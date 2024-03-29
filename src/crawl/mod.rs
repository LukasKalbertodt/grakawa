use chrono::NaiveDateTime;
use failure::Error;
use regex::Regex;
use reqwest::{
    header::Cookie,
    Client,
};
use scraper::{Html, Selector};
use url::Url;


use db::product::Prices;
use util::Euro;



const BASE_URL: &str = "https://geizhals.de/";


/// Loads a page and parses it as HTML document.
fn get<S: AsRef<str>>(url: S) -> Result<Html, Error> {
    debug!("GETting: {}", url.as_ref());

    let body = ::reqwest::get(url.as_ref())?.text()?;
    Ok(Html::parse_document(&body))
}

/// Removes the base url "https://geizhals.de" if present.
pub fn remove_base(url: &str) -> String {
    lazy_static! {
        static ref POSSIBLE_PREFIXES: Regex
            = Regex::new(r#"^https?://(www\.)?geizhals\.(de|at|eu)/"#).unwrap();
    }

    POSSIBLE_PREFIXES.replace(url, "").into_owned()
}

/// Loads the price history of the product with the given id.
pub fn load_price_history(product_id: u32) -> Result<Prices, Error> {
    let url = format!("{}?phist={}", BASE_URL, product_id);
    let html = get(url)?;

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

/// ...
///
/// The given query string has to be the query part of the URL. It needs to
/// contain the `cat` parameter.
pub fn products_from_search(query_string: &str) -> Result<Vec<u32>, Error> {
    let raw_url = format!("{}{}", BASE_URL, query_string);

    // 1000 is the maximum value the Geizhals server will accept
    let cookies = {
        let mut c = Cookie::new();
        c.set("blaettern", "1000");
        c
    };

    let client = Client::new();
    let mut ids = vec![];
    let mut remaining = 0i32;

    for page in 1.. {
        // Set the `pg` query parameter to request the correct page. If the
        // query parameter is already present, we need to remove it first
        let mut url = Url::parse(&raw_url)?;
        if url.query_pairs().find(|(k, _)| k == "pg").is_some() {
            let pairs = url.query_pairs()
                .filter(|&(ref k, _)| k != "pg")
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect::<Vec<_>>();

            url.query_pairs_mut()
                .clear()
                .extend_pairs(pairs)
                .finish();

        }

        url.query_pairs_mut()
            .append_pair("pg", &page.to_string())
            .finish();


        // Next, get the page from the interwebz
        debug!("GETting '{}'", url.as_str());
        let body = client.get(url.as_str()).header(cookies.clone()).send()?.text()?;
        let html = Html::parse_document(&body);

        // If this is our first request, we need to find out the total number
        // of products
        if page == 1 {
            let count_selector = Selector::parse("div.productlist__name > a").unwrap();
            let node = html.select(&count_selector).next()
                .ok_or(format_err!("Unexpected HTML (missing article count)"))?;

            lazy_static! {
                static ref NUMBER: Regex = Regex::new(r#"[0-9]+"#).unwrap();
            }

            let s = node.inner_html();
            remaining = NUMBER.find(&s)
                .ok_or(format_err!("Unexpected HTML (invalid article count)"))?
                .as_str()
                .parse::<i32>()?;
        }


        // Search for the products in the list
        let product_item = Selector::parse(
            "div.productlist__product > div.productlist__compare > input"
        ).unwrap();


        for elem in html.select(&product_item) {
            let s = elem.value()
                .attr("value")
                .ok_or(format_err!("Unexpected HTML (missing value parameter)"))?;
            let id = s.parse()?;
            ids.push(id);
        }

        // If there are no more products to check
        remaining -= 1000;
        if remaining <= 0 {
            break;
        } else {
            debug!("{} products remaining in this search", remaining);
        }
    }

    Ok(ids)
}
