use ast::traits::AstNode;
use diagnostics::{
    Diagnostic,
    builder::DiagnosticBuilder,
};
use salsa::Accumulator;
use text_size::TextRange;

use crate::{
    Db,
    ElabDb,
    core::{
        BinderInfo,
        Term,
        TermArena,
        TermId,
    },
    elab::local::{
        LocalBinder,
        LocalCtx,
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
                let (path_strs, path): (Vec<String>, Vec<Symbol>) = name
                    .path()
                    .map(|seg| {
                        let text: String = seg.identifier()
                            .and_then(|s| s.text().map(str::to_owned))
                            .unwrap_or_else(|| "<unknown>".to_owned());
                        let symbol = Symbol::from_str(self.db, &text);
                        (text, symbol)
                    })
                    .unzip();
                let member = name.member();
                let Some(member_txt) = member.as_ref().and_then(|m| m.text()) else {
                    return self.error_mvar();
                };
                if member_txt == "Type" && path.is_empty() {
                    return self.arena.type0();
                }

                let member = Symbol::from_str(self.db, member_txt);
                if let Some(item) = self.namespace.resolve(self.db, &path, member) {
                    self.arena.alloc_term(Term::Const(item))
                } else {
                    let path_txt = path_strs.into_iter().map(|w| w + "::").collect::<String>();
                    let diag = self
                        .mk_error(
                            name.syntax().text_range(),
                            &format!("unresolved name '{path_txt}{member_txt}'"),
                        )
                        .build();
                    self.diagnostic(diag);
                    self.error_mvar()
                }
            }
            ast::Type::PiType(_) => {
                todo!();
            }
        }
    }

    pub fn error_mvar(&mut self) -> TermId {
        let u = self.gen_.fresh();
        self.arena.alloc_term(Term::MVar(u))
    }

    pub fn lower_body(&mut self, _expr: ast::Expr) -> TermId {
        self.arena.type0()
    }

    pub fn placeholder(&mut self) -> TermId {
        self.arena.type0()
    }

    pub fn mk_error(&mut self, range: TextRange, message: &str) -> DiagnosticBuilder {
        let file = self.current_decl.file(self.db);
        Diagnostic::error(message, file, range)
    }
}
