use failure::{Error, ResultExt};

use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use util::Euro;



const PRICE_FILE_NAME: &str = "prices.json";


pub struct Product<'db> {
    id: u32,
    path: PathBuf,
    _dummy: PhantomData<&'db ()>,
}

impl<'a> Product<'a> {
    pub fn create(id: u32, db_path: &Path) -> Result<Self, Error> {
        // Create product folder if it doesn't exist already
        let product_path = db_path.join(format!("p{:09}", id));
        match fs::metadata(&product_path) {
            Ok(ref metadata) if !metadata.is_dir() => {
                bail!(
                    "something called '{}' exists but is not a folder!",
                    product_path.display()
                );
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&product_path)?;
            }
            Err(e) => bail!(e),
        }

        Ok(Self {
            id,
            path: product_path,
            _dummy: PhantomData,
        })
    }

    pub fn open(id: u32, db_path: &Path) -> Result<Option<Self>, Error> {
        let product_path = db_path.join(format!("p{:09}", id));
        match fs::metadata(&product_path) {
            Ok(ref metadata) if !metadata.is_dir() => Ok(None),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Ok(_) => {
                Ok(Some(Self {
                    id,
                    path: product_path,
                    _dummy: PhantomData,
                }))
            }
            Err(e) => bail!(e),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn read_prices(&self) -> Result<Prices, Error> {
        let f = File::open(self.path.join(PRICE_FILE_NAME))?;
        Ok(::serde_json::from_reader(&f).context("couldn't read price file")?)
    }

    pub fn write_prices(&self, prices: &Prices) -> Result<(), Error> {
        let mut f = File::create(self.path.join(PRICE_FILE_NAME))?;
        ::serde_json::to_writer_pretty(&mut f, prices)?;

        Ok(())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Prices {
   #[serde(flatten)]
   pub prices: HashMap<String, Euro>,
}

impl Prices {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
        }
    }
}
