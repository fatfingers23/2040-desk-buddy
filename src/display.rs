use crate::env::env_value;
use crate::io::easy_format_str;
use crate::web_requests::{Current, CurrentUnits};
use crate::{weather_icons, InsideSensorData};
use defmt::*;
use embassy_rp::rtc::DateTime;
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use epd_waveshare::color::Color;
use heapless::{String, Vec};
use libm::{floor, roundf};
use tinybmp::Bmp;

pub fn draw_scd_data(
    starting_point: Point,
    sensor_data: InsideSensorData,
    display: &mut impl DrawTarget<Color = Color>,
) {
    let mut formatting_buffer = [0u8; 520];
    let fahrenheit = env_value("UNIT") == "fahrenheit";
    let temp = if fahrenheit {
        easy_format_str(
            format_args!("{}°F", roundf(sensor_data.temperature * 1.8 + 32.0)),
            &mut formatting_buffer,
        )
    } else {
        easy_format_str(
            format_args!("{}°C", roundf(sensor_data.temperature)),
            &mut formatting_buffer,
        )
    };

    let mut formatting_buffer = [0u8; 520];
    let humidity = easy_format_str(
        format_args!("{}%", roundf(sensor_data.humidity)),
        &mut formatting_buffer,
    );
    let mut formatting_buffer = [0u8; 520];
    let co2 = easy_format_str(
        format_args!("{}ppm", sensor_data.co2),
        &mut formatting_buffer,
    );
    // (5,50)
    draw_bmp(
        display,
        include_bytes!("../images/house_fill.bmp"),
        starting_point.x,
        starting_point.y,
    );
    // draw_text(display, temp.unwrap(), 38, 50);
    draw_text(
        display,
        temp.unwrap(),
        starting_point.x + 33,
        starting_point.y,
    );

    draw_text(
        display,
        humidity.unwrap(),
        starting_point.x + 33,
        starting_point.y + 15,
    );
    draw_text(
        display,
        co2.unwrap(),
        starting_point.x + 33,
        starting_point.y + 30,
    );
}

///Draw time
pub fn draw_time(date_time: DateTime, display: &mut impl DrawTarget<Color = Color>) {
    let mut am = true;
    let twelve_hour = if date_time.hour >= 12 {
        am = false;
        if date_time.hour == 12 {
            12
        } else {
            date_time.hour - 12
        }
    } else if date_time.hour == 0 {
        12
    } else {
        date_time.hour
    };

    let am_pm = if am { "AM" } else { "PM" };

    let mut formatting_buffer = [0u8; 520];
    let formatted_time = easy_format_str(
        format_args!(
            "{:02}:{:02} {} {}/{}/{}",
            twelve_hour, date_time.minute, am_pm, date_time.month, date_time.day, date_time.year
        ),
        &mut formatting_buffer,
    );

    draw_text(display, formatted_time.unwrap(), 5, 10);
}

/// Draw the current outside weather
pub fn draw_current_outside_weather(
    starting_point: Point,
    current: Current,
    units: CurrentUnits,
    display: &mut impl DrawTarget<Color = Color>,
) {
    //Place holders to help with design
    // let current_box_style = PrimitiveStyleBuilder::new()
    //     .stroke_color(Color::Black)
    //     .stroke_width(1)
    //     .fill_color(Color::White)
    //     .build();

    // let _ = Rectangle::new(starting_point, Size::new(100, 100))
    //     .into_styled(current_box_style)
    //     .draw(display);
    // let current_weather_starting_point = Point::new(300, 45);

    draw_bmp(
        display,
        weather_icons::get_weather_icon(current.weather_code).get_icon(),
        starting_point.x,
        starting_point.y,
    );

    let mut formatting_buffer = [0u8; 520];
    let current_temp = easy_format_str(
        format_args!("{}{}", current.temperature_2m, units.temperature_2m),
        &mut formatting_buffer,
    );

    draw_text(
        display,
        &current_temp.unwrap(),
        starting_point.x + 58,
        starting_point.y + 20,
    );

    let mut formatting_buffer = [0u8; 520];
    let current_humidity = easy_format_str(
        format_args!(
            "{}{}",
            current.relative_humidity_2m, units.relative_humidity_2m
        ),
        &mut formatting_buffer,
    );

    draw_text(
        display,
        &current_humidity.unwrap(),
        starting_point.x + 58,
        starting_point.y + 35,
    );
}

pub fn draw_weather_forecast_box(
    starting_point: Point,
    forecast_box_width: u32,
    daily_date: &str,
    units: &str,
    daily_max_temp: f64,
    daily_min_temp: f64,
    daily_weather_code: u8,
    sun_rise: String<16>,
    sun_set: String<16>,
    possible_current_datetime: Option<DateTime>,
    current_index: u8,
    display: &mut impl DrawTarget<Color = Color>,
) {
    //TODO need to see about measure icons placement from bottom not top
    //This is about how some weather icons are taller than others
    let daily_max_rounded = floor(daily_max_temp);
    let daily_min_rounded = floor(daily_min_temp);

    info!(
        "Date: {:?}, Max Temp: {:?}, Min Temp: {:?}, Weather Code: {:?}",
        daily_date, daily_max_temp, daily_min_temp, daily_weather_code
    );

    //forecast box style
    let forecast_box_style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::Black)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();

    //Top of rectangle showing date
    let _ = Rectangle::new(starting_point, Size::new(forecast_box_width, 25))
        .into_styled(forecast_box_style)
        .draw(display);

    //Outline of the daily forecast box
    let _ = Rectangle::new(
        Point::new(starting_point.x, starting_point.y + 25),
        Size::new(forecast_box_width, 125),
    )
    .into_styled(forecast_box_style)
    .draw(display);

    // Writing the forecast content
    let split: Vec<&str, 3> = daily_date.split("-").collect();
    let _year = split[0];
    let month = split[1];
    let day = split[2];

    //HACK need to move to a function
    let sun_rise_split: Vec<&str, 2> = sun_rise.split("T").collect();
    let sun_rise_time = sun_rise_split[1];
    let sun_set_split: Vec<&str, 2> = sun_set.split("T").collect();
    let sun_set_time = sun_set_split[1];

    //Month/day text
    let mut formatting_buffer = [0u8; 520];
    let month_day =
        easy_format_str(format_args!("{}/{}", month, day), &mut formatting_buffer).unwrap();

    if let Some(current_datetime) = possible_current_datetime {
        //TODO do the acutal logic to get the day of the week

        let mut day_of_week_index = current_datetime.day_of_week as u8 + current_index;
        if day_of_week_index > 6 {
            day_of_week_index = day_of_week_index - 7;
        }

        let short_hand_day_of_week = match day_of_week_index {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            6 => "Sat",
            _ => &month_day,
        };

        draw_text(
            display,
            short_hand_day_of_week,
            starting_point.x + 16,
            starting_point.y + 6,
        );
    } else {
        //TODO Index loops over need to if it goes over 6 to reset to 0
        draw_text(
            display,
            month_day,
            starting_point.x + 16,
            starting_point.y + 6,
        );
    }

    //TODO add a list of birthdays to the env file
    if month_day == "12/08" || month_day == "12/24" || month_day == "04/16" || month_day == "06/10"
    {
        draw_bmp(
            display,
            include_bytes!("../images/birthday_cake.bmp"),
            starting_point.x + 58,
            starting_point.y + 6,
        );
    }

    //Draw weather icon
    //TODO check precipitation_probability_max to decide if to show rain. then can check
    //if over freezing to decide if snow?
    draw_bmp(
        display,
        weather_icons::get_weather_icon(daily_weather_code).get_icon(),
        starting_point.x + 10,
        starting_point.y + 45,
    );

    //Max and min temp
    let mut formatting_buffer = [0u8; 520];
    let max_min_text = easy_format_str(
        format_args!(
            "{}{}/{}{}",
            daily_max_rounded, units, daily_min_rounded, units
        ),
        &mut formatting_buffer,
    );
    draw_text(
        display,
        max_min_text.unwrap(),
        starting_point.x + 5,
        starting_point.y + 35,
    );

    //Sun set and rise section

    draw_bmp(
        display,
        include_bytes!("../images/weather_icons/small_sun.bmp"),
        starting_point.x + 1,
        starting_point.y + 100,
    );

    draw_text(
        display,
        &sun_rise_time,
        starting_point.x + 30,
        starting_point.y + 105,
    );

    draw_bmp(
        display,
        include_bytes!("../images/weather_icons/small_moon.bmp"),
        starting_point.x + 1,
        starting_point.y + 125,
    );

    draw_text(
        display,
        &sun_set_time,
        starting_point.x + 30,
        starting_point.y + 130,
    );
}

///drawing helpers

fn draw_bmp(display: &mut impl DrawTarget<Color = Color>, bmp_data: &[u8], x: i32, y: i32) {
    let bmp: Bmp<BinaryColor> = Bmp::from_slice(bmp_data).unwrap();
    let _ = Image::new(&bmp, Point::new(x, y)).draw(&mut display.color_converted());
}

fn draw_text(display: &mut impl DrawTarget<Color = Color>, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&profont::PROFONT_12_POINT)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
    debug!("Draw text: {:?}", text);
}

fn _draw_text_font(
    display: &mut impl DrawTarget<Color = Color>,
    text: &str,
    x: i32,
    y: i32,
    font: &MonoFont,
) {
    let style = MonoTextStyleBuilder::new()
        .font(&font)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
    debug!("Draw text: {:?}", text);
}
