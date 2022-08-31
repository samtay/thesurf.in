use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use scraper::html::Html;
use scraper::selector::Selector;
use serde_json::{to_writer, Map, Value};
use std::{collections::HashMap, fs::File, io::Write, path::Path};

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
        let spot_id: u16 = anchor
            .value()
            .attr("href")
            .and_then(|href| href.trim_end_matches('/').rsplit_once('/'))
            .map(|(_, spot_id)| spot_id.to_owned())
            .ok_or(anyhow!("Failed to find spot ID in HTML anchor"))
            .and_then(|s| s.parse().context("Couldn't parse spot ID into integer"))?;
        let spot_name = anchor
            .inner_html()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-");
        spot_json_map.insert(spot_name, spot_id.into());
    }
    to_writer(writer, &Value::Object(spot_json_map))?;
    Ok(())
}

impl Default for Crawler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Spots {
    spots: HashMap<String, u16>,
}

impl Spots {
    /// Create a new Spots struct, pulling data from ./data/spots.json
    pub fn new() -> Result<Self> {
        Self::from_path("./data/spots.json")
    }

    /// Create a new Spots struct, pulling data from the given path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).context(format!(
            "Couldn't find spots json file at {:?}",
            path.as_ref()
        ))?;
        let spots = serde_json::from_reader(file).context(format!(
            "Couldn't parse file {:?} into spots json",
            path.as_ref()
        ))?;
        Ok(Self { spots })
    }

    /// Search the spots data for the MSW spot identifier (the integer)
    pub fn get_id<'a>(&self, name: impl Into<&'a str>) -> Option<u16> {
        self.spots.get(name.into()).copied()
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
        parse_spot_ids(html, &mut buffer).unwrap();
        buffer.set_position(0);
        let spots: Value = serde_json::from_reader(buffer).unwrap();
        assert_eq!(*spots.get("ormond-beach").unwrap(), json!(4203));
    }
}
