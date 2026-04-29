use std::collections::hash_map::Entry;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use ariadne::{Cache, Source};
use structure::Files;

pub struct FilesCache<'a> {
    pub files: &'a Files,
    sources: std::collections::HashMap<PathBuf, Source<Arc<str>>>,
}

impl<'a> FilesCache<'a> {
    pub fn new(files: &'a Files) -> Self {
        Self {
            files,
            sources: std::collections::HashMap::new(),
        }
    }
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
