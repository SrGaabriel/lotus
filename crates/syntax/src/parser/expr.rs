use diagnostics::Severity;

use crate::{
    kind::SyntaxKind,
    lexer::TokenKind,
    parser::{
        Parser,
        marker::CompletedMarker,
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

pub const BINDER_FIRST: TokenSet =
    TokenSet::new(&[TokenKind::LParen, TokenKind::LBrace, TokenKind::LBracket]);
pub const BINDER_RECOVERY: TokenSet = TokenSet::new(&[
    TokenKind::RParen,
    TokenKind::RBrace,
    TokenKind::RBracket,
    TokenKind::Eof,
]);

impl Parser<'_> {
    pub fn at_expr_start(&self) -> bool {
        self.at_ts(EXPR_FIRST)
    }

    pub fn parse_expr(&mut self, recovery: TokenSet) {
        if !self.at_expr_start() {
            self.error_expected("expression", recovery);
            return;
        }
        let mut lhs = self.parse_atom_expr(recovery);
        while self.at_expr_start() {
            let m = lhs.precede(self);
            self.parse_atom_expr(recovery);
            lhs = m.complete(self, SyntaxKind::AppExpr);
        }
    }

    fn parse_atom_expr(&mut self, recovery: TokenSet) -> CompletedMarker {
        match self.current() {
            TokenKind::Identifier => self.parse_name_expr(recovery),
            TokenKind::Number => self.parse_number_expr(recovery),
            TokenKind::LParen => self.parse_paren_expr(recovery),
            TokenKind::LBrace => self.parse_brace_block(recovery),
            _ => unreachable!(),
        }
    }

    pub fn parse_paren_expr(&mut self, recovery: TokenSet) -> CompletedMarker {
        let m = self.start();
        let inner = recovery.union(EXPR_RECOVERY);
        self.expect_recover(TokenKind::LParen, inner);
        if !self.at(TokenKind::RParen) {
            self.parse_expr(inner);
        }
        self.expect_recover(TokenKind::RParen, inner);
        m.complete(self, SyntaxKind::ParenExpr)
    }

    pub fn parse_brace_block(&mut self, recovery: TokenSet) -> CompletedMarker {
        let m = self.start();
        let inner = recovery.union(EXPR_RECOVERY).union(STMT_FIRST);
        self.expect_recover(TokenKind::LBrace, inner);
        while !self.check_at(TokenKind::RBrace) && !self.check_at(TokenKind::Eof) {
            match self.current() {
                TokenKind::LetKw => self.parse_let_stmt(inner),
                TokenKind::ReturnKw => self.parse_return_stmt(inner),
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
        m.complete(self, SyntaxKind::BraceBlock)
    }

    pub fn parse_let_stmt(&mut self, recovery: TokenSet) {
        let m = self.start();
        self.bump_remap(SyntaxKind::LetKw);
        let name = self.start();
        if self.expect_recover(TokenKind::Identifier, recovery) {
            name.complete(self, SyntaxKind::Identifier);
        } else {
            name.abandon(self);
            m.complete(self, SyntaxKind::LetStmt);
            return;
        }
        if self.check_at(TokenKind::Colon) {
            let ann = self.start();
            self.bump();
            let colon_label = self.label(self.current_range(), "`:` here");
            self.with_label(colon_label, |p| {
                p.parse_type(
                    recovery.union(TokenSet::new(&[TokenKind::DefEq, TokenKind::Semicolon])),
                );
            });
            ann.complete(self, SyntaxKind::TypeAnnotation);
        }

        self.parse_assignment_tail(m, recovery, SyntaxKind::LetStmt);
    }

    pub fn parse_mutation_stmt(&mut self, recovery: TokenSet) {
        let m = self.start();
        let name = self.start();
        if self.expect_recover(TokenKind::Identifier, recovery) {
            name.complete(self, SyntaxKind::Identifier);
        } else {
            name.abandon(self);
            m.complete(self, SyntaxKind::MutationStmt);
            return;
        }
        self.parse_assignment_tail(m, recovery, SyntaxKind::MutationStmt);
    }

    pub fn parse_return_stmt(&mut self, recovery: TokenSet) {
        let m = self.start();
        self.bump_remap(SyntaxKind::ReturnStmt);
        self.parse_expr(recovery);
        self.expect_semi(SEMI_FOLLOW, recovery);
        m.complete(self, SyntaxKind::ReturnStmt);
    }

    fn parse_assignment_tail(
        &mut self,
        m: crate::parser::marker::Marker,
        recovery: TokenSet,
        kind: SyntaxKind,
    ) {
        if !self.expect_recover(TokenKind::DefEq, recovery) {
            m.complete(self, kind);
            return;
        }
        let eq_label = self.label(self.prev_range().unwrap(), "`:=` here");
        self.with_label(eq_label, |p| p.parse_expr(recovery.union(SEMI_SET)));
        self.expect_semi(SEMI_FOLLOW, recovery);
        m.complete(self, kind);
    }

    pub fn parse_name_expr(&mut self, recovery: TokenSet) -> CompletedMarker {
        let m = self.start();
        loop {
            if let Some(next) = self.peek_nth(1)
                && next.kind == TokenKind::ColonColon
            {
                let ps = self.start();
                let ident = self.start();
                self.expect_recover(TokenKind::Identifier, recovery);
                ident.complete(self, SyntaxKind::Identifier);
                self.bump();
                ps.complete(self, SyntaxKind::PathSegment);
            } else {
                let ident = self.start();
                self.expect_recover(TokenKind::Identifier, recovery);
                ident.complete(self, SyntaxKind::Identifier);
                break;
            }
        }
        m.complete(self, SyntaxKind::Name)
    }

    pub fn parse_number_expr(&mut self, recovery: TokenSet) -> CompletedMarker {
        let m = self.start();
        self.expect_recover(TokenKind::Number, recovery);
        m.complete(self, SyntaxKind::NumberLit)
    }

    pub fn parse_binder(&mut self, recovery: TokenSet) {
        let m = self.start();
        let inner = recovery.union(BINDER_RECOVERY);
        let (open, close, kind) = match self.current() {
            TokenKind::LParen => (
                TokenKind::LParen,
                TokenKind::RParen,
                SyntaxKind::ParenBinder,
            ),
            TokenKind::LBrace => (
                TokenKind::LBrace,
                TokenKind::RBrace,
                SyntaxKind::BraceBinder,
            ),
            TokenKind::LBracket => (
                TokenKind::LBracket,
                TokenKind::RBracket,
                SyntaxKind::BracketBinder,
            ),
            _ => unreachable!(),
        };
        self.expect_recover(open, inner);
        if !self.at(close) {
            let name = self.start();
            if self.expect_recover(TokenKind::Identifier, inner) {
                name.complete(self, SyntaxKind::Identifier);
            } else {
                name.abandon(self);
                m.complete(self, kind);
                return;
            }
            if self.expect_recover(TokenKind::Colon, inner) {
                let colon_label = self.label(self.prev_range().unwrap(), "`:` here");
                self.with_label(colon_label, |p| p.parse_type(inner));
            }
        }
        self.expect_recover(close, inner);
        m.complete(self, kind);
    }

    pub fn parse_type(&mut self, recovery: TokenSet) {
        if self.at_binder_start() || self.at(TokenKind::RArrow) {
            self.parse_pi_type(recovery);
            return;
        }
        if !self.at_type_atom_start() {
            self.error_expected("type", recovery);
            return;
        }
        let mut lhs = self.parse_atom_type(recovery);
        while self.at_type_atom_start() && !self.at_binder_start() {
            let m = lhs.precede(self);
            self.parse_atom_type(recovery);
            lhs = m.complete(self, SyntaxKind::AppType);
        }
        if self.at(TokenKind::RArrow) {
            let m = lhs.precede(self);
            self.bump();
            self.parse_type(recovery);
            m.complete(self, SyntaxKind::ArrowType);
        }
    }

    fn at_type_atom_start(&self) -> bool {
        matches!(self.current(), TokenKind::Identifier | TokenKind::LParen)
    }

    fn at_binder_start(&self) -> bool {
        match self.current() {
            TokenKind::LBrace | TokenKind::LBracket => true,
            TokenKind::LParen => {
                self.peek_nth(1).map(|t| t.kind) == Some(TokenKind::Identifier)
                    && self.peek_nth(2).map(|t| t.kind) == Some(TokenKind::Colon)
            }
            _ => false,
        }
    }

    fn parse_atom_type(&mut self, recovery: TokenSet) -> CompletedMarker {
        match self.current() {
            TokenKind::LParen => self.parse_paren_type(recovery),
            TokenKind::Identifier => self.parse_name_type(recovery),
            _ => unreachable!(),
        }
    }

    fn parse_name_type(&mut self, recovery: TokenSet) -> CompletedMarker {
        let m = self.start();
        loop {
            if let Some(forthcoming) = self.peek_nth(1)
                && forthcoming.kind == TokenKind::ColonColon
            {
                let ps = self.start();
                let ident = self.start();
                self.expect_recover(TokenKind::Identifier, recovery);
                ident.complete(self, SyntaxKind::Identifier);
                self.bump();
                ps.complete(self, SyntaxKind::PathSegment);
            } else {
                let ident = self.start();
                self.expect_recover(TokenKind::Identifier, recovery);
                ident.complete(self, SyntaxKind::Identifier);
                break;
            }
        }
        m.complete(self, SyntaxKind::Name)
    }

    fn parse_paren_type(&mut self, recovery: TokenSet) -> CompletedMarker {
        let m = self.start();
        let inner = recovery.union(TokenSet::new(&[TokenKind::RParen]));
        self.expect_recover(TokenKind::LParen, inner);
        self.parse_type(inner);
        self.expect_recover(TokenKind::RParen, inner);
        m.complete(self, SyntaxKind::ParenType)
    }

    fn parse_pi_type(&mut self, recovery: TokenSet) {
        let m = self.start();
        let inner = recovery.union(TokenSet::new(&[TokenKind::RArrow]));
        self.parse_binder(inner);
        self.expect_recover(TokenKind::RArrow, recovery);
        self.parse_type(recovery);
        m.complete(self, SyntaxKind::PiType);
    }
}
