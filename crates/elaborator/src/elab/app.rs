use ast::traits::AstNode;
use diagnostics::builder::Diag;
use text_size::TextRange;

use crate::{
    core::{
        BinderInfo,
        Term,
        TermKind,
    },
    elab::{
        ctx::ElabCtx,
        expected::{
            Expected,
            ExpectedReason,
        },
        subst::instantiate,
    },
};

struct SurfaceArg {
    expr: Option<ast::Expr>,
    range: TextRange,
}

struct AppState<'db> {
    term: Term<'db>,
    ty: Term<'db>,
}

impl<'db> ElabCtx<'db> {
    pub fn infer_app(&mut self, expr: &ast::AppExpr) -> (Term<'db>, Term<'db>) {
        let (head, args) = flatten_app(expr);

        let (head_term, head_ty) = match head {
            Some(head) => self.infer(head),
            None => self.poison(),
        };

        let mut state = AppState::new(head_term, head_ty);
        for arg in args {
            if let Err(builder) = state.consume_arg(self, arg) {
                return self.error(builder);
            }
        }
        state.finish()
    }

    pub fn insert_implicits_for_expected(
        &mut self,
        term: Term<'db>,
        ty: Term<'db>,
        expected_ty: Term<'db>,
        range: TextRange,
    ) -> Option<(Term<'db>, Term<'db>)> {
        let mut state = AppState::new(term, ty);
        match state.insert_implicits_for_expected(self, expected_ty, range) {
            Ok(()) => Some(state.finish()),
            Err(builder) => {
                self.error(builder);
                None
            }
        }
    }
}

impl<'db> AppState<'db> {
    fn new(term: Term<'db>, ty: Term<'db>) -> Self {
        Self { term, ty }
    }

    fn consume_arg(&mut self, cx: &mut ElabCtx<'db>, arg: SurfaceArg) -> Result<(), Diag> {
        self.insert_implicit_args(cx, arg.range)?;

        let func_ty = cx.whnf(self.ty);
        let (dom, cod) = ensure_explicit_function_type(cx, func_ty, arg.range)?;

        let arg_term = match arg.expr {
            Some(expr) => {
                let expected = Expected {
                    ty: dom,
                    reason: ExpectedReason::None,
                };
                cx.check(expr, &expected)
            }
            None => cx.poison().0,
        };

        self.term = Term::app(cx.db, self.term, arg_term);
        self.ty = instantiate(cx.db, &cod, arg_term);
        Ok(())
    }

    fn finish(self) -> (Term<'db>, Term<'db>) {
        (self.term, self.ty)
    }

    fn insert_implicits_for_expected(
        &mut self,
        cx: &mut ElabCtx<'db>,
        expected_ty: Term<'db>,
        range: TextRange,
    ) -> Result<(), Diag> {
        loop {
            self.ty = cx.whnf(self.ty);
            if expected_keeps_implicit(cx, self.ty, expected_ty) {
                return Ok(());
            }

            let inserted = self.insert_one_implicit(cx, range)?;
            if !inserted {
                return Ok(());
            }
        }
    }

    fn insert_implicit_args(
        &mut self,
        cx: &mut ElabCtx<'db>,
        range: TextRange,
    ) -> Result<(), Diag> {
        loop {
            self.ty = cx.whnf(self.ty);
            let inserted = self.insert_one_implicit(cx, range)?;
            if !inserted {
                return Ok(());
            }
        }
    }

    fn insert_one_implicit(
        &mut self,
        cx: &mut ElabCtx<'db>,
        range: TextRange,
    ) -> Result<bool, Diag> {
        match self.ty.kind(cx.db) {
            TermKind::Pi(info, dom, cod) if is_ordinary_implicit(*info) => {
                let arg = cx.fresh_mvar(*dom);
                self.term = Term::app(cx.db, self.term, arg);
                self.ty = instantiate(cx.db, cod, arg);
                Ok(true)
            }
            TermKind::Pi(BinderInfo::InstanceImplicit, _, _) => {
                Err(cx.unsupported_instance_implicit(range))
            }
            _ => Ok(false),
        }
    }
}

fn ensure_explicit_function_type<'db>(
    cx: &mut ElabCtx<'db>,
    func_ty: Term<'db>,
    range: TextRange,
) -> Result<(Term<'db>, Term<'db>), Diag> {
    match func_ty.kind(cx.db) {
        TermKind::Pi(BinderInfo::Explicit, param_ty, body_ty) => Ok((*param_ty, *body_ty)),
        TermKind::Pi(info, _, _) if is_ordinary_implicit(*info) => {
            unreachable!("ordinary implicit binders should be inserted before explicit arguments")
        }
        TermKind::Pi(BinderInfo::InstanceImplicit, _, _) => {
            Err(cx.unsupported_instance_implicit(range))
        }
        TermKind::Error(_) => Ok((func_ty, func_ty)),
        _ => Err(cx.expected_function(range, func_ty)),
    }
}

fn expected_keeps_implicit<'db>(
    cx: &ElabCtx<'db>,
    inferred_ty: Term<'db>,
    expected_ty: Term<'db>,
) -> bool {
    let expected_ty = cx.whnf(expected_ty);
    match (inferred_ty.kind(cx.db), expected_ty.kind(cx.db)) {
        (TermKind::Pi(found, _, _), TermKind::Pi(expected, _, _))
            if is_ordinary_implicit(*found) && found == expected =>
        {
            true
        }
        (_, TermKind::MVar(_)) => true,
        _ => false,
    }
}

fn is_ordinary_implicit(info: BinderInfo) -> bool {
    matches!(info, BinderInfo::Implicit | BinderInfo::StrictImplicit)
}

fn flatten_app(expr: &ast::AppExpr) -> (Option<ast::Expr>, Vec<SurfaceArg>) {
    let mut args = Vec::new();
    let head = collect_app(ast::Expr::AppExpr(expr.clone()), &mut args);
    (head, args)
}

fn collect_app(expr: ast::Expr, args: &mut Vec<SurfaceArg>) -> Option<ast::Expr> {
    match expr {
        ast::Expr::AppExpr(app) => {
            let head = match app.func() {
                Some(func) => collect_app(func, args),
                None => None,
            };
            let arg = app.arg();
            let range = arg.as_ref().map_or_else(
                || app.syntax().text_range(),
                |arg| arg.syntax().text_range(),
            );
            args.push(SurfaceArg { expr: arg, range });
            head
        }
        expr => Some(expr),
    }
}
