use ast::parse_file;
use db::SourceFile;

use crate::{
    ElabDatabase,
    ids::{
        ItemId,
        ItemKind,
        Symbol,
    },
};

#[salsa::tracked(debug)]
pub struct ItemTree<'db> {
    #[tracked]
    #[returns(ref)]
    pub items: Vec<ItemId<'db>>,
}

#[salsa::tracked]
pub fn item_tree(db: &dyn ElabDatabase, file: SourceFile) -> ItemTree<'_> {
    tracing::info!("Computing item tree for file {:?}", file);
    let parse = parse_file(db, file);
    let source = parse.tree();

    let mut items = Vec::new();
    for (i, decl) in source.decl().enumerate() {
        let ast::Decl::DefDecl(def) = decl;
        let Some(ident_node) = def.ident() else {
            continue;
        };
        let Some(ident) = ident_node.text() else {
            continue;
        };
        let symbol = Symbol::from_str(db, ident);
        items.push(ItemId::new(db, file, symbol, i as u32, ItemKind::Def));
    }

    ItemTree::new(db, items)
}
