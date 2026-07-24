use literals::{
    Literal,
    NumberSuffix,
    NumberValue,
};
use text_size::TextRange;
use thiserror::Error;

use crate::{
    ElabDb,
    core::{
        Level,
        LevelKind,
        Term,
        TermKind,
    },
    elab::{
        ctx::ElabCtx,
        expected::{
            Expected,
            ExpectedReason,
        },
        subst::{
            abstract_fvar,
            instantiate,
        },
        unify::UnifyError,
    },
    env::lang_items::LangItem,
    ids::Unique,
};

#[derive(Debug, Clone, Error)]
pub enum InferTermError<'db> {
    #[error("unbound de Bruijn variable #{0}")]
    UnboundBVar(usize),

    #[error("unknown free variable {0:?}")]
    UnknownFVar(Unique),

    #[error("unknown metavariable {0:?}")]
    UnknownMVar(Unique),

    #[error("expected a function")]
    ExpectedFunction { term: Term<'db>, found: Term<'db> },

    #[error("expected a type")]
    ExpectedType { term: Term<'db>, found: Term<'db> },

    #[error("term has an unexpected type")]
    TypeMismatch {
        term: Term<'db>,
        found: Term<'db>,
        expected: Term<'db>,
        cause: UnifyError<'db>,
    },

    #[error("missing language item `{0}`")]
    MissingLangItem(LangItem),
}

impl<'db> ElabCtx<'db> {
    pub fn infer_term(&mut self, term: Term<'db>) -> Result<Term<'db>, InferTermError<'db>> {
        match term.kind(self.db) {
            TermKind::BVar(index) => Err(InferTermError::UnboundBVar(*index)),

            TermKind::FVar(id) => self
                .lctx
                .find_by_unique(*id)
                .map(|binder| binder.ty)
                .ok_or(InferTermError::UnknownFVar(*id)),

            TermKind::MVar(id) => self
                .mctx
                .get_meta(*id)
                .map(|meta| meta.ty)
                .ok_or(InferTermError::UnknownMVar(*id)),

            TermKind::Const(id) => Ok(self.db.signature(*id).ty),

            TermKind::Sort(level) => Ok(Term::sort(self.db, Level::succ(self.db, *level))),

            TermKind::App(function, argument) => {
                let function_ty = self.infer_term(*function)?;
                let function_ty = self.whnf(function_ty);
                let TermKind::Pi(_, parameter_ty, result_ty) = function_ty.kind(self.db) else {
                    return Err(InferTermError::ExpectedFunction {
                        term: *function,
                        found: function_ty,
                    });
                };

                let argument_ty = self.infer_term(*argument)?;
                self.ensure_def_eq(*argument, argument_ty, *parameter_ty)?;
                Ok(instantiate(self.db, result_ty, *argument))
            }

            TermKind::Lam(info, parameter_ty, body) => {
                self.infer_sort(*parameter_ty)?;
                let (fvar, body_ty) = self.infer_under_binder(*info, *parameter_ty, *body)?;
                let body_ty = abstract_fvar(self.db, &body_ty, fvar);
                Ok(Term::pi(self.db, *info, *parameter_ty, body_ty))
            }

            TermKind::Pi(info, parameter_ty, body) => {
                let parameter_level = self.infer_sort(*parameter_ty)?;
                let (_, body_level) = self.infer_sort_under_binder(*info, *parameter_ty, *body)?;
                let level = Level::new(self.db, LevelKind::IMax(parameter_level, body_level));
                Ok(Term::sort(self.db, level))
            }

            TermKind::Sigma(info, parameter_ty, body) => {
                let parameter_level = self.infer_sort(*parameter_ty)?;
                let (_, body_level) = self.infer_sort_under_binder(*info, *parameter_ty, *body)?;
                let level = Level::new(self.db, LevelKind::Max(parameter_level, body_level));
                Ok(Term::sort(self.db, level))
            }

            TermKind::Let(ty, value, body) => {
                self.infer_sort(*ty)?;
                let value_ty = self.infer_term(*value)?;
                self.ensure_def_eq(*value, value_ty, *ty)?;

                let (fvar, body_ty) =
                    self.infer_under_binder(crate::core::BinderInfo::Explicit, *ty, *body)?;
                let body_ty = abstract_fvar(self.db, &body_ty, fvar);
                Ok(instantiate(self.db, &body_ty, *value))
            }

            TermKind::Lit(literal) => self.infer_literal_type(literal),
        }
    }

    pub fn infer_term_with_diagnostics(
        &mut self,
        term: Term<'db>,
        span: TextRange,
    ) -> Option<Term<'db>> {
        match self.infer_term(term) {
            Ok(ty) => Some(ty),
            Err(InferTermError::TypeMismatch {
                found,
                expected,
                cause,
                ..
            }) => {
                let expected = Expected::new(expected, ExpectedReason::None);
                self.report_mismatch(span, found, &expected, &cause);
                None
            }
            Err(InferTermError::ExpectedFunction { term, found }) => {
                let message = format!(
                    "expected `{}` to be a function, but its type is `{}`",
                    term.debug(self.db),
                    found.debug(self.db),
                );
                let diagnostic = self
                    .mk_error(span, "expected a function")
                    .with_primary_message(message)
                    .build();
                self.diagnostic(diagnostic);
                None
            }
            Err(InferTermError::ExpectedType { term, found }) => {
                let message = format!(
                    "expected `{}` to be a type, but its type is `{}`",
                    term.debug(self.db),
                    found.debug(self.db),
                );
                let diagnostic = self
                    .mk_error(span, "expected a type")
                    .with_primary_message(message)
                    .build();
                self.diagnostic(diagnostic);
                None
            }
            Err(error) => {
                let note = format!("while inferring `{}`", term.debug(self.db));
                let message = error.to_string();
                let diagnostic = self
                    .mk_error(span, "could not infer the type of this term")
                    .with_primary_message(message)
                    .with_note(note)
                    .build();
                self.diagnostic(diagnostic);
                None
            }
        }
    }

    fn infer_sort(&mut self, term: Term<'db>) -> Result<Level<'db>, InferTermError<'db>> {
        let ty = self.infer_term(term)?;
        let ty = self.whnf(ty);
        match ty.kind(self.db) {
            TermKind::Sort(level) => Ok(*level),
            _ => Err(InferTermError::ExpectedType { term, found: ty }),
        }
    }

    fn infer_under_binder(
        &mut self,
        info: crate::core::BinderInfo,
        parameter_ty: Term<'db>,
        body: Term<'db>,
    ) -> Result<(Unique, Term<'db>), InferTermError<'db>> {
        let level = self.lctx.level();
        let fvar = self.fresh_fvar(None, parameter_ty, info, TextRange::default(), None);
        let opened_body = instantiate(self.db, &body, Term::fvar(self.db, fvar));
        let result = self.infer_term(opened_body);
        self.lctx.pop_to(level);
        result.map(|ty| (fvar, ty))
    }

    fn infer_sort_under_binder(
        &mut self,
        info: crate::core::BinderInfo,
        parameter_ty: Term<'db>,
        body: Term<'db>,
    ) -> Result<(Unique, Level<'db>), InferTermError<'db>> {
        let level = self.lctx.level();
        let fvar = self.fresh_fvar(None, parameter_ty, info, TextRange::default(), None);
        let opened_body = instantiate(self.db, &body, Term::fvar(self.db, fvar));
        let result = self.infer_sort(opened_body);
        self.lctx.pop_to(level);
        result.map(|sort| (fvar, sort))
    }

    fn ensure_def_eq(
        &mut self,
        term: Term<'db>,
        found: Term<'db>,
        expected: Term<'db>,
    ) -> Result<(), InferTermError<'db>> {
        self.unify(found, expected)
            .map_err(|cause| InferTermError::TypeMismatch {
                term,
                found,
                expected,
                cause,
            })
    }

    fn infer_literal_type(&self, literal: &Literal) -> Result<Term<'db>, InferTermError<'db>> {
        let lang_item = match literal {
            Literal::Text(_) => LangItem::Str,
            Literal::Numeric(number) => match number.suffix {
                Some(NumberSuffix::I8) => LangItem::Int8,
                Some(NumberSuffix::I16) => LangItem::Int16,
                Some(NumberSuffix::I32) => LangItem::Int32,
                Some(NumberSuffix::I64) => LangItem::Int64,
                Some(NumberSuffix::U8) => LangItem::UInt8,
                Some(NumberSuffix::U16) => LangItem::UInt16,
                Some(NumberSuffix::U32) => LangItem::UInt32,
                Some(NumberSuffix::U64) => LangItem::UInt64,
                Some(NumberSuffix::F32) => LangItem::Float32,
                Some(NumberSuffix::F64) => LangItem::Float64,
                None => match number.value {
                    NumberValue::Integer(_) => LangItem::Int32,
                    NumberValue::Float(_) => LangItem::Float64,
                },
            },
        };

        self.db
            .lang_items(self.current_decl.file(self.db))
            .get(&lang_item)
            .copied()
            .map(|item| Term::constant(self.db, item))
            .ok_or(InferTermError::MissingLangItem(lang_item))
    }
}
