use diagnostics::Diagnostic;
use salsa::Accumulator;

use crate::{
    Db,
    ElabDb,
    core::{
        BinderInfo,
        Term,
        TermArena,
        TermId,
    },
    elab::{
        local::{
            LocalBinder,
            LocalCtx,
        },
        meta::MetaOrigin,
    },
    env::Namespace,
    ids::{
        ItemId,
        Symbol,
        Unique,
        UniqueGen,
    },
};

pub struct ElabCtx<'db> {
    pub db: Db<'db>,
    pub current_decl: ItemId<'db>,

    pub arena: TermArena<'db>,
    pub gen_: UniqueGen,

    pub lctx: LocalCtx<'db>,
    pub namespace: Namespace<'db>,
}

impl<'db> ElabCtx<'db> {
    pub fn new(db: Db<'db>, current_decl: ItemId<'db>) -> Self {
        let file = current_decl.file(db);
        let namespace = db.def_map(file);
        Self {
            db,
            current_decl,
            arena: TermArena::new(),
            gen_: UniqueGen::new(),
            lctx: LocalCtx::default(),
            namespace,
        }
    }

    pub fn diagnostic(&self, diag: Diagnostic) {
        diag.accumulate(self.db);
    }

    pub fn fresh_fvar(
        &mut self,
        name: Option<Symbol<'db>>,
        ty: TermId,
        info: BinderInfo,
    ) -> Unique {
        let unique = self.gen_.fresh();
        self.lctx.push(LocalBinder {
            unique,
            name,
            ty,
            info,
            value: None,
        });
        unique
    }

    pub fn lower_type(&mut self, ty: ast::Type) -> TermId {
        match ty {
            ast::Type::Name(name) => {
                let path: Vec<Symbol> = name
                    .path()
                    .filter_map(|seg| {
                        seg.identifier()?
                            .text()
                            .map(|t| Symbol::from_str(self.db, t))
                    })
                    .collect();
                let member = name.member();
                let Some(member_text) = member.as_ref().and_then(|m| m.text()) else {
                    return self
                        .error_mvar(&MetaOrigin::Error("expected a member name".to_string()));
                };

                let member = Symbol::from_str(self.db, member_text);
                match self.namespace.resolve(self.db, &path, member) {
                    Some(item) => self.arena.alloc_term(Term::Const(item)),
                    None => self.error_mvar(&MetaOrigin::Error("unresolved name".to_string())),
                }
            }
            ast::Type::PiType(_) => {
                todo!();
            }
        }
    }

    pub fn error_mvar(&mut self, origin: &MetaOrigin) -> TermId {
        tracing::error!("encountered error metavariable: {origin:?}");
        let u = self.gen_.fresh();
        self.arena.alloc_term(Term::MVar(u))
    }

    pub fn lower_body(&mut self, _expr: ast::Expr) -> TermId {
        self.arena.type0()
    }

    pub fn placeholder(&mut self) -> TermId {
        self.arena.type0()
    }
}
