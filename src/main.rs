#![no_std]
#![no_main]

use assign_resources::assign_resources;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{self};
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    peripherals::SPI1,
    spi::{self, Spi},
};
use embassy_sync::{
    blocking_mutex::{
        raw::{CriticalSectionRawMutex, NoopRawMutex},
        Mutex,
    },
    channel,
};
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::TimeSource;
use epd_waveshare::{
    color::*,
    epd4in2_v2::{Display4in2, Epd4in2},
    prelude::*,
};
use static_cell::StaticCell;
use tinybmp::Bmp;
use {defmt_rtt as _, panic_probe as _};

mod cyw43_driver;
mod env;
mod io;

type Spi1Bus = Mutex<NoopRawMutex, Spi<'static, SPI1, spi::Blocking>>;

/// This is the type of Events that we will send from the worker tasks to the orchestrating task.
enum Events {
    UsbPowered(bool),
    VsysVoltage(f32),
    FirstRandomSeed(u32),
    SecondRandomSeed(u32),
    ThirdRandomSeed(u32),
    ResetFirstRandomSeed,
}

/// This is the type of Commands that we will send from the orchestrating task to the worker tasks.
/// Note that we are lazy here and only have one command, you might want to have more.
enum Commands {
    /// This command will stop the appropriate worker task
    Stop,
}

#[derive(Default, Debug, Clone, Format)]
struct State {}

impl State {
    fn new() -> Self {
        Self {}
    }
}

static EVENT_CHANNEL: channel::Channel<CriticalSectionRawMutex, Events, 10> =
    channel::Channel::new();

static CONSUMER_CHANNEL: channel::Channel<CriticalSectionRawMutex, State, 1> =
    channel::Channel::new();

assign_resources! {
    display_peripherals: DisplayPeripherals {
        spi: SPI1,
        mosi: PIN_11,
        clk: PIN_10,
        cs: PIN_9,
        dc: PIN_8,
        rst: PIN_12,
        busy: PIN_13,

    }
    // add more resources to more structs if needed, for example defining one struct for each task
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_rp::init(Default::default());
    let r = split_resources! {p};
    // let (_device, mut control) = setup_cyw43(
    //     p.PIO0, p.PIN_23, p.PIN_24, p.PIN_25, p.PIN_29, p.DMA_CH0, spawner,
    // )
    // .await;

    info!("Starting up");
    spawner.must_spawn(orchestrate(spawner));
    spawner.must_spawn(display_task(r.display_peripherals));

    loop {
        Timer::after_millis(1_000).await;
        info!("Hello, World!");
    }
}

#[embassy_executor::task]
async fn orchestrate(_spawner: Spawner) {
    info!("Orchestrating task started");
    let mut state = State::new();

    // we need to have a receiver for the events
    let receiver = EVENT_CHANNEL.receiver();

    // and we need a sender for the consumer task
    let state_sender = CONSUMER_CHANNEL.sender();

    loop {
        //Wait for an event
        let event = receiver.receive().await;
    }
}

#[embassy_executor::task]
pub async fn display_task(display_pins: DisplayPeripherals) {
    info!("Display task started");
    let cs = Output::new(display_pins.cs, Level::High);
    let dc = Output::new(display_pins.dc, Level::High);
    let rst = Output::new(display_pins.rst, Level::High);
    let busy = Input::new(display_pins.busy, Pull::Up);

    let mut config = spi::Config::default();
    config.frequency = 4_000_000;

    let spi = Spi::new_blocking_txonly(
        display_pins.spi,
        display_pins.clk,
        display_pins.mosi,
        config,
    );
    let mut spi_dev = ExclusiveDevice::new(spi, cs, embassy_time::Delay);

    let mut epd4in2 = Epd4in2::new(&mut spi_dev, busy, dc, rst, &mut embassy_time::Delay, None)
        .expect("eink initalize error");

    let mut display = Display4in2::default();
    display.clear(Color::White).ok();

    draw_text(&mut display, "Hey", 5, 50);
    epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);
    draw_bmp(
        &mut display,
        include_bytes!("../ferris_w_a_knife.bmp"),
        5,
        100,
    );
    epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);
    epd4in2.sleep(&mut spi_dev, &mut Delay).unwrap();

    loop {
        Timer::after_millis(50).await;
    }
}

fn draw_bmp(display: &mut impl DrawTarget<Color = Color>, bmp_data: &[u8], x: i32, y: i32) {
    let bmp: Bmp<BinaryColor> = Bmp::from_slice(bmp_data).unwrap();
    let _ = Image::new(&bmp, Point::new(x, y)).draw(&mut display.color_converted());
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
