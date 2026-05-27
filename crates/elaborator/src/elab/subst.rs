use tracing::instrument;

use crate::{
    core::{
        Term,
        TermKind,
    },
    elab::ctx::ElabCtx,
    ids::Unique,
};

impl<'db> ElabCtx<'db> {
    pub fn abstract_fvar(&self, term: &Term<'db>, fvar: Unique) -> Term<'db> {
        self.abstract_fvar_at(term, fvar, 0)
    }

    #[instrument(skip(self))]
    pub fn abstract_fvar_at(&self, term: &Term<'db>, fvar: Unique, index: usize) -> Term<'db> {
        match term.kind(self.db) {
            TermKind::BVar(i) if *i >= index => Term::bvar(self.db, i + 1),
            TermKind::BVar(i) => Term::bvar(self.db, *i),
            TermKind::FVar(u) if *u == fvar => Term::bvar(self.db, index),
            TermKind::FVar(u) => Term::fvar(self.db, *u),
            TermKind::MVar(u) => Term::mvar(self.db, *u),
            TermKind::Const(d) => Term::constant(self.db, *d),
            TermKind::Sort(l) => Term::sort(self.db, *l),
            TermKind::App(f, x) => {
                let f = self.abstract_fvar_at(f, fvar, index);
                let x = self.abstract_fvar_at(x, fvar, index);
                Term::app(self.db, f, x)
            }
            TermKind::Lam(info, ty, body) => {
                let ty = self.abstract_fvar_at(ty, fvar, index);
                let body = self.abstract_fvar_at(body, fvar, index + 1);
                Term::lam(self.db, *info, ty, body)
            }
            TermKind::Pi(info, ty, body) => {
                let ty = self.abstract_fvar_at(ty, fvar, index);
                let body = self.abstract_fvar_at(body, fvar, index + 1);
                Term::pi(self.db, *info, ty, body)
            }
            TermKind::Sigma(info, ty, body) => {
                let ty = self.abstract_fvar_at(ty, fvar, index);
                let body = self.abstract_fvar_at(body, fvar, index + 1);
                Term::sigma(self.db, *info, ty, body)
            }
            TermKind::Let(ty, value, body) => {
                let ty = self.abstract_fvar_at(ty, fvar, index);
                let value = self.abstract_fvar_at(value, fvar, index);
                let body = self.abstract_fvar_at(body, fvar, index + 1);
                Term::let_(self.db, ty, value, body)
            }
            TermKind::Lit(lit) => Term::lit(self.db, lit.clone()),
        }
    }

    pub fn instantiate(&self, term: &Term<'db>, replacement: Term<'db>) -> Term<'db> {
        self.instantiate_at(term, replacement, 0)
    }

    #[instrument(skip(self))]
    pub fn instantiate_at(
        &self,
        term: &Term<'db>,
        replacement: Term<'db>,
        index: usize,
    ) -> Term<'db> {
        match term.kind(self.db) {
            TermKind::BVar(i) if *i == index => self.shift(&replacement, index),
            TermKind::BVar(i) if *i > index => Term::bvar(self.db, i - 1),
            TermKind::BVar(_)
            | TermKind::FVar(_)
            | TermKind::MVar(_)
            | TermKind::Const(_)
            | TermKind::Lit(_)
            | TermKind::Sort(_) => *term,

            TermKind::App(f, x) => {
                let f = self.instantiate_at(f, replacement, index);
                let x = self.instantiate_at(x, replacement, index);
                Term::app(self.db, f, x)
            }
            TermKind::Lam(info, ty, body) => {
                let ty = self.instantiate_at(ty, replacement, index);
                let body = self.instantiate_at(body, replacement, index + 1);
                Term::lam(self.db, *info, ty, body)
            }
            TermKind::Pi(info, ty, body) => {
                let ty = self.instantiate_at(ty, replacement, index);
                let body = self.instantiate_at(body, replacement, index + 1);
                Term::pi(self.db, *info, ty, body)
            }
            TermKind::Sigma(info, ty, body) => {
                let ty = self.instantiate_at(ty, replacement, index);
                let body = self.instantiate_at(body, replacement, index + 1);
                Term::sigma(self.db, *info, ty, body)
            }
            TermKind::Let(ty, value, body) => {
                let ty = self.instantiate_at(ty, replacement, index);
                let value = self.instantiate_at(value, replacement, index);
                let body = self.instantiate_at(body, replacement, index + 1);
                Term::let_(self.db, ty, value, body)
            }
        }
    }

    pub fn shift(&self, term: &Term<'db>, offset: usize) -> Term<'db> {
        if offset == 0 {
            *term
        } else {
            self.shift_at(term, offset, 0)
        }
    }

    pub fn shift_at(&self, term: &Term<'db>, offset: usize, index: usize) -> Term<'db> {
        match term.kind(self.db) {
            TermKind::BVar(i) if *i >= index => Term::bvar(self.db, i + offset),
            TermKind::App(f, x) => {
                let f = self.shift_at(f, offset, index);
                let x = self.shift_at(x, offset, index);
                Term::app(self.db, f, x)
            }
            TermKind::Lam(info, ty, body) => {
                let ty = self.shift_at(ty, offset, index);
                let body = self.shift_at(body, offset, index + 1);
                Term::lam(self.db, *info, ty, body)
            }
            TermKind::Pi(info, ty, body) => {
                let ty = self.shift_at(ty, offset, index);
                let body = self.shift_at(body, offset, index + 1);
                Term::pi(self.db, *info, ty, body)
            }
            TermKind::Sigma(info, ty, body) => {
                let ty = self.shift_at(ty, offset, index);
                let body = self.shift_at(body, offset, index + 1);
                Term::sigma(self.db, *info, ty, body)
            }
            TermKind::Let(ty, value, body) => {
                let ty = self.shift_at(ty, offset, index);
                let value = self.shift_at(value, offset, index);
                let body = self.shift_at(body, offset, index + 1);
                Term::let_(self.db, ty, value, body)
            }
            TermKind::Lit(lit) => Term::lit(self.db, lit.clone()),
            TermKind::FVar(_)
            | TermKind::BVar(_)
            | TermKind::MVar(_)
            | TermKind::Const(_)
            | TermKind::Sort(_) => *term,
        }
    }
}
