use crate::{
    Diagnostic,
    Label,
    Severity,
};
use db::SourceFile;
use text_size::TextRange;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct DiagInner {
    severity: Severity,
    code: Option<&'static str>,
    message: String,
    primary: Label,
    secondaries: Vec<Label>,
    notes: Vec<String>,
    helps: Vec<String>,
}

#[derive(Debug)]
#[must_use = "a Diag must be emitted with build() or explicitly cancel()ed"]
pub struct Diag {
    inner: Option<Box<DiagInner>>,
}

impl Diag {
    fn inner_mut(&mut self) -> &mut DiagInner {
        self.inner.as_mut().expect("Diag already consumed")
    }

    pub fn with_code(mut self, code: &'static str) -> Self {
        self.inner_mut().code = Some(code);
        self
    }

    pub fn with_secondary(
        mut self,
        file: SourceFile,
        range: TextRange,
        message: Option<String>,
    ) -> Self {
        self.inner_mut().secondaries.push(Label {
            file,
            range,
            message,
        });
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.inner_mut().notes.push(note);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.inner_mut().helps.push(help);
        self
    }

    pub fn with_primary_message(mut self, message: impl Into<String>) -> Self {
        self.inner_mut().primary.message = Some(message.into());
        self
    }

    pub fn with_secondary_label(mut self, label: Label) -> Self {
        self.inner_mut().secondaries.push(label);
        self
    }

    pub fn build(mut self) -> Diagnostic {
        let inner = self.inner.take().expect("Diag already consumed");
        Diagnostic {
            severity: inner.severity,
            code: inner.code,
            message: inner.message,
            primary: inner.primary,
            secondaries: inner.secondaries,
            notes: inner.notes,
            helps: inner.helps,
        }
    }

    pub fn cancel(mut self) {
        self.inner.take();
    }
}

impl Drop for Diag {
    fn drop(&mut self) {
        if let Some(inner) = &self.inner
            && cfg!(debug_assertions)
            && !std::thread::panicking()
        {
            panic!(
                "diagnostic dropped without build() or cancel(): {:?}",
                inner.message
            );
        }
    }
}

impl Diagnostic {
    pub fn warning(message: &str, file: SourceFile, range: TextRange) -> Diag {
        Self::builder(Severity::Warning, message, file, range)
    }

    pub fn error(message: &str, file: SourceFile, range: TextRange) -> Diag {
        Self::builder(Severity::Error, message, file, range)
    }

    pub fn lint(message: &str, file: SourceFile, range: TextRange) -> Diag {
        Self::builder(Severity::Lint, message, file, range)
    }

    pub fn builder(severity: Severity, message: &str, file: SourceFile, range: TextRange) -> Diag {
        Diag {
            inner: Some(Box::new(DiagInner {
                severity,
                code: None,
                message: message.to_string(),
                primary: Label {
                    file,
                    range,
                    message: None,
                },
                secondaries: Vec::new(),
                notes: Vec::new(),
                helps: Vec::new(),
            })),
        }
    }
}
