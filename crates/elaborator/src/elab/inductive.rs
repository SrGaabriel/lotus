use ast::parse_file;

use crate::{
    ElabDatabase,
    ElabDb,
    core::Term,
    elab::ctx::ElabCtx,
    ids::{
        ItemId,
        ItemKind,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, salsa::Update)]
pub struct InductiveData<'db> {
    pub num_params: usize,
    pub binders: Vec<Term<'db>>,
    pub ctors: Vec<ItemId<'db>>,
}

#[salsa::tracked(returns(ref))]
pub fn inductive_data<'db>(db: &'db dyn ElabDatabase, item: ItemId<'db>) -> InductiveData<'db> {
    debug_assert_eq!(item.kind(db), ItemKind::Inductive);

    let mut cx = ElabCtx::new(db, item);
    let file = item.file(db);

    let tree = db.item_tree(file);
    let ctors: Vec<ItemId<'db>> = tree
        .items(db)
        .iter()
        .copied()
        .filter(|c| c.parent(db) == Some(item) && c.kind(db) == ItemKind::Constructor)
        .collect();

    let parse = parse_file(db, file);
    let source = parse.tree();
    let mut binders: Vec<Term<'db>> = Vec::new();
    let mut num_params: usize = 0;

    if let Some(ast::Decl::InductiveDecl(decl)) = source.decl().nth(item.ast_index(db) as usize) {
        for binder in decl.params() {
            let ty = match binder.ty() {
                Some(t) => cx.lower_type(t),
                None => cx.error_mvar(),
            };
            binders.push(ty);
            num_params += 1;
        }
    }

    InductiveData {
        num_params,
        binders,
        ctors,
    }
}
