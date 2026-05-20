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
        Level,
        LevelId,
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
                        let text: String = seg
                            .identifier()
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

    pub fn infer(&mut self, expr: ast::Expr) -> (TermId, TermId) {
        // match expr {
        //     ast::Expr::Literal(lit) => {
        //         let term = self.lower_literal(lit);
        //         let ty = self.literal_type(lit);
        //         (term, ty)
        //     }
        //     ast::Expr::Name(n) => self.infer_name(n),
        //     ast::Expr::ParenExpr(p) => self.infer(p.expr().unwrap()),
        //     ast::Expr::BraceBlock(b) => self.infer_block(b),
        // }
        (self.error_mvar(), self.error_mvar())
    }

    pub fn check(&mut self, expr: ast::Expr, expected: TermId) -> TermId {
        let (term, ty) = self.infer(expr);
        term
    }

    pub fn placeholder(&mut self) -> TermId {
        self.arena.type0()
    }

    pub fn mk_error(&mut self, range: TextRange, message: &str) -> DiagnosticBuilder {
        let file = self.current_decl.file(self.db);
        Diagnostic::error(message, file, range)
    }

    pub fn import_term(&mut self, from: &TermArena<'db>, t: TermId) -> TermId {
        match from.get_term(t).clone() {
            Term::BVar(i) => self.arena.alloc_term(Term::BVar(i)),
            Term::FVar(u) => self.arena.alloc_term(Term::FVar(u)),
            Term::MVar(u) => self.arena.alloc_term(Term::MVar(u)),
            Term::Const(id) => self.arena.alloc_term(Term::Const(id)),
            Term::Unit => self.arena.alloc_term(Term::Unit),
            Term::Lit(l) => self.arena.alloc_term(Term::Lit(l)),
            Term::Sort(l) => {
                let l = self.import_level(from, l);
                self.arena.alloc_term(Term::Sort(l))
            }
            Term::App(f, x) => {
                let f = self.import_term(from, f);
                let x = self.import_term(from, x);
                self.arena.mk_app(f, x)
            }
            Term::Lam(i, t, b) => {
                let t = self.import_term(from, t);
                let b = self.import_term(from, b);
                self.arena.mk_lam(i, t, b)
            }
            Term::Pi(i, t, b) => {
                let t = self.import_term(from, t);
                let b = self.import_term(from, b);
                self.arena.mk_pi(i, t, b)
            }
            Term::Sigma(i, t, b) => {
                let t = self.import_term(from, t);
                let b = self.import_term(from, b);
                self.arena.mk_sigma(i, t, b)
            }
            Term::Let(t, v, b) => {
                let t = self.import_term(from, t);
                let v = self.import_term(from, v);
                let b = self.import_term(from, b);
                self.arena.mk_let(t, v, b)
            }
        }
    }

    fn import_level(&mut self, from: &TermArena<'db>, l: LevelId) -> LevelId {
        match from.get_level(l) {
            Level::Zero => self.arena.alloc_level(Level::Zero),
            Level::Succ(l) => {
                let l = self.import_level(from, *l);
                self.arena.alloc_level(Level::Succ(l))
            }
            Level::Max(lhs, rhs) => {
                let lhs = self.import_level(from, *lhs);
                let rhs = self.import_level(from, *rhs);
                self.arena.alloc_level(Level::Max(lhs, rhs))
            }
            Level::IMax(lhs, rhs) => {
                let lhs = self.import_level(from, *lhs);
                let rhs = self.import_level(from, *rhs);
                self.arena.alloc_level(Level::IMax(lhs, rhs))
            }
            Level::MVar(u) => self.arena.alloc_level(Level::MVar(*u)),
            Level::Param(s) => self.arena.alloc_level(Level::Param(*s)),
        }
    }
}
