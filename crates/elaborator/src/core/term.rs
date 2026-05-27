use crate::{
    Db,
    ids::{
        ItemId,
        Symbol,
        Unique,
    },
};

#[salsa::interned(debug)]
pub struct Term<'db> {
    #[returns(ref)]
    pub kind: TermKind<'db>,
}

#[salsa::interned(debug)]
pub struct Level<'db> {
    #[returns(ref)]
    pub kind: LevelKind<'db>,
}

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
pub enum TermKind<'db> {
    BVar(usize),
    FVar(Unique),
    MVar(Unique),
    App(Term<'db>, Term<'db>),
    Sort(Level<'db>),
    Const(ItemId<'db>),
    Lam(BinderInfo, Term<'db>, Term<'db>),
    Pi(BinderInfo, Term<'db>, Term<'db>),
    Sigma(BinderInfo, Term<'db>, Term<'db>),
    Let(Term<'db>, Term<'db>, Term<'db>),
    Lit(Literal),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum LevelKind<'db> {
    Zero,
    Succ(Level<'db>),
    Max(Level<'db>, Level<'db>),
    IMax(Level<'db>, Level<'db>),
    MVar(Unique),
    Param(Symbol<'db>),
}

impl<'db> Term<'db> {
    pub fn bvar(db: Db<'db>, i: usize) -> Self {
        Term::new(db, TermKind::BVar(i))
    }

    pub fn fvar(db: Db<'db>, u: Unique) -> Self {
        Term::new(db, TermKind::FVar(u))
    }

    pub fn mvar(db: Db<'db>, u: Unique) -> Self {
        Term::new(db, TermKind::MVar(u))
    }

    pub fn app(db: Db<'db>, f: Term<'db>, x: Term<'db>) -> Self {
        Term::new(db, TermKind::App(f, x))
    }

    pub fn sort(db: Db<'db>, level: Level<'db>) -> Self {
        Term::new(db, TermKind::Sort(level))
    }

    pub fn constant(db: Db<'db>, item: ItemId<'db>) -> Self {
        Term::new(db, TermKind::Const(item))
    }

    pub fn lam(db: Db<'db>, info: BinderInfo, ty: Term<'db>, body: Term<'db>) -> Self {
        Term::new(db, TermKind::Lam(info, ty, body))
    }

    pub fn pi(db: Db<'db>, info: BinderInfo, ty: Term<'db>, body: Term<'db>) -> Self {
        Term::new(db, TermKind::Pi(info, ty, body))
    }

    pub fn sigma(db: Db<'db>, info: BinderInfo, ty: Term<'db>, body: Term<'db>) -> Self {
        Term::new(db, TermKind::Sigma(info, ty, body))
    }

    pub fn let_(db: Db<'db>, ty: Term<'db>, value: Term<'db>, body: Term<'db>) -> Self {
        Term::new(db, TermKind::Let(ty, value, body))
    }

    pub fn lit(db: Db<'db>, lit: Literal) -> Self {
        Term::new(db, TermKind::Lit(lit))
    }

    pub fn type0(db: Db<'db>) -> Self {
        Term::sort(db, Level::one(db))
    }

    pub fn debug(self, db: Db<'db>) -> TermDisplay<'db> {
        TermDisplay { db, term: self }
    }
}

impl<'db> Level<'db> {
    pub fn zero(db: Db<'db>) -> Self {
        Level::new(db, LevelKind::Zero)
    }

    pub fn one(db: Db<'db>) -> Self {
        let zero = Level::new(db, LevelKind::Zero);
        Level::new(db, LevelKind::Succ(zero))
    }

    pub fn two(db: Db<'db>) -> Self {
        let one = Level::one(db);
        Level::new(db, LevelKind::Succ(one))
    }

    pub fn succ(db: Db<'db>, inner: Level<'db>) -> Self {
        Level::new(db, LevelKind::Succ(inner))
    }

    pub fn mvar(db: Db<'db>, u: Unique) -> Self {
        Level::new(db, LevelKind::MVar(u))
    }
}

pub fn uncurry<'db>(db: Db<'db>, term: Term<'db>) -> (Term<'db>, Vec<(BinderInfo, Term<'db>)>) {
    let mut args = Vec::new();
    let mut current = term;
    while let TermKind::Pi(info, param, body) = current.kind(db) {
        args.push((*info, *param));
        current = *body;
    }
    (current, args)
}

pub struct TermDisplay<'db> {
    pub db: Db<'db>,
    pub term: Term<'db>,
}

impl std::fmt::Display for TermDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let child = |term| TermDisplay { db: self.db, term };
        match self.term.kind(self.db) {
            TermKind::BVar(i) => write!(f, "#{i}"),
            TermKind::Const(d) => write!(f, "{}", d.name(self.db).text(self.db)),
            TermKind::App(g, x) => write!(f, "({} {})", child(*g), child(*x)),
            TermKind::Lam(info, ty, body) => {
                write!(f, "(λ {:?} : {} => {})", info, child(*ty), child(*body))
            }
            TermKind::Pi(info, ty, body) => {
                write!(f, "({:?} {} -> {})", info, child(*ty), child(*body))
            }
            TermKind::Sigma(info, ty, body) => {
                write!(f, "(Σ {:?} {} , {})", info, child(*ty), child(*body))
            }
            TermKind::Let(ty, val, body) => write!(
                f,
                "(let : {} := {} ; {})",
                child(*ty),
                child(*val),
                child(*body)
            ),
            TermKind::Sort(_) => write!(f, "Sort ?"),
            TermKind::Lit(lit) => write!(f, "{lit:?}"),
            TermKind::FVar(u) => write!(f, "?f{}", u.0),
            TermKind::MVar(u) => write!(f, "?m{}", u.0),
        }
    }
}

pub struct FreeBinder<'db> {
    pub fvar: Unique,
    pub info: BinderInfo,
    pub ty: Term<'db>,
}

impl<'db> FreeBinder<'db> {
    pub fn new(fvar: Unique, info: BinderInfo, ty: Term<'db>) -> Self {
        Self { fvar, info, ty }
    }
}
