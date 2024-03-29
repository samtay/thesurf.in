use std::fmt::Display;

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Swell {
    pub min_breaking_height: f32,
    pub abs_min_breaking_height: f32,
    pub max_breaking_height: f32,
    pub abs_max_breaking_height: f32,
    pub unit: UnitLength,
    pub components: SwellComponents,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwellComponents {
    pub combined: Option<SwellComponent>,
    pub primary: Option<SwellComponent>,
    pub secondary: Option<SwellComponent>,
    pub tertiary: Option<SwellComponent>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwellComponent {
    pub height: f32,
    pub period: u16,
    pub direction: f32,
    pub compass_direction: CompassDirection,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Wind {
    pub speed: u32,
    pub direction: f32,
    pub compass_direction: CompassDirection,
    #[serde(deserialize_with = "int_fmt::deserialize")]
    pub chill: i32,
    pub gusts: u32,
    pub unit: UnitSpeed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub pressure: u32,
    #[serde(deserialize_with = "int_fmt::deserialize")]
    pub temperature: i32,
    pub unit_pressure: String, // display purposes only
    #[serde(rename = "unit")]
    pub unit_temperature: UnitTemperature,
}

// or URL types
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Charts {
    pub swell: Option<String>,
    pub period: Option<String>,
    pub wind: Option<String>,
    pub pressure: Option<String>,
    pub sst: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum UnitLength {
    #[serde(rename = "ft")]
    Feet,
    #[serde(rename = "m")]
    Meters,
}

impl Display for UnitLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Feet => write!(f, "ft"),
            Self::Meters => write!(f, "m"),
        }
    }
}

impl Display for UnitSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Mph => write!(f, "mph"),
            Self::Kph => write!(f, "kph"),
        }
    }
}

impl Display for UnitTemperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::C => write!(f, "°C"),
            Self::F => write!(f, "°F"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UnitSpeed {
    Mph,
    Kph,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UnitTemperature {
    C,
    F,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
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

/// Unit options supported by MSW
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitType {
    Uk,
    Us,
    Eu,
}

pub struct ForecastAPI {
    client: Client,
    units: Option<UnitType>,
}

impl ForecastAPI {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            units: None,
        }
    }

    pub fn units(mut self, unit_type: Option<UnitType>) -> Self {
        self.units = unit_type;
        self
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
        {
            let mut query_pairs = api_url.query_pairs_mut();
            query_pairs.append_pair("spot_id", &spot_id.to_string());
            if let Some(ut) = self.units {
                query_pairs.append_pair("units", serde_json::to_value(ut)?.as_str().unwrap());
            }
        }
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

// MSW sometimes sends "-0" which serde fails to serialize to an integer
mod int_fmt {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer).map(|x| x.round() as i32)
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
                min_breaking_height: 2.0,
                max_breaking_height: 3.0,
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

    #[test]
    fn json_doesnt_bomb_on_neg_0() {
        // For some reason MSW sends "-0" as a value for wind chill. Since we're
        // parsing as an int, this fails deserialization.
        // Side note: should really only request what we want from MSW but w/e
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
            "chill": -0,
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
                min_breaking_height: 2.0,
                max_breaking_height: 3.0,
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
                chill: 0,
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
            serde_json::from_str::<'_, Forecast>(msw_json)
                .unwrap()
                .wind
                .chill,
            expected_forecast.wind.chill
        );
    }
}
