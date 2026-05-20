use ast::parse_file;

use crate::{
    ElabDatabase, ElabDb, elab::ctx::ElabCtx, env::DefBody, ids::ItemId
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
                let sig = db.signature(item);
                let expected = cx.import_term(&sig.arena, sig.ty);
                cx.check(expr, expected)
            }
            None => cx.placeholder(),
        },
        _ => cx.placeholder(),
    };

    DefBody {
        arena: cx.arena,
        value,
    }
}
