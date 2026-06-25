use tracing::instrument;

use crate::{
    ElabDatabase,
    core::{
        Term,
        TermKind,
    },
    ids::Unique,
};

pub fn abstract_fvar<'db>(db: &'db dyn ElabDatabase, term: &Term<'db>, fvar: Unique) -> Term<'db> {
    abstract_fvar_at(db, term, fvar, 0)
}

#[instrument(skip(db))]
pub fn abstract_fvar_at<'db>(
    db: &'db dyn ElabDatabase,
    term: &Term<'db>,
    fvar: Unique,
    index: usize,
) -> Term<'db> {
    match term.kind(db) {
        TermKind::BVar(i) if *i >= index => Term::bvar(db, i + 1),
        TermKind::BVar(i) => Term::bvar(db, *i),
        TermKind::FVar(u) if *u == fvar => Term::bvar(db, index),
        TermKind::FVar(u) => Term::fvar(db, *u),
        TermKind::MVar(u) => Term::mvar(db, *u),
        TermKind::Const(d) => Term::constant(db, *d),
        TermKind::Sort(l) => Term::sort(db, *l),
        TermKind::App(f, x) => {
            let f = abstract_fvar_at(db, f, fvar, index);
            let x = abstract_fvar_at(db, x, fvar, index);
            Term::app(db, f, x)
        }
        TermKind::Lam(info, ty, body) => {
            let ty = abstract_fvar_at(db, ty, fvar, index);
            let body = abstract_fvar_at(db, body, fvar, index + 1);
            Term::lam(db, *info, ty, body)
        }
        TermKind::Pi(info, ty, body) => {
            let ty = abstract_fvar_at(db, ty, fvar, index);
            let body = abstract_fvar_at(db, body, fvar, index + 1);
            Term::pi(db, *info, ty, body)
        }
        TermKind::Sigma(info, ty, body) => {
            let ty = abstract_fvar_at(db, ty, fvar, index);
            let body = abstract_fvar_at(db, body, fvar, index + 1);
            Term::sigma(db, *info, ty, body)
        }
        TermKind::Let(ty, value, body) => {
            let ty = abstract_fvar_at(db, ty, fvar, index);
            let value = abstract_fvar_at(db, value, fvar, index);
            let body = abstract_fvar_at(db, body, fvar, index + 1);
            Term::let_(db, ty, value, body)
        }
        TermKind::Lit(lit) => Term::lit(db, lit.clone()),
    }
}

pub fn instantiate<'db>(
    db: &'db dyn ElabDatabase,
    term: &Term<'db>,
    replacement: Term<'db>,
) -> Term<'db> {
    instantiate_at(db, term, replacement, 0)
}

#[instrument(skip(db))]
pub fn instantiate_at<'db>(
    db: &'db dyn ElabDatabase,
    term: &Term<'db>,
    replacement: Term<'db>,
    index: usize,
) -> Term<'db> {
    match term.kind(db) {
        TermKind::BVar(i) if *i == index => shift(db, &replacement, index),
        TermKind::BVar(i) if *i > index => Term::bvar(db, i - 1),
        TermKind::BVar(_)
        | TermKind::FVar(_)
        | TermKind::MVar(_)
        | TermKind::Const(_)
        | TermKind::Lit(_)
        | TermKind::Sort(_) => *term,

        TermKind::App(f, x) => {
            let f = instantiate_at(db, f, replacement, index);
            let x = instantiate_at(db, x, replacement, index);
            Term::app(db, f, x)
        }
        TermKind::Lam(info, ty, body) => {
            let ty = instantiate_at(db, ty, replacement, index);
            let body = instantiate_at(db, body, replacement, index + 1);
            Term::lam(db, *info, ty, body)
        }
        TermKind::Pi(info, ty, body) => {
            let ty = instantiate_at(db, ty, replacement, index);
            let body = instantiate_at(db, body, replacement, index + 1);
            Term::pi(db, *info, ty, body)
        }
        TermKind::Sigma(info, ty, body) => {
            let ty = instantiate_at(db, ty, replacement, index);
            let body = instantiate_at(db, body, replacement, index + 1);
            Term::sigma(db, *info, ty, body)
        }
        TermKind::Let(ty, value, body) => {
            let ty = instantiate_at(db, ty, replacement, index);
            let value = instantiate_at(db, value, replacement, index);
            let body = instantiate_at(db, body, replacement, index + 1);
            Term::let_(db, ty, value, body)
        }
    }
}

pub fn shift<'db>(db: &'db dyn ElabDatabase, term: &Term<'db>, offset: usize) -> Term<'db> {
    if offset == 0 {
        *term
    } else {
        shift_at(db, term, offset, 0)
    }
}

pub fn shift_at<'db>(
    db: &'db dyn ElabDatabase,
    term: &Term<'db>,
    offset: usize,
    index: usize,
) -> Term<'db> {
    match term.kind(db) {
        TermKind::BVar(i) if *i >= index => Term::bvar(db, i + offset),
        TermKind::App(f, x) => {
            let f = shift_at(db, f, offset, index);
            let x = shift_at(db, x, offset, index);
            Term::app(db, f, x)
        }
        TermKind::Lam(info, ty, body) => {
            let ty = shift_at(db, ty, offset, index);
            let body = shift_at(db, body, offset, index + 1);
            Term::lam(db, *info, ty, body)
        }
        TermKind::Pi(info, ty, body) => {
            let ty = shift_at(db, ty, offset, index);
            let body = shift_at(db, body, offset, index + 1);
            Term::pi(db, *info, ty, body)
        }
        TermKind::Sigma(info, ty, body) => {
            let ty = shift_at(db, ty, offset, index);
            let body = shift_at(db, body, offset, index + 1);
            Term::sigma(db, *info, ty, body)
        }
        TermKind::Let(ty, value, body) => {
            let ty = shift_at(db, ty, offset, index);
            let value = shift_at(db, value, offset, index);
            let body = shift_at(db, body, offset, index + 1);
            Term::let_(db, ty, value, body)
        }
        TermKind::Lit(lit) => Term::lit(db, lit.clone()),
        TermKind::FVar(_)
        | TermKind::BVar(_)
        | TermKind::MVar(_)
        | TermKind::Const(_)
        | TermKind::Sort(_) => *term,
    }
}
