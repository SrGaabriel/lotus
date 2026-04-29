use std::path::PathBuf;

use ast::{Parse, SourceFile as AstSourceFile, parse_file};
use db::{RootDatabase, SourceDatabase, SourceFile, SourceRoot};
use diagnostics::Diagnostic;
use salsa::{CancellationToken, Database, Durability};
use structure::{File, Files, Program};

pub struct Compiler {
    db: RootDatabase,
    root: Option<SourceRoot>,
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

    pub fn ingest_program(&mut self, program: Program) -> SourceRoot {
        let (name, entry_path, files) = match program {
            Program::File(file) => {
                let name = file
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map_or_else(|| "main".to_string(), str::to_string);
                let entry_path = Some(file.path.clone());
                let mut bundle = Files::new();
                bundle.add(file);
                (name, entry_path, bundle)
            }
            Program::Project(project) => (project.name, None, project.files),
        };

        let mut interned = Vec::new();
        let mut entrypoint = None;
        for (_, file) in files {
            let is_entry = entry_path.as_deref() == Some(file.path.as_path());
            let id = self.add_file_inner(file);
            if is_entry {
                entrypoint = Some(id);
            }
            interned.push(id);
        }

        let root = SourceRoot::new(&self.db, name, interned, entrypoint);
        self.root = Some(root);
        root
    }

    pub fn add_file(&mut self, file: File) -> SourceFile {
        let id = self.add_file_inner(file);
        if let Some(root) = self.root {
            use salsa::Setter;
            let mut files = root.files(&self.db).clone();
            if !files.contains(&id) {
                files.push(id);
                root.set_files(&mut self.db).to(files);
            }
        }
        id
    }

    fn add_file_inner(&mut self, file: File) -> SourceFile {
        self.db.intern_file(file.path, file.text.to_string())
    }

    pub fn update_file(&mut self, path: PathBuf, text: String) -> SourceFile {
        self.db.intern_file(path, text)
    }

    pub fn parse(&self, file: SourceFile) -> &Parse<AstSourceFile> {
        parse_file(&self.db, file)
    }

    pub fn diagnostics(&self, file: SourceFile) -> Vec<Diagnostic> {
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
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
