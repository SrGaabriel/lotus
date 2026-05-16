use rustc_hash::FxHashMap;

use crate::{
    Db,
    ids::{
        ItemId,
        Symbol,
    },
};

pub type SymbolMap<'db, T> = FxHashMap<Symbol<'db>, T>;
pub type ItemMap<'db> = SymbolMap<'db, ItemId<'db>>;

#[salsa::tracked]
pub struct Namespace<'db> {
    #[tracked]
    #[returns(ref)]
    pub decls: ItemMap<'db>,
    #[tracked]
    #[returns(ref)]
    pub children: SymbolMap<'db, Namespace<'db>>,
}

impl<'db> Namespace<'db> {
    pub fn walk(self, db: Db<'db>, path: &[Symbol<'db>]) -> Option<Namespace<'db>> {
        let mut current = self;
        for seg in path {
            current = current.children(db).get(seg).copied()?;
        }
        Some(current)
    }

    pub fn resolve(
        self,
        db: Db<'db>,
        path: &[Symbol<'db>],
        member: Symbol<'db>,
    ) -> Option<ItemId<'db>> {
        self.walk(db, path)?.decls(db).get(&member).copied()
    }
}
