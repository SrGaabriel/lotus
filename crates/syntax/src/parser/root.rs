use super::token_set::TokenSet;
use crate::{
    kind::SyntaxKind,
    lexer::TokenKind,
    parser::{Parser, expr::EXPR_FIRST},
};

pub const DECL_FIRST: TokenSet = TokenSet::new(&[TokenKind::Identifier]);

pub fn is_contextual_kw(text: &str) -> bool {
    matches!(text, "def")
}

impl Parser<'_> {
    pub fn at_kw(&self, kw: &str) -> bool {
        self.current() == TokenKind::Identifier && self.current_text() == kw
    }

    pub fn at_decl_start(&self) -> bool {
        self.at_kw("def")
    }

    pub fn parse_source_file(&mut self) {
        let m = self.start();
        while !self.at(TokenKind::Eof) {
            if self.at_decl_start() {
                self.parse_decl();
            } else {
                self.error_and_bump("expected a declaration");
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
        let recovery = DECL_FIRST.union(EXPR_FIRST);

        let name = self.start();
        if self.expect_recover(TokenKind::Identifier, recovery) {
            name.complete(self, SyntaxKind::Name);
        } else {
            name.abandon(self);
            m.complete(self, SyntaxKind::DefDecl);
            return;
        }
        if !self.expect_recover(TokenKind::DefEq, recovery) {
            m.complete(self, SyntaxKind::DefDecl);
            return;
        }

        if self.at(TokenKind::Eof) {
            self.error_expected_with("expression", recovery, |b| b
            .with_help("either add a body to this declaration, or remove the `:=` if you intended to declare a name without defining it".into())
        );
        } else {
            let prev_range = self.prev_range().unwrap();
            let eq_label = self.label(prev_range, "`:=` here");
            self.parse_expr(recovery, |b| b.with_secondary_label(eq_label));
        }
        m.complete(self, SyntaxKind::DefDecl);
    }  
}
