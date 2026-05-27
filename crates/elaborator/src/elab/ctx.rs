use ast::traits::AstNode;
use diagnostics::{
    Diagnostic,
    Label,
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
        FreeBinder,
        Level,
        Literal,
        Term,
    },
    elab::{
        expected::{
            Expected,
            ExpectedReason,
        },
        local::{
            LocalBinder,
            LocalCtx,
        },
        unify::UnifyError,
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
    util::naming::is_autobindable,
};

#[derive(Debug, Clone)]
pub enum Frame<'db> {
    DefBody { name: Symbol<'db> },
}

pub struct ElabCtx<'db> {
    pub db: Db<'db>,
    pub current_decl: ItemId<'db>,

    pub gen_: UniqueGen,

    pub lctx: LocalCtx<'db>,
    pub namespace: Namespace<'db>,

    pub frames: Vec<Frame<'db>>,
    pub erroneous_mvars: Vec<Unique>,

    pub autobound: Vec<FreeBinder<'db>>,
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
            frames: Vec::new(),
            erroneous_mvars: Vec::new(),
            autobound: Vec::new(),
        }
    }

    pub fn diagnostic(&self, diag: Diagnostic) {
        diag.accumulate(self.db);
    }

    pub fn with_frame<R>(&mut self, frame: Frame<'db>, body: impl FnOnce(&mut Self) -> R) -> R {
        self.frames.push(frame);
        let result = body(self);
        self.frames.pop();
        result
    }

    fn frame_notes(&self) -> Vec<String> {
        self.frames
            .iter()
            .rev()
            .map(|frame| match frame {
                Frame::DefBody { name } => {
                    format!("while checking the body of `{}`", name.text(self.db))
                }
            })
            .collect()
    }

    pub fn fresh_fvar(
        &mut self,
        name: Option<Symbol<'db>>,
        ty: Term<'db>,
        info: BinderInfo,
        origin: TextRange,
    ) -> Unique {
        let unique = self.gen_.fresh();
        self.lctx.push(LocalBinder {
            unique,
            name,
            ty,
            info,
            value: None,
            origin,
        });
        unique
    }

    pub fn lower_type(&mut self, ty: ast::Type) -> Term<'db> {
        match ty {
            ast::Type::Name(name) => match self.try_resolve_name(&name) {
                Some((term, _term_ty)) => term,
                None => self.autobind_or_error(&name),
            },
            ast::Type::PiType(_) => {
                todo!();
            }
        }
    }

    pub fn fresh_mvar(&mut self) -> Term<'db> {
        let u = self.gen_.fresh();
        Term::mvar(self.db, u)
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
            ast::Expr::ParenExpr(expr) => match expr.expr() {
                Some(inner) => self.infer(inner),
                None => (self.error_mvar(), self.error_mvar()),
            },
            ast::Expr::BraceBlock(block) => self.infer_block(&block),
        }
    }

    pub fn check(&mut self, expr: ast::Expr, expected: &Expected<'db>) -> Term<'db> {
        if let ast::Expr::BraceBlock(block) = expr {
            self.check_block(&block, expected)
        } else {
            let range = expr.syntax().text_range();
            let (term, ty) = self.infer(expr);
            if let Err(err) = self.unify(ty, expected.ty) {
                self.report_mismatch(range, ty, expected, &err);
            }
            term
        }
    }

    pub fn placeholder(&mut self) -> Term<'db> {
        Term::type0(self.db)
    }

    fn check_block(&mut self, block: &ast::BraceBlock, expected: &Expected<'db>) -> Term<'db> {
        let (term, _) = self.lower_stmt(block.stmt(), block.syntax().text_range(), Some(expected));
        term
    }

    fn infer_block(&mut self, block: &ast::BraceBlock) -> (Term<'db>, Term<'db>) {
        self.lower_stmt(block.stmt(), block.syntax().text_range(), None)
    }

    fn lower_stmt<I>(
        &mut self,
        mut iter: I,
        range: TextRange,
        expected: Option<&Expected<'db>>,
    ) -> (Term<'db>, Term<'db>)
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
                let origin = let_stmt.syntax().text_range();
                let (value, ty) = if let Some(expr) = let_stmt.expr() {
                    self.infer(expr)
                } else {
                    (self.error_mvar(), self.error_mvar())
                };
                let saved_lctx = self.lctx.clone();
                let fvar = self.fresh_fvar(name, ty, BinderInfo::Explicit, origin);
                let (body, body_ty) = self.lower_stmt(iter, range, expected);
                let body = self.abstract_fvar(&body, fvar);
                self.lctx = saved_lctx;
                let let_expr = Term::let_(self.db, ty, value, body);
                (let_expr, body_ty)
            }
            Some(ast::Stmt::MutationStmt(mutation)) => {
                let (value, ty) = if let Some(expr) = mutation.expr() {
                    self.infer(expr)
                } else {
                    (self.error_mvar(), self.error_mvar())
                };
                let (body, body_ty) = self.lower_stmt(iter, range, expected);
                let let_expr = Term::let_(self.db, ty, value, body);
                (let_expr, body_ty)
            }
            Some(ast::Stmt::ReturnStmt(return_)) => match return_.expr() {
                Some(expr) => {
                    if let Some(expected) = expected {
                        let term = self.check(expr, expected);
                        (term, expected.ty)
                    } else {
                        self.infer(expr)
                    }
                }
                None => self.unit_value(range, expected),
            },
            None => self.unit_value(range, expected),
        }
    }

    fn unit_value(
        &mut self,
        range: TextRange,
        expected: Option<&Expected<'db>>,
    ) -> (Term<'db>, Term<'db>) {
        let unit_ty = self.lang_item(&LangItem::Unit, range);
        let unit_const = self.lang_item(&LangItem::UnitConstructor, range);
        if let Some(expected) = expected
            && let Err(err) = self.unify(unit_ty, expected.ty)
        {
            self.report_mismatch(range, unit_ty, expected, &err);
        }
        (unit_const, unit_ty)
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

    fn resolve_name(&mut self, name: &ast::Name) -> (Term<'db>, Term<'db>) {
        match self.try_resolve_name(name) {
            Some(resolved) => resolved,
            None => self.unresolved_name(name),
        }
    }

    #[instrument(skip(self))]
    fn try_resolve_name(&mut self, name: &ast::Name) -> Option<(Term<'db>, Term<'db>)> {
        let path: Vec<Symbol> = name
            .path()
            .map(|seg| {
                let text = seg
                    .identifier()
                    .and_then(|s| s.text().map(str::to_owned))
                    .unwrap_or_else(|| "<unknown>".to_owned());
                Symbol::from_str(self.db, &text)
            })
            .collect();
        let member = name.member();
        let member_txt = member.as_ref().and_then(|m| m.text())?;
        if member_txt == "Type" && path.is_empty() {
            let type0 = Term::type0(self.db);
            let type1 = Term::sort(self.db, Level::two(self.db));
            return Some((type0, type1));
        }

        let member = Symbol::from_str(self.db, member_txt);
        if let Some(local) = self.lctx.find_by_name(member) {
            let ty = local.ty;
            let reference = Term::fvar(self.db, local.unique);
            return Some((reference, ty));
        }

        let item = self.namespace.resolve(self.db, &path, member)?;
        let item_ty = self.db.signature(item).ty;
        let item_term = Term::constant(self.db, item);
        Some((item_term, item_ty))
    }

    fn autobind_or_error(&mut self, name: &ast::Name) -> Term<'db> {
        let is_qualified = name.path().next().is_some();
        let member = name.member();
        if let Some(member_txt) = member.as_ref().and_then(|m| m.text())
            && !is_qualified
            && is_autobindable(member_txt)
        {
            let symbol = Symbol::from_str(self.db, member_txt);
            self.fresh_autobound(symbol, name.syntax().text_range())
        } else {
            self.unresolved_name(name).0
        }
    }

    fn fresh_autobound(&mut self, name: Symbol<'db>, origin: TextRange) -> Term<'db> {
        let u = self.gen_.fresh();
        let sort = Term::sort(self.db, Level::mvar(self.db, u));
        let fvar = self.fresh_fvar(Some(name), sort, BinderInfo::Implicit, origin);
        self.autobound
            .push(FreeBinder::new(fvar, BinderInfo::Implicit, sort));
        Term::fvar(self.db, fvar)
    }

    fn unresolved_name(&mut self, name: &ast::Name) -> (Term<'db>, Term<'db>) {
        let path_txt: String = name
            .path()
            .map(|seg| {
                let text = seg
                    .identifier()
                    .and_then(|s| s.text().map(str::to_owned))
                    .unwrap_or_else(|| "<unknown>".to_owned());
                text + "::"
            })
            .collect();
        let member_txt = name
            .member()
            .as_ref()
            .and_then(|m| m.text())
            .unwrap_or("<unknown>")
            .to_owned();
        let diag = self
            .mk_error(
                name.syntax().text_range(),
                &format!("unresolved name '{path_txt}{member_txt}'"),
            )
            .build();
        self.diagnostic(diag);
        (self.error_mvar(), self.error_mvar())
    }

    fn report_mismatch(
        &mut self,
        range: TextRange,
        found: Term<'db>,
        expected: &Expected<'db>,
        err: &UnifyError<'db>,
    ) {
        let expected_txt = expected.ty.debug(self.db).to_string();
        let found_txt = found.debug(self.db).to_string();

        let mut builder = self
            .mk_error(
                range,
                &format!("type mismatch: expected `{expected_txt}`, found `{found_txt}`"),
            )
            .with_primary_message(format!("this is `{found_txt}`, expected `{expected_txt}`"));

        match expected.reason {
            ExpectedReason::ReturnType { annotation } => {
                let label = self.mk_label(
                    annotation,
                    &format!("expected `{expected_txt}` because of this return type"),
                );
                builder = builder.with_secondary_label(label);
            }
            ExpectedReason::Annotation { range: ann } => {
                let label = self.mk_label(
                    ann,
                    &format!("expected `{expected_txt}` because of this annotation"),
                );
                builder = builder.with_secondary_label(label);
            }
            ExpectedReason::None => {}
        }

        let (root_found, root_expected) = err.root();
        if root_found != found || root_expected != expected.ty {
            builder = builder.with_note(format!(
                "the conflict is between `{}` and `{}`",
                root_found.debug(self.db),
                root_expected.debug(self.db)
            ));
        }

        for note in self.frame_notes() {
            builder = builder.with_note(note);
        }

        self.diagnostic(builder.build());
    }

    pub fn elaborate_binders<I>(&mut self, binders: I) -> Vec<FreeBinder<'db>>
    where
        I: Iterator<Item = ast::Binder>,
    {
        binders.map(|b| self.elaborate_binder(&b)).collect()
    }

    pub fn elaborate_binder(&mut self, binder: &ast::Binder) -> FreeBinder<'db> {
        let info = match binder.info() {
            ast::BinderInfo::Implicit => BinderInfo::Implicit,
            ast::BinderInfo::InstanceImplicit => BinderInfo::InstanceImplicit,
            ast::BinderInfo::Explicit => BinderInfo::Explicit,
        };
        let binder_name = binder
            .name()
            .and_then(|n| n.ident())
            .as_ref()
            .map(|n| Symbol::from_str(self.db, n.text()));
        let ty = if let Some(ty) = binder.ty() {
            self.lower_type(ty)
        } else {
            self.error_mvar()
        };

        let unique = self.fresh_fvar(binder_name, ty, info, binder.syntax().text_range());
        FreeBinder::new(unique, info, ty)
    }

    pub fn with_binders<I>(
        &mut self,
        binders: I,
        body: impl FnOnce(&mut Self) -> Term<'db>,
    ) -> Term<'db>
    where
        I: Iterator<Item = ast::Binder>,
    {
        self.with_binders_impl(binders, body, Term::lam)
    }

    pub fn with_pi_binders<I>(
        &mut self,
        binders: I,
        body: impl FnOnce(&mut Self) -> Term<'db>,
    ) -> Term<'db>
    where
        I: Iterator<Item = ast::Binder>,
    {
        self.with_binders_impl(binders, body, Term::pi)
    }

    fn with_binders_impl<I>(
        &mut self,
        binders: I,
        body: impl FnOnce(&mut Self) -> Term<'db>,
        mk: fn(Db<'db>, BinderInfo, Term<'db>, Term<'db>) -> Term<'db>,
    ) -> Term<'db>
    where
        I: Iterator<Item = ast::Binder>,
    {
        let free_binders = self.elaborate_binders(binders);
        let saved_lctx = self.lctx.clone();
        let free_result = body(self);
        let result = self.abstract_binders_with(&free_binders, free_result, mk);
        self.lctx = saved_lctx;
        result
    }

    pub fn abstract_autobound_pi(&mut self, body: Term<'db>) -> Term<'db> {
        let autobound = std::mem::take(&mut self.autobound);
        self.abstract_binders_with(&autobound, body, Term::pi)
    }

    pub fn abstract_autobound_lam(&mut self, body: Term<'db>) -> Term<'db> {
        let autobound = std::mem::take(&mut self.autobound);
        self.abstract_binders_with(&autobound, body, Term::lam)
    }

    pub fn abstract_binders(
        &mut self,
        binder_fvars: &[FreeBinder<'db>],
        body: Term<'db>,
    ) -> Term<'db> {
        self.abstract_binders_with(binder_fvars, body, Term::lam)
    }

    fn abstract_binders_with(
        &self,
        binder_fvars: &[FreeBinder<'db>],
        body: Term<'db>,
        mk: fn(Db<'db>, BinderInfo, Term<'db>, Term<'db>) -> Term<'db>,
    ) -> Term<'db> {
        let mut term = body;
        for binder in binder_fvars.iter().rev() {
            term = self.abstract_fvar(&term, binder.fvar);
            term = mk(self.db, binder.info, binder.ty, term);
        }
        term
    }

    pub fn mk_error(&mut self, range: TextRange, message: &str) -> DiagnosticBuilder {
        let file = self.current_decl.file(self.db);
        Diagnostic::error(message, file, range)
    }

    pub fn mk_label(&mut self, range: TextRange, message: &str) -> Label {
        let file = self.current_decl.file(self.db);
        Label {
            file,
            range,
            message: Some(message.to_string()),
        }
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
