use anyhow::Result;
use reqwest::{blocking::Client, Url};

pub struct Forecast {
    client: Client,
}

impl Forecast {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Gets forecast for the given spot ID
    ///
    /// https://magicseaweed.com/api/a6b398fd38c1730c375d12526ae26a42/forecast/?spot_id=1
    /// https://magicseaweed.com/api/YOURAPIKEY/forecast/?spot_id=10
    pub fn get(&self, spot_id: u16) -> Result<()> {
        let mut api_url = Url::parse("https://magicseaweed.com/api/")?
            .join(env!("MSW_API_KEY"))?
            .join("forecast")?;
        api_url
            .query_pairs_mut()
            .append_pair("spot_id", &spot_id.to_string());
        let _forecast = self.client.get(api_url).send()?.json()?;
        Ok(())
    }
}

impl Default for Forecast {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forecast_works() {
        let forecast = Forecast::new().get(4203);
        assert!(forecast.is_ok());
    }
}
