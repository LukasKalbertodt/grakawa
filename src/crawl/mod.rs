use chrono::NaiveDateTime;
use failure::Error;
use regex::Regex;
use scraper::{Html, Selector};


use db::product::Prices;
use util::Euro;



const BASE_URL: &str = "https://geizhals.de/";


/// Loads a page from Geizhals and parses it as HTML document. The given `path`
/// is prepended by `BASE_URL`.
fn get<S: AsRef<str>>(path: S) -> Result<Html, Error> {
    let url = format!("{}{}", BASE_URL, path.as_ref());
    debug!("GETting: {}", url);

    let body = ::reqwest::get(&url)?.text()?;
    Ok(Html::parse_document(&body))
}

/// Loads the price history of the product with the given id.
pub fn load_price_history(product_id: u32) -> Result<Prices, Error> {
    let path = format!("?phist={}", product_id);
    let html = get(path)?;

    // Find the `<script>` tag which contains the data we are after
    let selector = Selector::parse("script").unwrap();
    let script = html.select(&selector)
        .map(|elem| elem.inner_html())
        .find(|inner| inner.contains(".plot("))
        .ok_or(format_err!("price history <script> tag not found"))?;

    // Next we extract the information via two regexes.
    lazy_static! {
        // This one simply extracts the list of arrays as a big string
        static ref EXTRACT_DATA: Regex
            = Regex::new(r#"_gh\.plot\([0-9]+, \[(\[.+?\])\]"#).unwrap();

        // This one searches for timestamp-price pairs
        static ref SPLIT_DATA: Regex
            = Regex::new(r#"\[(?P<timestamp>[0-9]+),(?P<price>[0-9\.]+)\]"#).unwrap();
    }

    // Find the array of data. We can unwrap at the end because if we have a
    // match, we know that we also have that capture group.
    let data = EXTRACT_DATA.captures(&script)
        .ok_or(format_err!("price history data not found in <script> tag"))?
        .get(1).unwrap();

    // Iterate over all matches of the second regex, parse the captured groups
    // into number representation and collect everything in a map.
    let prices = SPLIT_DATA
        .captures_iter(data.as_str())
        .map(|caps| {
            let price = caps["price"].parse::<Euro>()?;

            // We divide by 1000 because Geizhals uses millisecond timestamps
            let timestamp = caps["timestamp"].parse::<i64>()? / 1000;
            let date = NaiveDateTime::from_timestamp(timestamp, 0).date();

            Ok((date, price))
        })
        .collect::<Result<_, Error>>()?;

    Ok(Prices { prices })
}
