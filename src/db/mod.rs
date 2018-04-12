use failure::{Error, ResultExt};

use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};


mod index;
mod lock;
pub mod product;


use self::index::Index;
use self::lock::LockFile;
use self::product::Product;




const INDEX_FILE_NAME: &str = "index.json";
const LOCK_FILE_NAME: &str = ".lock";

pub struct Db {
    db_path: PathBuf,
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
        let index = Index::open_or_create(&index_path)
            .context("couldn't open index")?;

        // TODO: check if index file corresponds to folder structure

        Ok(Self {
            db_path,
            index,
            _lock: lock,
        })
    }

    pub fn add_product(&mut self, id: u32) -> Result<Option<Product>, Error> {
        if self.index.add_product_id(id)? {
            let p = Product::create(id, &self.db_path)?;
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }

    pub fn get_product(&self, id: u32) -> Result<Option<Product>, Error> {
        Product::open(id, &self.db_path)
    }

    pub fn get_or_add_product(&mut self, id: u32) -> Result<Product, Error> {
        match Product::open(id, &self.db_path)? {
            Some(p) => Ok(p),
            None => Ok(self.add_product(id)?.unwrap()),
        }
    }

    pub fn product_ids<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.index.product_ids().iter().cloned()
    }
}
