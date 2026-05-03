use ast::parse_file;
use db::SourceFile;

use crate::{
    ElabDatabase,
    ElabDb,
    core::TermArena,
    env::Body,
    ids::{
        DefId,
        UniqueGen,
    },
};

#[salsa::tracked(returns(ref))]
pub fn elaborate_def<'db>(db: &'db dyn ElabDatabase, def: DefId<'db>) -> Body<'db> {
    let file = def.file(db);
    let parse = parse_file(db, file);
    let source = parse.tree();

    let Some(ast::Decl::DefDecl(decl)) = source.decl().nth(def.ast_index(db) as usize) else {
        return Body::empty();
    };

    let mut arena = TermArena::new();
    let mut _gen = UniqueGen::new();

    let value = decl.body().map(|_expr| arena.type0());

    Body {
        arena,
        value,
        ty: None,
    }
}

#[salsa::tracked]
pub fn elaborate_file(db: &dyn ElabDatabase, file: SourceFile) {
    let ns = db.def_map(file);
    for def in ns.decls(db).values() {
        let _ = db.elaborate_def(*def);
    }
}
