use ast::traits::AstNode;
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
            None => (self.error_mvar(), self.error_mvar()),
        };

        let mut state = AppState::new(head_term, head_ty);
        for arg in args {
            if !state.consume_arg(self, arg) {
                return (self.error_mvar(), self.error_mvar());
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
        state.insert_implicits_for_expected(self, expected_ty, range)?;
        Some(state.finish())
    }
}

impl<'db> AppState<'db> {
    fn new(term: Term<'db>, ty: Term<'db>) -> Self {
        Self { term, ty }
    }

    fn consume_arg(&mut self, cx: &mut ElabCtx<'db>, arg: SurfaceArg) -> bool {
        if self.insert_implicit_args(cx, arg.range).is_none() {
            return false;
        }

        let func_ty = cx.whnf(self.ty);
        let Some((dom, cod)) = ensure_explicit_function_type(cx, func_ty, arg.range) else {
            return false;
        };

        let arg_term = match arg.expr {
            Some(expr) => {
                let expected = Expected {
                    ty: dom,
                    reason: ExpectedReason::None,
                };
                cx.check(expr, &expected)
            }
            None => cx.error_mvar(),
        };

        self.term = Term::app(cx.db, self.term, arg_term);
        self.ty = cx.instantiate(&cod, arg_term);
        true
    }

    fn finish(self) -> (Term<'db>, Term<'db>) {
        (self.term, self.ty)
    }

    fn insert_implicits_for_expected(
        &mut self,
        cx: &mut ElabCtx<'db>,
        expected_ty: Term<'db>,
        range: TextRange,
    ) -> Option<()> {
        loop {
            self.ty = cx.whnf(self.ty);
            if expected_keeps_implicit(cx, self.ty, expected_ty) {
                return Some(());
            }

            let inserted = self.insert_one_implicit(cx, range)?;
            if !inserted {
                return Some(());
            }
        }
    }

    fn insert_implicit_args(&mut self, cx: &mut ElabCtx<'db>, range: TextRange) -> Option<()> {
        loop {
            self.ty = cx.whnf(self.ty);
            let inserted = self.insert_one_implicit(cx, range)?;
            if !inserted {
                return Some(());
            }
        }
    }

    fn insert_one_implicit(&mut self, cx: &mut ElabCtx<'db>, range: TextRange) -> Option<bool> {
        match self.ty.kind(cx.db) {
            TermKind::Pi(info, dom, cod) if is_ordinary_implicit(*info) => {
                let arg = cx.fresh_mvar(*dom);
                self.term = Term::app(cx.db, self.term, arg);
                self.ty = cx.instantiate(cod, arg);
                Some(true)
            }
            TermKind::Pi(BinderInfo::InstanceImplicit, _, _) => {
                cx.report_unsupported_instance_implicit(range);
                None
            }
            _ => Some(false),
        }
    }
}

fn ensure_explicit_function_type<'db>(
    cx: &mut ElabCtx<'db>,
    func_ty: Term<'db>,
    range: TextRange,
) -> Option<(Term<'db>, Term<'db>)> {
    match func_ty.kind(cx.db) {
        TermKind::Pi(BinderInfo::Explicit, param_ty, body_ty) => Some((*param_ty, *body_ty)),
        TermKind::Pi(info, _, _) if is_ordinary_implicit(*info) => {
            unreachable!("ordinary implicit binders should be inserted before explicit arguments")
        }
        TermKind::Pi(BinderInfo::InstanceImplicit, _, _) => {
            cx.report_unsupported_instance_implicit(range);
            None
        }
        _ => {
            cx.report_expected_function(range, func_ty);
            None
        }
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

impl<'db> ElabCtx<'db> {
    fn report_expected_function(&mut self, range: TextRange, found: Term<'db>) {
        let found_txt = found.debug(self.db).to_string();
        let diag = self
            .mk_error(range, &format!("expected a function, found `{found_txt}`"))
            .with_primary_message(format!(
                "this has type `{found_txt}`, which is not a function"
            ))
            .build();
        self.diagnostic(diag);
    }

    fn report_unsupported_instance_implicit(&mut self, range: TextRange) {
        let diag = self
            .mk_error(range, "instance implicit arguments are not supported yet")
            .with_primary_message(
                "this call needs instance search, but instance resolution is not implemented",
            )
            .build();
        self.diagnostic(diag);
    }
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
