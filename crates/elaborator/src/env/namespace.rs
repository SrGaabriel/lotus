use rustc_hash::FxHashMap;

use crate::{
    Db,
    ids::{
        DefId,
        Symbol,
    },
};

#[salsa::tracked]
pub struct Namespace<'db> {
    #[tracked]
    #[returns(ref)]
    pub decls: FxHashMap<Symbol<'db>, DefId<'db>>,
    #[tracked]
    #[returns(ref)]
    pub children: FxHashMap<Symbol<'db>, Namespace<'db>>,
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
    ) -> Option<DefId<'db>> {
        self.walk(db, path)?.decls(db).get(&member).copied()
    }
}
