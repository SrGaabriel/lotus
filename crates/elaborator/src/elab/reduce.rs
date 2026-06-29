use crate::{
    ElabDatabase,
    ElabDb,
    core::{
        Term,
        TermKind,
    },
    elab::{
        ctx::ElabCtx,
        subst::instantiate,
    },
    ids::Unique,
};

impl<'db> ElabCtx<'db> {
    pub fn whnf(&self, term: Term<'db>) -> Term<'db> {
        whnf_with_mvars(self.db, term, |u| self.mctx.get_solution(u).copied())
    }
}

pub fn whnf<'db>(db: &'db dyn ElabDatabase, term: Term<'db>) -> Term<'db> {
    whnf_with_mvars(db, term, |_| None)
}

pub fn whnf_spine<'db>(db: &'db dyn ElabDatabase, term: Term<'db>) -> (Term<'db>, Vec<Term<'db>>) {
    let mut current = whnf(db, term);
    let mut args = Vec::new();

    while let TermKind::App(f, x) = current.kind(db) {
        args.push(*x);
        current = whnf(db, *f);
    }

    args.reverse();
    (current, args)
}

pub fn instantiate_many<'db>(
    db: &'db dyn ElabDatabase,
    term: Term<'db>,
    args: impl IntoIterator<Item = Term<'db>>,
) -> Term<'db> {
    let mut current = term;
    for arg in args {
        current = whnf(db, current);
        current = match current.kind(db) {
            TermKind::Lam(_, _, body) => instantiate(db, body, arg),
            _ => Term::app(db, current, arg),
        };
    }
    whnf(db, current)
}

fn whnf_with_mvars<'db>(
    db: &'db dyn ElabDatabase,
    term: Term<'db>,
    resolve_mvar: impl Fn(Unique) -> Option<Term<'db>> + Copy,
) -> Term<'db> {
    match term.kind(db) {
        TermKind::App(f, x) => {
            let f = whnf_with_mvars(db, *f, resolve_mvar);
            if let TermKind::Lam(_, _, body) = f.kind(db) {
                whnf_with_mvars(db, instantiate(db, body, *x), resolve_mvar)
            } else {
                Term::app(db, f, *x)
            }
        }

        TermKind::MVar(u) => {
            if let Some(value) = resolve_mvar(*u) {
                whnf_with_mvars(db, value, resolve_mvar)
            } else {
                term
            }
        }

        TermKind::Let(_, value, body) => {
            whnf_with_mvars(db, instantiate(db, body, *value), resolve_mvar)
        }

        TermKind::Const(name) => {
            if let Some(body) = db.def_body(*name) {
                whnf_with_mvars(db, body.value, resolve_mvar)
            } else {
                term
            }
        }

        _ => term,
    }
}
