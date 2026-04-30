use diagnostics::EnrichTy;

use super::token_set::TokenSet;
use crate::{
    kind::SyntaxKind,
    lexer::TokenKind,
    parser::Parser,
};

const DECL_FIRST: TokenSet = TokenSet::new(&[TokenKind::Identifier]);
const EXPR_FIRST: TokenSet =
    TokenSet::new(&[TokenKind::Identifier, TokenKind::Number, TokenKind::LParen]);

fn is_contextual_kw(text: &str) -> bool {
    matches!(text, "def")
}

fn at_kw(p: &Parser, kw: &str) -> bool {
    p.current() == TokenKind::Identifier && p.current_text() == kw
}

fn at_decl_start(p: &Parser) -> bool {
    at_kw(p, "def")
}

fn at_expr_start(p: &Parser) -> bool {
    if !p.at_ts(EXPR_FIRST) {
        return false;
    }
    !(p.current() == TokenKind::Identifier && is_contextual_kw(p.current_text()))
}

pub fn parse_source_file(p: &mut Parser) {
    let m = p.start();
    while !p.at(TokenKind::Eof) {
        if at_decl_start(p) {
            parse_decl(p);
        } else {
            p.error_and_bump("expected a declaration");
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

    if p.at(TokenKind::Eof) {
        p.error_expected_with("expression", recovery, |b| b
            .with_help("either add a body to this declaration, or remove the `:=` if you intended to declare a name without defining it".into())
        );
    } else {
        let eq_label = p.label(p.prev_range().unwrap(), "`:=` here");
        parse_expr(p, recovery, |b| b.with_secondary_label(eq_label));
    }
    m.complete(p, SyntaxKind::DefDecl);
}

fn parse_expr(p: &mut Parser, recovery: TokenSet, enrich: EnrichTy!()) {
    if !at_expr_start(p) {
        p.error_expected_with("expression", recovery, enrich);
        return;
    }
    let m = p.start();
    p.bump();
    m.complete(p, SyntaxKind::Expr);
}
