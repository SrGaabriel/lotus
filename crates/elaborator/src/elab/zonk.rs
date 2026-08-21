use crate::{
    core::{
        Term,
        TermKind,
    },
    elab::ctx::ElabCtx,
    ids::Unique,
};

impl<'db> ElabCtx<'db> {
    pub fn zonk(&self, term: Term<'db>) -> Term<'db> {
        self.zonk_with_seen(term, &mut Vec::new())
    }

    fn zonk_with_seen(&self, term: Term<'db>, seen: &mut Vec<Unique>) -> Term<'db> {
        match term.kind(self.db) {
            TermKind::MVar(meta) => {
                if seen.contains(meta) {
                    return term;
                }
                let Some(solution) = self.mctx.get_solution(*meta).copied() else {
                    return term;
                };
                seen.push(*meta);
                let result = self.zonk_with_seen(solution, seen);
                seen.pop();
                result
            }
            TermKind::App(f, x) => {
                let f = self.zonk_with_seen(*f, seen);
                let x = self.zonk_with_seen(*x, seen);
                Term::app(self.db, f, x)
            }
            TermKind::Lam(info, ty, body) => {
                let ty = self.zonk_with_seen(*ty, seen);
                let body = self.zonk_with_seen(*body, seen);
                Term::lam(self.db, *info, ty, body)
            }
            TermKind::Pi(info, ty, body) => {
                let ty = self.zonk_with_seen(*ty, seen);
                let body = self.zonk_with_seen(*body, seen);
                Term::pi(self.db, *info, ty, body)
            }
            TermKind::Sigma(info, ty, body) => {
                let ty = self.zonk_with_seen(*ty, seen);
                let body = self.zonk_with_seen(*body, seen);
                Term::sigma(self.db, *info, ty, body)
            }
            TermKind::Let(ty, value, body) => {
                let ty = self.zonk_with_seen(*ty, seen);
                let value = self.zonk_with_seen(*value, seen);
                let body = self.zonk_with_seen(*body, seen);
                Term::let_(self.db, ty, value, body)
            }
            TermKind::BVar(_)
            | TermKind::FVar(_)
            | TermKind::Const(_)
            | TermKind::Sort(_)
            | TermKind::Lit(_)
            | TermKind::Error(_) => term,
        }
    }
}
