use ast::{
    parse_file,
    traits::AstNode,
};

use crate::{
    ElabDatabase,
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
pub fn def_body<'db>(db: &'db dyn ElabDatabase, item: ItemId<'db>) -> Option<DefBody<'db>> {
    let mut cx = ElabCtx::new(db, item);

    let file = item.file(db);
    let parse = parse_file(db, file);
    let source = parse.tree();

    let value = match source.decl().nth(item.ast_index(db) as usize) {
        Some(ast::Decl::DefDecl(decl)) => match decl.body() {
            Some(expr) => {
                let return_type_tok = decl.return_type().and_then(|r| r.r#type());

                let annotation = return_type_tok.as_ref().map_or_else(
                    || expr.syntax().text_range(),
                    |ret| ret.syntax().text_range(),
                );

                let name = item.name(db);
                cx.with_binders(decl.binders(), |cx| {
                    let body_type = match return_type_tok {
                        Some(t) => cx.lower_type(t),
                        None => cx.error_mvar(),
                    };
                    let expected =
                        Expected::new(body_type, ExpectedReason::ReturnType { annotation });
                    cx.with_frame(Frame::DefBody { name }, |cx| cx.check(expr, &expected))
                })
            }
            None => cx.placeholder_ty(),
        },
        _ => return None,
    };

    let value = cx.abstract_autobound_lam(value);
    Some(DefBody { value })
}
