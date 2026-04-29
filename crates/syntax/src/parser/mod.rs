use crate::{green::Parser, lexer::TokenKind, red::SyntaxKind};

pub fn parse_source_file(p: &mut Parser) {
    p.start_node(SyntaxKind::SourceFile);
    while p.current() != TokenKind::Eof {
        if at_decl_start(p) {
            parse_decl(p);
        } else {
            p.error_and_bump("expected a declaration");
        }
    }
    p.finish_node();
}

fn parse_decl(p: &mut Parser) {
    if at_decl_start(p) {
        parse_def_decl(p);
    } else {
        unreachable!()
    }
}

fn parse_def_decl(p: &mut Parser) {
    p.start_node(SyntaxKind::DefDecl);
    p.expect(TokenKind::Identifier);
    p.expect(TokenKind::Identifier);
    p.finish_node();
}

fn at_decl_start(p: &Parser) -> bool {
    p.current() == TokenKind::Identifier && p.current_text() == "def"
}
