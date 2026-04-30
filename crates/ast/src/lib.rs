pub mod generated;
pub mod traits;

use crate::traits::AstNode;
use db::{
    SourceDatabase,
    SourceFile as DbFile,
};
use diagnostics::Diagnostic;
pub use generated::*;
use salsa::Accumulator;
use std::marker::PhantomData;
use syntax::ResolvedNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parse<T> {
    syntax: ResolvedNode,
    _ty: PhantomData<fn() -> T>,
}

impl<T: AstNode> Parse<T> {
    pub fn tree(&self) -> T {
        T::cast(self.syntax.clone()).expect("root kind mismatch")
    }

    pub fn syntax_node(&self) -> &ResolvedNode {
        &self.syntax
    }
}

pub fn parse(file: DbFile, text: &str) -> (Parse<SourceFile>, Vec<Diagnostic>) {
    let parsed = syntax::parse(file, text);
    let diagnostics = parsed.diagnostics.clone();
    let parse = Parse {
        syntax: parsed.into_node(),
        _ty: PhantomData,
    };
    (parse, diagnostics)
}

#[salsa::db]
pub trait AstDatabase: SourceDatabase {}

#[salsa::db]
impl<DB: SourceDatabase> AstDatabase for DB {}

#[salsa::tracked(returns(ref))]
pub fn parse_file(db: &dyn AstDatabase, file: DbFile) -> Parse<SourceFile> {
    let (parse, diagnostics) = parse(file, file.text(db));
    for diag in diagnostics {
        diag.accumulate(db);
    }
    parse
}
