use text_size::TextRange;

use crate::{Diagnostic, Label, Severity, files::FileId};

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
        file: FileId,
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
    pub fn warning(message: &str, file: FileId, range: TextRange) -> DiagnosticBuilder {
        DiagnosticBuilder {
            severity: Severity::Warning,
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

    pub fn error(message: &str, file: FileId, range: TextRange) -> DiagnosticBuilder {
        DiagnosticBuilder {
            severity: Severity::Error,
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

    pub fn lint(message: &str, file: FileId, range: TextRange) -> DiagnosticBuilder {
        DiagnosticBuilder {
            severity: Severity::Lint,
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

pub struct DiagnosticFrame<'a> {
    pub message: String,
    pub code: Option<&'a str>,
    pub help: Option<&'a str>,
    pub note: Option<&'a str>,
}

impl<'a> DiagnosticFrame<'a> {
    pub fn new(message: String) -> Self {
        Self {
            message,
            code: None,
            help: None,
            note: None,
        }
    }

    pub fn with_code(mut self, code: &'a str) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_help(mut self, help: &'a str) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_note(mut self, note: &'a str) -> Self {
        self.note = Some(note);
        self
    }
}
