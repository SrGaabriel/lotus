use crate::{kind::SyntaxKind, lexer::TokenKind, parser::Parser};

use super::token_set::TokenSet;

const DECL_FIRST: TokenSet = TokenSet::new(&[TokenKind::Identifier]);
const EXPR_FIRST: TokenSet =
    TokenSet::new(&[TokenKind::Identifier, TokenKind::Number, TokenKind::LParen]);

pub fn parse_source_file(p: &mut Parser) {
    let m = p.start();
    while !p.at(TokenKind::Eof) {
        if at_decl_start(p) {
            parse_decl(p);
        } else {
            p.err_recover(DECL_FIRST);
        }
    }
    m.complete(p, SyntaxKind::SourceFile);
}

fn parse_decl(p: &mut Parser) {
    debug_assert!(at_decl_start(p));
    parse_def_decl(p);
}

fn parse_def_decl(p: &mut Parser) {
    let m = p.start();
    p.bump_remap(SyntaxKind::DefKw);

    let recovery = DECL_FIRST.union(EXPR_FIRST);

    let name = p.start();
    if p.expect_recover(TokenKind::Identifier, recovery) {
        name.complete(p, SyntaxKind::Name);
    } else {
        name.abandon(p);
        m.complete(p, SyntaxKind::DefDecl);
        return;
    }

    if !p.expect_recover(TokenKind::DefEq, recovery) {
        m.complete(p, SyntaxKind::DefDecl);
        return;
    }

    parse_expr(p, DECL_FIRST);
    m.complete(p, SyntaxKind::DefDecl);
}

fn at_decl_start(p: &Parser) -> bool {
    p.current() == TokenKind::Identifier && p.current_text() == "def"
}

fn parse_expr(p: &mut Parser, recovery: TokenSet) {
    if !p.at_ts(EXPR_FIRST) {
        p.err_recover(recovery);
        return;
    }
    let m = p.start();
    p.bump();
    m.complete(p, SyntaxKind::Expr);
}
