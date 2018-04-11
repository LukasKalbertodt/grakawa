extern crate env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate scraper;

use failure::Error;

mod db;


fn main() {
    env_logger::init();

    if let Err(e) = run() {
        error!("An error occured: {}", e.cause());

        for cause in e.causes().skip(1) {
            error!("... caused by: {}", cause);
        }

        if let Some(bt) = e.cause().backtrace() {
            error!("Backtrace: {}", bt);
        }
    }
}


fn run() -> Result<(), Error> {
    db::Db::new("db")?;

    Ok(())
}
