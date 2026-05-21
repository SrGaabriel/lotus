use crate::{
    ElabDatabase,
    ids::{
        ItemId,
        Symbol,
        Unique,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
#[repr(transparent)]
pub struct TermId(pub u32);

impl TermId {
    pub fn debug<'a, 'db>(
        &self,
        db: &'db dyn ElabDatabase,
        arena: &'a TermArena<'db>,
    ) -> TermDisplay<'a, 'db> {
        TermDisplay {
            db,
            arena,
            term: *self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
#[repr(transparent)]
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
    Number(u64),
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum Term<'db> {
    BVar(usize),
    FVar(Unique),
    MVar(Unique),
    App(TermId, TermId),
    Sort(LevelId),
    Const(ItemId<'db>),
    Lam(BinderInfo, TermId, TermId),
    Pi(BinderInfo, TermId, TermId),
    Sigma(BinderInfo, TermId, TermId),
    Let(TermId, TermId, TermId),
    Lit(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum Level<'db> {
    Zero,
    Succ(LevelId),
    Max(LevelId, LevelId),
    IMax(LevelId, LevelId),
    MVar(Unique),
    Param(Symbol<'db>),
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

    pub fn mk_const(&mut self, item: ItemId<'db>) -> TermId {
        self.alloc_term(Term::Const(item))
    }

    pub fn mk_lit(&mut self, lit: Literal) -> TermId {
        self.alloc_term(Term::Lit(lit))
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

pub struct TermDisplay<'a, 'db> {
    pub db: &'db dyn ElabDatabase,
    pub arena: &'a TermArena<'db>,
    pub term: TermId,
}

impl std::fmt::Display for TermDisplay<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let child = |id| TermDisplay {
            db: self.db,
            arena: self.arena,
            term: id,
        };
        match self.arena.get_term(self.term) {
            Term::BVar(i) => write!(f, "#{i}"),
            Term::Const(d) => write!(f, "{}", d.name(self.db).text(self.db)),
            Term::App(g, x) => write!(f, "({} {})", child(*g), child(*x)),
            Term::Lam(info, ty, body) => {
                write!(f, "(λ {:?} : {} => {})", info, child(*ty), child(*body))
            }
            Term::Pi(info, ty, body) => {
                write!(f, "({:?} {} -> {})", info, child(*ty), child(*body))
            }
            Term::Sigma(info, ty, body) => {
                write!(f, "(Σ {:?} {} , {})", info, child(*ty), child(*body))
            }
            Term::Let(ty, val, body) => write!(
                f,
                "(let : {} := {} ; {})",
                child(*ty),
                child(*val),
                child(*body)
            ),
            Term::Sort(_) => write!(f, "Sort ?"),
            Term::Lit(lit) => write!(f, "{lit:?}"),
            Term::FVar(u) => write!(f, "?f{}", u.0),
            Term::MVar(u) => write!(f, "?m{}", u.0),
        }
    }
}
