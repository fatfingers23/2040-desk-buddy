#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use assign_resources::assign_resources;
use core::str::from_utf8;
use cyw43::JoinOptions;
use cyw43_driver::{net_task, setup_cyw43};
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Config, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_rp::peripherals::{self};
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    spi::{self, Spi},
};
use embassy_sync::signal;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{
    image::Image,
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal_bus::spi::ExclusiveDevice;
use env::env_value;
use epd_waveshare::{
    color::*,
    epd4in2_v2::{Display4in2, Epd4in2},
    prelude::*,
};
use io::easy_format_str;
use rand::RngCore;
use reqwless::client::{HttpClient, TlsConfig, TlsVerify};
use reqwless::request::Method;
use response_models::WeatherResponse;
use static_cell::StaticCell;
use tinybmp::Bmp;
use {defmt_rtt as _, panic_probe as _};

mod cyw43_driver;
mod env;
mod io;
mod response_models;

#[allow(dead_code)]
/// This is the type of Events that we will send from the worker tasks to the orchestrating task.
enum Events {
    //Can send data with these so make sure to check the example
    UpdateWeather,
    UpdateOfficeStatus,
}

/// This is the type of Commands that we will send from the orchestrating task to the worker tasks.
/// Note that we are lazy here and only have one command, you might want to have more.
#[allow(dead_code)]
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

/// Signal for stopping the first random signal task. We use a signal here, because we need no queue. It is suffiient to have one signal active.
static STOP_FIRST_RANDOM_SIGNAL: signal::Signal<CriticalSectionRawMutex, Commands> =
    signal::Signal::new();

assign_resources! {
    display_peripherals: DisplayPeripherals {
        spi: SPI1,
        mosi: PIN_11,
        clk: PIN_10,
        cs: PIN_9,
        dc: PIN_8,
        rst: PIN_12,
        busy: PIN_13,

    },
    cyw43_peripherals: Cyw43Peripherals {
        pio: PIO0,
        cs: PIN_23,
        sck: PIN_24,
        mosi: PIN_25,
        miso: PIN_29,
        dma: DMA_CH0,
    },
    // add more resources to more structs if needed, for example defining one struct for each task
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources! {p};

    spawner.must_spawn(orchestrate(spawner));
    spawner.must_spawn(wireless_task(spawner, r.cyw43_peripherals));
    //Proof of concept caller
    spawner.must_spawn(random_30s(spawner));
    //TODO display commented out while disconnected for wifi development
    // spawner.must_spawn(display_task(r.display_peripherals));
}

#[embassy_executor::task]
async fn orchestrate(_spawner: Spawner) {
    let mut test = 1;
    info!("{:?}", test);

    test = 2;
    info!("{:?}", test);

    let mut _state = State::new();

    // we need to have a receiver for the events
    let receiver = EVENT_CHANNEL.receiver();

    // and we need a sender for the consumer task
    let _state_sender = CONSUMER_CHANNEL.sender();

    loop {
        //Wait for an event
        let _event = receiver.receive().await;
    }
}

#[embassy_executor::task]
pub async fn display_task(display_pins: DisplayPeripherals) {
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
    // epd4in2.clear_frame(&mut spi_dev, &mut Delay);
    let _ = epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);

    draw_text(&mut display, "Hey", 5, 50);
    draw_bmp(
        &mut display,
        include_bytes!("../ferris_w_a_knife.bmp"),
        5,
        100,
    );
    let _ = epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);
    epd4in2.sleep(&mut spi_dev, &mut Delay).unwrap();

    loop {
        Timer::after_millis(50).await;
    }
}

#[embassy_executor::task]
async fn wireless_task(spawner: Spawner, cyw43_peripherals: Cyw43Peripherals) {
    let mut rng: RoscRng = RoscRng;
    let (net_device, mut control) = setup_cyw43(
        cyw43_peripherals.pio,
        cyw43_peripherals.cs,
        cyw43_peripherals.sck,
        cyw43_peripherals.mosi,
        cyw43_peripherals.miso,
        cyw43_peripherals.dma,
        spawner,
    )
    .await;
    debug!("Wireless task started");
    control.gpio_set(0, true).await;

    let config = Config::dhcpv4(Default::default());

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    unwrap!(spawner.spawn(net_task(runner)));
    let wifi_network = env_value("WIFI_SSID");
    let wifi_password = env_value("WIFI_PASSWORD");

    loop {
        match control
            .join(wifi_network, JoinOptions::new(wifi_password.as_bytes()))
            .await
        {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    info!("waiting for link up...");
    while !stack.is_link_up() {
        Timer::after_millis(500).await;
    }
    info!("Link is up!");

    info!("waiting for stack to be up...");
    stack.wait_config_up().await;
    info!("Stack is up!");
    //Turns LED on so I know it's connected and ready
    control.gpio_set(0, true).await;

    //TODO do this like orchestrate task where it listens for a method like "update weather", "update office status", etc
    // we need to have a receiver for the events
    let receiver = EVENT_CHANNEL.receiver();

    loop {
        //Wait for an event
        let event = receiver.receive().await;
        match event {
            Events::UpdateWeather => {
                let mut rx_buffer = [0; 8320];
                let result = get_weather_updates(stack, seed, &mut rx_buffer).await;
                if let Ok(weather) = result {
                    info!("Task 1: {:?}", weather.daily.time[0]);
                    // let weather = get_weather_updates(stack, seed
                }
            }
            Events::UpdateOfficeStatus => {
                let mut rx_buffer = [0; 8320];

                let result = get_weather_updates(stack, seed, &mut rx_buffer).await;
                if let Ok(weather) = result {
                    info!("Task 2: {:?}", weather.daily.time[0]);
                    // let weather = get_weather_updates(stack, seed
                }
            }
        }
    }
}

///Proof of concept on something to call the tasks
#[embassy_executor::task]
async fn random_30s(_spawner: Spawner) {
    let sender = EVENT_CHANNEL.sender();
    loop {
        // we either await on the timer or the signal, whichever comes first.
        let futures = select(
            Timer::after(Duration::from_secs(30)),
            STOP_FIRST_RANDOM_SIGNAL.wait(),
        )
        .await;
        match futures {
            Either::First(_) => {
                // we received are operating on the timer
                info!("30s are up, generating random number");

                sender.send(Events::UpdateWeather).await;
                Timer::after(Duration::from_secs(4)).await;
                sender.send(Events::UpdateOfficeStatus).await;
            }
            Either::Second(_) => {
                // we received the signal to stop
                info!("Received signal to stop, goodbye!");
                break;
            }
        }
    }
}

pub enum WebCallError {
    HttpError(u16),
    WebRequestError,
    FailedToReadResponse,
    DeserializationError,
    UrlFormatError,
}

async fn get_weather_updates<'a>(
    stack: embassy_net::Stack<'static>,
    seed: u64,
    rx_buffer: &'a mut [u8],
) -> Result<WeatherResponse<'a>, WebCallError> {
    //TODO i think this can be a lot less

    let mut tls_read_buffer = [0; 16640];
    let mut tls_write_buffer = [0; 16640];

    let client_state = TcpClientState::<1, 1024, 1024>::new();
    let tcp_client = TcpClient::new(stack, &client_state);
    let dns_client = DnsSocket::new(stack);
    let tls_config = TlsConfig::new(
        seed,
        &mut tls_read_buffer,
        &mut tls_write_buffer,
        TlsVerify::None,
    );

    info!("Making HTTP request");
    let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

    let lat = env_value("LAT");
    let long = env_value("LON");
    let unit = env_value("UNIT");
    let timezone = env_value("TIMEZONE");

    let mut url_buffer = [0u8; 8_192]; // im sure this can be much smaller

    let formatted_url = easy_format_str(
        format_args!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_probability_max&temperature_unit={}&timezone={}",
         lat, long, unit, timezone), &mut url_buffer);
    let url = match formatted_url {
        Ok(url) => url,
        Err(_) => {
            error!("Failed to format URL");
            return Err(WebCallError::UrlFormatError);
        }
    };

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
    match serde_json_core::de::from_slice::<WeatherResponse>(body.as_bytes()) {
        Ok((output, _used)) => Ok(output),
        Err(_) => {
            error!("There was an error deserlizing the weather reuqest");
            return Err(WebCallError::DeserializationError);
        }
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
    debug!("Draw text: {:?}", text);
}
