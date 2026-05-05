use super::token_set::TokenSet;
use crate::{
    kind::SyntaxKind,
    lexer::TokenKind,
    parser::{
        Parser,
        expr::EXPR_FIRST,
    },
};
use diagnostics::Severity;

pub const DECL_FIRST: TokenSet = TokenSet::new(&[TokenKind::DefKw]);

impl Parser<'_> {
    pub fn at_decl_start(&self) -> bool {
        let kind = self.current();
        DECL_FIRST.contains(kind)
    }

    pub fn parse_source_file(&mut self) {
        let m = self.start();
        while !self.at(TokenKind::Eof) {
            if self.at_decl_start() {
                self.parse_decl();
            } else {
                self.recover_until_with(
                    DECL_FIRST,
                    None,
                    Severity::Error,
                    "expected a declaration",
                    |b| b,
                );
            }
        }
        m.complete(self, SyntaxKind::SourceFile);
    }

    pub fn parse_decl(&mut self) {
        debug_assert!(self.at_decl_start());
        self.parse_def_decl();
    }

    pub fn parse_def_decl(&mut self) {
        let m = self.start();
        self.bump_remap(SyntaxKind::DefKw);

        let name = self.start();
        if !self.expect_recover(TokenKind::Identifier, DECL_FIRST) {
            name.abandon(self);
            m.complete(self, SyntaxKind::DefDecl);
            return;
        }
        name.complete(self, SyntaxKind::Identifier);

        self.with_help("parameters must be declared before the `:=`", |p| {
            while p.at(TokenKind::LParen) || p.at(TokenKind::LBrace) || p.at(TokenKind::LBracket) {
                p.parse_binder(DECL_FIRST);
            }
        });

        let header_recovery = DECL_FIRST
            .union(TokenSet::new(&[TokenKind::DefEq]))
            .union(EXPR_FIRST);
        let ret = self.start();
        let colon = self.with_help("all definitions need an explicit return type", |p| {
            p.expect_recover(TokenKind::Colon, header_recovery)
        });
        if colon {
            self.parse_type(header_recovery);
            ret.complete(self, SyntaxKind::DefReturnType);
        } else {
            ret.abandon(self);
        }

        let body_recovery = DECL_FIRST.union(EXPR_FIRST);
        let has_eq = self.expect_recover(TokenKind::DefEq, body_recovery);

        if self.at(TokenKind::Eof) {
            if has_eq {
                self.with_help(
                    "either add a body to this declaration, or remove the `:=` if you intended to declare a name without defining it",
                    |p| p.error_expected("expression", body_recovery),
                );
            }
        } else if self.at_expr_start() {
            let prev_range = self.prev_range().unwrap();
            let eq_label = self.label(prev_range, "`:=` here");
            self.with_label(eq_label, |p| p.parse_expr(body_recovery));
        }

        m.complete(self, SyntaxKind::DefDecl);
    }
}
