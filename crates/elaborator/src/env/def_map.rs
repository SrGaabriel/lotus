use rustc_hash::FxHashMap;

use db::SourceFile;

use crate::{
    ElabDatabase,
    ElabDb,
    env::Namespace,
};

#[salsa::tracked]
pub fn def_map(db: &dyn ElabDatabase, file: SourceFile) -> Namespace<'_> {
    let tree = db.item_tree(file);
    let mut decls = FxHashMap::default();
    for def in tree.items(db) {
        decls.insert(def.name(db), *def);
    }
    Namespace::new(db, decls, FxHashMap::default())
}
