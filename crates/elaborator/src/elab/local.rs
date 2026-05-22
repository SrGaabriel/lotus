use text_size::TextRange;

use crate::{
    core::{
        BinderInfo,
        Term,
    },
    ids::{
        Symbol,
        Unique,
    },
};

#[derive(Debug, Clone)]
pub struct LocalBinder<'db> {
    pub unique: Unique,
    pub name: Option<Symbol<'db>>,
    pub ty: Term<'db>,
    pub info: BinderInfo,
    pub value: Option<Term<'db>>,
    pub origin: TextRange,
}

#[derive(Debug, Default, Clone)]
pub struct LocalCtx<'db> {
    binders: Vec<LocalBinder<'db>>,
}

impl<'db> LocalCtx<'db> {
    pub fn push(&mut self, b: LocalBinder<'db>) -> Unique {
        let u = b.unique;
        self.binders.push(b);
        u
    }

    pub fn pop(&mut self) -> Option<LocalBinder<'db>> {
        self.binders.pop()
    }

    pub fn level(&self) -> usize {
        self.binders.len()
    }

    pub fn pop_to(&mut self, l: usize) {
        self.binders.truncate(l);
    }

    pub fn find_by_name(&self, name: Symbol<'db>) -> Option<&LocalBinder<'db>> {
        self.binders.iter().rev().find(|b| b.name == Some(name))
    }

    pub fn find_by_unique(&self, u: Unique) -> Option<&LocalBinder<'db>> {
        self.binders.iter().rev().find(|b| b.unique == u)
    }

    pub fn iter(&self) -> impl Iterator<Item = &LocalBinder<'db>> {
        self.binders.iter()
    }

    pub fn len(&self) -> usize {
        self.binders.len()
    }

    pub fn is_empty(&self) -> bool {
        self.binders.is_empty()
    }
}
