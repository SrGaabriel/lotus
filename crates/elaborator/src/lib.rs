pub mod core;
pub mod elab;
pub mod env;
pub mod ids;

use ast::AstDatabase;
use db::SourceFile;

pub use crate::{
    elab::{
        elaborate_def,
        elaborate_file,
    },
    env::Body,
};
use crate::{
    env::{
        ItemTree,
        Namespace,
    },
    ids::{
        DefId,
        Symbol,
    },
};

#[salsa::db]
pub trait ElabDatabase: AstDatabase {}

#[salsa::db]
impl<DB: AstDatabase> ElabDatabase for DB {}

pub type Db<'db> = &'db dyn ElabDatabase;

pub trait ElabDb<'db> {
    fn intern_symbol(self, text: &str) -> Symbol<'db>;
    fn item_tree(self, file: SourceFile) -> ItemTree<'db>;
    fn def_map(self, file: SourceFile) -> Namespace<'db>;
    fn elaborate_def(self, def: DefId<'db>) -> &'db Body<'db>;
    fn elaborate_file(self, file: SourceFile);
}

impl<'db> ElabDb<'db> for Db<'db> {
    fn intern_symbol(self, text: &str) -> Symbol<'db> {
        Symbol::from_str(self, text)
    }

    fn item_tree(self, file: SourceFile) -> ItemTree<'db> {
        env::item_tree::item_tree(self, file)
    }

    fn def_map(self, file: SourceFile) -> Namespace<'db> {
        env::def_map::def_map(self, file)
    }

    fn elaborate_def(self, def: DefId<'db>) -> &'db Body<'db> {
        elab::elaborate_def(self, def)
    }

    fn elaborate_file(self, file: SourceFile) {
        elab::elaborate_file(self, file);
    }
}

#[macro_export]
macro_rules! sym {
    ($db:expr, $text:expr) => {
        $crate::ids::Symbol::new($db, ::std::string::String::from($text))
    };
}
