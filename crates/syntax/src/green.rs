use cstree::Syntax;
use cstree::build::{Checkpoint, GreenNodeBuilder};
use cstree::green::GreenNode;

use crate::{
    lexer::{Lexer, Token, TokenKind},
    red::SyntaxKind,
};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub offset: u32,
}

pub struct Parsed {
    pub green: GreenNode,
    pub errors: Vec<ParseError>,
}

pub struct Parser<'input> {
    tokens: Vec<Token<'input>>,
    cursor: usize,
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    errors: Vec<ParseError>,
}

impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            tokens: Lexer::new(input).collect(),
            cursor: 0,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Parsed {
        self.builder.start_node(SyntaxKind::Root);
        self.eat_trivia();
        self.builder.finish_node();

        let (green, _interner) = self.builder.finish();
        Parsed {
            green,
            errors: self.errors,
        }
    }

    pub fn current(&self) -> TokenKind {
        self.nth(0)
    }

    pub fn nth(&self, n: usize) -> TokenKind {
        self.peek_nth(n).map_or(TokenKind::Eof, |t| t.kind)
    }

    pub fn at(&self, kind: TokenKind) -> bool {
        self.current() == kind
    }

    pub fn at_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.current())
    }

    fn peek_nth(&self, n: usize) -> Option<Token<'input>> {
        let mut idx = self.cursor;
        let mut remaining = n;
        loop {
            let tok = *self.tokens.get(idx)?;
            if SyntaxKind::from_token(tok.kind).is_trivia() {
                idx += 1;
                continue;
            }
            if remaining == 0 {
                return Some(tok);
            }
            remaining -= 1;
            idx += 1;
        }
    }

    fn eat_trivia(&mut self) {
        while let Some(tok) = self.tokens.get(self.cursor).copied() {
            let kind = SyntaxKind::from_token(tok.kind);
            if !kind.is_trivia() {
                break;
            }
            self.builder.token(kind, tok.text);
            self.cursor += 1;
        }
    }

    pub fn bump(&mut self) {
        self.eat_trivia();
        let Some(tok) = self.tokens.get(self.cursor).copied() else {
            return;
        };
        let kind = SyntaxKind::from_token(tok.kind);
        if kind.static_text().is_some() {
            self.builder.static_token(kind);
        } else {
            self.builder.token(kind, tok.text);
        }
        self.cursor += 1;
    }

    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: TokenKind) {
        if !self.eat(kind) {
            self.error(format!("expected {:?}, found {:?}", kind, self.current()));
        }
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.eat_trivia();
        self.builder.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    pub fn checkpoint(&mut self) -> Checkpoint {
        self.eat_trivia();
        self.builder.checkpoint()
    }

    pub fn start_node_at(&mut self, cp: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(cp, kind);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        let offset = self
            .tokens
            .get(self.cursor)
            .map(|t| t.offset)
            .unwrap_or_default();
        self.errors.push(ParseError {
            message: message.into(),
            offset,
        });
    }

    pub fn error_and_bump(&mut self, message: impl Into<String>) {
        self.start_node(SyntaxKind::Error);
        self.error(message);
        self.bump();
        self.finish_node();
    }
}
