use failure::{Error};

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
};


const INDEX_FILE_NAME: &str = "index.json";
const LOCK_FILE_NAME: &str = ".lock";

pub struct Db {
    db_path: PathBuf,
    product_ids: Vec<u32>,
    index_file: File,

    /// We keep the lock file opened to provide information about which process
    /// is holding the lock. The `Option` is only here for technical reasons:
    /// We need to drop this file to close it before doing other work in
    /// `drop()`. This field is always `Some`.
    lock_file: Option<File>,
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

        // Prepare OpenOptions to atomically create a new file
        let atomic_create = {
            let mut oo = OpenOptions::new();
            oo.write(true);
            oo.create_new(true);
            oo
        };

        // Atomically check if a .lock file exists and create one if not
        let lock_path = db_path.join(LOCK_FILE_NAME);
        let lock_file = match atomic_create.open(&lock_path) {
            Ok(f) => f,
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                bail!(
                    "The database is in a locked state -- there is probably \
                     another instance of this process running. If that's not \
                     the case, you can probably simply delete '{}' and \
                     restart this program.",
                     lock_path.display()
                );
            }
            Err(e) => bail!(e),
        };

        // Check if a `index.json` file already exists
        let index_path = db_path.join(INDEX_FILE_NAME);
        let index_file = match atomic_create.open(&index_path) {
            Ok(f) => f,
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
                File::open(index_path)?
            }
            Err(e) => bail!(e),
        };

        Ok(Self {
            db_path,
            product_ids: vec![],
            index_file,
            lock_file: Some(lock_file),
        })
    }

    // pub fn create_product(&mut self, id: u32) -> Product {
    //     unimplemented!()
    // }
}

impl Drop for Db {
    fn drop(&mut self) {
        // We drop the lock_file here to free the file handle and close the
        // file.
        self.lock_file.take();
        if let Err(e) = fs::remove_file(self.db_path.join(LOCK_FILE_NAME)) {
            warn!("Could not delete .lock file! Details: {}", e);
        }
    }
}


// pub struct Product {
//     id: u32,
// }
