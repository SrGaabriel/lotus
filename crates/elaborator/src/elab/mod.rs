pub mod app;
pub mod ctx;
pub mod def;
pub mod expected;
pub mod inductive;
pub mod local;
pub mod meta;
pub mod reduce;
pub mod sig;
pub mod subst;
pub mod unify;
pub mod zonk;

pub use sig::Signature;

use db::SourceFile;

use crate::{
    ElabDatabase,
    ElabDb,
    ElaboratedFile,
    ElaboratedItem,
    ids::{
        ItemId,
        ItemKind,
    },
};

pub fn elaborate_decl<'db>(db: &'db dyn ElabDatabase, item: ItemId<'db>) -> ElaboratedItem<'db> {
    let signature = db.signature(item);
    let def_body = if item.kind(db) == ItemKind::Def {
        db.def_body(item).as_ref()
    } else {
        None
    };
    let inductive_data = match item.kind(db) {
        ItemKind::Inductive => Some(db.inductive_data(item)),
        _ => None,
    };
    ElaboratedItem {
        id: item,
        signature,
        def_body,
        inductive_data,
    }
}

pub fn elaborate_file(db: &dyn ElabDatabase, file: SourceFile) -> ElaboratedFile<'_> {
    let namespace = db.def_map(file);
    let lang_items = db.lang_items(file);
    let items = namespace
        .decls(db)
        .values()
        .map(|&id| db.elaborate_decl(id))
        .collect();
    ElaboratedFile {
        namespace,
        items,
        lang_items,
    }
}
