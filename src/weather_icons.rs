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
    // ChanceFlurries,
    ChanceRain,
    ChanceSleet,
    // ChanceSnow,
    // ChanceStorms,
    Clear,
    Cloudy,
    Flurries,
    Fog,
    // Hazy,
    MostlyCloudy,
    // MostlySunny,
    //Saving space and removing night time for now
    // NtChanceFlurries,
    // NtChanceRain,
    // NtChanceSleet,
    // NtChanceSnow,
    // NtChanceStorms,
    NtClear,
    // NtCloudy,
    // NtFlurries,
    // NtFog,
    // NtHazy,
    // NtMostlyCloudy,
    // NtMostlySunny,
    // NtPartlyCloudy,
    // NtPartlySunny,
    // NtRain,
    // NtSleet,
    // NtSnow,
    // NtSunny,
    // NtTStorms,
    // NtUnknown,
    PartlyCloudy,
    // PartlySunny,
    Rain,
    Sleet,
    Snow,
    // Sunny,
    TStorms,
    Unknown,
}

impl WeatherIcon {
    pub fn get_icon(&self) -> &'static [u8] {
        match self {
            // WeatherIcon::ChanceFlurries => {
            //     include_bytes!("../images/weather_icons/chanceflurries.bmp")
            // }
            WeatherIcon::ChanceRain => {
                include_bytes!("../images/weather_icons/chancerain.bmp")
            }
            WeatherIcon::ChanceSleet => {
                include_bytes!("../images/weather_icons/chancesleet.bmp")
            }
            // WeatherIcon::ChanceSnow => {
            //     include_bytes!("../images/weather_icons/chancesnow.bmp")
            // }
            // WeatherIcon::ChanceStorms => {
            //     include_bytes!("../images/weather_icons/chancetstorms.bmp")
            // }
            WeatherIcon::Clear => {
                include_bytes!("../images/weather_icons/clear.bmp")
            }
            WeatherIcon::Cloudy => {
                include_bytes!("../images/weather_icons/cloudy.bmp")
            }
            WeatherIcon::Flurries => {
                include_bytes!("../images/weather_icons/flurries.bmp")
            }
            WeatherIcon::Fog => {
                include_bytes!("../images/weather_icons/fog.bmp")
            }
            // WeatherIcon::Hazy => {
            //     include_bytes!("../images/weather_icons/hazy.bmp")
            // }
            WeatherIcon::MostlyCloudy => {
                include_bytes!("../images/weather_icons/mostlycloudy.bmp")
            }
            // WeatherIcon::MostlySunny => {
            //     include_bytes!("../images/weather_icons/mostlysunny.bmp")
            // }
            // WeatherIcon::NtChanceFlurries => {
            //     include_bytes!("../images/weather_icons/nt_chanceflurries.bmp")
            // }
            // WeatherIcon::NtChanceRain => {
            //     include_bytes!("../images/weather_icons/nt_chancerain.bmp")
            // }
            // WeatherIcon::NtChanceSleet => {
            //     include_bytes!("../images/weather_icons/nt_chancesleet.bmp")
            // }
            // WeatherIcon::NtChanceSnow => {
            //     include_bytes!("../images/weather_icons/nt_chancesnow.bmp")
            // }
            // WeatherIcon::NtChanceStorms => {
            //     include_bytes!("../images/weather_icons/nt_chancetstorms.bmp")
            // }
            WeatherIcon::NtClear => {
                include_bytes!("../images/weather_icons/nt_clear.bmp")
            }
            // WeatherIcon::NtCloudy => {
            //     include_bytes!("../images/weather_icons/nt_cloudy.bmp")
            // }
            // WeatherIcon::NtFlurries => {
            //     include_bytes!("../images/weather_icons/nt_flurries.bmp")
            // }
            // WeatherIcon::NtFog => {
            //     include_bytes!("../images/weather_icons/nt_fog.bmp")
            // }
            // WeatherIcon::NtHazy => {
            //     include_bytes!("../images/weather_icons/nt_hazy.bmp")
            // }
            // WeatherIcon::NtMostlyCloudy => {
            //     include_bytes!("../images/weather_icons/nt_mostlycloudy.bmp")
            // }
            // WeatherIcon::NtMostlySunny => {
            //     include_bytes!("../images/weather_icons/nt_mostlysunny.bmp")
            // }
            // WeatherIcon::NtPartlyCloudy => {
            //     include_bytes!("../images/weather_icons/nt_partlycloudy.bmp")
            // }
            // WeatherIcon::NtPartlySunny => {
            //     include_bytes!("../images/weather_icons/nt_partlysunny.bmp")
            // }
            // WeatherIcon::NtRain => {
            //     include_bytes!("../images/weather_icons/nt_rain.bmp")
            // }
            // WeatherIcon::NtSleet => {
            //     include_bytes!("../images/weather_icons/nt_sleet.bmp")
            // }
            // WeatherIcon::NtSnow => {
            //     include_bytes!("../images/weather_icons/nt_snow.bmp")
            // }
            // WeatherIcon::NtSunny => {
            //     include_bytes!("../images/weather_icons/nt_sunny.bmp")
            // }
            // WeatherIcon::NtTStorms => {
            //     include_bytes!("../images/weather_icons/nt_tstorms.bmp")
            // }
            // WeatherIcon::NtUnknown => {
            //     include_bytes!("../images/weather_icons/nt_unknown.bmp")
            // }
            WeatherIcon::PartlyCloudy => {
                include_bytes!("../images/weather_icons/partlycloudy.bmp")
            }
            // WeatherIcon::PartlySunny => {
            //     include_bytes!("../images/weather_icons/partlysunny.bmp")
            // }
            WeatherIcon::Rain => {
                include_bytes!("../images/weather_icons/rain.bmp")
            }
            WeatherIcon::Sleet => {
                include_bytes!("../images/weather_icons/sleet.bmp")
            }
            WeatherIcon::Snow => {
                include_bytes!("../images/weather_icons/snow.bmp")
            }
            // WeatherIcon::Sunny => {
            //     include_bytes!("../images/weather_icons/sunny.bmp")
            // }
            WeatherIcon::TStorms => {
                include_bytes!("../images/weather_icons/tstorms.bmp")
            }
            WeatherIcon::Unknown => {
                include_bytes!("../images/weather_icons/unknown.bmp")
            }
        }
    }
}

pub fn get_weather_icon(code: u8) -> WeatherIcon {
    //TODO pretty sure these need adjusting
    match code {
        0 => WeatherIcon::Clear,
        1 => WeatherIcon::PartlyCloudy,
        2 => WeatherIcon::MostlyCloudy,
        3 => WeatherIcon::Cloudy,
        45 | 48 => WeatherIcon::Fog,
        51 | 53 | 55 => WeatherIcon::ChanceRain,
        56 | 57 => WeatherIcon::ChanceSleet,
        61 | 63 | 65 => WeatherIcon::Rain,
        66 | 67 => WeatherIcon::Sleet,
        71 => WeatherIcon::Flurries,
        73 | 75 | 77 => WeatherIcon::Snow,
        80 | 81 | 82 => WeatherIcon::Rain,
        85 | 86 => WeatherIcon::Snow,
        95 | 96 | 99 => WeatherIcon::TStorms,
        _ => WeatherIcon::Unknown,
    }
}

pub fn get_night_weather_icon(code: u8) -> WeatherIcon {
    //TODO Need to implement night time icons
    match code {
        0 => WeatherIcon::NtClear,
        // 1 => WeatherIcon::PartlyCloudy,
        // 2 => WeatherIcon::MostlyCloudy,
        // 3 => WeatherIcon::Cloudy,
        // 45 | 48 => WeatherIcon::Fog,
        // 51 | 53 | 55 => WeatherIcon::ChanceRain,
        // 56 | 57 => WeatherIcon::ChanceSleet,
        // 61 | 63 | 65 => WeatherIcon::Rain,
        // 66 | 67 => WeatherIcon::Sleet,
        // 71 => WeatherIcon::Flurries,
        // 73 | 75 | 77 => WeatherIcon::Snow,
        // 80 | 81 | 82 => WeatherIcon::Rain,
        // 85 | 86 => WeatherIcon::Snow,
        // 95 | 96 | 99 => WeatherIcon::TStorms,
        _ => WeatherIcon::NtClear, // _ => WeatherIcon::Unknown,
    }
}
