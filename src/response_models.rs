use heapless::{String, Vec};
use serde::Deserialize;

// https://api.open-meteo.com/v1/forecast?latitude=35.7512&longitude=-86.93&current=temperature_2m,relative_humidity_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_probability_max&temperature_unit=fahrenheit&timezone=America/Chicago
//Weather code meanings
//     WMO Weather interpretation codes (WW)
// Code 	Description
// 0 	Clear sky
// 1, 2, 3 	Mainly clear, partly cloudy, and overcast
// 45, 48 	Fog and depositing rime fog
// 51, 53, 55 	Drizzle: Light, moderate, and dense intensity
// 56, 57 	Freezing Drizzle: Light and dense intensity
// 61, 63, 65 	Rain: Slight, moderate and heavy intensity
// 66, 67 	Freezing Rain: Light and heavy intensity
// 71, 73, 75 	Snow fall: Slight, moderate, and heavy intensity
// 77 	Snow grains
// 80, 81, 82 	Rain showers: Slight, moderate, and violent
// 85, 86 	Snow showers slight and heavy
// 95 * 	Thunderstorm: Slight or moderate
// 96, 99 * 	Thunderstorm with slight and heavy hail
// loop {
//TODO maybe the buffer needs to be bigger for the response?

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherResponse<'a> {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "generationtime_ms")]
    pub generationtime_ms: f64,
    #[serde(rename = "utc_offset_seconds")]
    pub utc_offset_seconds: i64,
    pub timezone: &'a str,
    #[serde(rename = "timezone_abbreviation")]
    pub timezone_abbreviation: &'a str,
    pub elevation: f64,
    #[serde(rename = "current_units")]
    pub current_units: CurrentUnits<'a>,
    pub current: Current<'a>,
    #[serde(rename = "daily_units")]
    pub daily_units: DailyUnits<'a>,
    pub daily: Daily,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUnits<'a> {
    pub time: &'a str,
    pub interval: &'a str,
    #[serde(rename = "temperature_2m")]
    pub temperature_2m: &'a str,
    #[serde(rename = "relative_humidity_2m")]
    pub relative_humidity_2m: &'a str,
    #[serde(rename = "weather_code")]
    pub weather_code: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Current<'a> {
    pub time: &'a str,
    pub interval: i64,
    #[serde(rename = "temperature_2m")]
    pub temperature_2m: f64,
    #[serde(rename = "relative_humidity_2m")]
    pub relative_humidity_2m: i64,
    ///See top for weather code meanings
    #[serde(rename = "weather_code")]
    pub weather_code: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUnits<'a> {
    pub time: &'a str,
    ///See top for weather code meanings
    #[serde(rename = "weather_code")]
    pub weather_code: &'a str,
    #[serde(rename = "temperature_2m_max")]
    pub temperature_2m_max: &'a str,
    #[serde(rename = "temperature_2m_min")]
    pub temperature_2m_min: &'a str,
    pub sunrise: &'a str,
    pub sunset: &'a str,
    #[serde(rename = "precipitation_probability_max")]
    pub precipitation_probability_max: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
//Hack
//I know the vecs will always be 7(for my use case) since i get 7 week forecast
//I know the Strings length will always be 10 or 16 because it's dates
//Reason for the second was I was having lifetime issues with 'a &str in heapless::vec
pub struct Daily {
    // "2024-11-29",
    pub time: Vec<String<10>, 7>,
    ///See top for weather code meanings
    #[serde(rename = "weather_code")]
    pub weather_code: Vec<i64, 7>,
    #[serde(rename = "temperature_2m_max")]
    pub temperature_2m_max: Vec<f64, 7>,
    #[serde(rename = "temperature_2m_min")]
    pub temperature_2m_min: Vec<f64, 7>,
    // 2024-11-29T06:37
    pub sunrise: Vec<String<16>, 7>,
    // 2024-11-29T06:37
    pub sunset: Vec<String<16>, 7>,
    #[serde(rename = "precipitation_probability_max")]
    pub precipitation_probability_max: Vec<i64, 7>,
}
