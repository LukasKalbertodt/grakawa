use chrono::NaiveDate;
use failure::{Error, ResultExt};

use std::{
    collections::BTreeMap,
    fs::{self, File},
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

    #[allow(unused)]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Reads the current price data from file. If there is no price data,
    /// an empty data set will be created.
    #[allow(unused)]
    pub fn read_prices(&self) -> Result<Prices, Error> {
        let path = self.path.join(PRICE_FILE_NAME);
        if !path.exists() {
            self.write_prices(&Prices::empty())?;
        }

        let f = File::open(path)?;
        Ok(
            ::serde_json::from_reader(&f)
                .context("couldn't read price file")?
        )
    }

    /// Writes the given price data to file, overwriting all prior data.
    pub fn write_prices(&self, prices: &Prices) -> Result<(), Error> {
        let mut f = File::create(self.path.join(PRICE_FILE_NAME))?;
        ::serde_json::to_writer_pretty(&mut f, prices)?;

        Ok(())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Prices {
   #[serde(flatten)]
   pub prices: BTreeMap<NaiveDate, Euro>,
}

impl Prices {
    #[allow(unused)]
    pub fn empty() -> Self {
        Self {
            prices: BTreeMap::new(),
        }
    }
}
