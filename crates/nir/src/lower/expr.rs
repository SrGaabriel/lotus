use std::collections::HashMap;

use elaborator::{
    ItemId,
    core::{
        BinderInfo,
        Term,
        TermKind,
    },
    elab::{
        ctx::ElabCtx,
        subst::instantiate,
    },
    ids::Unique,
};
use text_size::TextRange;

use crate::{
    NirDatabase,
    lower::types::lower_type,
    types::{
        Atom,
        Code,
        LetValue,
        LocalId,
    },
};

struct ExprLoweringCtx<'db> {
    db: &'db dyn NirDatabase,
    elab_ctx: ElabCtx<'db>,
    next_local: LocalId,
    runtime_fvars: HashMap<Unique, LocalId>,
}

type Continuation<'db, 'a> =
    Box<dyn FnOnce(&mut ExprLoweringCtx<'db>, Atom<'db>) -> Code<'db> + 'a>;

impl<'db> ExprLoweringCtx<'db> {
    fn new(db: &'db dyn NirDatabase, decl: ItemId<'db>) -> Self {
        Self {
            db,
            elab_ctx: ElabCtx::new(db, decl),
            next_local: 0,
            runtime_fvars: HashMap::new(),
        }
    }

    fn lower_expr<'a>(
        &mut self,
        term: Term<'db>,
        continuation: Continuation<'db, 'a>,
    ) -> Code<'db> {
        if let TermKind::Let(value_ty, value, body) = term.kind(self.db)
            && lower_type(self.db, *value_ty).is_none()
        {
            let body = instantiate(self.db, body, *value);
            return self.lower_expr(body, continuation);
        }

        // todo: check if this is really needed
        let _ = self
            .elab_ctx
            .infer_term(term)
            .expect("elaborated core term should be well-typed");

        match term.kind(self.db) {
            TermKind::Lit(literal) => continuation(self, Atom::Literal(literal.clone())),

            TermKind::Const(item) => continuation(self, Atom::Global(*item)),

            TermKind::FVar(id) => {
                let local = self
                    .runtime_fvars
                    .get(id)
                    .copied()
                    .unwrap_or_else(|| panic!("unbound free variable {id:?}"));
                continuation(self, Atom::Local(local))
            }

            TermKind::Let(value_ty, value, body) => {
                self.lower_runtime_let(*value_ty, *value, *body, continuation)
            }

            TermKind::App(..) => {
                todo!("lower runtime applications")
            }

            TermKind::Lam(..) => todo!("lower runtime lambdas with static and runtime parameters"),

            TermKind::BVar(_)
            | TermKind::MVar(_)
            | TermKind::Sort(_)
            | TermKind::Pi(..)
            | TermKind::Sigma(..)
            | TermKind::Error(_) => {
                unreachable!("non-runtime term in NIR lowering: {}", term.debug(self.db))
            }
        }
    }

    fn lower_runtime_let<'a>(
        &mut self,
        value_ty: Term<'db>,
        value: Term<'db>,
        body: Term<'db>,
        continuation: Continuation<'db, 'a>,
    ) -> Code<'db> {
        let ty = lower_type(self.db, value_ty)
            .expect("a runtime let binder must have a runtime NIR type");

        self.lower_expr(
            value,
            Box::new(move |ctx, value| {
                let binder = ctx.fresh_local();

                let lctx_level = ctx.elab_ctx.lctx.level();
                let fvar = ctx.elab_ctx.fresh_fvar(
                    None,
                    value_ty,
                    BinderInfo::Explicit,
                    TextRange::default(),
                    None,
                );
                let body = instantiate(ctx.db, &body, Term::fvar(ctx.db, fvar));
                let previous = ctx.runtime_fvars.insert(fvar, binder);
                debug_assert!(previous.is_none());

                let body = ctx.lower_expr(body, continuation);

                ctx.runtime_fvars.remove(&fvar);
                ctx.elab_ctx.lctx.pop_to(lctx_level);

                Code::Let {
                    binder,
                    ty,
                    value: LetValue::Atom(value),
                    body: Box::new(body),
                }
            }),
        )
    }

    fn fresh_local(&mut self) -> LocalId {
        let local = self.next_local;
        self.next_local += 1;
        local
    }
}

pub fn lower_body<'db>(db: &'db dyn NirDatabase, decl: ItemId<'db>, term: Term<'db>) -> Code<'db> {
    let mut ctx = ExprLoweringCtx::new(db, decl);
    ctx.lower_expr(term, Box::new(|_, result| Code::Return(result)))
}
