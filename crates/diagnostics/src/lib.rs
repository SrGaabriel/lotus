pub mod builder;
pub mod files;
pub mod render;

use db::SourceFile;
use text_size::TextRange;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
    Lint,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub file: SourceFile,
    pub range: TextRange,
    pub message: Option<String>,
}

#[salsa::accumulator]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<&'static str>,
    pub message: String,
    pub primary: Label,
    pub secondaries: Vec<Label>,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
}
