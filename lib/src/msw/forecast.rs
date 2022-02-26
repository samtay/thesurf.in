use anyhow::{bail, Result};
use reqwest::{blocking::Client, Url};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Forecast {
    pub timestamp: i64, // or decode into date time type?
    pub local_timestamp: i64,
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
    pub unit: String, // TODO enum "ft",
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
    pub compass_direction: String, // TODO enum "W"
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Wind {
    pub speed: u16,
    pub direction: f32,
    pub compass_direction: String, // or custom direction enum
    pub chill: i16,
    pub gusts: u16,
    pub unit: String, // or custom unit enum "mph"
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub pressure: u16,
    pub temperature: i16,
    pub unit_pressure: String, // or custom enum
    #[serde(rename = "unit")]
    pub unit_temperature: String, // custom enum
}

// or URL types
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Charts {
    pub swell: String,
    pub period: String,
    pub wind: String,
    pub pressure: String,
    pub sst: String,
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
    pub fn get(&self, spot_id: u16) -> Result<Vec<Forecast>> {
        let mut api_url = Url::parse("https://magicseaweed.com/api/")?;
        api_url
            .path_segments_mut()
            .expect("https:// scheme implies URL can be a base")
            .push(env!("MSW_API_KEY"))
            .push("forecast");
        api_url
            .query_pairs_mut()
            .append_pair("spot_id", &spot_id.to_string());
        let forecast = self.client.get(api_url).send()?.json()?;
        Ok(forecast)
    }
}

impl Default for ForecastAPI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    #[ignore]
    fn forecast_works() {
        let forecast = ForecastAPI::new().get(4203);
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
        let expected_forecast = Forecast {
            timestamp: 1645678800,
            local_timestamp: 1645660800,
            faded_rating: 1, // or custom star rating enum
            solid_rating: 1,
            swell: Swell {
                abs_min_breaking_height: 2.23,
                abs_max_breaking_height: 3.48,
                min_breaking_height: 2,
                max_breaking_height: 3,
                unit: "ft".to_string(),
                components: SwellComponents {
                    combined: Some(SwellComponent {
                        height: 4.0,
                        period: 10,
                        direction: 271.53,
                        compass_direction: "E".to_string(),
                    }),
                    primary: Some(SwellComponent {
                        height: 4.0,
                        period: 10,
                        direction: 271.53,
                        compass_direction: "E".to_string(),
                    }),
                    secondary: None,
                    tertiary: None,
                },
            },
            wind: Wind {
                speed: 7,
                direction: 335.0,
                compass_direction: "SSE".to_string(),
                chill: 21,
                gusts: 12,
                unit: "mph".to_string(),
            },
            condition: Condition {
                pressure: 1023,
                temperature: 20,
                //weather: 10,
                unit_pressure: "mb".to_string(),
                unit_temperature: "c".to_string(),
            },
            charts: Charts {
                swell: "https://charts-s3.msw.ms/archive/wave/750/21-1645671600-24.gif".to_string(),
                period: "https://charts-s3.msw.ms/archive/wave/750/21-1645671600-25.gif"
                    .to_string(),
                wind: "https://charts-s3.msw.ms/archive/gfs/750/21-1645671600-4.gif".to_string(),
                pressure: "https://charts-s3.msw.ms/archive/gfs/750/21-1645671600-3.gif"
                    .to_string(),
                sst: "https://charts-s3.msw.ms/archive/sst/750/21-1645671600-10.gif".to_string(),
            },
        };
        assert_eq!(
            serde_json::from_str::<'_, Forecast>(msw_json).unwrap(),
            expected_forecast
        );
    }
}
