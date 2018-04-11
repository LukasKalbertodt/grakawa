use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};


pub struct LockFile {
    /// We keep the lock file opened to provide information about which process
    /// is holding the lock. The `Option` is only here for technical reasons:
    /// We need to drop this file to close it before doing other work in
    /// `drop()`. This field is always `Some`.
    file: Option<File>,
    path: PathBuf,
}

impl LockFile {
    pub fn lock(path: &Path) -> Result<Option<Self>, io::Error> {
        // Atomically check if the file exists and create one if not
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(file) => {
                Ok(
                    Some(
                        Self {
                            file: Some(file),
                            path: path.to_owned(),
                        }
                    )
                )
            }
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        // We drop the lock_file here to free the file handle and close the
        // file.
        self.file.take();
        if let Err(e) = fs::remove_file(&self.path) {
            warn!("Could not delete .lock file! Details: {}", e);
        }
    }
}
