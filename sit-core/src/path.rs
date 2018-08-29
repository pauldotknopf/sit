//! Path related utilities

use std::path::{Path, PathBuf};
use std::io;
use std::fs;

/// Denotes value as having path on a file system
pub trait HasPath {
    /// Returns a reference to a path
    fn path(&self) -> &Path;
}

/// Allows for directory resolution
pub trait ResolvePath {
    /// Resolves to an actual directory
    ///
    /// It will interpret common conventions for resolving a directory
    /// (so, for example, a file with a textual link in it will be resolved
    /// to the directory it points to)
    fn resolve_dir(&self) -> Result<PathBuf, io::Error>;
}

impl<T> ResolvePath for T where T: AsRef<Path> {
    fn resolve_dir(&self) -> Result<PathBuf, io::Error> {
        let mut path: PathBuf = self.as_ref().into();
        if path.is_dir() {
            Ok(path)
        } else {
            fs::File::open(&path)
                .and_then(|mut f| {
                    use std::io::Read;
                    let mut s = String::new();
                    f.read_to_string(&mut s).map(|_| s)
                })
                .and_then(|s| {
                    #[cfg(windows)]
                    let s = s.replace("/", "\\");
                    let trimmed_path = s.trim();
                    path.pop(); // remove the file name
                    path.join(PathBuf::from(trimmed_path)).resolve_dir()
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ResolvePath;
    use std::fs;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn resolve_dir() {
        let tmp = TempDir::new("sit").unwrap().into_path();
        assert_eq!(tmp.resolve_dir().unwrap(), tmp);
    }

    #[test]
    fn resolve_link() {
        let tmp = TempDir::new("sit").unwrap().into_path();
        fs::create_dir_all(tmp.join("dir")).unwrap();
        let mut f = fs::File::create(tmp.join("1")).unwrap();
        f.write(b"dir").unwrap();
        assert_eq!(tmp.join("1").resolve_dir().unwrap(), tmp.join("dir"));
    }

    #[test]
    fn resolve_broken_link() {
        let tmp = TempDir::new("sit").unwrap().into_path();
        let mut f = fs::File::create(tmp.join("1")).unwrap();
        f.write(b"dir").unwrap();
        assert!(tmp.join("1").resolve_dir().is_err());
    }

    #[test]
    fn resolve_link_nested() {
        let tmp = TempDir::new("sit").unwrap().into_path();
        fs::create_dir_all(tmp.join("dir")).unwrap();
        let mut f = fs::File::create(tmp.join("1")).unwrap();
        f.write(b"dir").unwrap();
        let mut f = fs::File::create(tmp.join("2")).unwrap();
        f.write(b"1").unwrap();
        assert_eq!(tmp.join("2").resolve_dir().unwrap(), tmp.join("dir"));
    }
}