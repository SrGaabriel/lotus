use crate::NirDatabase;

struct ExprLoweringCtx<'db> {
    db: &'db dyn NirDatabase,
}

impl<'db> ExprLoweringCtx<'db> {
    fn new(db: &'db dyn NirDatabase) -> Self {
        Self { db }
    }
}

pub fn lower_expr(db: &dyn NirDatabase, expr: &elaborator::core::Term) -> Expr {}
