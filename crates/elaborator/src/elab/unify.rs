use crate::{
    core::{
        Level,
        Term,
        TermId,
    },
    elab::ctx::ElabCtx,
};

impl ElabCtx<'_> {
    pub fn unify(&mut self, a: TermId, b: TermId) -> bool {
        tracing::debug!("unify {} and {}", a.debug(self.db, &self.arena), b.debug(self.db, &self.arena));
        let a = self.arena.get_term(a);
        let b = self.arena.get_term(b);

        if self.structural_eq(a, b) {
            return true;
        }
        false
    }

    fn structural_eq(&self, a: &Term, b: &Term) -> bool {
        match (a, b) {
            (Term::BVar(i), Term::BVar(j)) => i == j,
            (Term::FVar(n1), Term::FVar(n2)) => n1 == n2,
            (Term::MVar(u1), Term::MVar(u2)) => u1 == u2,
            (Term::Lit(l1), Term::Lit(l2)) => l1 == l2,
            (Term::Sort(l1), Term::Sort(l2)) => {
                let l1 = self.arena.get_level(*l1);
                let l2 = self.arena.get_level(*l2);
                self.structural_eq_level(l1, l2)
            }
            (Term::App(f1, a1), Term::App(f2, a2)) => {
                let f1 = self.arena.get_term(*f1);
                let f2 = self.arena.get_term(*f2);
                self.structural_eq(f1, f2)
                    && self.structural_eq(self.arena.get_term(*a1), self.arena.get_term(*a2))
            }
            (Term::Const(n1), Term::Const(n2)) => n1 == n2,
            (Term::Lam(_, ty1, b1), Term::Lam(_, ty2, b2))
            | (Term::Pi(_, ty1, b1), Term::Pi(_, ty2, b2))
            | (Term::Sigma(_, ty1, b1), Term::Sigma(_, ty2, b2)) => {
                let ty1 = self.arena.get_term(*ty1);
                let ty2 = self.arena.get_term(*ty2);
                self.structural_eq(ty1, ty2)
                    && self.structural_eq(self.arena.get_term(*b1), self.arena.get_term(*b2))
            }
            (Term::Let(ty1, v1, b1), Term::Let(ty2, v2, b2)) => {
                let ty1 = self.arena.get_term(*ty1);
                let ty2 = self.arena.get_term(*ty2);
                self.structural_eq(ty1, ty2)
                    && self.structural_eq(self.arena.get_term(*v1), self.arena.get_term(*v2))
                    && self.structural_eq(self.arena.get_term(*b1), self.arena.get_term(*b2))
            }
            _ => false,
        }
    }

    fn structural_eq_level(&self, a: &Level, b: &Level) -> bool {
        match (a, b) {
            (Level::Zero, Level::Zero) => true,
            (Level::Succ(a), Level::Succ(b)) => {
                let a = self.arena.get_level(*a);
                let b = self.arena.get_level(*b);
                self.structural_eq_level(a, b)
            }
            (Level::Max(a1, a2), Level::Max(b1, b2))
            | (Level::IMax(a1, a2), Level::IMax(b1, b2)) => {
                let a1 = self.arena.get_level(*a1);
                let b1 = self.arena.get_level(*b1);
                self.structural_eq_level(a1, b1)
                    && self
                        .structural_eq_level(self.arena.get_level(*a2), self.arena.get_level(*b2))
            }
            (Level::MVar(u1), Level::MVar(u2)) => u1 == u2,
            (Level::Param(u1), Level::Param(u2)) => u1 == u2,
            _ => false,
        }
    }
}
