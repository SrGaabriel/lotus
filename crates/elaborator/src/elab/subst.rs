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
}
