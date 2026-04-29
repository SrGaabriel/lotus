pub mod builder;
pub mod files;
pub mod render;

use text_size::TextRange;

use crate::files::FileId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Lint,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub file: FileId,
    pub range: TextRange,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<&'static str>,
    pub message: String,
    pub primary: Label,
    pub secondaries: Vec<Label>,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
}
