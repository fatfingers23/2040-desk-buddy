#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use assign_resources::assign_resources;
use core::cell::RefCell;
use cyw43::JoinOptions;
use cyw43_driver::{net_task, setup_cyw43};
use defmt::*;
use display::{draw_current_outside_weather, draw_time, draw_weather_forecast_box};
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::dns::DnsSocket;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::{Config, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::i2c::I2c;
use embassy_rp::i2c::{self, InterruptHandler};
use embassy_rp::peripherals::{self, I2C0};
use embassy_rp::rtc::{DateTime, DayOfWeek, RtcError};
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    spi::{self, Spi},
};
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::signal;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use env::env_value;
use epd_waveshare::{
    color::*,
    epd4in2_v2::{Display4in2, Epd4in2},
    prelude::*,
};
use heapless::Vec;
use io::easy_format_str;
use rand::RngCore;
use reqwless::client::{HttpClient, TlsConfig, TlsVerify};
use scd4x::types::SensorData;
use scd4x::Scd4x;
use static_cell::StaticCell;
use web_requests::{get_web_request, ForecastResponse, TimeApiResponse};
use {defmt_rtt as _, panic_probe as _};

mod cyw43_driver;
mod display;
mod env;
mod io;
mod weather_icons;
mod web_requests;

type I2c0Bus = NoopMutex<RefCell<I2c<'static, I2C0, i2c::Blocking>>>;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

#[allow(dead_code)]
#[derive(Debug, Format)]
/// These are events that trigger web requests.
enum WebRequestEvents {
    UpdateForecast,
    UpdateOfficeStatus,
    GetTime,
}

enum GeneralEvents {
    ForecastUpdated(ForecastResponse),
    TimeFromApi(DateTime),
    //TODO also pass what was changed? Like hour, minute etc
    TimeDigitChanged(DateTime),
    SensorUpdate(SensorData),
}

//TODO need to go to each channel and event and log which is getting called when cause theres a wait happening somewhere
impl GeneralEvents {
    fn as_str(&self) -> &str {
        match self {
            GeneralEvents::ForecastUpdated(_) => "ForecastUpdated",
            GeneralEvents::TimeFromApi(_) => "TimeFromApi",
            GeneralEvents::TimeDigitChanged(_) => "TimeDigitChanged",
            GeneralEvents::SensorUpdate(_) => "SensorUpdate",
        }
    }
}

/// This is the type of Commands that we will send from the orchestrating task to the worker tasks.
/// Note that we are lazy here and only have one command, you might want to have more.
#[allow(dead_code)]
enum Commands {
    /// This command will stop the appropriate worker task
    Stop,
}

///Just a copy of SensorData to have debug, clone and format
#[derive(Debug, Clone, Format)]
pub struct InsideSensorData {
    pub co2: u16,
    pub temperature: f32,
    pub humidity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Format)]
enum StateChanges {
    None,
    ForecastUpdated,
    OfficeStatusUpdated,
    TimeSet,
    NewTimeDigit,
    SensorUpdate,
}

#[derive(Debug, Clone)]
struct State {
    forecast: Option<ForecastResponse>,
    date_time_from_api: Option<DateTime>,
    approximately_current_time: Option<DateTime>,
    sensor_data: Option<InsideSensorData>,
    state_change: StateChanges,
}

impl State {
    fn new() -> Self {
        Self {
            forecast: None,
            date_time_from_api: None,
            approximately_current_time: None,
            sensor_data: None,
            state_change: StateChanges::None,
        }
    }
}

static WEB_REQUEST_EVENT_CHANNEL: channel::Channel<CriticalSectionRawMutex, WebRequestEvents, 10> =
    channel::Channel::new();

static GENERAL_EVENT_CHANNEL: channel::Channel<CriticalSectionRawMutex, GeneralEvents, 10> =
    channel::Channel::new();

//TODO i think having multiple things listening to the consumer channel is the mix up
//Will come back later to see. Looks like that is it
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
    rtc: ClockPeripherals {
        rtc: RTC,
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources! {p};

    let i2c = I2c::new_blocking(p.I2C0, p.PIN_21, p.PIN_20, i2c::Config::default());
    static I2C_BUS: StaticCell<I2c0Bus> = StaticCell::new();
    // let i2c_bus = I2C_BUS.init(Mutex::new(i2c));
    let i2c_bus = NoopMutex::new(RefCell::new(i2c));
    let i2c_bus = I2C_BUS.init(i2c_bus);

    spawner.must_spawn(orchestrate(spawner));
    spawner.must_spawn(wireless_task(spawner, r.cyw43_peripherals));

    //Sensors/RTC tasks
    spawner.must_spawn(rtc_task(spawner, r.rtc));
    spawner.must_spawn(scd_task(spawner, i2c_bus));

    //Timings tasks?
    spawner.must_spawn(random_10s(spawner));

    //Display task
    spawner.must_spawn(display_task(r.display_peripherals));

    loop {
        info!("I'm still alive");
        Timer::after(Duration::from_secs(300)).await;
    }
}

#[embassy_executor::task]
async fn orchestrate(_spawner: Spawner) {
    let mut state = State::new();

    // we need to have a receiver for the events
    let receiver = GENERAL_EVENT_CHANNEL.receiver();

    // and we need a sender for the consumer task
    let state_sender = CONSUMER_CHANNEL.sender();

    loop {
        //Wait for an event
        let event = receiver.receive().await;
        info!("Event received: {:?}", event.as_str());
        match event {
            GeneralEvents::ForecastUpdated(forecast_response) => {
                state.forecast = Some(forecast_response);
                state.state_change = StateChanges::ForecastUpdated;
            }
            GeneralEvents::TimeFromApi(time) => {
                info!("Time received from API");
                state.date_time_from_api = Some(time);
                state.state_change = StateChanges::TimeSet;
            }
            GeneralEvents::TimeDigitChanged(time) => {
                info!("Time digit changed");
                state.approximately_current_time = Some(time);
                state.state_change = StateChanges::NewTimeDigit;
            }
            GeneralEvents::SensorUpdate(sensor_data) => {
                state.sensor_data = Some(InsideSensorData {
                    co2: sensor_data.co2,
                    temperature: sensor_data.temperature,
                    humidity: sensor_data.humidity,
                });
                state.state_change = StateChanges::SensorUpdate;
            }
        }
        info!("State change: {:?}", state.state_change);
        state_sender.send(state.clone()).await;
    }
}

#[embassy_executor::task]
async fn rtc_task(_spawner: Spawner, rtc_peripheral: ClockPeripherals) {
    let mut rtc = embassy_rp::rtc::Rtc::new(rtc_peripheral.rtc);

    let receiver = CONSUMER_CHANNEL.receiver();
    let sender = GENERAL_EVENT_CHANNEL.sender();

    loop {
        //Wait for an event
        let state = receiver.receive().await;
        info!("State received RTC: {:?}", state.state_change);
        match state.state_change {
            StateChanges::TimeSet => {
                if let Some(time) = state.date_time_from_api {
                    let result = rtc.set_datetime(time);
                    match result {
                        Ok(_) => {
                            info!("Time received and set");
                            break;
                        }
                        Err(e) => {
                            print_rtc_error(e);
                        }
                    }
                }
            }

            _ => {}
        }
        // state_sender.send(state.clone()).await;
    }

    let mut hour = 0;
    let mut minute = 0;

    loop {
        let possible_time = rtc.now();
        match possible_time {
            Ok(time) => {
                if time.hour != hour || time.minute != minute {
                    info!("Time: {}:{} {}", time.hour, time.minute, time.second);
                    hour = time.hour;
                    minute = time.minute;
                    sender.send(GeneralEvents::TimeDigitChanged(time)).await;
                }
            }
            Err(e) => {
                print_rtc_error(e);
            }
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}

fn print_rtc_error(e: RtcError) {
    match e {
        embassy_rp::rtc::RtcError::NotRunning => {
            error!("RTC not running");
        }
        embassy_rp::rtc::RtcError::InvalidDateTime(e) => {
            match e {
                embassy_rp::rtc::DateTimeError::InvalidYear => error!("Invalid year"),
                embassy_rp::rtc::DateTimeError::InvalidMonth => error!("Invalid month"),
                embassy_rp::rtc::DateTimeError::InvalidDay => error!("Invalid day"),
                embassy_rp::rtc::DateTimeError::InvalidDayOfWeek(dow) => {
                    error!("Invalid day of week: {}", dow)
                }
                embassy_rp::rtc::DateTimeError::InvalidHour => error!("Invalid hour"),
                embassy_rp::rtc::DateTimeError::InvalidMinute => error!("Invalid minute"),
                embassy_rp::rtc::DateTimeError::InvalidSecond => error!("Invalid second"),
            }
            error!("Invalid date time");
        }
    };
}

#[embassy_executor::task]
async fn scd_task(_spawner: Spawner, i2c_bus: &'static I2c0Bus) {
    // let fahrenheit = env_value("UNIT") == "fahrenheit";
    let sender = GENERAL_EVENT_CHANNEL.sender();

    let i2c_dev = I2cDevice::new(i2c_bus);
    let mut sensor = Scd4x::new(i2c_dev, Delay);

    sensor.stop_periodic_measurement().unwrap();
    sensor.reinit().unwrap();

    sensor.start_periodic_measurement().unwrap();
    //Need to wait 5 seconds before first measurement
    Timer::after(Duration::from_secs(5)).await;
    loop {
        let data = sensor.measurement().unwrap();
        // if fahrenheit {
        //     info!(
        //         "CO2: {} ppm, Temperature: {}°F, Humidity: {}%",
        //         data.co2,
        //         data.temperature * 1.8 + 32.0,
        //         data.humidity
        //     );
        // } else {
        //     info!(
        //         "CO2: {} ppm, Temperature: {}°C, Humidity: {}%",
        //         data.co2, data.temperature, data.humidity
        //     );
        // }
        sender.send(GeneralEvents::SensorUpdate(data)).await;
        Timer::after(Duration::from_secs(30)).await;
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
    //TODO need to come back and look at the epd driver I think there should be a cleaner clear function
    display.clear(Color::White).ok();
    // epd4in2.clear_frame(&mut spi_dev, &mut Delay);
    let _ = epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);

    epd4in2.sleep(&mut spi_dev, &mut Delay).unwrap();

    let receiver = CONSUMER_CHANNEL.receiver();

    // let sender = EVENT_CHANNEL.sender();
    loop {
        //TODO how do we decide what has changed?
        let state = receiver.receive().await;
        info!("State received Display: {:?}", state.state_change);
        match state.state_change {
            StateChanges::None => {}
            StateChanges::ForecastUpdated => {
                if let Some(forecast) = state.forecast {
                    let mut forecast_starting_point = Point::new(0, 145);
                    let forecast_box_width = 80;

                    //Only have room for a 5 day forecast
                    for i in 0..5 {
                        let daily_date = &forecast.daily.time[i];
                        let daily_max_temp = &forecast.daily.temperature_2m_max[i];
                        let daily_min_temp = &forecast.daily.temperature_2m_min[i];
                        let daily_weather_code = &forecast.daily.weather_code[i];
                        let sunrise = &forecast.daily.sunrise[i];
                        let sunset = &forecast.daily.sunset[i];
                        //I think all units are the same so just going to use this one
                        let unit = &forecast.daily_units.temperature_2m_max;
                        draw_weather_forecast_box(
                            forecast_starting_point,
                            forecast_box_width,
                            daily_date,
                            &unit,
                            *daily_max_temp,
                            *daily_min_temp,
                            *daily_weather_code,
                            sunrise.clone(),
                            sunset.clone(),
                            //TODO prob change to the updated datetime
                            state.date_time_from_api.clone(),
                            i as u8,
                            &mut display,
                        );
                        forecast_starting_point.x += forecast_box_width as i32;
                    }

                    let current_weather_starting_point = Point::new(300, 45);
                    draw_current_outside_weather(
                        current_weather_starting_point,
                        forecast.current,
                        forecast.current_units,
                        &mut display,
                    );
                    //Do not need to wake up till right before I write since display is just handled on the RP2040
                    let _ = epd4in2.wake_up(&mut spi_dev, &mut Delay);
                    let _ = epd4in2.update_and_display_frame(
                        &mut spi_dev,
                        display.buffer(),
                        &mut Delay,
                    );
                    epd4in2.sleep(&mut spi_dev, &mut Delay).unwrap();
                }
            }
            StateChanges::OfficeStatusUpdated => {}
            StateChanges::TimeSet => {
                if let Some(date_time) = state.date_time_from_api {
                    draw_time(date_time, &mut display);
                }
                let _ = epd4in2.wake_up(&mut spi_dev, &mut Delay);
                let _ =
                    epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);
                epd4in2.sleep(&mut spi_dev, &mut Delay).unwrap();
            }
            StateChanges::NewTimeDigit => {
                if let Some(date_time) = state.approximately_current_time {
                    draw_time(date_time, &mut display);
                }
                let _ = epd4in2.wake_up(&mut spi_dev, &mut Delay);
                let _ =
                    epd4in2.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay);
                epd4in2.sleep(&mut spi_dev, &mut Delay).unwrap();
            }
            StateChanges::SensorUpdate => {}
        }
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

    spawner.must_spawn(net_task(runner));
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
    let receiver = WEB_REQUEST_EVENT_CHANNEL.receiver();
    let sender = GENERAL_EVENT_CHANNEL.sender();

    loop {
        //Wait for an event
        let event = receiver.receive().await;
        info!("Display Event received: {:?}", event);
        //Build the http client
        let mut tls_read_buffer = [0; 16640];
        let mut tls_write_buffer = [0; 16640];

        let client_state = TcpClientState::<4, 1024, 1024>::new();
        let tcp_client = TcpClient::new(stack, &client_state);
        let dns_client = DnsSocket::new(stack);
        let tls_config = TlsConfig::new(
            seed,
            &mut tls_read_buffer,
            &mut tls_write_buffer,
            TlsVerify::None,
        );

        let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

        match event {
            WebRequestEvents::UpdateForecast => {
                let mut rx_buffer = [0; 8320];
                let lat = env_value("LAT");
                let long = env_value("LON");
                let unit = env_value("UNIT");
                let timezone = env_value("TIMEZONE");

                let mut url_buffer = [0u8; 1_028];

                let formatted_url = easy_format_str(format_args!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_probability_max&temperature_unit={}&timezone={}",
                lat, long, unit, timezone), &mut url_buffer);

                let result = get_web_request::<ForecastResponse>(
                    &mut http_client,
                    formatted_url.unwrap(),
                    &mut rx_buffer,
                )
                .await;

                match result {
                    Ok(forecast) => {
                        sender.send(GeneralEvents::ForecastUpdated(forecast)).await;
                    }
                    Err(e) => {
                        error!("Failed to get forecast: {:?}", e);
                    }
                }
            }
            WebRequestEvents::UpdateOfficeStatus => {
                //Call the office status update web request when implemented
            }
            WebRequestEvents::GetTime => {
                let mut rx_buffer = [0; 8320];
                let timezone = env_value("TIMEZONE");

                let mut url_buffer = [0u8; 1_028]; // im sure this can be much smaller

                let formatted_url = easy_format_str(
                    format_args!("https://worldtimeapi.org/api/timezone/{}", timezone),
                    &mut url_buffer,
                );

                let result = get_web_request::<TimeApiResponse>(
                    &mut http_client,
                    formatted_url.unwrap(),
                    &mut rx_buffer,
                )
                .await;

                if let Ok(response) = result {
                    //TODO need to hide this away
                    //Hell is empty and all the odd string manipulation code is here
                    let datetime = response.datetime.split('T').collect::<Vec<&str, 2>>();
                    //split at -
                    let date = datetime[0].split('-').collect::<Vec<&str, 3>>();
                    let year = date[0].parse::<u16>().unwrap();
                    let month = date[1].parse::<u8>().unwrap();
                    let day = date[2].parse::<u8>().unwrap();
                    //split at :
                    let time = datetime[1].split(':').collect::<Vec<&str, 4>>();
                    let hour = time[0].parse::<u8>().unwrap();
                    let minute = time[1].parse::<u8>().unwrap();
                    //split at .
                    let second_split = time[2].split('.').collect::<Vec<&str, 2>>();
                    let second = second_split[0].parse::<f64>().unwrap();
                    let rtc_time = DateTime {
                        year: year,
                        month: month,
                        day: day,
                        day_of_week: match response.day_of_week {
                            0 => DayOfWeek::Sunday,
                            1 => DayOfWeek::Monday,
                            2 => DayOfWeek::Tuesday,
                            3 => DayOfWeek::Wednesday,
                            4 => DayOfWeek::Thursday,
                            5 => DayOfWeek::Friday,
                            6 => DayOfWeek::Saturday,
                            _ => DayOfWeek::Sunday,
                        },
                        hour,
                        minute,
                        second: second as u8,
                    };
                    info!("sending time to rtc");
                    sender.send(GeneralEvents::TimeFromApi(rtc_time)).await;
                }
            }
        }
    }
}

///Proof of concept on something to call the tasks
#[embassy_executor::task]
async fn random_10s(_spawner: Spawner) {
    let sender = WEB_REQUEST_EVENT_CHANNEL.sender();
    Timer::after(Duration::from_secs(10)).await;
    info!("10s are up, calling time api");
    //Calls to get time from the an API
    sender.send(WebRequestEvents::GetTime).await;
    Timer::after(Duration::from_secs(30)).await;
    sender.send(WebRequestEvents::UpdateForecast).await;

    loop {
        // we either await on the timer or the signal, whichever comes first.
        let futures = select(
            Timer::after(Duration::from_secs(150)),
            STOP_FIRST_RANDOM_SIGNAL.wait(),
        )
        .await;
        match futures {
            Either::First(_) => {
                // we received are operating on the timer
                // info!("10s are up, calling forecast update");
                // sender.send(WebRequestEvents::UpdateForecast).await;
                sender.send(WebRequestEvents::UpdateForecast).await;
            }
            Either::Second(_) => {
                // we received the signal to stop
                info!("Received signal to stop, goodbye!");
                break;
            }
        }
    }
}
