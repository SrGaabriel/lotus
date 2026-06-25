use crate::src::{
    PirFile,
    PirItem,
};
use db::SourceFile;
use elaborator::ItemId;

use crate::PirDatabase;

pub mod types;

#[salsa::tracked(returns(ref))]
pub fn lower_file(db: &dyn PirDatabase, file: SourceFile) -> PirFile<'_> {
    let namespace = elaborator::env::def_map::def_map(db, file);
    let items = namespace
        .decls(db)
        .values()
        .filter_map(|&item| lower_item(db, item))
        .collect();

    PirFile::new(db, items)
}

#[salsa::tracked]
pub fn lower_item<'db>(db: &'db dyn PirDatabase, item: ItemId<'db>) -> Option<PirItem<'db>> {
    let signature = elaborator::elab::sig::signature(db, item);
    let ty = types::lower_type(db, signature.ty)?;

    Some(PirItem::new(db, item, ty))
}
