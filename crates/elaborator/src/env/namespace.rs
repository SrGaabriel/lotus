use rustc_hash::FxHashMap;
use strsim::osa_distance;

use crate::{
    Db,
    ids::{
        ItemId,
        Qualified,
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

    pub fn similar(self, db: Db<'db>, qual: &Qualified<'db>) -> Option<Qualified<'db>> {
        let mut path = qual.path.clone();
        let mut current = self;
        for seg in &mut path {
            current = if let Some(child) = current.children(db).get(seg).copied() {
                child
            } else {
                *seg = best_match(db, *seg, current.children(db).keys().copied())?;
                current.children(db).get(seg).copied()?
            };
        }

        let member = best_match(db, qual.member, current.decls(db).keys().copied())?;
        let suggestion = Qualified { path, member };
        (suggestion != *qual).then_some(suggestion)
    }
}

pub fn best_match<'db>(
    db: Db<'db>,
    target: Symbol<'db>,
    candidates: impl IntoIterator<Item = Symbol<'db>>,
) -> Option<Symbol<'db>> {
    let target = target.text(db);
    let budget = target.chars().count().max(3) / 3;

    candidates
        .into_iter()
        .filter_map(|name| {
            let text = name.text(db);
            if text.chars().count().abs_diff(target.chars().count()) > budget {
                return None;
            }
            let dist = osa_distance(target, text);
            (dist <= budget).then_some((dist, text, name))
        })
        .min_by(|lhs, rhs| lhs.0.cmp(&rhs.0).then_with(|| lhs.1.cmp(rhs.1)))
        .map(|(_, _, name)| name)
}
