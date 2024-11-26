#![no_std]
#![no_main]

use cyw43_driver::setup_cyw43;
use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    peripherals::{SPI0, SPI1},
    spi::{self, Spi},
};
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, Mutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{
    color::*,
    epd4in2_v2::{self, Display4in2, Epd4in2},
    graphics::{DisplayRotation, VarDisplay},
    prelude::*,
};
use serde::de;
use static_cell::StaticCell;
use tinybmp::Bmp;
use {defmt_rtt as _, panic_probe as _};

mod cyw43_driver;
mod env;
mod io;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    // let (_device, mut control) = setup_cyw43(
    //     p.PIO0, p.PIN_23, p.PIN_24, p.PIN_25, p.PIN_29, p.DMA_CH0, spawner,
    // )
    // .await;

    let mosi = p.PIN_11;
    let clk = p.PIN_10;
    let cs = Output::new(p.PIN_9, Level::High);

    let dc = Output::new(p.PIN_8, Level::High);

    let rst = Output::new(p.PIN_12, Level::High);
    let busy = Input::new(p.PIN_13, Pull::Up);

    let mut config = spi::Config::default();
    config.frequency = 4_000_000;

    let spi = Spi::new_blocking_txonly(p.SPI1, clk, mosi, config);

    let mut spi_dev = ExclusiveDevice::new(spi, cs, embassy_time::Delay);

    let mut epd4in2 = Epd4in2::new(&mut spi_dev, busy, dc, rst, &mut embassy_time::Delay, None)
        .expect("eink initalize error");

    epd4in2
        .set_refresh(&mut spi_dev, &mut Delay, RefreshLut::Quick)
        .unwrap();
    let (x, y, width, height) = (50, 50, 250, 250);
    info!("Display setup");
    //250*250
    let mut display = Display4in2::default();
    display.clear(Color::White).ok();
    epd4in2
        .update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay)
        .unwrap();

    display.set_rotation(DisplayRotation::Rotate0);
    draw_text(&mut display, "Are you going to stop flashing?", 5, 50);

    epd4in2.update_frame(&mut spi_dev, display.buffer(), &mut Delay);
    epd4in2
        .display_frame(&mut spi_dev, &mut Delay)
        .expect("display frame new graphics");

    let bmp_data = include_bytes!("../ferris_w_a_knife.bmp");
    let bmp: Bmp<BinaryColor> = Bmp::from_slice(bmp_data).unwrap();

    Image::new(&bmp, Point::new(50, 100)).draw(&mut display.color_converted());
    epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);

    loop {}
}

fn draw_text(display: &mut impl DrawTarget<Color = Color>, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
    info!("Draw text: {:?}", text);
}
