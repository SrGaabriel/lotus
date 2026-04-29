use cstree::Syntax;
use cstree::build::{Checkpoint, GreenNodeBuilder};
use cstree::green::GreenNode;
use cstree::interning::TokenInterner;
use cstree::text::TextRange;
use diagnostics::Diagnostic;
use structure::FileId;
use text_size::TextSize;

use crate::parser::parse_source_file;
use crate::{ResolvedNode, SyntaxNode};
use crate::{
    lexer::{Lexer, Token, TokenKind},
    red::SyntaxKind,
};

pub struct Parsed {
    pub file: FileId,
    pub green: GreenNode,
    pub diagnostics: Vec<Diagnostic>,
    pub interner: TokenInterner,
}

impl Parsed {
    pub fn into_node(self) -> ResolvedNode {
        SyntaxNode::new_root_with_resolver(self.green, self.interner)
    }
}

pub struct Parser<'input> {
    file: FileId,
    tokens: Vec<Token<'input>>,
    cursor: usize,
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    diagnostics: Vec<Diagnostic>,
}

impl<'input> Parser<'input> {
    pub fn new(file: FileId, input: &'input str) -> Self {
        Self {
            file,
            tokens: Lexer::new(input).collect(),
            cursor: 0,
            builder: GreenNodeBuilder::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Parsed {
        self.builder.start_node(SyntaxKind::Root);
        parse_source_file(&mut self);
        self.builder.finish_node();

        let (green, cache) = self.builder.finish();
        let interner = cache.unwrap().into_interner().unwrap();
        Parsed {
            file: self.file,
            green,
            diagnostics: self.diagnostics,
            interner,
        }
    }

    pub fn current(&self) -> TokenKind {
        self.nth(0)
    }

    pub fn current_text(&self) -> &str {
        self.peek_nth(0).map_or("", |t| t.text)
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
            self.error(&format!("expected {:?}, found {:?}", kind, self.current()));
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

    pub fn diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn error(&mut self, message: &str) {
        let range = self.current_range();
        self.diagnostics
            .push(Diagnostic::error(message, self.file, range).build());
    }

    pub fn error_and_bump(&mut self, message: &str) {
        self.start_node(SyntaxKind::Error);
        self.error(message);
        self.bump();
        self.finish_node();
    }

    pub fn current_range(&self) -> TextRange {
        if let Some(tok) = self.peek_nth(0) {
            let start = TextSize::new(tok.offset);
            let end = start + TextSize::new(tok.text.len() as u32);
            TextRange::new(start, end)
        } else {
            let pos = self
                .tokens
                .last()
                .map_or(0, |t| t.offset + t.text.len() as u32);
            TextRange::empty(TextSize::new(pos))
        }
    }

    pub fn prev_range(&self) -> Option<TextRange> {
        self.tokens[..self.cursor]
            .iter()
            .rev()
            .find(|t| !SyntaxKind::from_token(t.kind).is_trivia())
            .map(|t| {
                let start = TextSize::new(t.offset);
                let end = start + TextSize::new(t.text.len() as u32);
                TextRange::new(start, end)
            })
    }
}
