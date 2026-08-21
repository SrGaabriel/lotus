#![feature(iterator_try_collect)]

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
    Setter,
};
use std::{
    path::PathBuf,
    sync::Arc,
};
use structure::Program;

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
        let main = if let Some(main) = program.main {
            Some(self.read_and_intern(main)?)
        } else {
            None
        };
        let source_files: Vec<_> = program
            .files
            .into_iter()
            .map(|p| self.read_and_intern(p))
            .try_collect()?;

        let root = if let Some(root) = self.root {
            root.set_name(&mut self.db).to(program.name);
            root.set_path(&mut self.db).to(program.root);
            root.set_files(&mut self.db).to(source_files);
            root.set_entrypoint(&mut self.db).to(main);
            root
        } else {
            SourceRoot::new(&self.db, program.name, program.root, source_files, main)
        };
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
        elaborator::file_diagnostics(&self.db, file)
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
