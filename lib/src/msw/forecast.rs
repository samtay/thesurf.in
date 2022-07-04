use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use reqwest::{Client, Url};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Forecast {
    pub timestamp: i64,
    #[serde(deserialize_with = "timestamp_fmt::deserialize")]
    pub local_timestamp: NaiveDateTime,
    pub faded_rating: u8, // or custom star rating enum
    pub solid_rating: u8,
    pub swell: Swell,
    pub wind: Wind,
    pub condition: Condition,
    pub charts: Charts,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Swell {
    pub min_breaking_height: u8,
    pub abs_min_breaking_height: f32,
    pub max_breaking_height: u8,
    pub abs_max_breaking_height: f32,
    pub unit: UnitLength,
    pub components: SwellComponents,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwellComponents {
    pub combined: Option<SwellComponent>,
    pub primary: Option<SwellComponent>,
    pub secondary: Option<SwellComponent>,
    pub tertiary: Option<SwellComponent>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwellComponent {
    pub height: f32,
    pub period: u8,
    pub direction: f32,
    pub compass_direction: CompassDirection,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Wind {
    pub speed: u16,
    pub direction: f32,
    pub compass_direction: CompassDirection,
    pub chill: i16,
    pub gusts: u16,
    pub unit: UnitSpeed,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub pressure: u16,
    pub temperature: i16,
    pub unit_pressure: String, // display purposes only
    #[serde(rename = "unit")]
    pub unit_temperature: UnitTemperature,
}

// or URL types
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Charts {
    pub swell: Option<String>,
    pub period: Option<String>,
    pub wind: Option<String>,
    pub pressure: Option<String>,
    pub sst: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum UnitLength {
    #[serde(rename = "ft")]
    Feet,
    #[serde(rename = "m")]
    Meters,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UnitSpeed {
    Mph,
    Kph,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UnitTemperature {
    C,
    F,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CompassDirection {
    N,
    NNE,
    NE,
    ENE,
    E,
    ESE,
    SE,
    SSE,
    S,
    SSW,
    SW,
    WSW,
    W,
    WNW,
    NW,
    NNW,
}

pub struct ForecastAPI {
    client: Client,
}

impl ForecastAPI {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Gets forecast for the given spot ID
    ///
    /// https://magicseaweed.com/api/YOURAPIKEY/forecast/?spot_id=10
    pub async fn get(&self, spot_id: u16) -> Result<Vec<Forecast>> {
        let mut api_url = Url::parse("https://magicseaweed.com/api/")?;
        let api_key = option_env!("MSW_API_KEY").ok_or(anyhow!("Missing API Key"))?;
        api_url
            .path_segments_mut()
            .expect("https:// scheme implies URL can be a base")
            .push(api_key)
            .push("forecast");
        api_url
            .query_pairs_mut()
            .append_pair("spot_id", &spot_id.to_string());
        let forecast = self.client.get(api_url).send().await?.json().await?;
        Ok(forecast)
    }
}

impl Default for ForecastAPI {
    fn default() -> Self {
        Self::new()
    }
}

mod timestamp_fmt {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .ok_or_else(|| serde::de::Error::custom("Timestamp seconds out of range"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    #[ignore = "Dont hit MSW API in default test runs"]
    async fn forecast_works() {
        let forecast = ForecastAPI::new().get(4203).await;
        assert!(forecast.is_ok());
    }

    #[test]
    fn json_parses() {
        let msw_json = r#"{
          "timestamp": 1645678800,
          "localTimestamp": 1645660800,
          "issueTimestamp": 1645660800,
          "fadedRating": 1,
          "solidRating": 1,
          "swell": {
            "absMinBreakingHeight": 2.23,
            "absMaxBreakingHeight": 3.48,
            "probability": 100,
            "unit": "ft",
            "minBreakingHeight": 2,
            "maxBreakingHeight": 3,
            "components": {
              "combined": {
                "height": 4,
                "period": 10,
                "direction": 271.53,
                "compassDirection": "E"
              },
              "primary": {
                "height": 4,
                "period": 10,
                "direction": 271.53,
                "compassDirection": "E"
              }
            }
          },
          "wind": {
            "speed": 7,
            "direction": 335,
            "compassDirection": "SSE",
            "chill": 21,
            "gusts": 12,
            "unit": "mph"
          },
          "condition": {
            "pressure": 1023,
            "temperature": 20,
            "unitPressure": "mb",
            "unit": "c"
          },
          "charts": {
            "swell": "https://charts-s3.msw.ms/archive/wave/750/21-1645671600-24.gif",
            "period": "https://charts-s3.msw.ms/archive/wave/750/21-1645671600-25.gif",
            "wind": "https://charts-s3.msw.ms/archive/gfs/750/21-1645671600-4.gif",
            "pressure": "https://charts-s3.msw.ms/archive/gfs/750/21-1645671600-3.gif",
            "sst": "https://charts-s3.msw.ms/archive/sst/750/21-1645671600-10.gif"
          }
        }"#;
        let expected_local_timestamp = NaiveDateTime::from_timestamp(1645660800, 0);
        let expected_forecast = Forecast {
            timestamp: 1645678800,
            local_timestamp: expected_local_timestamp,
            faded_rating: 1, // or custom star rating enum
            solid_rating: 1,
            swell: Swell {
                abs_min_breaking_height: 2.23,
                abs_max_breaking_height: 3.48,
                min_breaking_height: 2,
                max_breaking_height: 3,
                unit: UnitLength::Feet,
                components: SwellComponents {
                    combined: Some(SwellComponent {
                        height: 4.0,
                        period: 10,
                        direction: 271.53,
                        compass_direction: CompassDirection::E,
                    }),
                    primary: Some(SwellComponent {
                        height: 4.0,
                        period: 10,
                        direction: 271.53,
                        compass_direction: CompassDirection::E,
                    }),
                    secondary: None,
                    tertiary: None,
                },
            },
            wind: Wind {
                speed: 7,
                direction: 335.0,
                compass_direction: CompassDirection::SSE,
                chill: 21,
                gusts: 12,
                unit: UnitSpeed::Mph,
            },
            condition: Condition {
                pressure: 1023,
                temperature: 20,
                //weather: 10,
                unit_pressure: "mb".to_string(),
                unit_temperature: UnitTemperature::C,
            },
            charts: Charts {
                swell: Some(
                    "https://charts-s3.msw.ms/archive/wave/750/21-1645671600-24.gif".to_string(),
                ),
                period: Some(
                    "https://charts-s3.msw.ms/archive/wave/750/21-1645671600-25.gif".to_string(),
                ),
                wind: Some(
                    "https://charts-s3.msw.ms/archive/gfs/750/21-1645671600-4.gif".to_string(),
                ),
                pressure: Some(
                    "https://charts-s3.msw.ms/archive/gfs/750/21-1645671600-3.gif".to_string(),
                ),
                sst: Some(
                    "https://charts-s3.msw.ms/archive/sst/750/21-1645671600-10.gif".to_string(),
                ),
            },
        };
        assert_eq!(
            serde_json::from_str::<'_, Forecast>(msw_json).unwrap(),
            expected_forecast
        );
        assert_eq!(
            expected_local_timestamp.to_string(),
            "2022-02-24 00:00:00".to_string()
        );
    }
}
