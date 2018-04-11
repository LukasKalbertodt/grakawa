use failure::{Error, ResultExt};

use std::{
    fs::{self, File, OpenOptions},
    io::{self},
    path::{Path, PathBuf},
};


const INDEX_FILE_NAME: &str = "index.json";
const LOCK_FILE_NAME: &str = ".lock";

pub struct Db {
    db_path: PathBuf,
    index_file: File,
    index: Index,

    _lock: LockFile,
}

impl Db {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, Error> {
        let db_path = db_path.as_ref().to_owned();

        // Check if the folder "db" already exists
        match fs::metadata(&db_path) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    bail!("something called 'db' exists but is not a folder!");
                }
                debug!("Using existing `db/` folder");
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("Creating new `db/` folder");
                fs::create_dir(&db_path)?;
            }
            Err(e) => bail!(e),
        }

        // Create a (file) lock on the database
        let lock_path = db_path.join(LOCK_FILE_NAME);
        let lock = match LockFile::lock(&lock_path)? {
            Some(l) => l,
            None => {
                bail!(
                    "Cannot open database: the database is in a locked state.
                     There is probably another instance of this process \
                     running. If that's not the case, you can probably simply \
                     delete '{}' and restart this program.",
                     lock_path.display()
                );
            }
        };

        // Open `index.json` file or create it if it doesn't exist
        let index_path = db_path.join(INDEX_FILE_NAME);
        let open_options = {
            let mut oo = OpenOptions::new();
            oo.write(true);
            oo.read(true);
            oo
        };

        let (index_file, index) = match open_options.open(&index_path) {
            Ok(f) => {
                let index = ::serde_json::from_reader(&f)
                    .context("couldn't read index file")?;

                (f, index)
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                let mut f = File::create(index_path)?;

                // Write empty index to new file
                let index = Index::empty();
                ::serde_json::to_writer_pretty(&mut f, &index)
                    .context("Couldn't initialize the lock file")?;

                (f, index)
            }
            Err(e) => bail!(e),
        };

        Ok(Self {
            db_path,
            index_file,
            _lock: lock,
            index,
        })
    }

    // pub fn create_product(&mut self, id: u32) -> Product {
    //     unimplemented!()
    // }
}




struct LockFile {
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


#[derive(Serialize, Deserialize)]
pub struct Index {
    product_ids: Vec<u32>,
}

impl Index {
    fn empty() -> Self {
        Self {
            product_ids: vec![],
        }
    }
}


// pub struct Product {
//     id: u32,
// }
