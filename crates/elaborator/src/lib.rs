pub mod core;
pub mod expr;
pub mod namespace;
pub mod unique;

use ast::{
    AstDatabase,
    parse_file,
};
use db::SourceFile;
use rustc_hash::FxHashMap;

use crate::{
    expr::Term,
    namespace::Namespace,
    unique::Name,
};

#[derive(Debug, Clone)]
pub struct Environment {
    pub source: SourceFile,
    pub externals: FxHashMap<Name, Term>,
    pub main_fn: Option<Name>,
    pub aliases: FxHashMap<Name, (Term, Term)>,
    pub root_namespace: Namespace,
    pub match_fns: FxHashMap<Name, Name>,
}

impl Environment {
    pub fn lookup_alias(&self, qname: &Name) -> Option<(&Term, &Term)> {
        self.aliases.get(qname).map(|(value, type_)| (value, type_))
    }
}

#[salsa::db]
pub trait ElabDatabase: AstDatabase {}

#[salsa::db]
impl<DB: AstDatabase> ElabDatabase for DB {}

#[salsa::tracked(returns(ref))]
pub fn elaborate_file(db: &dyn ElabDatabase, file: SourceFile) {
    let parse = parse_file(db, file);
    let _source = parse.tree();
}
