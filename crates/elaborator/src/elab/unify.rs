use crate::{
    core::{
        Level,
        LevelKind,
        Term,
        TermKind,
    },
    elab::ctx::ElabCtx,
};

impl<'db> ElabCtx<'db> {
    pub fn unify(&mut self, a: Term<'db>, b: Term<'db>) -> bool {
        tracing::debug!("unify {} and {}", a.debug(self.db), b.debug(self.db));
        self.structural_eq(a, b)
    }

    fn structural_eq(&self, a: Term<'db>, b: Term<'db>) -> bool {
        if a == b {
            return true;
        }
        match (a.kind(self.db), b.kind(self.db)) {
            (TermKind::BVar(i), TermKind::BVar(j)) => i == j,
            (TermKind::FVar(n1), TermKind::FVar(n2)) => n1 == n2,
            (TermKind::MVar(u1), TermKind::MVar(u2)) => u1 == u2,
            (TermKind::Lit(l1), TermKind::Lit(l2)) => l1 == l2,
            (TermKind::Const(n1), TermKind::Const(n2)) => n1 == n2,
            (TermKind::Sort(l1), TermKind::Sort(l2)) => self.structural_eq_level(*l1, *l2),
            (TermKind::App(f1, a1), TermKind::App(f2, a2)) => {
                self.structural_eq(*f1, *f2) && self.structural_eq(*a1, *a2)
            }
            (TermKind::Lam(_, ty1, b1), TermKind::Lam(_, ty2, b2))
            | (TermKind::Pi(_, ty1, b1), TermKind::Pi(_, ty2, b2))
            | (TermKind::Sigma(_, ty1, b1), TermKind::Sigma(_, ty2, b2)) => {
                self.structural_eq(*ty1, *ty2) && self.structural_eq(*b1, *b2)
            }
            (TermKind::Let(ty1, v1, b1), TermKind::Let(ty2, v2, b2)) => {
                self.structural_eq(*ty1, *ty2)
                    && self.structural_eq(*v1, *v2)
                    && self.structural_eq(*b1, *b2)
            }
            _ => false,
        }
    }

    fn structural_eq_level(&self, a: Level<'db>, b: Level<'db>) -> bool {
        if a == b {
            return true;
        }
        match (a.kind(self.db), b.kind(self.db)) {
            (LevelKind::Zero, LevelKind::Zero) => true,
            (LevelKind::Succ(a), LevelKind::Succ(b)) => self.structural_eq_level(*a, *b),
            (LevelKind::Max(a1, a2), LevelKind::Max(b1, b2))
            | (LevelKind::IMax(a1, a2), LevelKind::IMax(b1, b2)) => {
                self.structural_eq_level(*a1, *b1) && self.structural_eq_level(*a2, *b2)
            }
            (LevelKind::MVar(u1), LevelKind::MVar(u2)) => u1 == u2,
            (LevelKind::Param(u1), LevelKind::Param(u2)) => u1 == u2,
            _ => false,
        }
    }
}
