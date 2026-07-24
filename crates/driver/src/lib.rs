use ast::{
    Parse,
    SourceFile as AstSourceFile,
    parse_file,
};
use db::{
    RootDatabase,
    SourceDatabase,
    SourceFile,
    SourceRoot,
};
use diagnostics::Diagnostic;
use elaborator::{
    ElabDatabase,
    ElabDb,
    ElaboratedFile,
    ItemKind,
};
use nir::{
    NirDatabase,
    NirDb,
    src::NirFile,
};
use salsa::{
    CancellationToken,
    Database,
    Durability,
};
use std::{
    path::PathBuf,
    sync::Arc,
};
use structure::{
    Package,
    Program,
};

pub struct Compiler {
    db: RootDatabase,
    root: Option<SourceRoot>,
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            db: RootDatabase::new(),
            root: None,
        }
    }

    pub fn db(&self) -> &RootDatabase {
        &self.db
    }

    pub fn db_mut(&mut self) -> &mut RootDatabase {
        &mut self.db
    }

    pub fn root(&self) -> Option<SourceRoot> {
        self.root
    }

    pub fn files(&self) -> Vec<SourceFile> {
        match self.root {
            Some(root) => root.files(&self.db).clone(),
            None => self.db.all_files(),
        }
    }

    pub fn ingest_program(&mut self, program: Program) -> Result<SourceRoot, IngestError> {
        let (name, files, entrypoint) = match program {
            Program::File(path) => {
                let name = stem_or_default(&path, "main");
                let entry = self.read_and_intern(path)?;
                (name, vec![entry], Some(entry))
            }
            Program::Package(Package { name, files, .. }) => {
                let mut interned = Vec::with_capacity(files.len());
                for path in files {
                    interned.push(self.read_and_intern(path)?);
                }
                (name, interned, None)
            }
        };

        let root = SourceRoot::new(&self.db, name, files, entrypoint);
        self.root = Some(root);
        Ok(root)
    }

    pub fn update_file(&mut self, path: PathBuf, text: Arc<str>) -> SourceFile {
        self.db.intern_file(path, text)
    }

    pub fn parse(&self, file: SourceFile) -> &Parse<AstSourceFile> {
        parse_file(&self.db, file)
    }

    pub fn elaborate(&self, file: SourceFile) -> ElaboratedFile<'_> {
        let db: &dyn ElabDatabase = &self.db;
        db.elaborate_file(file)
    }

    pub fn lower(&self, file: SourceFile) -> &NirFile<'_> {
        let db: &dyn NirDatabase = &self.db;
        db.lower_file(file)
    }

    pub fn diagnostics(&self, file: SourceFile) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        out.extend(
            parse_file::accumulated::<Diagnostic>(&self.db, file)
                .into_iter()
                .cloned(),
        );
        let db: &dyn ElabDatabase = &self.db;
        let namespace = db.def_map(file);
        let _ = db.lang_items(file);
        out.extend(
            elaborator::env::lang_items::file_lang_items::accumulated::<Diagnostic>(&self.db, file)
                .into_iter()
                .cloned(),
        );

        for &item in namespace.decls(db).values() {
            let _ = db.signature(item);
            out.extend(
                elaborator::elab::sig::signature::accumulated::<Diagnostic>(&self.db, item)
                    .into_iter()
                    .cloned(),
            );

            match item.kind(db) {
                ItemKind::Def => {
                    let _ = db.def_body(item);
                    out.extend(
                        elaborator::elab::def::def_body::accumulated::<Diagnostic>(&self.db, item)
                            .into_iter()
                            .cloned(),
                    );
                }
                ItemKind::Inductive => {
                    let _ = db.inductive_data(item);
                    out.extend(
                        elaborator::elab::inductive::inductive_data::accumulated::<Diagnostic>(
                            &self.db, item,
                        )
                        .into_iter()
                        .cloned(),
                    );
                }
                ItemKind::Constructor => {}
            }
        }
        out
    }

    pub fn parsing_diagnostics(&self, file: SourceFile) -> Vec<Diagnostic> {
        parse_file::accumulated::<Diagnostic>(&self.db, file)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.db.cancellation_token()
    }

    pub fn cancel(&mut self) {
        self.db.trigger_cancellation();
    }

    pub fn synthetic_write(&mut self, durability: Durability) {
        self.db.synthetic_write(durability);
    }

    fn read_and_intern(&mut self, path: PathBuf) -> Result<SourceFile, IngestError> {
        let text = std::fs::read_to_string(&path).map_err(|source| IngestError::Io {
            path: path.clone(),
            source,
        })?;
        let text: Arc<str> = Arc::from(text.into_boxed_str());
        Ok(self.db.intern_file(path, text))
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

fn stem_or_default(path: &std::path::Path, default: &str) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map_or_else(|| default.to_string(), str::to_string)
}
