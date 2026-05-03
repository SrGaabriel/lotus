use db::SourceFile;

use crate::Db;

#[salsa::interned(debug)]
pub struct Symbol<'db> {
    #[returns(ref)]
    pub text: String,
}

impl<'db> Symbol<'db> {
    pub fn from_str(db: Db<'db>, text: &str) -> Self {
        Self::new(db, text.to_owned())
    }

    pub fn as_str(self, db: Db<'db>) -> &'db str {
        self.text(db).as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
pub struct Unique(pub u32);

#[derive(Debug, Default)]
pub struct UniqueGen {
    next: u32,
}

impl UniqueGen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh(&mut self) -> Unique {
        let id = self.next;
        self.next += 1;
        Unique(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
pub enum ItemKind {
    Def,
}

#[salsa::tracked(debug)]
pub struct DefId<'db> {
    pub file: SourceFile,
    pub name: Symbol<'db>,
    pub ast_index: u32,
    pub kind: ItemKind,
}
