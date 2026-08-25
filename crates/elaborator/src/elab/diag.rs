use diagnostics::{
    Diagnostic,
    Label,
    builder::Diag,
};
use salsa::Accumulator;
use text_size::TextRange;

use crate::{
    core::{
        Term,
        TermKind,
        error::ErrorToken,
    },
    elab::{
        ctx::ElabCtx,
        expected::{
            Expected,
            ExpectedReason,
        },
        unify::UnifyError,
    },
    env::namespace::best_match,
    ids::{
        Qualified,
        Symbol,
        Unique,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Frame<'db> {
    DefBody { name: Symbol<'db> },
}

impl<'db> ElabCtx<'db> {
    pub fn unresolved_name(
        &mut self,
        qualified: &Qualified<'db>,
        range: TextRange,
    ) -> (Term<'db>, Term<'db>) {
        let mut diag = self.mk_error(
            range,
            &format!("unresolved name '{}'", qualified.to_string(self.db)),
        );

        if let Some(suggestion) = self.suggest(qualified) {
            diag = diag.with_help(format!("did you mean `{}`", suggestion.to_string(self.db)));
        }

        self.error(diag)
    }

    fn suggest(&self, qualified: &Qualified<'db>) -> Option<Qualified<'db>> {
        if qualified.path.is_empty() {
            let locals = self.lctx.iter().filter_map(|b| b.name);
            let decls = self.namespace.decls(self.db).keys().copied();
            let member = best_match(self.db, qualified.member, locals.chain(decls))?;
            return (member != qualified.member).then(|| Qualified::unqualified(member));
        }
        self.namespace.similar(self.db, qualified)
    }

    pub fn mismatch(
        &mut self,
        range: TextRange,
        found: Term<'db>,
        expected: &Expected<'db>,
        err: &UnifyError<'db>,
    ) -> Diag {
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

        builder
    }

    fn diagnostic(&self, diag: Diagnostic) {
        diag.accumulate(self.db);
    }

    pub fn with_frame<R>(&mut self, frame: Frame<'db>, body: impl FnOnce(&mut Self) -> R) -> R {
        self.frames.push(frame);
        let result = body(self);
        let popped = self.frames.pop();
        debug_assert!(popped.is_some_and(|p| p == frame));
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

    pub fn mk_error(&mut self, range: TextRange, message: &str) -> Diag {
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

    pub fn error(&self, builder: Diag) -> (Term<'db>, Term<'db>) {
        self.diagnostic(builder.build());
        let poison = Term::error(self.db, ErrorToken::new());
        (poison, poison)
    }

    pub fn emit(&self, builder: Diag) {
        self.diagnostic(builder.build());
    }

    pub fn poison(&self) -> (Term<'db>, Term<'db>) {
        let poison = Term::error(self.db, ErrorToken::new());
        (poison, poison)
    }

    pub fn error_term(&self) -> Term<'db> {
        self.poison().0
    }

    pub fn report_unsolved_mvars(&mut self, term: Term<'db>, range: TextRange) {
        if term.has_error(self.db) {
            return;
        }
        let mut unsolved = Vec::new();
        self.collect_unsolved(term, &mut unsolved);
        if unsolved.is_empty() {
            return;
        }
        let builder = self
            .mk_error(
                range,
                "could not infer all implicit values in this declaration",
            )
            .with_note(format!(
                "{} metavariable(s) could not be solved",
                unsolved.len()
            ));
        self.emit(builder);
    }

    fn collect_unsolved(&self, term: Term<'db>, out: &mut Vec<Unique>) {
        match term.kind(self.db) {
            TermKind::MVar(u) => {
                if !self.mctx.is_solved(*u) && !out.contains(u) {
                    out.push(*u);
                }
            }
            TermKind::App(f, x) => {
                self.collect_unsolved(*f, out);
                self.collect_unsolved(*x, out);
            }
            TermKind::Lam(_, ty, body)
            | TermKind::Pi(_, ty, body)
            | TermKind::Sigma(_, ty, body) => {
                self.collect_unsolved(*ty, out);
                self.collect_unsolved(*body, out);
            }
            TermKind::Let(ty, value, body) => {
                self.collect_unsolved(*ty, out);
                self.collect_unsolved(*value, out);
                self.collect_unsolved(*body, out);
            }
            TermKind::BVar(_)
            | TermKind::FVar(_)
            | TermKind::Const(_)
            | TermKind::Sort(_)
            | TermKind::Lit(_)
            | TermKind::Error(_) => {}
        }
    }

    pub fn expected_function(&mut self, range: TextRange, found: Term<'db>) -> Diag {
        let found_txt = found.debug(self.db).to_string();
        self.mk_error(range, &format!("expected a function, found `{found_txt}`"))
            .with_primary_message(format!(
                "this has type `{found_txt}`, which is not a function"
            ))
    }

    pub fn unsupported_instance_implicit(&mut self, range: TextRange) -> Diag {
        self.mk_error(range, "instance implicit arguments are not supported yet")
            .with_primary_message(
                "this call needs instance search, but instance resolution is not implemented",
            )
    }
}
