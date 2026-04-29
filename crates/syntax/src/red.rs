use cstree::Syntax;

use crate::lexer::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Syntax)]
#[repr(u32)]
pub enum SyntaxKind {
    // === Tokens ===
    Whitespace,
    LineComment,
    BlockComment,

    Identifier,
    OpIdentifier,

    NumberLit,
    StringLit,

    #[static_text("(")]
    LParen,
    #[static_text(")")]
    RParen,
    #[static_text("{")]
    LBrace,
    #[static_text("}")]
    RBrace,
    #[static_text("[")]
    LBracket,
    #[static_text("]")]
    RBracket,

    #[static_text(";")]
    Semicolon,
    #[static_text(",")]
    Comma,
    #[static_text(".")]
    Dot,

    Unknown,

    DefKw,

    // === Nodes ===
    Root,
    SourceFile,
    DefDecl,
    Name,
    Expr,
    Error,
}

impl SyntaxKind {
    pub fn from_token(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Whitespace => Self::Whitespace,
            TokenKind::LineComment => Self::LineComment,
            TokenKind::BlockComment { .. } => Self::BlockComment,
            TokenKind::Identifier => Self::Identifier,
            TokenKind::OpIdentifier => Self::OpIdentifier,
            TokenKind::Number => Self::NumberLit,
            TokenKind::LParen => Self::LParen,
            TokenKind::RParen => Self::RParen,
            TokenKind::LBrace => Self::LBrace,
            TokenKind::RBrace => Self::RBrace,
            TokenKind::LBracket => Self::LBracket,
            TokenKind::RBracket => Self::RBracket,
            TokenKind::Semicolon => Self::Semicolon,
            TokenKind::Comma => Self::Comma,
            TokenKind::Dot => Self::Dot,
            TokenKind::Unknown => Self::Unknown,
            TokenKind::Eof => unreachable!("Eof never reaches the syntax tree"),
        }
    }

    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            Self::Whitespace | Self::LineComment | Self::BlockComment
        )
    }
}
