use rustc_hash::FxHashMap;
use salsa::Accumulator;
use text_size::TextRange;

use db::{
    SourceFile,
    SourceRoot,
};
use diagnostics::Diagnostic;

use crate::{
    ElabDatabase,
    ElabDb,
    env::{
        Namespace,
        import::{
            Import,
            ModulePath,
            file_imports,
            module_map,
        },
        namespace::{
            ItemMap,
            SymbolMap,
        },
    },
    ids::ItemId,
};

#[salsa::tracked]
pub fn local_namespace<'db>(db: &'db dyn ElabDatabase, file: SourceFile) -> Namespace<'db> {
    let tree = db.item_tree(file);
    let mut decls: ItemMap = FxHashMap::default();
    let mut children: SymbolMap<'db, Namespace<'db>> = FxHashMap::default();

    for &id in tree.items(db) {
        if id.parent(db).is_none() {
            decls.insert(id.name(db), id);
        }
    }

    let mut child_groups: SymbolMap<'db, ItemMap<'db>> = FxHashMap::default();
    for &id in tree.items(db) {
        if let Some(parent) = id.parent(db) {
            child_groups
                .entry(parent.name(db))
                .or_default()
                .insert(id.name(db), id);
        }
    }
    for (parent_name, kids) in child_groups {
        children.insert(parent_name, Namespace::new(db, kids, FxHashMap::default()));
    }

    Namespace::new(db, decls, children)
}

#[salsa::tracked]
pub fn def_map(db: &dyn ElabDatabase, file: SourceFile) -> Namespace<'_> {
    let local = local_namespace(db, file);
    let imports = file_imports(db, file);
    if imports.is_empty() {
        return local;
    }

    let mut decls = local.decls(db).clone();
    let mut children = local.children(db).clone();
    let modules = SourceRoot::try_get(db).map(|root| module_map(db, root));

    for import in imports {
        match resolve_import(db, modules, import) {
            Some(Resolved::Item { item, child }) => {
                decls.insert(import.name, item);
                if let Some(child) = child {
                    children.insert(import.name, child);
                }
            }
            Some(Resolved::Module(namespace)) => {
                children.insert(import.name, namespace);
            }
            None => {
                let diag = Diagnostic::error(
                    &format!("unresolved import `{}`", import_display(db, import)),
                    file,
                    TextRange::new(import.start.into(), import.end.into()),
                )
                .build();
                diag.accumulate(db);
            }
        }
    }

    Namespace::new(db, decls, children)
}

enum Resolved<'db> {
    Item {
        item: ItemId<'db>,
        child: Option<Namespace<'db>>,
    },
    Module(Namespace<'db>),
}

fn resolve_import<'db>(
    db: &'db dyn ElabDatabase,
    modules: Option<&FxHashMap<ModulePath<'db>, SourceFile>>,
    import: &Import<'db>,
) -> Option<Resolved<'db>> {
    let modules = modules?;

    if !import.module.is_empty()
        && let Some(&target) = modules.get(&import.module)
    {
        let namespace = local_namespace(db, target);
        if let Some(&item) = namespace.decls(db).get(&import.name) {
            let child = namespace.children(db).get(&import.name).copied();
            return Some(Resolved::Item { item, child });
        }
    }

    let mut full = import.module.clone();
    full.push(import.name);
    let &target = modules.get(&full)?;
    Some(Resolved::Module(local_namespace(db, target)))
}

fn import_display<'db>(db: &'db dyn ElabDatabase, import: &Import<'db>) -> String {
    let mut out = String::new();
    for seg in &import.module {
        out.push_str(seg.into_str(db));
        out.push_str("::");
    }
    out.push_str(import.name.into_str(db));
    out
}
