use ast::{
    parse_file,
    traits::AstNode,
};

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
    let decl_range = node
        .as_ref()
        .map(|n| n.syntax().text_range())
        .unwrap_or_default();
    let ty = match item.kind(db) {
        ItemKind::Def => match node {
            Some(ast::Decl::DefDecl(decl)) => match decl.return_type().and_then(|r| r.r#type()) {
                Some(ret_ty) => cx.with_pi_binders(decl.binders(), |cx| cx.lower_type(ret_ty)),
                None => cx.error_term(),
            },
            _ => cx.error_term(),
        },
        ItemKind::Inductive => match node {
            Some(ast::Decl::InductiveDecl(decl)) => {
                match decl.return_type().and_then(|r| r.r#type()) {
                    Some(ret_ty) => cx.lower_type(ret_ty),
                    None => cx.error_term(),
                }
            }
            _ => cx.error_term(),
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
            if let Some(ctor) = ctor
                && let Some(return_type_node) = ctor.return_type()
                && let Some(ret_ty) = return_type_node.r#type()
            {
                cx.with_pi_binders(ctor.binders(), |cx| cx.lower_type(ret_ty))
            } else {
                cx.error_term()
            }
        }
    };

    tracing::debug!(
        "signature for {} is {}",
        item.name(db).text(db),
        ty.debug(db)
    );
    let ty = cx.zonk(ty);
    let ty = cx.abstract_autobound_pi(ty);
    cx.report_unsolved_mvars(ty, decl_range);
    Signature { ty }
}
