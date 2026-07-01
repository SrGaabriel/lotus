use crate::{
    lower::expr::lower_body,
    src::{
        NirFile,
        NirItem,
    },
};
use db::SourceFile;
use elaborator::{
    ItemId,
    elab::def::def_body,
};

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
    let def_body = def_body(db, item).to_owned()?.value;
    let _body = lower_body(db, item, def_body);

    Some(NirItem::new(db, item, ty))
}
