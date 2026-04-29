use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;

use ariadne::{Cache, Source};
use db::{SourceDatabase, SourceFile};

/// An ariadne [`Cache`] that resolves [`SourceFile`] ids through the db.
pub struct FilesCache<'db> {
    pub db: &'db dyn SourceDatabase,
    sources: HashMap<SourceFile, Source<String>>,
}

impl<'db> FilesCache<'db> {
    pub fn new(db: &'db dyn SourceDatabase) -> Self {
        Self {
            db,
            sources: HashMap::new(),
        }
    }
}

impl Cache<SourceFile> for FilesCache<'_> {
    type Storage = String;

    fn fetch(&mut self, id: &SourceFile) -> Result<&Source<Self::Storage>, impl fmt::Debug> {
        match self.sources.entry(*id) {
            Entry::Occupied(entry) => Ok::<&Source<Self::Storage>, &str>(entry.into_mut()),
            Entry::Vacant(entry) => {
                let text = id.text(self.db).clone();
                Ok(entry.insert(Source::from(text)))
            }
        }
    }

    fn display<'b>(&self, id: &'b SourceFile) -> Option<impl fmt::Display + 'b> {
        Some(id.path(self.db).display().to_string())
    }
}
