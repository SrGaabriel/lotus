use std::path::{
    Path,
    PathBuf,
};

pub const EXTENSION: &str = "lo";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Edition {
    #[default]
    Lotus2026,
}

#[derive(Debug, Clone)]
pub enum Program {
    File(PathBuf),
    Package(Package),
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
    pub edition: Edition,
}

#[derive(Debug, thiserror::Error)]
pub enum ProgramError {
    #[error("Input path is not a file or directory")]
    InvalidInput,
    #[error("File has invalid extension: {0}")]
    InvalidExtension(PathBuf),
    #[error("Could not infer name for package from directory name")]
    InvalidPackageName,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Program {
    pub fn from_path(path: PathBuf, name: Option<String>) -> Result<Self, ProgramError> {
        if path.is_file() {
            require_extension(&path)?;
            Ok(Self::File(path))
        } else if path.is_dir() {
            Ok(Self::Package(Package::from_path(path, name)?))
        } else {
            Err(ProgramError::InvalidInput)
        }
    }
}

impl Package {
    pub fn from_path(root: PathBuf, name: Option<String>) -> Result<Self, ProgramError> {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
            .or(name)
            .ok_or(ProgramError::InvalidPackageName)?;

        let mut files = Vec::new();
        for entry in std::fs::read_dir(&root)? {
            let path = entry?.path();
            if has_extension(&path) {
                files.push(path);
            }
        }

        Ok(Self {
            name,
            root,
            files,
            edition: Edition::default(),
        })
    }
}

fn has_extension(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some(EXTENSION)
}

fn require_extension(path: &Path) -> Result<(), ProgramError> {
    if has_extension(path) {
        Ok(())
    } else {
        Err(ProgramError::InvalidExtension(path.to_path_buf()))
    }
}
