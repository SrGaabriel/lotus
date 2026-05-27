use crate::{
    ElabDb,
    core::{
        Term,
        TermKind,
    },
    elab::ctx::ElabCtx,
};

impl<'db> ElabCtx<'db> {
    pub fn whnf(&self, term: Term<'db>) -> Term<'db> {
        match term.kind(self.db) {
            TermKind::App(f, x) => {
                let f = self.whnf(*f);
                if let TermKind::Lam(_, _, body) = f.kind(self.db) {
                    self.whnf(self.instantiate(body, *x))
                } else {
                    Term::app(self.db, f, *x)
                }
            }

            TermKind::MVar(u) => {
                if let Some(value) = self.mctx.get_solution(*u) {
                    self.whnf(*value)
                } else {
                    term
                }
            }

            TermKind::Let(_, value, body) => self.whnf(self.instantiate(body, *value)),

            TermKind::Const(name) => {
                if let Some(body) = self.db.def_body(*name) {
                    self.whnf(body.value)
                } else {
                    term
                }
            }

            _ => term,
        }
    }
}
