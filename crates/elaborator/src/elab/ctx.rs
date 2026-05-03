use crate::{
    Db,
    ElabDb,
    core::{
        TermArena,
        TermId,
    },
    env::Namespace,
    ids::{
        ItemId,
        UniqueGen,
    },
};

pub struct ElabCtx<'db> {
    pub db: Db<'db>,
    pub current_decl: ItemId<'db>,

    pub arena: TermArena<'db>,
    pub gen_: UniqueGen,

    pub namespace: Namespace<'db>,
}

impl<'db> ElabCtx<'db> {
    pub fn new(db: Db<'db>, current_decl: ItemId<'db>) -> Self {
        let file = current_decl.file(db);
        let namespace = db.def_map(file);
        Self {
            db,
            current_decl,
            arena: TermArena::new(),
            gen_: UniqueGen::new(),
            namespace,
        }
    }

    pub fn lower_return_type(&mut self, _ty: ast::Type) -> TermId {
        self.arena.type0()
    }

    pub fn lower_body(&mut self, _expr: ast::Expr) -> TermId {
        self.arena.type0()
    }

    pub fn placeholder(&mut self) -> TermId {
        self.arena.type0()
    }
}
