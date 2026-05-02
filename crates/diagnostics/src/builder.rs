use crate::{
    Diagnostic,
    Label,
    Severity,
};
use db::SourceFile;
use text_size::TextRange;

#[macro_export]
macro_rules! EnrichTy {
    () => {
        impl FnOnce(diagnostics::builder::DiagnosticBuilder) -> diagnostics::builder::DiagnosticBuilder
    };
}

pub fn conserve() -> impl FnOnce(DiagnosticBuilder) -> DiagnosticBuilder {
    |b| b
}

pub struct DiagnosticBuilder {
    pub severity: Severity,
    pub code: Option<&'static str>,
    pub message: String,
    pub primary: Label,
    pub secondaries: Vec<Label>,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
}

impl DiagnosticBuilder {
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_secondary(
        mut self,
        file: SourceFile,
        range: TextRange,
        message: Option<String>,
    ) -> Self {
        self.secondaries.push(Label {
            file,
            range,
            message,
        });
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.helps.push(help);
        self
    }

    pub fn with_primary_message(mut self, message: impl Into<String>) -> Self {
        self.primary.message = Some(message.into());
        self
    }

    pub fn with_secondary_label(mut self, label: Label) -> Self {
        self.secondaries.push(label);
        self
    }

    pub fn build(self) -> Diagnostic {
        Diagnostic {
            severity: self.severity,
            code: self.code,
            message: self.message,
            primary: self.primary,
            secondaries: self.secondaries,
            notes: self.notes,
            helps: self.helps,
        }
    }
}

impl Diagnostic {
    pub fn warning(message: &str, file: SourceFile, range: TextRange) -> DiagnosticBuilder {
        Self::builder(Severity::Warning, message, file, range)
    }

    pub fn error(message: &str, file: SourceFile, range: TextRange) -> DiagnosticBuilder {
        Self::builder(Severity::Error, message, file, range)
    }

    pub fn lint(message: &str, file: SourceFile, range: TextRange) -> DiagnosticBuilder {
        Self::builder(Severity::Lint, message, file, range)
    }

    pub fn builder(
        severity: Severity,
        message: &str,
        file: SourceFile,
        range: TextRange,
    ) -> DiagnosticBuilder {
        DiagnosticBuilder {
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
        }
    }
}
