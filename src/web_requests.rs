use core::str::from_utf8;
use defmt::Format;
use defmt::*;
use embassy_net::{dns::DnsSocket, tcp::client::TcpClient};
use heapless::{String, Vec};
use reqwless::{
    client::{HttpClient, HttpConnection},
    request::{self, Method, Request, RequestBody},
};
use serde::{Deserialize, Serialize};

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

/// This is the response from the weather api
/// You will notice I am using heapless::String instead of &str. I was having issues with sharing the struct between tasks
/// because of str and decided to just go simple to keep moving
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForecastResponse {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "generationtime_ms")]
    pub generationtime_ms: f64,
    #[serde(rename = "utc_offset_seconds")]
    pub utc_offset_seconds: i64,
    pub timezone: String<32>,
    #[serde(rename = "timezone_abbreviation")]
    pub timezone_abbreviation: String<8>,
    pub elevation: f64,
    #[serde(rename = "current_units")]
    pub current_units: CurrentUnits,
    pub current: Current,
    #[serde(rename = "daily_units")]
    pub daily_units: DailyUnits,
    pub daily: Daily,
}

///This is the units used for each of the current measurements
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUnits {
    pub time: String<7>,
    pub interval: String<7>,
    #[serde(rename = "temperature_2m")]
    pub temperature_2m: String<3>,
    #[serde(rename = "relative_humidity_2m")]
    pub relative_humidity_2m: String<2>,
    //I think this will always be wmo code. Going to assume it is
    // #[serde(rename = "weather_code")]
    // pub weather_code: &'a str,
}
///This is the actual current weather measurements
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Current {
    pub time: String<16>,
    pub interval: i64,
    #[serde(rename = "temperature_2m")]
    pub temperature_2m: f64,
    #[serde(rename = "relative_humidity_2m")]
    pub relative_humidity_2m: i64,
    ///See top for weather code meanings
    #[serde(rename = "weather_code")]
    pub weather_code: u8,
}

///This is the units used for each of the daily measurements
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUnits {
    pub time: String<7>,
    //I think this will always be wmo code. Going to assume it is
    // #[serde(rename = "weather_code")]
    // pub weather_code: &'a str,
    #[serde(rename = "temperature_2m_max")]
    pub temperature_2m_max: String<3>,
    #[serde(rename = "temperature_2m_min")]
    pub temperature_2m_min: String<3>,
    //Just going to comment these out cause it's all just going to use the same time format
    // pub sunrise: &'a str,
    // pub sunset: &'a str,
    #[serde(rename = "precipitation_probability_max")]
    pub precipitation_probability_max: String<1>,
}

///This is the actual daily weather measurements
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
    pub weather_code: Vec<u8, 7>,
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

///time response
#[derive(Deserialize)]
pub struct TimeApiResponse<'a> {
    pub datetime: &'a str,
    pub day_of_week: u8,
}

///Blyesky CreateSession Request
#[derive(Serialize)]
pub struct CreateSessionRequest<'a> {
    pub identifier: &'a str,
    pub password: &'a str,
}

impl RequestBody for CreateSessionRequest<'_> {
    async fn write<W: embedded_io_async::Write>(&self, writer: &mut W) -> Result<(), W::Error> {
        //TODO i think this can be done better
        // let request_body: heapless::String<256> = serde_json_core::to_string(&self).unwrap();
        // writer.write_all(request_body.as_bytes()).await?;
        let mut buffer = [0u8; 256];
        let bytes = serde_json_core::to_slice(&self, &mut buffer).unwrap();
        let only_used_bytes = &buffer[..bytes];
        info!("Request Body: {}", from_utf8(&only_used_bytes).unwrap());
        writer.write_all(&only_used_bytes).await?;

        Ok(())
    }
}

///BlueSky CreateSession Response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionResponse<'a> {
    #[serde(rename = "accessJwt")]
    pub access_jwt: &'a str,
    // pub refresh_jwt: &'a str,
    // pub handle: &'a str,
    // pub did: &'a str,
    // //Not sur what this is
    // // pub did_doc: Option<serde_json::Value>,
    // pub email: &'a str,
    // pub email_confirmed: bool,
    // pub email_auth_factor: bool,
    // pub active: bool,
    // pub status: &'a str,
}

#[derive(Debug, Format)]
pub enum WebCallError {
    HttpError(u16),
    WebRequestError,
    FailedToReadResponse,
    DeserializationError,
}

pub async fn send_request<'a, T, ResponseType>(
    http_client: &mut HttpClient<'a, TcpClient<'a, 4>, DnsSocket<'a>>,
    base_url: &str,
    request: Request<'a, T>,
    rx_buffer: &'a mut [u8],
) -> Result<ResponseType, WebCallError>
where
    T: RequestBody,
    ResponseType: serde::Deserialize<'a>,
{
    let mut conn = match http_client.resource(base_url).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to make HTTP request: {:?}", e);
            return Err(WebCallError::WebRequestError);
        }
    };

    let response = match conn.send(request, rx_buffer).await {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to make HTTP request: {:?}", e);
            return Err(WebCallError::WebRequestError);
        }
    };

    // info!("Response: {:?}", response.);

    if !response.status.is_successful() {
        let status_code = response.status.0.clone();
        error!("HTTP request failed with status: {:?}", response.status);
        // error!("Failed response: {}", response);
        match from_utf8(response.body().read_to_end().await.unwrap()) {
            Ok(body) => {
                error!("Response body: {}", body);
            }
            Err(_e) => {
                error!("Failed to read response body");
            }
        }

        return Err(WebCallError::HttpError(status_code));
    }

    // let body_str = from_utf8(response.body().read_to_end().await.unwrap()).unwrap();
    // info!("Response body: {}", body_str);
    // if body_str.is_empty() {
    //     return Err(WebCallError::FailedToReadResponse);
    // }

    let body = match from_utf8(response.body().read_to_end().await.unwrap()) {
        Ok(b) => b,
        Err(_e) => {
            error!("Failed to read response body");
            return Err(WebCallError::FailedToReadResponse);
        }
    };

    // info!("Response body: {}", body);
    // Err(WebCallError::DeserializationError)
    match serde_json_core::de::from_slice::<ResponseType>(body.as_bytes()) {
        Ok((output, _used)) => Ok(output),
        Err(e) => {
            info!("Here");
            error!("Response body: {}", body);
            print_serde_json_error(e);
            return Err(WebCallError::DeserializationError);
        }
    }
}

/// Wrapper to help you make a GET request that responses with JSON
pub async fn get_web_request<'a, ResponseType>(
    http_client: &mut HttpClient<'a, TcpClient<'a, 4>, DnsSocket<'a>>,
    url: &str,
    rx_buffer: &'a mut [u8],
) -> Result<ResponseType, WebCallError>
where
    ResponseType: serde::Deserialize<'a>,
{
    let mut request = match http_client.request(Method::GET, &url).await {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to make HTTP request: {:?}", e);
            return Err(WebCallError::WebRequestError);
        }
    };

    let response = match request.send(rx_buffer).await {
        Ok(resp) => resp,
        Err(_e) => {
            error!("Failed to send HTTP request");
            return Err(WebCallError::WebRequestError);
        }
    };

    if !response.status.is_successful() {
        error!("HTTP request failed with status: {:?}", response.status);
        return Err(WebCallError::HttpError(response.status.0));
    }

    let body = match from_utf8(response.body().read_to_end().await.unwrap()) {
        Ok(b) => b,
        Err(_e) => {
            error!("Failed to read response body");
            return Err(WebCallError::FailedToReadResponse);
        }
    };
    match serde_json_core::de::from_slice::<ResponseType>(body.as_bytes()) {
        Ok((output, _used)) => Ok(output),
        Err(e) => {
            print_serde_json_error(e);
            return Err(WebCallError::DeserializationError);
        }
    }
}

fn print_serde_json_error(error: serde_json_core::de::Error) {
    match error {
        serde_json_core::de::Error::AnyIsUnsupported => {
            error!("Deserialization error: AnyIsUnsupported")
        }
        serde_json_core::de::Error::BytesIsUnsupported => {
            error!("Deserialization error: BytesIsUnsupported")
        }
        serde_json_core::de::Error::EofWhileParsingList => {
            error!("Deserialization error: EofWhileParsingList")
        }
        serde_json_core::de::Error::EofWhileParsingObject => {
            error!("Deserialization error: EofWhileParsingObject")
        }
        serde_json_core::de::Error::EofWhileParsingString => {
            error!("Deserialization error: EofWhileParsingString")
        }
        serde_json_core::de::Error::EofWhileParsingNumber => {
            error!("Deserialization error: EofWhileParsingNumber")
        }
        serde_json_core::de::Error::EofWhileParsingValue => {
            error!("Deserialization error: EofWhileParsingValue")
        }
        serde_json_core::de::Error::ExpectedColon => {
            error!("Deserialization error: ExpectedColon")
        }
        serde_json_core::de::Error::ExpectedListCommaOrEnd => {
            error!("Deserialization error: ExpectedListCommaOrEnd")
        }
        serde_json_core::de::Error::ExpectedObjectCommaOrEnd => {
            error!("Deserialization error: ExpectedObjectCommaOrEnd")
        }
        serde_json_core::de::Error::ExpectedSomeIdent => {
            error!("Deserialization error: ExpectedSomeIdent")
        }
        serde_json_core::de::Error::ExpectedSomeValue => {
            error!("Deserialization error: ExpectedSomeValue")
        }
        serde_json_core::de::Error::InvalidNumber => {
            error!("Deserialization error: InvalidNumber")
        }
        serde_json_core::de::Error::InvalidType => {
            error!("Deserialization error: InvalidType")
        }
        serde_json_core::de::Error::InvalidUnicodeCodePoint => {
            error!("Deserialization error: InvalidUnicodeCodePoint")
        }
        serde_json_core::de::Error::InvalidEscapeSequence => {
            error!("Deserialization error: InvalidEscapeSequence")
        }
        serde_json_core::de::Error::EscapedStringIsTooLong => {
            error!("Deserialization error: EscapedStringIsTooLong")
        }
        serde_json_core::de::Error::KeyMustBeAString => {
            error!("Deserialization error: KeyMustBeAString")
        }
        serde_json_core::de::Error::TrailingCharacters => {
            error!("Deserialization error: TrailingCharacters")
        }
        serde_json_core::de::Error::TrailingComma => {
            error!("Deserialization error: TrailingComma")
        }
        serde_json_core::de::Error::CustomError => {
            error!("Deserialization error: CustomError")
        }
        _ => error!("Deserialization error: Unknown"),
    }
}
