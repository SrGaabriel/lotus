use std::collections::HashSet;

use db::SourceFile;
use diagnostics::Diagnostic;

use crate::{
    ElabDatabase,
    ElabDb,
    elab::{
        def,
        inductive::inductive_data,
        sig::signature,
    },
    env::{
        def_map,
        lang_items,
    },
    ids::ItemKind,
};

pub fn file_diagnostics(db: &dyn ElabDatabase, file: SourceFile) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    out.extend(
        ast::parse_file::accumulated::<Diagnostic>(db, file)
            .into_iter()
            .cloned(),
    );

    let _ = db.def_map(file);
    out.extend(
        def_map::def_map::accumulated::<Diagnostic>(db, file)
            .into_iter()
            .cloned(),
    );

    let _ = db.lang_items(file);
    out.extend(
        lang_items::file_lang_items::accumulated::<Diagnostic>(db, file)
            .into_iter()
            .cloned(),
    );

    for &item in db.item_tree(file).items(db) {
        let _ = db.signature(item);
        out.extend(
            signature::accumulated::<Diagnostic>(db, item)
                .into_iter()
                .cloned(),
        );
        match item.kind(db) {
            ItemKind::Def => {
                let _ = db.def_body(item);
                out.extend(
                    def::def_body::accumulated::<Diagnostic>(db, item)
                        .into_iter()
                        .cloned(),
                );
            }
            ItemKind::Inductive => {
                let _ = db.inductive_data(item);
                out.extend(
                    inductive_data::accumulated::<Diagnostic>(db, item)
                        .into_iter()
                        .cloned(),
                );
            }
            ItemKind::Constructor => {}
        }
    }

    out.retain(|diag| diag.primary.file == file);
    let mut seen = HashSet::new();
    out.retain(|diag| seen.insert(diag.clone()));
    out
}
