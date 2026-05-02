use diagnostics::EnrichTy;

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
            TokenKind::Identifier => self.parse_name_expr(enrich),
            TokenKind::Number => self.parse_number_expr(enrich),
            TokenKind::LParen => self.parse_paren_expr(recovery, enrich),
            TokenKind::LBrace => self.parse_brace_expr(recovery, enrich),
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

    pub fn parse_brace_expr(&mut self, recovery: TokenSet, enrich: EnrichTy!()) {
        let m = self.start();
        let inner = recovery.union(EXPR_RECOVERY);
        self.expect_recover(TokenKind::LBrace, inner);
        if !self.at(TokenKind::RBrace) {
            self.parse_expr(inner, enrich);
        }
        self.expect_recover(TokenKind::RBrace, inner);
        m.complete(self, SyntaxKind::BraceBlock);
    }

    pub fn parse_name_expr(&mut self, _enrich: EnrichTy!()) {
        let m = self.start();
        self.expect_recover(TokenKind::Identifier, EXPR_RECOVERY);
        m.complete(self, SyntaxKind::Name);
    }

    pub fn parse_number_expr(&mut self, _enrich: EnrichTy!()) {
        let m = self.start();
        self.expect_recover(TokenKind::Number, EXPR_RECOVERY);
        m.complete(self, SyntaxKind::NumberLit);
    }
}
