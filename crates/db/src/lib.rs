use rustc_hash::FxHashMap;
use salsa::Setter;
use std::{
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Arc,
        RwLock,
    },
};

#[salsa::input(debug)]
pub struct SourceFile {
    #[returns(ref)]
    pub path: PathBuf,
    #[returns(ref)]
    pub text: Arc<str>,
}

#[salsa::input(singleton, debug)]
pub struct SourceRoot {
    #[returns(ref)]
    pub name: String,
    #[returns(ref)]
    pub path: PathBuf,
    #[returns(ref)]
    pub files: Vec<SourceFile>,
    pub entrypoint: Option<SourceFile>,
}

#[salsa::db]
pub trait SourceDatabase: salsa::Database {
    fn source_file(&self, path: &Path) -> Option<SourceFile>;
    fn intern_file(&mut self, path: PathBuf, text: Arc<str>) -> SourceFile;
    fn all_files(&self) -> Vec<SourceFile>;
}

#[salsa::db]
#[derive(Clone, Default)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
    files: Arc<RwLock<FxHashMap<PathBuf, SourceFile>>>,
}

impl RootDatabase {
    pub fn new() -> Self {
        Self::default()
    }
}

#[salsa::db]
impl salsa::Database for RootDatabase {}

#[salsa::db]
impl SourceDatabase for RootDatabase {
    fn source_file(&self, path: &Path) -> Option<SourceFile> {
        self.files.read().unwrap().get(path).copied()
    }

    fn intern_file(&mut self, path: PathBuf, text: Arc<str>) -> SourceFile {
        let existing = self.files.read().unwrap().get(&path).copied();
        if let Some(file) = existing {
            file.set_text(self).to(text);
            return file;
        }
        let file = SourceFile::new(self, path.clone(), text);
        self.files.write().unwrap().insert(path, file);
        file
    }

    fn all_files(&self) -> Vec<SourceFile> {
        self.files.read().unwrap().values().copied().collect()
    }
}

#[salsa::db]
#[derive(Clone, Default)]
pub struct MockDatabase {
    storage: salsa::Storage<Self>,
    files: Arc<RwLock<FxHashMap<PathBuf, SourceFile>>>,
}

#[salsa::db]
impl salsa::Database for MockDatabase {}

#[salsa::db]
impl SourceDatabase for MockDatabase {
    fn source_file(&self, path: &Path) -> Option<SourceFile> {
        self.files.read().unwrap().get(path).copied()
    }

    fn intern_file(&mut self, path: PathBuf, text: Arc<str>) -> SourceFile {
        let existing = self.files.read().unwrap().get(&path).copied();
        if let Some(file) = existing {
            file.set_text(self).to(text);
            return file;
        }
        let file = SourceFile::new(self, path.clone(), text);
        self.files.write().unwrap().insert(path, file);
        file
    }

    fn all_files(&self) -> Vec<SourceFile> {
        self.files.read().unwrap().values().copied().collect()
    }
}
