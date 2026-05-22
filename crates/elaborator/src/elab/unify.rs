use crate::{
    core::{
        Level,
        LevelKind,
        Term,
        TermKind,
    },
    elab::ctx::ElabCtx,
};

#[derive(Debug, Clone)]
pub enum UnifyError<'db> {
    Mismatch {
        a: Term<'db>,
        b: Term<'db>,
    },
    Below {
        a: Term<'db>,
        b: Term<'db>,
        cause: Box<UnifyError<'db>>,
    },
}

impl<'db> UnifyError<'db> {
    pub fn root(&self) -> (Term<'db>, Term<'db>) {
        match self {
            UnifyError::Mismatch { a, b } => (*a, *b),
            UnifyError::Below { cause, .. } => cause.root(),
        }
    }
}

impl<'db> ElabCtx<'db> {
    pub fn unify(&mut self, a: Term<'db>, b: Term<'db>) -> Result<(), UnifyError<'db>> {
        tracing::debug!("unify {} and {}", a.debug(self.db), b.debug(self.db));
        self.eq_term(a, b)
    }

    fn eq_term(&self, a: Term<'db>, b: Term<'db>) -> Result<(), UnifyError<'db>> {
        if a == b {
            return Ok(());
        }
        match (a.kind(self.db), b.kind(self.db)) {
            (TermKind::BVar(i), TermKind::BVar(j)) if i == j => Ok(()),
            (TermKind::FVar(u), TermKind::FVar(v)) if u == v => Ok(()),
            (TermKind::MVar(u), TermKind::MVar(v)) if u == v => Ok(()),
            (TermKind::Lit(l1), TermKind::Lit(l2)) if l1 == l2 => Ok(()),
            (TermKind::Const(c1), TermKind::Const(c2)) if c1 == c2 => Ok(()),
            (TermKind::Sort(l1), TermKind::Sort(l2)) if self.eq_level(*l1, *l2) => Ok(()),
            (TermKind::App(f1, x1), TermKind::App(f2, x2)) => {
                self.eq_term(*f1, *f2).map_err(|cause| UnifyError::Below {
                    a,
                    b,
                    cause: Box::new(cause),
                })?;
                self.eq_term(*x1, *x2).map_err(|cause| UnifyError::Below {
                    a,
                    b,
                    cause: Box::new(cause),
                })?;
                Ok(())
            }
            (TermKind::Lam(_, t1, b1), TermKind::Lam(_, t2, b2))
            | (TermKind::Pi(_, t1, b1), TermKind::Pi(_, t2, b2))
            | (TermKind::Sigma(_, t1, b1), TermKind::Sigma(_, t2, b2)) => {
                self.eq_term(*t1, *t2).map_err(|cause| UnifyError::Below {
                    a,
                    b,
                    cause: Box::new(cause),
                })?;
                self.eq_term(*b1, *b2).map_err(|cause| UnifyError::Below {
                    a,
                    b,
                    cause: Box::new(cause),
                })?;
                Ok(())
            }
            (TermKind::Let(t1, v1, b1), TermKind::Let(t2, v2, b2)) => {
                self.eq_term(*t1, *t2).map_err(|cause| UnifyError::Below {
                    a,
                    b,
                    cause: Box::new(cause),
                })?;
                self.eq_term(*v1, *v2).map_err(|cause| UnifyError::Below {
                    a,
                    b,
                    cause: Box::new(cause),
                })?;
                self.eq_term(*b1, *b2).map_err(|cause| UnifyError::Below {
                    a,
                    b,
                    cause: Box::new(cause),
                })?;
                Ok(())
            }
            _ => Err(UnifyError::Mismatch { a, b }),
        }
    }

    fn eq_level(&self, a: Level<'db>, b: Level<'db>) -> bool {
        if a == b {
            return true;
        }
        match (a.kind(self.db), b.kind(self.db)) {
            (LevelKind::Zero, LevelKind::Zero) => true,
            (LevelKind::Succ(a), LevelKind::Succ(b)) => self.eq_level(*a, *b),
            (LevelKind::Max(a1, a2), LevelKind::Max(b1, b2))
            | (LevelKind::IMax(a1, a2), LevelKind::IMax(b1, b2)) => {
                self.eq_level(*a1, *b1) && self.eq_level(*a2, *b2)
            }
            (LevelKind::MVar(u), LevelKind::MVar(v)) => u == v,
            (LevelKind::Param(u), LevelKind::Param(v)) => u == v,
            _ => false,
        }
    }
}
