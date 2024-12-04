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

pub enum WeatherIcon {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    Drizzle,
    FreezingDrizzle,
    Rain,
    Sleet,
    ChanceSnow,
    SnowFallModerate,
    SnowFallHeavy,
    SnowGrains,
    RainShowersSlight,
    RainShowersModerate,
    RainShowersViolent,
    SnowShowersSlight,
    SnowShowersHeavy,
    ThunderstormSlight,
    ThunderstormHeavy,
    ThunderstormWithSlightHail,
    ThunderstormWithHeavyHail,
}

impl WeatherIcon {
    pub fn get_icon(&self) -> &'static [u8] {
        match self {
            WeatherIcon::ClearSky => include_bytes!("../images/weather_icons/clear.bmp"),
            WeatherIcon::MainlyClear => include_bytes!("../images/weather_icons/mostlysunny.bmp"),
            WeatherIcon::PartlyCloudy => {
                include_bytes!("../images/weather_icons/partlycloudy.bmp")
            }
            WeatherIcon::Overcast => include_bytes!("../images/weather_icons/cloudy.bmp"),
            WeatherIcon::Fog => include_bytes!("../images/weather_icons/fog.bmp"),
            WeatherIcon::Drizzle => include_bytes!("../images/weather_icons/chancerain.bmp"),
            WeatherIcon::FreezingDrizzle => {
                include_bytes!("../images/weather_icons/chancesleet.bmp")
            }
            WeatherIcon::Rain => include_bytes!("../images/weather_icons/rain.bmp"),
            WeatherIcon::Sleet => include_bytes!("../images/weather_icons/sleet.bmp"),
            // WeatherIcon::RainModerate => include_bytes!("../images/weather_icons/63.png"),
            // WeatherIcon::RainHeavy => include_bytes!("../images/weather_icons/65.png"),
            // WeatherIcon::FreezingRainLight => include_bytes!("../images/weather_icons/66.png"),
            // WeatherIcon::FreezingRainHeavy => include_bytes!("../images/weather_icons/67.png"),
            // WeatherIcon::SnowFallSlight => include_bytes!("../images/weather_icons/71.png"),
            // WeatherIcon::SnowFallModerate => include_bytes!("../images/weather_icons/73.png"),
            // WeatherIcon::SnowFallHeavy => include_bytes!("../images/weather_icons/75.png"),
            // WeatherIcon::SnowGrains => include_bytes!("../images/weather_icons/77.png"),
            // WeatherIcon::RainShowersSlight => include_bytes!("../images/weather_icons/80.png"),
            // WeatherIcon::RainShowersModerate => include_bytes!("../images/weather_icons/81.png"),
            // WeatherIcon::RainShowersViolent => include_bytes!("../images/weather_icons/82.png"),
            // WeatherIcon::SnowShowersSlight => include_bytes!("../images/weather_icons/85.png"),
            // WeatherIcon::SnowShowersHeavy => include_bytes!("../images/weather_icons/86.png"),
            // WeatherIcon::ThunderstormSlight => include_bytes!("../images/weather_icons/95.png"),
            // WeatherIcon::ThunderstormHeavy => include_bytes!("../images/weather_icons/96.png"),
            // WeatherIcon::ThunderstormWithSlightHail => include_bytes!("../images/weather_icons/99.png"),
            // WeatherIcon::ThunderstormWithHeavyHail => include_bytes!("../images/weather_icons/99.png"),
        }
    }
}

pub fn get_weather_icon(code: u8) -> WeatherIcon {
    match code {
        0 => WeatherIcon::ClearSky,
        1 | 2 | 3 => WeatherIcon::PartlyCloudy,
        _ => WeatherIcon::PartlyCloudy,
    }
}
