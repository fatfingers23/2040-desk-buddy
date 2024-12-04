use defmt::*;
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
use libm::floor;
use tinybmp::Bmp;

use crate::io::easy_format_str;
use crate::response_models::{Current, CurrentUnits};
use crate::weather_icons;

/// Forecast display

pub fn draw_current_outside_weather(
    starting_point: Point,
    current: Current,
    units: CurrentUnits,
    display: &mut impl DrawTarget<Color = Color>,
) {
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
        starting_point.x + 20,
        starting_point.y + 30,
    );

    let mut formatting_buffer = [0u8; 520];
    let current_temp = easy_format_str(
        format_args!("{}{}", current.temperature_2m, units.temperature_2m),
        &mut formatting_buffer,
    );

    draw_text(
        display,
        &current_temp.unwrap(),
        starting_point.x + 30,
        starting_point.y + 5,
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
        starting_point.x + 30,
        starting_point.y + 20,
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
    display: &mut impl DrawTarget<Color = Color>,
) {
    //TODO need to lower it all about 10 pixels
    //TODO need to see about measure icons placement from bottom not top
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

    //TODO find the day of the week. I think i'll need the RTC set for that
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
    let month_day = easy_format_str(format_args!("{}/{}", month, day), &mut formatting_buffer);

    draw_text(
        display,
        month_day.unwrap(),
        starting_point.x + 16,
        starting_point.y + 6,
    );

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
