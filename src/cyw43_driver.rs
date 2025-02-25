use cyw43::Control;
use cyw43_pio::PioSpi;
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_net_wiznet::Device;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::pio::{InterruptHandler, Pio};
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

pub async fn setup_cyw43<'a>(
    pio0: PIO0,
    p_23: PIN_23,
    p_24: PIN_24,
    p_25: PIN_25,
    p_29: PIN_29,
    dma_ch0: DMA_CH0,
    spawner: Spawner,
) -> (Device<'a>, Control<'a>) {
    // let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    // let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    // To make flashing faster for development, you may want to flash the firmwares independently
    // at hardcoded addresses, instead of baking them into the program with `include_bytes!`:
    //     probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x101b0000
    //     probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x101f8000
    //     probe-rs download 43439A0_btfw.bin --format bin --chip RP2040 --base-address 0x10141400
    const FW_SZ: usize = include_bytes!("../cyw43-firmware/43439A0.bin").len();
    const CLM_SZ: usize = include_bytes!("../cyw43-firmware/43439A0_clm.bin").len();
    let fw = unsafe { core::slice::from_raw_parts(0x101b0000 as *const u8, FW_SZ) };
    let clm = unsafe { core::slice::from_raw_parts(0x101f8000 as *const u8, CLM_SZ) };
    //let btfw = unsafe { core::slice::from_raw_parts(0x10141400 as *const u8, 6164) };

    let pwr = Output::new(p_23, Level::Low);
    let cs = Output::new(p_25, Level::High);
    let mut pio = Pio::new(pio0, Irqs);
    let spi = PioSpi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p_24, p_29, dma_ch0);

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    (net_device, control)
}
