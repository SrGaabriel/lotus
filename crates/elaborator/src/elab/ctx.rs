use ast::traits::AstNode;
use diagnostics::{
    Diagnostic,
    builder::DiagnosticBuilder,
};
use salsa::Accumulator;
use text_size::TextRange;
use tracing::instrument;

use crate::{
    Db,
    ElabDb,
    core::{
        BinderInfo,
        Level,
        Literal,
        Term,
    },
    elab::local::{
        LocalBinder,
        LocalCtx,
    },
    env::{
        Namespace,
        lang_items::LangItem,
    },
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

    pub gen_: UniqueGen,

    pub lctx: LocalCtx<'db>,
    pub namespace: Namespace<'db>,

    pub erroneous_mvars: Vec<Unique>,
}

impl<'db> ElabCtx<'db> {
    pub fn new(db: Db<'db>, current_decl: ItemId<'db>) -> Self {
        let file = current_decl.file(db);
        let namespace = db.def_map(file);
        Self {
            db,
            current_decl,
            gen_: UniqueGen::new(),
            lctx: LocalCtx::default(),
            namespace,
            erroneous_mvars: Vec::new(),
        }
    }

    pub fn diagnostic(&self, diag: Diagnostic) {
        diag.accumulate(self.db);
    }

    pub fn fresh_fvar(
        &mut self,
        name: Option<Symbol<'db>>,
        ty: Term<'db>,
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

    pub fn lower_type(&mut self, ty: ast::Type) -> Term<'db> {
        match ty {
            ast::Type::Name(name) => {
                let (term, _term_ty) = self.resolve_name(&name);
                term
            }
            ast::Type::PiType(_) => {
                todo!();
            }
        }
    }

    pub fn error_mvar(&mut self) -> Term<'db> {
        let u = self.gen_.fresh();
        self.erroneous_mvars.push(u);
        Term::mvar(self.db, u)
    }

    pub fn infer(&mut self, expr: ast::Expr) -> (Term<'db>, Term<'db>) {
        match expr {
            ast::Expr::Literal(lit) => {
                let term = self.lower_literal(lit.clone());
                let ty = self.literal_type(lit);
                (term, ty)
            }
            ast::Expr::Name(name) => self.resolve_name(&name),
            ast::Expr::ParenExpr(expr) => {
                if let Some(inner) = expr.expr() {
                    self.infer(inner)
                } else {
                    (self.error_mvar(), self.error_mvar())
                }
            }
            ast::Expr::BraceBlock(block) => self.lower_block(&block),
        }
    }

    pub fn check(&mut self, expr: ast::Expr, expected: Term<'db>) -> Term<'db> {
        let text_range = expr.syntax().text_range();
        let (term, ty) = self.infer(expr);
        if !self.unify(ty, expected) {
            let expected_txt = expected.debug(self.db);
            let ty_txt = ty.debug(self.db);
            let diag = self
                .mk_error(
                    text_range,
                    &format!("expected {expected_txt}, found {ty_txt}"),
                )
                .build();
            self.diagnostic(diag);
        }
        term
    }

    pub fn placeholder(&mut self) -> Term<'db> {
        Term::type0(self.db)
    }

    fn lower_block(&mut self, block: &ast::BraceBlock) -> (Term<'db>, Term<'db>) {
        self.lower_stmt(block.stmt(), block.syntax().text_range())
    }

    fn lower_stmt<I>(&mut self, mut iter: I, range: TextRange) -> (Term<'db>, Term<'db>)
    where
        I: Iterator<Item = ast::Stmt>,
    {
        match iter.next() {
            Some(ast::Stmt::LetStmt(let_stmt)) => {
                let name = let_stmt
                    .name()
                    .and_then(|n| n.ident())
                    .as_ref()
                    .map(|n| Symbol::from_str(self.db, n.text()));
                let (value, ty) = if let Some(expr) = let_stmt.expr() {
                    tracing::debug!("expr: {:?}", expr);
                    self.infer(expr)
                } else {
                    (self.error_mvar(), self.error_mvar())
                };
                let saved_lctx = self.lctx.clone();
                tracing::debug!(
                    "fresh fvar {:?} {}",
                    name.map(|t| t.text(self.db).clone()),
                    ty.debug(self.db),
                );
                self.fresh_fvar(name, ty, BinderInfo::Explicit);
                let (body, body_ty) = self.lower_stmt(iter, range);
                self.lctx = saved_lctx;
                let let_expr = Term::let_(self.db, value, body_ty, body);
                (let_expr, body_ty)
            }
            Some(ast::Stmt::MutationStmt(mutation)) => {
                let (value, _ty) = if let Some(expr) = mutation.expr() {
                    self.infer(expr)
                } else {
                    (self.error_mvar(), self.error_mvar())
                };
                let (body, body_ty) = self.lower_stmt(iter, range);
                let let_expr = Term::let_(self.db, value, body_ty, body);
                (let_expr, body_ty)
            }
            Some(ast::Stmt::ReturnStmt(return_)) => {
                let (value, ty) = if let Some(expr) = return_.expr() {
                    self.infer(expr)
                } else {
                    (self.error_mvar(), self.error_mvar())
                };
                tracing::debug!(
                    "KIRE {} and {}",
                    value.debug(self.db),
                    ty.debug(self.db)
                );
                (value, ty)
            }
            None => {
                let unit_ty = self.lang_item(&LangItem::Unit, range);
                let unit_const = self.lang_item(&LangItem::UnitConstructor, range);
                (unit_const, unit_ty)
            }
        }
    }

    fn lower_literal(&mut self, lit: ast::Literal) -> Term<'db> {
        match lit {
            ast::Literal::NumberLit(num) => {
                let Some(value) = num.text().and_then(|s| s.parse::<u64>().ok()) else {
                    return self.error_mvar();
                };
                Term::lit(self.db, Literal::Number(value))
            }
            ast::Literal::StringLit(s) => {
                let Some(value) = s.unquoted().map(std::string::ToString::to_string) else {
                    return self.error_mvar();
                };
                Term::lit(self.db, Literal::Str(value))
            }
        }
    }

    fn literal_type(&mut self, lit: ast::Literal) -> Term<'db> {
        match lit {
            ast::Literal::NumberLit(num) => {
                self.lang_item(&LangItem::Int32, num.syntax().text_range())
            }
            ast::Literal::StringLit(s) => self.lang_item(&LangItem::Str, s.syntax().text_range()),
        }
    }

    #[instrument(skip(self))]
    fn resolve_name(&mut self, name: &ast::Name) -> (Term<'db>, Term<'db>) {
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
            return (self.error_mvar(), self.error_mvar());
        };
        if member_txt == "Type" && path.is_empty() {
            let u = self.gen_.fresh();
            let level = Level::mvar(self.db, u);
            let succ = Level::succ(self.db, level);
            return (Term::sort(self.db, succ), Term::sort(self.db, level));
        }

        let member = Symbol::from_str(self.db, member_txt);
        if let Some(local) = self.lctx.find_by_name(member) {
            let ty = local.ty;
            let reference = Term::fvar(self.db, local.unique);
            return (reference, ty);
        }

        if let Some(item) = self.namespace.resolve(self.db, &path, member) {
            let item_ty = self.db.signature(item).ty;
            let item_term = Term::constant(self.db, item);
            (item_term, item_ty)
        } else {
            let path_txt = path_strs.into_iter().map(|w| w + "::").collect::<String>();
            let diag = self
                .mk_error(
                    name.syntax().text_range(),
                    &format!("unresolved name '{path_txt}{member_txt}'"),
                )
                .build();
            self.diagnostic(diag);
            (self.error_mvar(), self.error_mvar())
        }
    }

    pub fn mk_error(&mut self, range: TextRange, message: &str) -> DiagnosticBuilder {
        let file = self.current_decl.file(self.db);
        Diagnostic::error(message, file, range)
    }

    pub fn lang_item(&mut self, lang_item: &LangItem, range: TextRange) -> Term<'db> {
        let file = self.current_decl.file(self.db);
        let Some(item_id) = self.db.lang_items(file).get(lang_item).copied() else {
            let diag = self
                .mk_error(range, &format!("missing language item: {lang_item}"))
                .build();
            self.diagnostic(diag);
            return self.error_mvar();
        };
        Term::constant(self.db, item_id)
    }
}
