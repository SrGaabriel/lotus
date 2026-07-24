use rustc_hash::FxHashMap;

use crate::{
    core::Term,
    elab::local::LocalCtx,
    ids::Unique,
};

pub struct Meta<'db> {
    pub id: Unique,
    pub ty: Term<'db>,
    pub lctx: LocalCtx<'db>,
}

pub struct MetaCtx<'db> {
    metas: Vec<Meta<'db>>,
    solutions: FxHashMap<Unique, Term<'db>>,
}

impl<'db> MetaCtx<'db> {
    pub fn new() -> Self {
        Self {
            metas: Vec::new(),
            solutions: FxHashMap::default(),
        }
    }

    pub fn register_meta(&mut self, id: Unique, ty: Term<'db>, lctx: LocalCtx<'db>) {
        self.metas.push(Meta { id, ty, lctx });
    }

    pub fn solve_meta(&mut self, id: Unique, value: Term<'db>) {
        assert!(!self.is_solved(id), "meta is already solved");
        self.solutions.insert(id, value);
    }

    pub fn is_solved(&self, id: Unique) -> bool {
        self.solutions.contains_key(&id)
    }

    pub fn get_meta(&self, id: Unique) -> Option<&Meta<'db>> {
        self.metas.iter().find(|m| m.id == id)
    }

    pub fn get_solution(&self, id: Unique) -> Option<&Term<'db>> {
        self.solutions.get(&id)
    }
}

impl Default for MetaCtx<'_> {
    fn default() -> Self {
        Self::new()
    }
}
