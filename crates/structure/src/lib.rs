use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

pub const EXTENSION: &str = "lo";

pub enum Program {
    File(File),
    Project(Project),
}

#[derive(Debug, thiserror::Error)]
pub enum ProgramError {
    #[error("Input path is not a file or directory")]
    InvalidInput,
    #[error("File has invalid extension: {0}")]
    InvalidExtension(PathBuf),
    #[error("Could not infer name for project from directory name")]
    InvalidProjectName,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Program {
    pub fn from_path(path: PathBuf, name: Option<String>) -> Result<Self, ProgramError> {
        if path.is_file() {
            if path.extension().and_then(|ext| ext.to_str()) == Some(EXTENSION) {
                let file = File::from_path(path)?;
                Ok(Program::File(file))
            } else {
                Err(ProgramError::InvalidExtension(path))
            }
        } else if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
                .or(name)
                .ok_or(ProgramError::InvalidProjectName)?;
            let mut files = Files::new();
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some(EXTENSION) {
                    let file = File::from_path(path)?;
                    files.add(file);
                }
            }
            let project = Project::new(name, files);
            Ok(Program::Project(project))
        } else {
            Err(ProgramError::InvalidInput)
        }
    }

    pub fn into_files(self) -> Files {
        match self {
            Program::File(file) => {
                let mut files = Files::new();
                files.add(file);
                files
            }
            Program::Project(project) => project.files,
        }
    }
}

pub struct Project {
    pub name: String,
    pub files: Files,
}

impl Project {
    pub fn new(name: String, files: Files) -> Self {
        Self { name, files }
    }

    pub fn get_file(&self, id: FileId) -> &File {
        self.files.get(id)
    }

    pub fn find_file_by_path(&self, path: &Path) -> Option<&File> {
        self.files.find_by_path(path)
    }
}

pub type FileId = u32;

#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub text: Arc<str>,
}

impl File {
    pub fn new(path: PathBuf, text: String) -> Self {
        Self {
            path,
            text: Arc::from(text),
        }
    }

    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(&path)?;
        Ok(Self::new(path, text))
    }
}

#[derive(Debug, Default)]
pub struct Files {
    files: Vec<File>,
    by_path: HashMap<PathBuf, FileId>,
}

impl Files {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            by_path: HashMap::new(),
        }
    }

    pub fn add(&mut self, file: File) -> FileId {
        let id = self.files.len() as FileId;
        self.by_path.insert(file.path.clone(), id);
        self.files.push(file);
        id
    }

    pub fn get(&self, id: FileId) -> &File {
        &self.files[id as usize]
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&File> {
        self.by_path.get(path).map(|id| self.get(*id))
    }

    pub fn iter(&self) -> impl Iterator<Item = (FileId, &File)> {
        self.files
            .iter()
            .enumerate()
            .map(|(id, file)| (id as FileId, file))
    }
}

impl IntoIterator for Files {
    type Item = (FileId, File);
    type IntoIter = std::iter::Map<
        std::iter::Enumerate<std::vec::IntoIter<File>>,
        fn((usize, File)) -> (FileId, File),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.files
            .into_iter()
            .enumerate()
            .map(|(id, file)| (id as FileId, file))
    }
}
