pub mod generated;
pub mod traits;

use std::marker::PhantomData;

use diagnostics::{Diagnostic, files::FileId};
pub use generated::*;
use syntax::ResolvedNode;

use crate::traits::AstNode;

#[derive(Debug, Clone)]
pub struct Parse<T> {
    syntax: ResolvedNode,
    diagnostics: Vec<Diagnostic>,
    _ty: PhantomData<T>,
}

impl<T: AstNode> Parse<T> {
    pub fn tree(&self) -> T {
        T::cast(self.syntax.clone()).expect("root kind mismatch")
    }

    pub fn syntax_node(&self) -> &ResolvedNode {
        &self.syntax
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn ok(self) -> Result<T, Vec<Diagnostic>> {
        if self.diagnostics().is_empty() {
            Ok(self.tree())
        } else {
            Err(self.diagnostics)
        }
    }
}

pub fn parse(file: FileId, text: &str) -> Parse<SourceFile> {
    let parsed = syntax::parse(file, text);
    let diagnostics = parsed.diagnostics.clone();
    Parse {
        syntax: parsed.into_node(),
        diagnostics,
        _ty: PhantomData,
    }
}
