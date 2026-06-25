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
    PirFile,
    PirItem,
};

#[salsa::db]
pub trait PirDatabase: ElabDatabase {}

#[salsa::db]
impl<DB: ElabDatabase> PirDatabase for DB {}

pub type Db<'db> = &'db dyn PirDatabase;

pub trait PirDb<'db> {
    fn lower_file(self, file: SourceFile) -> &'db PirFile<'db>;
    fn lower_item(self, item: ItemId<'db>) -> Option<PirItem<'db>>;
}

impl<'db> PirDb<'db> for Db<'db> {
    fn lower_file(self, file: SourceFile) -> &'db PirFile<'db> {
        lower::lower_file(self, file)
    }

    fn lower_item(self, item: ItemId<'db>) -> Option<PirItem<'db>> {
        lower::lower_item(self, item)
    }
}
