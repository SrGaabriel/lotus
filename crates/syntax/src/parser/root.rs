use super::token_set::TokenSet;
use crate::{
    kind::SyntaxKind,
    lexer::TokenKind,
    parser::{
        Parser,
        expr::EXPR_FIRST,
        marker::Marker,
    },
};
use diagnostics::Severity;

pub const DECL_FIRST: TokenSet =
    TokenSet::new(&[TokenKind::DefKw, TokenKind::InductiveKw, TokenKind::At]);

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
        let m = self.start();

        while self.at(TokenKind::At) {
            self.parse_attr();
        }

        let peek = self.current();
        match peek {
            TokenKind::DefKw => self.parse_def_decl(m),
            TokenKind::InductiveKw => self.parse_inductive_decl(m),
            _ => {
                self.error_expected("a declaration (`def` or `inductive`)", DECL_FIRST);
                m.complete(self, SyntaxKind::Error);
            }
        }
    }

    pub fn parse_def_decl(&mut self, m: Marker) {
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
            ret.complete(self, SyntaxKind::ReturnType);
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

    pub fn parse_inductive_decl(&mut self, m: Marker) {
        self.bump_remap(SyntaxKind::InductiveKw);

        let name = self.start();
        if !self.expect_recover(TokenKind::Identifier, DECL_FIRST) {
            name.abandon(self);
            m.complete(self, SyntaxKind::InductiveDecl);
            return;
        }
        name.complete(self, SyntaxKind::Identifier);

        self.with_help("binders must be declared before the `:=`", |p| {
            while p.at(TokenKind::LParen) || p.at(TokenKind::LBrace) || p.at(TokenKind::LBracket) {
                p.parse_binder(DECL_FIRST);
            }
        });
        let index_recovery = DECL_FIRST
            .union(TokenSet::new(&[TokenKind::DefEq]))
            .union(EXPR_FIRST);
        let ret = self.start();
        let colon = self.with_help(
            "all inductive declarations need an explicit return type",
            |p| p.expect_recover(TokenKind::Colon, index_recovery),
        );
        if colon {
            self.parse_type(index_recovery);
            ret.complete(self, SyntaxKind::ReturnType);
        } else {
            ret.abandon(self);
        }

        if !self.expect_recover(TokenKind::DefEq, DECL_FIRST) {
            m.complete(self, SyntaxKind::InductiveDecl);
            return;
        }
        let constructors = self.start();
        let constructor_recovery = DECL_FIRST.union(TokenSet::new(&[TokenKind::Semicolon]));
        self.parse_constructor_decl(constructor_recovery);
        while self.at(TokenKind::Pipe) {
            self.bump();
            self.parse_constructor_decl(constructor_recovery);
        }
        constructors.complete(self, SyntaxKind::InductiveConstructors);
        self.expect_recover(TokenKind::Semicolon, DECL_FIRST);

        m.complete(self, SyntaxKind::InductiveDecl);
    }

    pub fn parse_constructor_decl(&mut self, recovery: TokenSet) {
        let m = self.start();

        while self.at(TokenKind::At) {
            self.parse_attr();
        }

        let name = self.start();
        if !self.expect_recover(TokenKind::Identifier, recovery) {
            name.abandon(self);
            m.complete(self, SyntaxKind::ConstructorDecl);
            return;
        }
        name.complete(self, SyntaxKind::Identifier);

        self.with_help("binders must be declared before the `:`", |p| {
            while p.at(TokenKind::LParen) || p.at(TokenKind::LBrace) || p.at(TokenKind::LBracket) {
                p.parse_binder(recovery);
            }
        });

        let ret_recovery = recovery.union(EXPR_FIRST);
        let ret = self.start();
        let colon = self.with_help("all constructors need an explicit return type", |p| {
            p.expect_recover(TokenKind::Colon, ret_recovery)
        });
        if colon {
            self.parse_type(ret_recovery);
            ret.complete(self, SyntaxKind::ReturnType);
        } else {
            ret.abandon(self);
        }

        m.complete(self, SyntaxKind::ConstructorDecl);
    }

    pub fn parse_attr(&mut self) {
        let m = self.start();
        self.bump_remap(SyntaxKind::At);

        if !self.expect_recover(TokenKind::LBracket, DECL_FIRST) {
            m.complete(self, SyntaxKind::Attribute);
            return;
        }

        let name = self.start();
        if self.expect_recover(
            TokenKind::Identifier,
            DECL_FIRST.union(TokenSet::new(&[TokenKind::RBracket])),
        ) {
            name.complete(self, SyntaxKind::Identifier);
        } else {
            name.abandon(self);
        }

        if self.at(TokenKind::String) {
            let v = self.start();
            self.bump();
            v.complete(self, SyntaxKind::StringLit);
        } else if self.at(TokenKind::Number) {
            let v = self.start();
            self.bump();
            v.complete(self, SyntaxKind::NumberLit);
        } else if !self.at(TokenKind::RBracket) {
            let recovery = DECL_FIRST.union(TokenSet::new(&[TokenKind::RBracket]));
            self.parse_expr(recovery);
        }

        self.expect_recover(TokenKind::RBracket, DECL_FIRST);

        m.complete(self, SyntaxKind::Attribute);
    }
}
