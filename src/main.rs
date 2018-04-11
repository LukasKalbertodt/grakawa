extern crate env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate scraper;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use failure::Error;

mod db;
mod util;


fn main() {
    env_logger::init();

    if let Err(e) = run() {
        error!("An error occured: {}", e.cause());

        for cause in e.causes().skip(1) {
            error!("... caused by: {}", cause);
        }

        info!("If you set RUST_BACKTRACE=1 a backtrace is printed to stderr");
        eprintln!("{}", e.backtrace());
    }
}


fn run() -> Result<(), Error> {
    let mut db = db::Db::new("db")?;

    let product = db.get_or_create_product(123)?;

    let mut prices = product.read_prices()?;
    prices.prices.insert("peter".into(), util::Euro::from_cents(710));
    product.write_prices(&prices)?;


    Ok(())
}
