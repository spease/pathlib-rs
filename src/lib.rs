#[macro_use] extern crate failure;
extern crate url;

use failure::{Error, ResultExt};
use std::ffi::{OsStr, OsString};
use std::fs::{DirEntry, File, OpenOptions, ReadDir};
use std::ops::{Deref, Div};
use std::path::{Path, PathBuf};
use std::io::{BufReader, Read};
use std::iter::Map;

type Result<T> = std::result::Result<T, Error>;

struct ConcretePathOpen {
    path: PathBuf,
    mode: String,
    buffering: isize,
}

impl ConcretePathOpen {
    fn new(path: PathBuf) -> Self {
        ConcretePathOpen {
            path,
            mode: "r".to_owned(),
            buffering: -1,
        }
    }

    pub fn buffering(mut self, i_buffering: isize) -> Self {
        self.buffering = i_buffering;
        self
    }

    // FIXME: Add mode

    pub fn open(self) -> Result<BufReader<File>> {
        let o = OpenOptions::new().open(self.path)?;
        let b = if self.buffering < 0 {
            BufReader::new(o)
        } else {
            BufReader::with_capacity(self.buffering as usize, o)
        };
        Ok(b)
    }
}

#[derive(PartialEq)]
struct PurePath(PathBuf);

impl PurePath {
    fn new() -> Self {
        PurePath(PathBuf::new())
    }
}

/*
#[derive(PartialEq)]
struct PurePosixPath(std::sys::unix::ext::ffi::OsString);

impl PurePosixPath {
    fn new() -> Self {
        PurePosixPath(std::sys::unix::ext::ffi::OsString::new())
    }    
}
*/

impl Deref for PurePath {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<Path> for PurePath {
    fn as_ref(&self) -> &Path {
        return self.0.as_ref()
    }
}

impl From<PathBuf> for PurePath {
    fn from(p: PathBuf) -> Self {
        PurePath(p)
    }
}

impl<'a> Div<&'a Path> for PurePath {
    type Output = PurePath;
    fn div(self, rhs: &'a Path) -> Self::Output {
        PurePath(self.0.join(rhs))
    }
}

impl PathLike for PurePath {
    type Value = PurePath;
}

trait PathLike: AsRef<Path> {
    type Value: PathLike + From<PathBuf>;

    fn anchor(&self) -> Option<Self::Value> {
        unimplemented!()
    }

    fn as_posix(&self) -> Self::Value {
        unimplemented!()
    }

    fn as_uri(&self) -> Result<url::Url> {
        unimplemented!()
    }

    fn drive(&self) -> Option<&OsStr> {
        unimplemented!()
    }

    fn name(&self) -> Option<&OsStr> {
        self.as_ref().file_name()
    }

    fn parent(&self) -> Option<Self::Value> {
        self.as_ref().parent().map(PathBuf::from).map(Self::Value::from)
    }

    fn parents(&self) -> Option<Vec<&OsStr>> {
        self.as_ref().parent().map(|p|p.iter().collect())
    }

    fn parts(&self) -> Vec<&OsStr> {
        self.as_ref().iter().collect()
    }

    fn relative_to<P>(&self, i_path: P) -> Result<Self::Value>
        where P: PathLike {
        self.as_ref().strip_prefix(i_path.as_ref()).map(PathBuf::from).map(Self::Value::from).map_err(|e|e.into())
    }

    fn root(&self) -> Result<Self::Value> {
        unimplemented!()
    }

    fn stem(&self) -> Option<&OsStr> {
        self.as_ref().file_stem()
    }

    fn suffix(&self) -> Option<&OsStr> {
        unimplemented!()
    }

    fn suffixes(&self) -> Vec<&OsStr> {
        unimplemented!()
    }

    fn with_name(&self, i_name: &OsStr) -> Self::Value {
        Self::Value::from(self.as_ref().with_file_name(i_name))
    }

    fn with_suffix(&self, i_suffix: &OsStr) -> Self::Value {
        let p = self.as_ref();
        let stem = p.file_stem().and_then(|stem|{let mut s=stem.to_os_string(); s.push(i_suffix); Some(s)}).unwrap_or(i_suffix.to_os_string());
        Self::Value::from(p.join(stem))
    }
}

trait ConcretePathLike: PathLike {
    fn cwd() -> Result<Self::Value> {
        std::env::current_dir().map(Self::Value::from).map_err(|e|e.into())
    }

    fn home() -> Option<Self::Value> {
        std::env::home_dir().map(Self::Value::from)
    }

    fn exists(&self) -> bool {
        self.as_ref().exists()
    }

    #[cfg(unix)]
    fn is_block_device(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;
        self.as_ref()
            .symlink_metadata()
            .map(|m|m.file_type().is_block_device())
            .map_err(|e|e.into())
    }

    #[cfg(not(unix))]
    fn is_block_device(&self) -> Result<bool> {
      unimplemented!()
    }

    #[cfg(unix)]
    fn is_char_device(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;
        self.as_ref()
            .symlink_metadata()
            .map(|m|m.file_type().is_char_device())
            .map_err(|e|e.into())
    }

    #[cfg(not(unix))]
    fn is_char_device(&self) -> Result<bool> {
      unimplemented!()
    }

    fn is_dir(&self) -> bool {
        self.as_ref().is_dir()
    }

    #[cfg(unix)]
    fn is_fifo(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;
        self.as_ref()
            .symlink_metadata()
            .map(|m|m.file_type().is_fifo())
            .map_err(|e|e.into())
    }

    #[cfg(not(unix))]
    fn is_fifo(&self) -> Result<bool> {
      unimplemented!()
    }

    fn is_file(&self) -> bool {
        self.as_ref().is_file()
    }

    fn is_symlink(&self) -> Result<bool> {
        self.as_ref()
            .symlink_metadata()
            .map(|m|m.file_type().is_symlink())
            .map_err(|e|e.into())
    }

    #[cfg(unix)]
    fn is_socket(&self) -> Result<bool> {
        use std::os::unix::fs::FileTypeExt;
        self.as_ref()
            .symlink_metadata()
            .map(|m|m.file_type().is_socket())
            .map_err(|e|e.into())
    }

    #[cfg(not(unix))]
    fn is_socket(&self) -> Result<bool> {
        unimplemented!()
    }

    /*
    fn iterdir(&self) -> Result<Map<ReadDir, fn(Result<DirEntry>) -> Result<Self::Value>>> {
        fn read_dir_to_value<T: From<PathBuf>>(entry: ReadDir) -> Map<ReadDir, fn(Result<DirEntry>) -> Result<T>> {
            entry.map(|entry|entry.map(|e|T::from(e.path())).map_err(|e|e.into()))
        }
        std::fs::read_dir(self.as_ref()).map(read_dir_to_value).map_err(|e|e.into())
    }
    */

    fn open(&self) -> ConcretePathOpen {
        ConcretePathOpen::new(self.as_ref().to_owned())
    }

    fn read_bytes(&self) -> Result<Vec<u8>> {
        let p = self.as_ref();
        let mut b = Vec::with_capacity(p.metadata()?.len() as usize);
        File::open(p)?.read_to_end(&mut b)?;
        Ok(b)
    }

    fn read_text(&self) -> Result<String> {
        let p = self.as_ref();
        let mut s = String::with_capacity(p.metadata()?.len() as usize);
        File::open(p)?.read_to_string(&mut s)?;
        Ok(s)
    }

    fn rename(&self, i_destination: &Path) -> Result<()> {
        std::fs::rename(self.as_ref(), i_destination).map_err(|e|e.into())
    }

    fn replace(&self, i_destination: &Path) -> Result<()> {
        self.rename(i_destination)
    }

    fn resolve(&self) -> Result<Self::Value> {
        self.as_ref().canonicalize().map(Self::Value::from).map_err(|e|e.into())
    }

    fn rmdir(&self) -> Result<()> {
        std::fs::remove_dir(self.as_ref()).map_err(|e|e.into())
    }

    fn unlink(&self) -> Result<()> {
        std::fs::remove_file(self.as_ref()).map_err(|e|e.into())
    }
}

struct ConcretePath(PathBuf);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
