extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate reqwest;
extern crate scraper;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate structopt;

use failure::Error;
use structopt::StructOpt;

mod crawl;
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
    // Parse command line arguments
    let args = Args::from_args();

    // Open DB
    let mut db = db::Db::new(&args.db_path)?;


    // Execute subcommand
    match args.command {
        // Add a specific ID to the database
        Command::Add { id: Some(id), from_search: None } => {
            let p = db.add_product(id)?;
            if p.is_some() {
                println!("Product was added");
            } else {
                println!("Product couldn't be added: ID '{}' already exists in DB", id);
            }
        }
        // Add all products from the result of a search to the database
        Command::Add { id: None, from_search: Some(search_url) } => {
            eprintln!("Not implemented yet...");
        }
        Command::Add { .. } => unreachable!(),

        // Update all products in the database
        Command::Update { all: true, id: None } => {
            for product_id in db.product_ids() {
                let p = db.get_product(product_id)?.unwrap();
                let prices = crawl::load_price_history(product_id)?;
                p.write_prices(&prices)?;
            }
        }
        // Update a specific product in the database
        Command::Update { all: false, id: Some(id) } => {
            match db.get_product(id)? {
                Some(p) => {
                    let prices = crawl::load_price_history(id)?;
                    p.write_prices(&prices)?;
                }
                None => {
                    eprintln!("Product with ID '{}' not found in DB!", id)
                }
            }
        }
        Command::Update { .. } => unreachable!(),
    }


    Ok(())
}


#[derive(Debug, StructOpt)]
#[structopt(name = "grakawa")]
struct Args {
    #[structopt(subcommand)]
    command: Command,

    #[structopt(long = "db-path", default_value = "db")]
    db_path: String,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "add")]
    Add {
        #[structopt(
            long = "id",
            help = "ID of product to add to the database",
            conflicts_with = "from_search",
            raw(required_unless = r#""from_search""#),
        )]
        id: Option<u32>,

        #[structopt(
            long = "from-search",
            help = "URL of the Geizhals search from which to add all products",
            raw(required_unless = r#""id""#),
        )]
        from_search: Option<String>,
    },
    #[structopt(name = "update")]
    Update {
        #[structopt(
            long = "all",
            help = "Update all products in the database",
            conflicts_with = "id",
            raw(required_unless = r#""id""#),
        )]
        all: bool,

        #[structopt(
            long = "id",
            help = "ID of the product which should be updated",
            raw(required_unless = r#""all""#),
        )]
        id: Option<u32>,
    }
}
