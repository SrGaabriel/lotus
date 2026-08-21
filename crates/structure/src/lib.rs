use std::path::{
    Path,
    PathBuf,
};

pub const EXTENSION: &str = "lt";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Edition {
    #[default]
    Lotus2026,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub name: String,
    pub root: PathBuf,
    pub main: Option<PathBuf>,
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
        let name = name
            .or_else(|| {
                path.file_stem()
                    .and_then(|n| n.to_str())
                    .map(str::to_string)
            })
            .ok_or(ProgramError::InvalidPackageName)?;

        let (root, main, files) = if path.is_dir() {
            let mut files = discover_files(&path)?;
            files.sort();
            (path, None, files)
        } else {
            let root = path.parent().map(Path::to_path_buf).unwrap_or_default();
            (root, Some(path.clone()), vec![path])
        };

        Ok(Self {
            name,
            root,
            main,
            files,
            edition: Edition::default(),
        })
    }
}

fn discover_files(path: &Path) -> Result<Vec<PathBuf>, ProgramError> {
    let entries = std::fs::read_dir(path)?;
    let mut paths = vec![];
    for entry_res in entries {
        let Ok(entry) = entry_res else {
            tracing::warn!("could not read entry in {:?}", path.file_name());
            continue;
        };
        let path = entry.path();
        if path.is_dir() {
            paths.extend(discover_files(&path)?);
            continue;
        }
        if !has_extension(&path) {
            continue;
        }
        paths.push(path);
    }
    Ok(paths)
}

fn has_extension(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some(EXTENSION)
}
