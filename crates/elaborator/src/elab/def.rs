use ast::{
    parse_file,
    traits::AstNode,
};

use crate::{
    ElabDatabase,
    ElabDb,
    elab::{
        ctx::{
            ElabCtx,
            Frame,
        },
        expected::{
            Expected,
            ExpectedReason,
        },
    },
    env::DefBody,
    ids::ItemId,
};

#[salsa::tracked(returns(ref))]
pub fn def_body<'db>(db: &'db dyn ElabDatabase, item: ItemId<'db>) -> DefBody<'db> {
    let mut cx = ElabCtx::new(db, item);

    let file = item.file(db);
    let parse = parse_file(db, file);
    let source = parse.tree();

    let value = match source.decl().nth(item.ast_index(db) as usize) {
        Some(ast::Decl::DefDecl(decl)) => match decl.body() {
            Some(expr) => {
                let ty = db.signature(item).ty;
                let annotation = decl.return_type().map_or_else(
                    || expr.syntax().text_range(),
                    |ret| ret.syntax().text_range(),
                );
                let expected = Expected::new(ty, ExpectedReason::ReturnType { annotation });
                let name = item.name(db);
                let free_binders = cx.elaborate_binders(decl.params());
                let body = cx.with_frame(Frame::DefBody { name }, |cx| cx.check(expr, &expected));
                cx.abstract_binders(&free_binders, body)
            }
            None => cx.placeholder(),
        },
        _ => cx.placeholder(),
    };

    DefBody { value }
}
