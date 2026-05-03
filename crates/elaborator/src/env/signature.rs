use ast::parse_file;

use crate::{
    ElabDatabase,
    core::{
        TermArena,
        TermId,
    },
    elab::ctx::ElabCtx,
    ids::{
        ItemId,
        ItemKind,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, salsa::Update)]
pub struct Signature<'db> {
    pub arena: TermArena<'db>,
    pub ty: TermId,
}

#[salsa::tracked(returns(ref))]
pub fn signature<'db>(db: &'db dyn ElabDatabase, item: ItemId<'db>) -> Signature<'db> {
    let mut cx = ElabCtx::new(db, item);

    let file = item.file(db);
    let parse = parse_file(db, file);
    let source = parse.tree();

    let ty = match item.kind(db) {
        ItemKind::Def => match source.decl().nth(item.ast_index(db) as usize) {
            Some(ast::Decl::DefDecl(decl)) => match decl.return_type() {
                Some(ret_ty) => cx.lower_return_type(ret_ty),
                None => cx.placeholder(),
            },
            _ => cx.placeholder(),
        },
    };

    Signature {
        arena: cx.arena,
        ty,
    }
}
