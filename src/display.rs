use crate::env::env_value;
use crate::io::{easy_format_str, format_date, return_str_time};
use crate::weather_icons;
use crate::web_requests::{Current, CurrentUnits};
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
use heapless::String;
use libm::{floor, roundf};
use tinybmp::Bmp;

//Some display models

///Just a copy of SensorData to have debug, clone and format
#[derive(Debug, Clone, Format)]
pub struct InsideSensorData {
    pub co2: u16,
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Debug, Clone, Format)]
pub struct BlueSkyNotificationData {
    pub unread_notifications: i32,
    pub last_notification: String<256>,
}

//The draw functions

pub fn draw_blue_sky_notification(
    starting_point: Point,
    notification: BlueSkyNotificationData,
    display: &mut impl DrawTarget<Color = Color>,
) {
    draw_bmp(
        display,
        include_bytes!("../images/bluesky_logo.bmp"),
        starting_point.x + 10,
        starting_point.y,
    );

    let mut formatting_buffer = [0u8; 10];
    let unread_notifications = easy_format_str(
        format_args!("{}", notification.unread_notifications),
        &mut formatting_buffer,
    );

    //Unread count
    draw_text(
        display,
        unread_notifications.unwrap(),
        starting_point.x + 40,
        starting_point.y + 0,
    );

    //Last notification
    draw_text(
        display,
        &notification.last_notification.as_str(),
        starting_point.x + 40,
        starting_point.y + 15,
    );
}

///Draws the inside sensor data from the scd40 sensor
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

    draw_bmp(
        display,
        include_bytes!("../images/house_fill.bmp"),
        starting_point.x,
        starting_point.y,
    );

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
    //Need to white out the time before drawing the new time. Differences in date size can leave one digit hanging
    let rectangle_style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::White)
        .stroke_width(1)
        .fill_color(Color::White)
        .build();

    let _ = Rectangle::new(Point::new(0, 0), Size::new(155, 25))
        .into_styled(rectangle_style)
        .draw(display);

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
    daytime: bool,
    display: &mut impl DrawTarget<Color = Color>,
) {
    let current_image = match daytime {
        true => weather_icons::get_weather_icon(current.weather_code).get_icon(),
        //TODO add more of night icons cause I think its just clear atm
        false => weather_icons::get_night_weather_icon(current.weather_code).get_icon(),
    };

    draw_bmp(
        display,
        &current_image,
        starting_point.x,
        starting_point.y - 15,
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
        starting_point.y,
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
        starting_point.y + 15,
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
    //Updated note 12-06 I think it's fine?
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
    let formatted_date = format_date(daily_date);

    let sun_rise_time = return_str_time(sun_rise.as_str());
    let sun_set_time = return_str_time(sun_set.as_str());

    //Month/day text
    let mut formatting_buffer = [0u8; 520];
    let month_day = easy_format_str(
        format_args!("{}/{}", formatted_date.month, formatted_date.day),
        &mut formatting_buffer,
    )
    .unwrap();

    if let Some(current_datetime) = possible_current_datetime {
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
            include_bytes!("../images/birthday_cake_24.bmp"),
            starting_point.x + 54,
            starting_point.y + 1,
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
