use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use ariadne::{Cache, Source};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub text: Arc<str>,
}

pub struct Files {
    files: Vec<File>,
    by_path: HashMap<PathBuf, FileId>,
}

impl Files {
    pub fn add(&mut self, file: File) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.by_path.insert(file.path.clone(), id);
        self.files.push(file);
        id
    }

    pub fn get(&self, id: FileId) -> &File {
        &self.files[id.0 as usize]
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&File> {
        self.by_path.get(path).map(|id| self.get(*id))
    }
}

pub struct FilesCache<'a> {
    pub files: &'a Files,
    sources: std::collections::HashMap<PathBuf, Source<Arc<str>>>,
}

impl Cache<PathBuf> for FilesCache<'_> {
    type Storage = Arc<str>;

    fn fetch(&mut self, id: &PathBuf) -> Result<&Source<Self::Storage>, impl fmt::Debug> {
        match self.sources.entry(id.clone()) {
            Entry::Occupied(entry) => Ok::<&Source<Self::Storage>, &str>(entry.into_mut()),
            Entry::Vacant(entry) => {
                let file = self
                    .files
                    .find_by_path(id)
                    .ok_or("file not found in cache")?;
                let source = Source::from(file.text.clone());
                Ok(entry.insert(source))
            }
        }
    }

    fn display<'b>(&self, path: &'b PathBuf) -> Option<impl std::fmt::Display + 'b> {
        Some(path.display())
    }
}
