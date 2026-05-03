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
    let parse = parse_file(db, file);
    let source = parse.tree();

    let mut items = Vec::new();
    for (i, decl) in source.decl().enumerate() {
        let ast::Decl::DefDecl(def) = decl;
        let Some(name_node) = def.name() else {
            continue;
        };
        let Some(ident) = name_node.ident() else {
            continue;
        };
        let symbol = Symbol::from_str(db, ident.text());
        items.push(ItemId::new(db, file, symbol, i as u32, ItemKind::Def));
    }

    ItemTree::new(db, items)
}
