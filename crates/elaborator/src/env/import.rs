use std::path::Path;

use ast::{
    parse_file,
    traits::AstNode,
};
use db::{
    SourceFile,
    SourceRoot,
};
use rustc_hash::FxHashMap;

use crate::{
    ElabDatabase,
    ids::Symbol,
};

pub type ModulePath<'db> = Vec<Symbol<'db>>;

#[derive(Debug, Clone, PartialEq, Eq, salsa::Update)]
pub struct Import<'db> {
    pub module: ModulePath<'db>,
    pub name: Symbol<'db>,
    pub start: u32,
    pub end: u32,
}

#[salsa::tracked(returns(ref))]
pub fn module_map(
    db: &dyn ElabDatabase,
    root: SourceRoot,
) -> FxHashMap<ModulePath<'_>, SourceFile> {
    let root_dir = root.path(db).clone();
    let mut map = FxHashMap::default();
    for &file in root.files(db) {
        if let Some(path) = module_path(db, &root_dir, file) {
            map.insert(path, file);
        }
    }
    map
}

fn module_path<'db>(
    db: &'db dyn ElabDatabase,
    root_dir: &Path,
    file: SourceFile,
) -> Option<ModulePath<'db>> {
    let rel = file.path(db).strip_prefix(root_dir).ok()?;
    let stem = rel.file_stem()?.to_str()?;
    let mut segments = Vec::new();
    if let Some(parent) = rel.parent() {
        for component in parent.components() {
            let text = component.as_os_str().to_str()?;
            segments.push(Symbol::from_str(db, text));
        }
    }
    segments.push(Symbol::from_str(db, stem));
    Some(segments)
}

#[salsa::tracked(returns(ref))]
pub fn file_imports(db: &dyn ElabDatabase, file: SourceFile) -> Vec<Import<'_>> {
    let parse = parse_file(db, file);
    let source = parse.tree();

    let mut imports = Vec::new();
    for decl in source.decl() {
        if let ast::Decl::ImportDecl(import) = decl
            && let Some(group) = import.import_group()
        {
            collect_group(db, &group, &[], &mut imports);
        }
    }
    imports
}

fn collect_group<'db>(
    db: &'db dyn ElabDatabase,
    group: &ast::ImportGroup,
    prefix: &[Symbol<'db>],
    out: &mut Vec<Import<'db>>,
) {
    let mut module = prefix.to_vec();
    for seg in group.path() {
        let Some(ident) = seg.identifier() else {
            return;
        };
        let Some(text) = ident.text() else {
            return;
        };
        module.push(Symbol::from_str(db, text));
    }
    match group.import_target() {
        Some(ast::ImportTarget::Identifier(ident)) => {
            let Some(text) = ident.text() else {
                return;
            };
            let range = group.syntax().text_range();
            out.push(Import {
                module,
                name: Symbol::from_str(db, text),
                start: range.start().into(),
                end: range.end().into(),
            });
        }
        Some(ast::ImportTarget::ImportList(list)) => {
            for sub in list.import_group() {
                collect_group(db, &sub, &module, out);
            }
        }
        None => {}
    }
}
