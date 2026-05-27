pub mod core;
pub mod elab;
pub mod env;
pub mod ids;
pub mod util;

use std::fmt::Debug;

use ast::AstDatabase;
use db::SourceFile;

pub use crate::{
    elab::{
        Signature,
        elaborate_decl,
        elaborate_file,
        inductive::InductiveData,
    },
    env::DefBody,
    ids::{
        ItemId,
        ItemKind,
    },
};
use crate::{
    env::{
        ItemTree,
        Namespace,
        lang_items::LanguageItems,
    },
    ids::Symbol,
};

pub struct ElaboratedFile<'db> {
    pub namespace: Namespace<'db>,
    pub items: Vec<ElaboratedItem<'db>>,
}

pub struct ElaboratedItem<'db> {
    pub id: ItemId<'db>,
    pub signature: &'db Signature<'db>,
    pub def_body: Option<&'db DefBody<'db>>,
    pub inductive_data: Option<&'db InductiveData<'db>>,
}

impl Debug for ElaboratedFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElaboratedFile")
            .field("items", &self.items.len())
            .finish()
    }
}

impl Debug for ElaboratedItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElaboratedItem")
            .field("signature", &self.signature)
            .field("def_body", &self.def_body)
            .finish()
    }
}

#[salsa::db]
pub trait ElabDatabase: AstDatabase {}

#[salsa::db]
impl<DB: AstDatabase> ElabDatabase for DB {}

pub type Db<'db> = &'db dyn ElabDatabase;

pub trait ElabDb<'db> {
    fn intern_symbol(self, text: &str) -> Symbol<'db>;
    fn item_tree(self, file: SourceFile) -> ItemTree<'db>;
    fn lang_items(self, file: SourceFile) -> &'db LanguageItems<'db>;
    fn def_map(self, file: SourceFile) -> Namespace<'db>;
    fn signature(self, item: ItemId<'db>) -> &'db Signature<'db>;
    fn def_body(self, item: ItemId<'db>) -> &'db Option<DefBody<'db>>;
    fn inductive_data(self, item: ItemId<'db>) -> &'db InductiveData<'db>;
    fn elaborate_decl(self, item: ItemId<'db>);
    fn elaborate_file(self, file: SourceFile);
    fn dbg_elaborate_file(self, file: SourceFile) -> ElaboratedFile<'db>;
}

impl<'db> ElabDb<'db> for Db<'db> {
    fn intern_symbol(self, text: &str) -> Symbol<'db> {
        Symbol::from_str(self, text)
    }

    fn item_tree(self, file: SourceFile) -> ItemTree<'db> {
        env::item_tree::item_tree(self, file)
    }

    fn lang_items(self, file: SourceFile) -> &'db LanguageItems<'db> {
        env::lang_items::file_lang_items(self, file)
    }

    fn def_map(self, file: SourceFile) -> Namespace<'db> {
        env::def_map::def_map(self, file)
    }

    fn signature(self, item: ItemId<'db>) -> &'db Signature<'db> {
        elab::sig::signature(self, item)
    }

    fn def_body(self, item: ItemId<'db>) -> &'db Option<DefBody<'db>> {
        elab::def::def_body(self, item)
    }

    fn inductive_data(self, item: ItemId<'db>) -> &'db InductiveData<'db> {
        elab::inductive::inductive_data(self, item)
    }

    fn elaborate_decl(self, item: ItemId<'db>) {
        elab::elaborate_decl(self, item);
    }

    fn elaborate_file(self, file: SourceFile) {
        elab::elaborate_file(self, file);
    }

    fn dbg_elaborate_file(self, file: SourceFile) -> ElaboratedFile<'db> {
        tracing::info!("Elaborating file: {:?}", file);
        let namespace = self.def_map(file);
        let items = namespace
            .decls(self)
            .values()
            .map(|&id| {
                let signature = self.signature(id);
                let def_body = if id.kind(self) == ItemKind::Def {
                    self.def_body(id).as_ref()
                } else {
                    None
                };
                let inductive_data = match id.kind(self) {
                    ItemKind::Inductive => Some(self.inductive_data(id)),
                    _ => None,
                };
                ElaboratedItem {
                    id,
                    signature,
                    def_body,
                    inductive_data,
                }
            })
            .collect();
        ElaboratedFile { namespace, items }
    }
}

#[macro_export]
macro_rules! sym {
    ($db:expr, $text:expr) => {
        $crate::ids::Symbol::new($db, ::std::string::String::from($text))
    };
}
