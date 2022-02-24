use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use scraper::html::Html;
use scraper::selector::Selector;
use serde_json::{to_writer, Map, Value};
use std::{fs::File, io::Write};

pub struct Crawler {
    client: Client,
}

impl Crawler {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Crawls the sitemap.php and finds spot IDs and names. Writes to the buffer in a JSON format, i.e.
    /// `{ "ormond-beach": 4203 }`
    ///
    /// TODO: support "ormond-beach-fl" too...
    /// TODO: async and concurrent requests to crawl each spot for lat/long
    pub fn crawl_spot_ids(&self, writer: &mut impl Write) -> Result<()> {
        let html = self
            .client
            .get("https://magicseaweed.com/site-map.php")
            .send()?
            .text()?;
        let mut file = File::create("site-map.html")?;
        file.write_all(html.as_bytes())?;
        parse_spot_ids(&html, writer)
    }
}

fn parse_spot_ids<T: Write>(html: &str, writer: T) -> Result<()> {
    let mut spot_json_map = Map::new();
    let spot_anchors = Selector::parse("h1.header + table a").unwrap();
    let document = Html::parse_document(html);
    for anchor in document.select(&spot_anchors) {
        // TODO actually just make this an integer
        let spot_id = anchor
            .value()
            .attr("href")
            .and_then(|href| href.trim_end_matches('/').rsplit_once('/'))
            .map(|(_, spot_id)| spot_id.to_owned())
            .filter(|s| s.chars().all(|c| c.is_digit(10)))
            .ok_or(anyhow!("Failed to parse spot ID from HTML anchor"))?;
        // TODO kebab case
        let spot_name = anchor.inner_html();
        spot_json_map.insert(spot_name, Value::String(spot_id));
    }
    to_writer(writer, &Value::Object(spot_json_map))?;
    Ok(())
}

impl Default for Crawler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    #[test]
    fn crawl_works() {
        let html = include_str!("../../../test/msw/site-map.html");
        let mut buffer = Cursor::new(Vec::new());
        parse_spot_ids(&html, &mut buffer).unwrap();
        buffer.set_position(0);
        let spots: Value = serde_json::from_reader(buffer).unwrap();
        assert_eq!(*spots.get("Ormond Beach").unwrap(), json!("4203"));
    }
}
