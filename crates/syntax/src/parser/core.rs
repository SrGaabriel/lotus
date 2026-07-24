use crate::{
    kind::SyntaxKind,
    lexer::TokenKind,
    parser::{
        Parser,
        token_set::TokenSet,
    },
};

impl Parser<'_> {
    pub fn parse_identifier(&mut self, recovery: TokenSet) {
        let ident = self.start();
        self.expect_recover(TokenKind::Identifier, recovery);
        ident.complete(self, SyntaxKind::Identifier);
    }

    pub fn parse_path(&mut self, recovery: TokenSet) {
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
                break;
            }
        }
    }
}
