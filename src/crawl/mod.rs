use failure::Error;
use regex::Regex;
use scraper::{Html, Selector};

use db::product::Prices;



const BASE_URL: &str = "https://geizhals.de/";



pub fn load_price_history(product_id: u32) -> Result<(), Error> {
    let url = format!("{}?phist={}", BASE_URL, product_id);
    println!("getting: {}", url);

    let body = ::reqwest::get(&url)?.text()?;
    let html = Html::parse_document(&body);
    let selector = Selector::parse("script").unwrap();

    let script = html.select(&selector)
        .map(|elem| elem.inner_html())
        .find(|inner| inner.contains(".plot("))
        .unwrap();

    lazy_static! {
        static ref EXTRACT_DATA: Regex
            = Regex::new(r#"_gh\.plot\([0-9]+, \[(\[.+?\])\]"#).unwrap();
        static ref SPLIT_DATA: Regex
            = Regex::new(r#"\[(?P<timestamp>[0-9]+),(?P<price>[0-9\.]+)\]"#).unwrap();
    }

    let data = EXTRACT_DATA.captures(&script).unwrap().get(1).unwrap();

    for caps in SPLIT_DATA.captures_iter(data.as_str()) {
        println!("{} -> {}", &caps["timestamp"], &caps["price"]);
    }


    // println!("{}", data.as_str());

    Ok(())
}
