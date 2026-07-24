use crate::{
    core::{
        Level,
        LevelKind,
        Term,
        TermKind,
    },
    elab::ctx::ElabCtx,
    ids::Unique,
};

#[derive(Debug, Clone)]
pub enum UnifyError<'db> {
    Mismatch {
        a: Term<'db>,
        b: Term<'db>,
    },
    Occurs {
        meta: Term<'db>,
        value: Term<'db>,
    },
    Escaping {
        meta: Term<'db>,
        value: Term<'db>,
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
            UnifyError::Occurs { meta, value } | UnifyError::Escaping { meta, value } => {
                (*meta, *value)
            }
            UnifyError::Below { cause, .. } => cause.root(),
        }
    }
}

impl<'db> ElabCtx<'db> {
    pub fn unify(&mut self, a: Term<'db>, b: Term<'db>) -> Result<(), UnifyError<'db>> {
        tracing::debug!("unify {} and {}", a.debug(self.db), b.debug(self.db));
        self.eq_term(a, b)
    }

    fn eq_term(&mut self, a: Term<'db>, b: Term<'db>) -> Result<(), UnifyError<'db>> {
        let a = self.whnf(a);
        let b = self.whnf(b);
        if a == b {
            return Ok(());
        }
        match (a.kind(self.db), b.kind(self.db)) {
            (TermKind::BVar(i), TermKind::BVar(j)) if i == j => Ok(()),
            (TermKind::FVar(u), TermKind::FVar(v)) if u == v => Ok(()),
            (TermKind::MVar(u), TermKind::MVar(v)) if u == v => Ok(()),
            (TermKind::MVar(u), _) => self.solve_mvar(*u, b),
            (_, TermKind::MVar(v)) => self.solve_mvar(*v, a),
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

    fn solve_mvar(&mut self, meta: Unique, value: Term<'db>) -> Result<(), UnifyError<'db>> {
        if let Some(solution) = self.mctx.get_solution(meta).copied() {
            return self.eq_term(solution, value);
        }

        let value = self.whnf(value);
        if let TermKind::MVar(other) = value.kind(self.db)
            && *other == meta
        {
            return Ok(());
        }

        if self.contains_mvar(meta, value) {
            return Err(UnifyError::Occurs {
                meta: Term::mvar(self.db, meta),
                value,
            });
        }

        if !self.term_fits_meta_scope(meta, value) {
            return Err(UnifyError::Escaping {
                meta: Term::mvar(self.db, meta),
                value,
            });
        }

        self.mctx.solve_meta(meta, value);
        Ok(())
    }

    fn contains_mvar(&self, needle: Unique, term: Term<'db>) -> bool {
        match term.kind(self.db) {
            TermKind::MVar(meta) if *meta == needle => true,
            TermKind::MVar(meta) => self
                .mctx
                .get_solution(*meta)
                .is_some_and(|solution| self.contains_mvar(needle, *solution)),
            TermKind::App(f, x) => self.contains_mvar(needle, *f) || self.contains_mvar(needle, *x),
            TermKind::Lam(_, ty, body)
            | TermKind::Pi(_, ty, body)
            | TermKind::Sigma(_, ty, body) => {
                self.contains_mvar(needle, *ty) || self.contains_mvar(needle, *body)
            }
            TermKind::Let(ty, value, body) => {
                self.contains_mvar(needle, *ty)
                    || self.contains_mvar(needle, *value)
                    || self.contains_mvar(needle, *body)
            }
            TermKind::BVar(_)
            | TermKind::FVar(_)
            | TermKind::Const(_)
            | TermKind::Sort(_)
            | TermKind::Lit(_) => false,
        }
    }

    fn term_fits_meta_scope(&self, meta: Unique, value: Term<'db>) -> bool {
        let Some(meta) = self.mctx.get_meta(meta) else {
            return false;
        };
        let allowed: Vec<Unique> = meta.lctx.iter().map(|binder| binder.unique).collect();
        self.term_fits_scope(value, &allowed, 0)
    }

    fn term_fits_scope(&self, term: Term<'db>, allowed: &[Unique], depth: usize) -> bool {
        match term.kind(self.db) {
            TermKind::BVar(index) => *index < depth,
            TermKind::FVar(unique) => allowed.contains(unique),
            TermKind::MVar(meta) => {
                if let Some(solution) = self.mctx.get_solution(*meta) {
                    self.term_fits_scope(*solution, allowed, depth)
                } else {
                    self.meta_context_fits_scope(*meta, allowed)
                }
            }
            TermKind::App(f, x) => {
                self.term_fits_scope(*f, allowed, depth) && self.term_fits_scope(*x, allowed, depth)
            }
            TermKind::Lam(_, ty, body)
            | TermKind::Pi(_, ty, body)
            | TermKind::Sigma(_, ty, body) => {
                self.term_fits_scope(*ty, allowed, depth)
                    && self.term_fits_scope(*body, allowed, depth + 1)
            }
            TermKind::Let(ty, value, body) => {
                self.term_fits_scope(*ty, allowed, depth)
                    && self.term_fits_scope(*value, allowed, depth)
                    && self.term_fits_scope(*body, allowed, depth + 1)
            }
            TermKind::Const(_) | TermKind::Sort(_) | TermKind::Lit(_) => true,
        }
    }

    fn meta_context_fits_scope(&self, meta: Unique, allowed: &[Unique]) -> bool {
        let Some(meta) = self.mctx.get_meta(meta) else {
            return false;
        };
        meta.lctx
            .iter()
            .all(|binder| allowed.contains(&binder.unique))
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
