use crate::ids::{
    DefId,
    Unique,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
pub struct TermId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
pub struct LevelId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
pub enum BinderInfo {
    Explicit,
    Implicit,
    InstanceImplicit,
    StrictImplicit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum Literal {
    Nat(u64),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum Term<'db> {
    BVar(usize),
    FVar(Unique),
    MVar(Unique),
    App(TermId, TermId),
    Sort(LevelId),
    Const(DefId<'db>),
    Lam(BinderInfo, TermId, TermId),
    Pi(BinderInfo, TermId, TermId),
    Sigma(BinderInfo, TermId, TermId),
    Let(TermId, TermId, TermId),
    Lit(Literal),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum Level<'db> {
    Zero,
    Succ(LevelId),
    Max(LevelId, LevelId),
    IMax(LevelId, LevelId),
    MVar(Unique),
    Param(crate::ids::Symbol<'db>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, salsa::Update)]
pub struct TermArena<'db> {
    pub terms: Vec<Term<'db>>,
    pub levels: Vec<Level<'db>>,
}

impl<'db> TermArena<'db> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_term(&mut self, term: Term<'db>) -> TermId {
        let id = TermId(self.terms.len() as u32);
        self.terms.push(term);
        id
    }

    pub fn alloc_level(&mut self, level: Level<'db>) -> LevelId {
        let id = LevelId(self.levels.len() as u32);
        self.levels.push(level);
        id
    }

    pub fn get_term(&self, id: TermId) -> &Term<'db> {
        &self.terms[id.0 as usize]
    }

    pub fn get_level(&self, id: LevelId) -> &Level<'db> {
        &self.levels[id.0 as usize]
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
