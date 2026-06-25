use elaborator::{
    ItemId,
    ItemKind,
    core::{
        Term,
        TermKind,
    },
    elab::reduce::{
        instantiate_many,
        whnf,
        whnf_spine,
    },
};

use crate::{
    PirDatabase,
    types::{
        Type,
        TypeArg,
    },
};

pub fn lower_type<'db>(db: &'db dyn PirDatabase, ty: Term<'db>) -> Option<Type<'db>> {
    TypeLoweringCtx::new(db).lower_type(ty)
}

#[derive(Clone, Copy)]
enum Binder {
    TypeParam(usize),
    ValueIndex,
}

#[derive(Clone, Copy)]
enum ArgKind {
    Type,
    Index,
}

struct TypeLoweringCtx<'db> {
    db: &'db dyn PirDatabase,
    binders: Vec<Binder>,
    next_param: usize,
}

impl<'db> TypeLoweringCtx<'db> {
    fn new(db: &'db dyn PirDatabase) -> Self {
        Self {
            db,
            binders: Vec::new(),
            next_param: 0,
        }
    }

    fn lower_type(&mut self, ty: Term<'db>) -> Option<Type<'db>> {
        let ty = whnf(self.db, ty);
        let (head, args) = whnf_spine(self.db, ty);
        match head.kind(self.db) {
            TermKind::BVar(index) if args.is_empty() => Some(self.lower_bvar(*index)),
            TermKind::Const(id) => self.lower_const_applied(*id, &args),
            TermKind::Pi(..) if args.is_empty() => self.lower_pi_type(ty),
            TermKind::Sort(_) => None,
            TermKind::Lit(_) => unreachable!("literals should not appear in types"),
            _ => Some(Type::Todo(ty.debug(self.db).to_string())),
        }
    }

    fn lower_bvar(&self, index: usize) -> Type<'db> {
        match self.binders.iter().rev().nth(index) {
            Some(Binder::TypeParam(param)) => Type::Param(*param),
            Some(Binder::ValueIndex) => Type::Todo(format!("dependent value #{index}")),
            None => Type::Todo(format!("unbound #{index}")),
        }
    }

    fn lower_const_applied(&mut self, id: ItemId<'db>, args: &[Term<'db>]) -> Option<Type<'db>> {
        match id.kind(self.db) {
            ItemKind::Inductive => {
                let arg_kinds = self.inductive_arg_kinds(id);

                if args.len() < arg_kinds.len() {
                    return None;
                }

                if args.len() > arg_kinds.len() {
                    return Some(Type::Todo("overapplied inductive".to_string()));
                }

                let args = args
                    .iter()
                    .zip(arg_kinds)
                    .map(|(arg, kind)| self.lower_type_arg(*arg, kind))
                    .collect::<Vec<_>>();

                Some(Type::Adt { id, args })
            }

            ItemKind::Def => {
                let Some(body) = elaborator::elab::def::def_body(self.db, id).as_ref() else {
                    return Some(Type::Todo(format!(
                        "opaque def {}",
                        id.name(self.db).text(self.db)
                    )));
                };

                let expanded = instantiate_many(self.db, body.value, args.iter().copied());
                self.lower_type(expanded)
            }

            ItemKind::Constructor => None,
        }
    }

    fn lower_type_arg(&mut self, arg: Term<'db>, kind: ArgKind) -> TypeArg<'db> {
        match kind {
            ArgKind::Type => {
                let ty = self.lower_type(arg).unwrap_or_else(|| {
                    Type::Todo(format!("erased type argument {}", arg.debug(self.db)))
                });
                TypeArg::Type(ty)
            }
            ArgKind::Index => TypeArg::Index(whnf(self.db, arg)),
        }
    }

    fn inductive_arg_kinds(&self, id: ItemId<'db>) -> Vec<ArgKind> {
        let data = elaborator::elab::inductive::inductive_data(self.db, id);
        let signature = elaborator::elab::sig::signature(self.db, id);

        let mut cx = TypeLoweringCtx::new(self.db);
        let mut kinds = Vec::new();

        for &binder in &data.binders {
            kinds.push(cx.push_formal_arg(binder));
        }

        let mut current = whnf(self.db, signature.ty);
        while let TermKind::Pi(_, param, body) = current.kind(self.db) {
            kinds.push(cx.push_formal_arg(*param));
            current = whnf(self.db, *body);
        }

        kinds
    }

    fn push_formal_arg(&mut self, ty: Term<'db>) -> ArgKind {
        if self.lower_type(ty).is_some() {
            self.binders.push(Binder::ValueIndex);
            ArgKind::Index
        } else {
            let param = self.next_param;
            self.next_param += 1;
            self.binders.push(Binder::TypeParam(param));
            ArgKind::Type
        }
    }

    fn lower_pi_type(&mut self, ty: Term<'db>) -> Option<Type<'db>> {
        let outer_binders = self.binders.len();
        let mut current = whnf(self.db, ty);
        let mut params = Vec::new();

        while let TermKind::Pi(_, param, body) = current.kind(self.db) {
            if let Some(param) = self.lower_type(*param) {
                params.push(param);
                self.binders.push(Binder::ValueIndex);
            } else {
                let param = self.next_param;
                self.next_param += 1;
                self.binders.push(Binder::TypeParam(param));
            }

            current = whnf(self.db, *body);
        }

        let ret = self.lower_type(current)?;
        self.binders.truncate(outer_binders);

        if params.is_empty() {
            Some(ret)
        } else {
            Some(Type::Function {
                params,
                ret: Box::new(ret),
            })
        }
    }
}
