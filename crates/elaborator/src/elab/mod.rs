pub mod ctx;
pub mod def;
pub mod expected;
pub mod inductive;
pub mod local;
pub mod meta;
pub mod sig;
pub mod subst;
pub mod unify;

pub use sig::Signature;

use db::SourceFile;

use crate::{
    ElabDatabase,
    ElabDb,
    ids::{
        ItemId,
        ItemKind,
    },
};

#[salsa::tracked]
pub fn elaborate_decl<'db>(db: &'db dyn ElabDatabase, item: ItemId<'db>) {
    let _ = db.signature(item);
    match item.kind(db) {
        ItemKind::Def => {
            let _ = db.def_body(item);
        }
        ItemKind::Inductive => {
            let _ = db.inductive_data(item);
        }
        ItemKind::Constructor => {}
    }
}

#[salsa::tracked]
pub fn elaborate_file(db: &dyn ElabDatabase, file: SourceFile) {
    let ns = db.def_map(file);
    for item in ns.decls(db).values() {
        db.elaborate_decl(*item);
    }
}
