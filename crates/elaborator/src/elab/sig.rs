use ast::parse_file;

use crate::{
    ElabDatabase,
    core::Term,
    elab::ctx::ElabCtx,
    ids::{
        ItemId,
        ItemKind,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, salsa::Update)]
pub struct Signature<'db> {
    pub ty: Term<'db>,
}

#[salsa::tracked(returns(ref))]
pub fn signature<'db>(db: &'db dyn ElabDatabase, item: ItemId<'db>) -> Signature<'db> {
    let mut cx = ElabCtx::new(db, item);

    let file = item.file(db);
    let parse = parse_file(db, file);
    let source = parse.tree();
    let node = source.decl().nth(item.ast_index(db) as usize);
    let ty = match item.kind(db) {
        ItemKind::Def => match node {
            Some(ast::Decl::DefDecl(decl)) => match decl.return_type().and_then(|r| r.r#type()) {
                Some(ret_ty) => cx.lower_type(ret_ty),
                None => cx.error_mvar(),
            },
            _ => cx.error_mvar(),
        },
        ItemKind::Inductive => match node {
            Some(ast::Decl::InductiveDecl(decl)) => {
                match decl.return_type().and_then(|r| r.r#type()) {
                    Some(ret_ty) => cx.lower_type(ret_ty),
                    None => cx.error_mvar(),
                }
            }
            _ => cx.error_mvar(),
        },
        ItemKind::Constructor => {
            let parent = item
                .parent(db)
                .expect("constructor ItemId must have a parent");
            let parent_node = source.decl().nth(parent.ast_index(db) as usize);
            let ctor = parent_node.and_then(|d| {
                if let ast::Decl::InductiveDecl(ind) = d {
                    ind.inductive_constructors()
                        .and_then(|cs| cs.constructor_decl().nth(item.ast_index(db) as usize))
                } else {
                    None
                }
            });
            match ctor.and_then(|c| c.return_type()).and_then(|r| r.r#type()) {
                Some(ret_ty) => cx.lower_type(ret_ty),
                None => cx.error_mvar(),
            }
        }
    };

    Signature { ty }
}
