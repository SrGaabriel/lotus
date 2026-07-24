use rustc_hash::FxHashMap;

use db::SourceFile;

use crate::{
    ElabDatabase,
    ElabDb,
    env::{
        Namespace,
        namespace::{
            ItemMap,
            SymbolMap,
        },
    },
};

#[salsa::tracked]
pub fn def_map<'db>(db: &'db dyn ElabDatabase, file: SourceFile) -> Namespace<'db> {
    let tree = db.item_tree(file);
    let mut decls: ItemMap = FxHashMap::default();
    let mut children: SymbolMap<'db, Namespace<'db>> = FxHashMap::default();

    for &id in tree.items(db) {
        if id.parent(db).is_none() {
            decls.insert(id.name(db), id);
        }
    }

    let mut child_groups: SymbolMap<'db, ItemMap<'db>> = FxHashMap::default();
    for &id in tree.items(db) {
        if let Some(parent) = id.parent(db) {
            child_groups
                .entry(parent.name(db))
                .or_default()
                .insert(id.name(db), id);
        }
    }
    for (parent_name, kids) in child_groups {
        children.insert(parent_name, Namespace::new(db, kids, FxHashMap::default()));
    }

    Namespace::new(db, decls, children)
}
