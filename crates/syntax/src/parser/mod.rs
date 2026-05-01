pub mod marker;
pub mod root;
pub mod token_set;
pub mod expr;

use crate::{
    ResolvedNode,
    SyntaxNode,
    kind::SyntaxKind,
    lexer::{
        Lexer,
        Token,
        TokenKind,
    },
    parser::{
        marker::Marker,
        token_set::TokenSet,
    },
};
use cstree::{
    Syntax,
    build::{
        Checkpoint,
        GreenNodeBuilder,
    },
    green::GreenNode,
    interning::TokenInterner,
    text::TextRange,
};
use db::SourceFile;
use diagnostics::{
    Diagnostic,
    EnrichTy,
    Label,
    Severity,
    builder::DiagnosticBuilder,
};
use text_size::TextSize;

pub struct Parsed {
    pub file: SourceFile,
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
    file: SourceFile,
    tokens: Vec<Token<'input>>,
    cursor: usize,
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    diagnostics: Vec<Diagnostic>,
    expected: Vec<TokenKind>,
}

impl<'input> Parser<'input> {
    pub fn new(file: SourceFile, input: &'input str) -> Self {
        Self {
            file,
            tokens: Lexer::new(input).collect(),
            cursor: 0,
            builder: GreenNodeBuilder::new(),
            diagnostics: Vec::new(),
            expected: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Parsed {
        self.builder.start_node(SyntaxKind::Root);
        self.parse_source_file();
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

    pub fn label(&self, range: TextRange, msg: impl Into<String>) -> Label {
        Label {
            file: self.file,
            range,
            message: Some(msg.into()),
        }
    }

    pub fn current(&self) -> TokenKind {
        self.nth(0)
    }

    pub fn try_current(&self) -> Option<TokenKind> {
        self.peek_nth(0).map(|t| t.kind)
    }

    pub fn try_current_text(&self) -> Option<&str> {
        self.peek_nth(0).map(|t| t.text)
    }

    pub fn current_text(&self) -> &str {
        self.peek_nth(0).map_or("", |t| t.text)
    }

    pub fn nth(&self, n: usize) -> TokenKind {
        self.peek_nth(n).map_or(TokenKind::Eof, |t| t.kind)
    }

    pub fn at(&mut self, kind: TokenKind) -> bool {
        self.push_expected(kind);
        self.current() == kind
    }

    pub fn at_any(&mut self, kinds: &[TokenKind]) -> bool {
        for &k in kinds {
            self.push_expected(k);
        }
        kinds.contains(&self.current())
    }

    pub fn at_ts(&self, ts: TokenSet) -> bool {
        ts.contains(self.current())
    }

    fn push_expected(&mut self, kind: TokenKind) {
        if !self.expected.contains(&kind) {
            self.expected.push(kind);
        }
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
        self.expected.clear();
    }

    pub fn bump_remap(&mut self, kind: SyntaxKind) {
        self.eat_trivia();
        let Some(tok) = self.tokens.get(self.cursor).copied() else {
            return;
        };
        if kind.static_text().is_some() {
            self.builder.static_token(kind);
        } else {
            self.builder.token(kind, tok.text);
        }
        self.cursor += 1;
        self.expected.clear();
    }

    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn expect_recover(&mut self, kind: TokenKind, recovery: TokenSet) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.err_recover(recovery);
        false
    }

    pub fn err_recover(&mut self, recovery: TokenSet) {
        let message = self.expected_message();
        if self.at(TokenKind::Eof) || self.at_ts(recovery) {
            self.error(&message);
            return;
        }
        self.start_node(SyntaxKind::Error);
        self.error(&message);
        self.bump();
        self.finish_node();
    }

    pub fn error_expected(&mut self, what: &str, recovery: TokenSet) {
        self.error_expected_with(what, recovery, |b| b);
    }

    pub fn error_expected_with(&mut self, what: &str, recovery: TokenSet, enrich: EnrichTy!()) {
        let found = self.current();
        let range = self.current_range();
        let msg = format!("expected {what}, found {found:?}");
        let mut builder = Diagnostic::builder(Severity::Error, &msg, self.file, range);
        builder.primary.message = Some(format!("this is not a valid {what}"));
        let diag = enrich(builder).build();
        self.expected.clear();

        if self.at(TokenKind::Eof) || self.at_ts(recovery) {
            self.diagnostics.push(diag);
            return;
        }
        self.start_node(SyntaxKind::Error);
        self.diagnostics.push(diag);
        self.bump();
        self.finish_node();
    }

    pub fn recover_until(&mut self, recovery: TokenSet) -> bool {
        if self.at(TokenKind::Eof) || self.at_ts(recovery) {
            return false;
        }
        self.start_node(SyntaxKind::Error);
        while !self.at(TokenKind::Eof) && !self.at_ts(recovery) {
            self.bump();
        }
        self.finish_node();
        true
    }

    pub fn start(&mut self) -> Marker {
        Marker::new(self.checkpoint())
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

    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind);
    }

    pub fn diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn diag(&self, severity: Severity, message: &str) -> DiagnosticBuilder {
        Diagnostic::builder(severity, message, self.file, self.current_range())
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

    fn expected_message(&mut self) -> String {
        let found = self.current();
        let mut expected = std::mem::take(&mut self.expected);
        if expected.is_empty() {
            return format!("unexpected {found:?}");
        }
        expected.sort_by_key(|k| k.as_index());
        let parts: Vec<String> = expected.iter().map(|k| format!("{k:?}")).collect();
        let list = match parts.as_slice() {
            [single] => single.clone(),
            [a, b] => format!("{a} or {b}"),
            rest => {
                let (last, init) = rest.split_last().unwrap();
                format!("{}, or {last}", init.join(", "))
            }
        };
        format!("expected {list}, found {found:?}")
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
