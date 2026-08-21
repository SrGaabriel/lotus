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
    let items = elaborator::env::item_tree::item_tree(db, file)
        .items(db)
        .iter()
        .filter(|item| item.parent(db).is_none())
        .filter_map(|&item| {
            lower_item(db, item).or_else(|| {
                tracing::debug!("erased '{}'", item.name(db).text(db));
                None
            })
        })
        .collect();

    NirFile::new(db, items)
}

#[salsa::tracked]
pub fn lower_item<'db>(db: &'db dyn NirDatabase, item: ItemId<'db>) -> Option<NirItem<'db>> {
    let signature = elaborator::elab::sig::signature(db, item);
    if signature.ty.has_error(db) {
        return None;
    }
    let ty = types::lower_type(db, signature.ty)?;
    let def_body = def_body(db, item).clone()?.value;
    if def_body.has_error(db) {
        return None;
    }
    let _body = lower_body(db, item, def_body);

    Some(NirItem::new(db, item, ty))
}
