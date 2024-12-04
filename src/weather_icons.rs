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
    // MainlyClear,
    PartlyCloudy,
    // Overcast,
    // Fog,
    // RimeFog,
    // DrizzleLight,
    // DrizzleModerate,
    // DrizzleDense,
    // FreezingDrizzleLight,
    // FreezingDrizzleDense,
    // RainSlight,
    // RainModerate,
    // RainHeavy,
    // FreezingRainLight,
    // FreezingRainHeavy,
    // SnowFallSlight,
    // SnowFallModerate,
    // SnowFallHeavy,
    // SnowGrains,
    // RainShowersSlight,
    // RainShowersModerate,
    // RainShowersViolent,
    // SnowShowersSlight,
    // SnowShowersHeavy,
    // ThunderstormSlight,
    // ThunderstormHeavy,
    // ThunderstormWithSlightHail,
    // ThunderstormWithHeavyHail,
}

impl WeatherIcon {
    pub fn get_icon(&self) -> &'static [u8] {
        match self {
            WeatherIcon::ClearSky => include_bytes!("../images/weather_icons/clear.bmp"),
            // WeatherIcon::MainlyClear => include_bytes!("../images/weather_icons/1.png"),
            WeatherIcon::PartlyCloudy => {
                include_bytes!("../images/weather_icons/partlycloudy.bmp")
            } // WeatherIcon::Overcast => include_bytes!("../images/weather_icons/3.png"),
              // WeatherIcon::Fog => include_bytes!("../images/weather_icons/45.png"),
              // WeatherIcon::RimeFog => include_bytes!("../images/weather_icons/48.png"),
              // WeatherIcon::DrizzleLight => include_bytes!("../images/weather_icons/51.png"),
              // WeatherIcon::DrizzleModerate => include_bytes!("../images/weather_icons/53.png"),
              // WeatherIcon::DrizzleDense => include_bytes!("../images/weather_icons/55.png"),
              // WeatherIcon::FreezingDrizzleLight => include_bytes!("../images/weather_icons/56.png"),
              // WeatherIcon::FreezingDrizzleDense => include_bytes!("../images/weather_icons/57.png"),
              // WeatherIcon::RainSlight => include_bytes!("../images/weather_icons/61.png"),
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
