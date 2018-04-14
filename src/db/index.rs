use failure::{Error, ResultExt};

use std::{
    fs::{File, OpenOptions},
    io::{self, Seek, SeekFrom},
    path::{Path},
};

pub struct Index {
    data: IndexData,

    /// The file storing the index
    file: File,
}

impl Index {
    pub fn open_or_create(path: &Path) -> Result<Self, Error> {
        debug!("Trying to open index file");

        let open_options = {
            let mut oo = OpenOptions::new();
            oo.write(true);
            oo.read(true);
            oo
        };

        let (file, data) = match open_options.open(path) {
            Ok(f) => {
                let data = ::serde_json::from_reader(&f)
                    .context("couldn't read index file")?;

                (f, data)
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("No index file found: creating new index file");

                let mut f = File::create(path)?;

                // Write empty index to new file
                let data = IndexData::empty();
                ::serde_json::to_writer_pretty(&mut f, &data)
                    .context("Couldn't initialize the lock file")?;

                (f, data)
            }
            Err(e) => bail!(e),
        };

        Ok(Self { file, data })
    }


    pub fn add_product_ids(&mut self, ids: &[u32]) -> Result<usize, Error> {
        let new_ids = ids.iter().cloned()
            .filter(|id| self.data.product_ids.binary_search(&id).is_err())
            .collect::<Vec<_>>();

        self.data.product_ids.extend_from_slice(&new_ids);
        self.data.product_ids.sort();
        self.write()?;

        Ok(new_ids.len())
    }

    fn write(&mut self) -> Result<(), Error> {
        debug!("Writing index");

        // Remove all file contents and write the new index
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        Ok(::serde_json::to_writer_pretty(&mut self.file, &self.data)?)
    }

    pub fn product_ids(&self) -> &[u32] {
        &self.data.product_ids
    }
}

#[derive(Serialize, Deserialize)]
struct IndexData {
    product_ids: Vec<u32>,
}

impl IndexData {
    fn empty() -> Self {
        Self {
            product_ids: vec![],
        }
    }
}
