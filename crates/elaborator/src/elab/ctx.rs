use std::str::FromStr;

use ast::traits::AstNode;
use diagnostics::Label;
use literals::{
    Literal,
    NumberLiteral,
};
use text_size::TextRange;
use tracing::instrument;

use crate::{
    Db,
    ElabDb,
    core::{
        BinderInfo,
        FreeBinder,
        Level,
        Term,
    },
    elab::{
        diag::Frame,
        expected::Expected,
        local::{
            LocalBinder,
            LocalCtx,
        },
        meta::MetaCtx,
        subst::abstract_fvar,
    },
    env::{
        Namespace,
        lang_items::{
            LangItem,
            item_range,
            visible_lang_items,
        },
    },
    ids::{
        ItemId,
        Symbol,
        Unique,
        UniqueGen,
    },
    util::naming::is_autobindable,
};

pub struct ElabCtx<'db> {
    pub db: Db<'db>,
    pub current_decl: ItemId<'db>,

    pub gen_: UniqueGen,

    pub lctx: LocalCtx<'db>,
    pub mctx: MetaCtx<'db>,
    pub namespace: Namespace<'db>,

    pub frames: Vec<Frame<'db>>,
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
            mctx: MetaCtx::new(),
            namespace,
            frames: Vec::new(),
            autobound: Vec::new(),
        }
    }

    pub fn fresh_fvar(
        &mut self,
        name: Option<Symbol<'db>>,
        ty: Term<'db>,
        info: BinderInfo,
        origin: TextRange,
        parent: Option<Unique>,
    ) -> Unique {
        let unique = self.gen_.fresh();
        self.lctx.push(LocalBinder {
            unique,
            name,
            ty,
            parent,
            info,
            value: None,
            origin,
        });
        unique
    }

    pub fn lower_type(&mut self, ty: ast::Type) -> Term<'db> {
        match ty {
            ast::Type::PiType(pi) => self.lower_pi_type(&pi),
            ast::Type::Name(name) => match self.try_resolve_name(&name) {
                Some((term, _term_ty)) => term,
                None => self.autobind_or_error(&name),
            },
            ast::Type::AppType(app) => {
                let func = match app.func() {
                    Some(f) => self.lower_type(f),
                    None => self.poison().0,
                };
                let arg = match app.arg() {
                    Some(a) => self.lower_type(a),
                    None => self.poison().0,
                };
                Term::app(self.db, func, arg)
            }
            ast::Type::ArrowType(arr) => {
                let dom = match arr.dom() {
                    Some(t) => self.lower_type(t),
                    None => self.poison().0,
                };
                let cod = match arr.cod() {
                    Some(t) => self.lower_type(t),
                    None => self.poison().0,
                };
                Term::pi(self.db, BinderInfo::Explicit, dom, cod)
            }
            ast::Type::ParenType(p) => match p.r#type() {
                Some(inner) => self.lower_type(inner),
                None => self.poison().0,
            },
        }
    }

    pub fn fresh_mvar(&mut self, ty: Term<'db>) -> Term<'db> {
        let u = self.gen_.fresh();
        self.mctx.register_meta(u, ty, self.lctx.clone());
        Term::mvar(self.db, u)
    }

    pub fn infer(&mut self, expr: ast::Expr) -> (Term<'db>, Term<'db>) {
        match expr {
            ast::Expr::Literal(lit) => {
                let span = lit.syntax().text_range();
                let term = self.lower_literal(lit.clone());
                let ty = self.infer_term_with_diagnostics(term, span);
                (term, ty)
            }
            ast::Expr::Name(name) => self.resolve_name(&name),
            ast::Expr::ParenExpr(expr) => match expr.expr() {
                Some(inner) => self.infer(inner),
                None => (self.error_term(), self.error_term()),
            },
            ast::Expr::BraceBlock(block) => self.infer_block(&block),
            ast::Expr::AppExpr(app) => self.infer_app(&app),
        }
    }

    pub fn check(&mut self, expr: ast::Expr, expected: &Expected<'db>) -> Term<'db> {
        if let ast::Expr::BraceBlock(block) = expr {
            self.check_block(&block, expected)
        } else {
            let range = expr.syntax().text_range();
            let (term, ty) = self.infer(expr);
            let Some((term, ty)) = self.insert_implicits_for_expected(term, ty, expected.ty, range)
            else {
                return self.error_term();
            };
            if let Err(err) = self.unify(ty, expected.ty) {
                let diag = self.mismatch(range, ty, expected, &err);
                self.emit(diag);
            }
            term
        }
    }

    pub fn placeholder_ty(&mut self) -> Term<'db> {
        let mvar = self.gen_.fresh();
        Term::sort(self.db, Level::mvar(self.db, mvar))
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
                    (self.error_term(), self.error_term())
                };
                let saved_lctx = self.lctx.clone();
                let fvar = self.fresh_fvar(name, ty, BinderInfo::Explicit, origin, None);
                let (body, body_ty) = self.lower_stmt(iter, range, expected);
                let body = abstract_fvar(self.db, &body, fvar);
                self.lctx = saved_lctx;
                let let_expr = Term::let_(self.db, ty, value, body);
                (let_expr, body_ty)
            }
            Some(ast::Stmt::MutationStmt(mutation)) => {
                let name = mutation
                    .name()
                    .and_then(|n| n.ident())
                    .as_ref()
                    .map(|n| Symbol::from_str(self.db, n.text()));
                let origin = mutation.syntax().text_range();
                let (value, ty) = if let Some(expr) = mutation.expr() {
                    self.infer(expr)
                } else {
                    (self.error_term(), self.error_term())
                };
                let saved_lctx = self.lctx.clone();
                if name.is_none() {
                    return (self.error_term(), self.error_term());
                }

                let Some(latest) = self.lctx.find_by_name(name.unwrap()) else {
                    let diag = self.mk_error(
                        origin,
                        &format!(
                            "cannot mutate undefined variable `{}`",
                            name.unwrap().text(self.db)
                        ),
                    );
                    return self.error(diag);
                };
                let fvar =
                    self.fresh_fvar(name, ty, BinderInfo::Explicit, origin, Some(latest.unique));

                let (body, body_ty) = self.lower_stmt(iter, range, expected);
                let body = abstract_fvar(self.db, &body, fvar);
                self.lctx = saved_lctx;
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
            let diag = self.mismatch(range, unit_ty, expected, &err);
            self.emit(diag);
        }
        (unit_const, unit_ty)
    }

    fn lower_literal(&mut self, lit: ast::Literal) -> Term<'db> {
        match lit {
            ast::Literal::NumberLit(num) => {
                let Some(text) = num.text() else {
                    return self.poison().0;
                };
                match NumberLiteral::from_str(text) {
                    Ok(number) => Term::lit(self.db, Literal::Numeric(number)),
                    Err(e) => {
                        let diagnostic = self.mk_error(num.syntax().text_range(), &e.to_string());
                        self.error(diagnostic).0
                    }
                }
            }
            ast::Literal::StringLit(s) => {
                let Some(value) = s.unquoted().map(std::string::ToString::to_string) else {
                    return self.poison().0;
                };
                Term::lit(self.db, Literal::Text(value))
            }
        }
    }

    fn lower_pi_type(&mut self, pi: &ast::PiType) -> Term<'db> {
        let binder = pi.binder();
        if let Some(b) = binder
            && let Some(cod) = pi.r#type()
        {
            self.with_pi_binders(std::iter::once(b), |cx| cx.lower_type(cod))
        } else {
            self.poison().1
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
        let fvar = self.fresh_fvar(Some(name), sort, BinderInfo::Implicit, origin, None);
        self.autobound
            .push(FreeBinder::new(fvar, BinderInfo::Implicit, sort));
        Term::fvar(self.db, fvar)
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
            self.poison().1
        };

        let unique = self.fresh_fvar(binder_name, ty, info, binder.syntax().text_range(), None);
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
            term = abstract_fvar(self.db, &term, binder.fvar);
            term = mk(self.db, binder.info, binder.ty, term);
        }
        term
    }

    pub fn lang_item(&mut self, lang_item: &LangItem, range: TextRange) -> Term<'db> {
        let file = self.current_decl.file(self.db);
        let candidates = visible_lang_items(self.db, file)
            .get(lang_item)
            .map_or(&[][..], Vec::as_slice);
        let Some(&item_id) = candidates.first() else {
            let builder = self
                .mk_error(range, &format!("missing language item: {lang_item}"))
                .with_help(format!(
                    "define or import an item annotated with `@[lang \"{lang_item}\"]`"
                ));
            return self.error(builder).0;
        };
        if candidates.len() > 1 {
            let mut builder =
                self.mk_error(range, &format!("ambiguous language item `{lang_item}`"));
            for &candidate in candidates {
                if let Some(defined_at) = item_range(self.db, candidate) {
                    builder = builder.with_secondary_label(Label {
                        file: candidate.file(self.db),
                        range: defined_at,
                        message: Some("candidate defined here".to_string()),
                    });
                }
            }
            return self.error(builder).0;
        }
        Term::constant(self.db, item_id)
    }
}
