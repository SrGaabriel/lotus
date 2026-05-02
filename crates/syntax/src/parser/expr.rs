use diagnostics::{
    EnrichTy,
    Severity,
    builder::conserve,
};

use crate::{
    kind::SyntaxKind,
    lexer::TokenKind,
    parser::{
        Parser,
        token_set::TokenSet,
    },
};

pub const EXPR_FIRST: TokenSet = TokenSet::new(&[
    TokenKind::Identifier,
    TokenKind::Number,
    TokenKind::LParen,
    TokenKind::LBrace,
]);
pub const EXPR_RECOVERY: TokenSet = TokenSet::new(&[
    TokenKind::Semicolon,
    TokenKind::RParen,
    TokenKind::RBrace,
    TokenKind::Eof,
]);
pub const STMT_FIRST: TokenSet = TokenSet::new(&[TokenKind::LetKw, TokenKind::Identifier]);
pub const STMT_SYNC: TokenSet =
    STMT_FIRST.union(TokenSet::new(&[TokenKind::RBrace, TokenKind::Eof]));
const SEMI_FOLLOW: TokenSet = STMT_FIRST.union(TokenSet::new(&[TokenKind::RBrace]));
const SEMI_SET: TokenSet = TokenSet::new(&[TokenKind::Semicolon]);

impl Parser<'_> {
    pub fn at_expr_start(&self) -> bool {
        self.at_ts(EXPR_FIRST)
    }

    pub fn parse_expr(&mut self, recovery: TokenSet, enrich: EnrichTy!()) {
        if !self.at_expr_start() {
            self.error_expected_with("expression", recovery, enrich);
            return;
        }
        match self.current() {
            TokenKind::Identifier => self.parse_name_expr(recovery, conserve()),
            TokenKind::Number => self.parse_number_expr(recovery, conserve()),
            TokenKind::LParen => self.parse_paren_expr(recovery, conserve()),
            TokenKind::LBrace => self.parse_brace_block(recovery, conserve()),
            _ => unreachable!(),
        }
    }

    pub fn parse_paren_expr(&mut self, recovery: TokenSet, enrich: EnrichTy!()) {
        let m = self.start();
        let inner = recovery.union(EXPR_RECOVERY);
        self.expect_recover(TokenKind::LParen, inner);
        if !self.at(TokenKind::RParen) {
            self.parse_expr(inner, enrich);
        }
        self.expect_recover(TokenKind::RParen, inner);
        m.complete(self, SyntaxKind::ParenExpr);
    }

    pub fn parse_brace_block(&mut self, recovery: TokenSet, _enrich: EnrichTy!()) {
        let m = self.start();
        let inner = recovery.union(EXPR_RECOVERY).union(STMT_FIRST);
        self.expect_recover(TokenKind::LBrace, inner);
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            match self.current() {
                TokenKind::LetKw => self.parse_let_stmt(inner),
                TokenKind::Identifier => self.parse_mutation_stmt(inner),
                _ => self.recover_until_with(
                    STMT_SYNC,
                    Some(TokenKind::Semicolon),
                    Severity::Error,
                    "expected statement",
                    |b| b.with_primary_message("not a valid statement"),
                ),
            }
        }
        self.expect_recover(TokenKind::RBrace, inner);
        m.complete(self, SyntaxKind::BraceBlock);
    }

    pub fn parse_let_stmt(&mut self, recovery: TokenSet) {
        let m = self.start();
        self.bump_remap(SyntaxKind::LetKw);
        self.parse_assignment_tail(m, recovery, SyntaxKind::LetStmt);
    }

    pub fn parse_mutation_stmt(&mut self, recovery: TokenSet) {
        let m = self.start();
        self.parse_assignment_tail(m, recovery, SyntaxKind::MutationStmt);
    }

    fn parse_assignment_tail(
        &mut self,
        m: crate::parser::marker::Marker,
        recovery: TokenSet,
        kind: SyntaxKind,
    ) {
        let name = self.start();
        if self.expect_recover(TokenKind::Identifier, recovery) {
            name.complete(self, SyntaxKind::Name);
        } else {
            name.abandon(self);
            m.complete(self, kind);
            return;
        }
        if !self.expect_recover(TokenKind::DefEq, recovery) {
            m.complete(self, kind);
            return;
        }
        let eq_label = self.label(self.prev_range().unwrap(), "`:=` here");
        self.parse_expr(recovery.union(SEMI_SET), |b| b.with_secondary_label(eq_label));
        self.expect_semi(SEMI_FOLLOW, recovery);
        m.complete(self, kind);
    }

    pub fn parse_name_expr(&mut self, recovery: TokenSet, _enrich: EnrichTy!()) {
        let m = self.start();
        self.expect_recover(TokenKind::Identifier, recovery);
        m.complete(self, SyntaxKind::Name);
    }

    pub fn parse_number_expr(&mut self, recovery: TokenSet, _enrich: EnrichTy!()) {
        let m = self.start();
        self.expect_recover(TokenKind::Number, recovery);
        m.complete(self, SyntaxKind::NumberLit);
    }
}
