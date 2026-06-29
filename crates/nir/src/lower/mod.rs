use crate::src::{
    NirFile,
    NirItem,
};
use db::SourceFile;
use elaborator::ItemId;

use crate::NirDatabase;

pub mod expr;
pub mod types;

#[salsa::tracked(returns(ref))]
pub fn lower_file(db: &dyn NirDatabase, file: SourceFile) -> NirFile<'_> {
    let namespace = elaborator::env::def_map::def_map(db, file);
    let items = namespace
        .decls(db)
        .values()
        .filter_map(|&item| lower_item(db, item))
        .collect();

    NirFile::new(db, items)
}

#[salsa::tracked]
pub fn lower_item<'db>(db: &'db dyn NirDatabase, item: ItemId<'db>) -> Option<NirItem<'db>> {
    let signature = elaborator::elab::sig::signature(db, item);
    let ty = types::lower_type(db, signature.ty)?;

    Some(NirItem::new(db, item, ty))
}
