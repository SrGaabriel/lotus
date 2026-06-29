pub mod debug;
pub mod lower;
pub mod src;
pub mod types;

use db::SourceFile;
use elaborator::{
    ElabDatabase,
    ItemId,
};
use src::{
    NirFile,
    NirItem,
};

#[salsa::db]
pub trait NirDatabase: ElabDatabase {}

#[salsa::db]
impl<DB: ElabDatabase> NirDatabase for DB {}

pub type Db<'db> = &'db dyn NirDatabase;

pub trait NirDb<'db> {
    fn lower_file(self, file: SourceFile) -> &'db NirFile<'db>;
    fn lower_item(self, item: ItemId<'db>) -> Option<NirItem<'db>>;
}

impl<'db> NirDb<'db> for Db<'db> {
    fn lower_file(self, file: SourceFile) -> &'db NirFile<'db> {
        lower::lower_file(self, file)
    }

    fn lower_item(self, item: ItemId<'db>) -> Option<NirItem<'db>> {
        lower::lower_item(self, item)
    }
}
