use crate::{
    kind::SyntaxKind,
    parser::{
        Parsed,
        Parser,
    },
};
use cstree::util::NodeOrToken;
use db::SourceFile;

pub mod kind;
pub mod lexer;
pub mod parser;

pub type SyntaxNode = cstree::syntax::SyntaxNode<SyntaxKind>;
pub type SyntaxToken = cstree::syntax::SyntaxToken<SyntaxKind>;
pub type ResolvedNode = cstree::syntax::ResolvedNode<SyntaxKind>;
pub type ResolvedToken = cstree::syntax::ResolvedToken<SyntaxKind>;
pub type SyntaxElement = NodeOrToken<SyntaxNode, SyntaxToken>;

pub fn parse(file: SourceFile, text: &str) -> Parsed {
    Parser::new(file, text).parse()
}
