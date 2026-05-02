use la_arena::{
    Arena,
    Idx,
};

use crate::unique::{
    Name,
    Unique,
};

pub type TermId = Idx<Term>;
pub type LevelId = Idx<Level>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TermArena {
    pub terms: Arena<Term>,
    pub levels: Arena<Level>,
}

impl TermArena {
    pub fn new() -> Self {
        Self {
            terms: Arena::new(),
            levels: Arena::new(),
        }
    }

    pub fn alloc_term(&mut self, term: Term) -> TermId {
        self.terms.alloc(term)
    }

    pub fn alloc_level(&mut self, level: Level) -> LevelId {
        self.levels.alloc(level)
    }

    pub fn mk_app(&mut self, l: TermId, r: TermId) -> TermId {
        self.alloc_term(Term::App(l, r))
    }

    pub fn mk_pi(&mut self, info: BinderInfo, param: TermId, body: TermId) -> TermId {
        self.alloc_term(Term::Pi(info, param, body))
    }

    pub fn mk_lam(&mut self, info: BinderInfo, param: TermId, body: TermId) -> TermId {
        self.alloc_term(Term::Lam(info, param, body))
    }

    pub fn mk_sigma(&mut self, info: BinderInfo, param: TermId, body: TermId) -> TermId {
        self.alloc_term(Term::Sigma(info, param, body))
    }

    pub fn mk_let(&mut self, ty: TermId, val: TermId, body: TermId) -> TermId {
        self.alloc_term(Term::Let(ty, val, body))
    }

    pub fn mk_sort(&mut self, level: LevelId) -> TermId {
        self.alloc_term(Term::Sort(level))
    }

    pub fn type0(&mut self) -> TermId {
        let zero = self.alloc_level(Level::Zero);
        let succ = self.alloc_level(Level::Succ(zero));
        self.alloc_term(Term::Sort(succ))
    }

    pub fn get_term(&self, id: TermId) -> &Term {
        &self.terms[id]
    }

    pub fn get_level(&self, id: LevelId) -> &Level {
        &self.levels[id]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BinderInfo {
    Explicit,
    Implicit,
    InstanceImplicit,
    StrictImplicit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Nat(u64),
    Str(String),
}

pub fn uncurry(arena: &TermArena, term: TermId) -> (TermId, Vec<(BinderInfo, TermId)>) {
    let mut args = Vec::new();
    let mut current = term;
    while let Term::Pi(info, param, body) = arena.get_term(current) {
        args.push((*info, *param));
        current = *body;
    }
    (current, args)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    BVar(usize),
    FVar(Name),
    MVar(Unique),
    App(TermId, TermId),
    Sort(LevelId),
    Const(Name),
    Lam(BinderInfo, TermId, TermId),
    Pi(BinderInfo, TermId, TermId),
    Sigma(BinderInfo, TermId, TermId),
    Let(TermId, TermId, TermId),
    Lit(Literal),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Zero,
    Succ(LevelId),
    Max(LevelId, LevelId),
    IMax(LevelId, LevelId),
    MVar(Unique),
    Param(Name),
}
