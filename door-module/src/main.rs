use std::sync::Arc;

use app::state::AppState;
use esp_idf_hal::peripherals::Peripherals;
use hal::power_window_driver::PowerWindowDriverPins;
use http::server::prepare_http_server;
use shared_lib::dto::pw_config::PowerWindowsConfig;
use shared_lib::system::{setup_system, run_tokio_runtime};
use shared_lib::wifi::client::connect_wifi_sync;
use shared_lib::wifi::config::{SYSTEM_AP_PASSWORD, SYSTEM_AP_SSID};
use shared_lib::wifi::ext::get_sta_mac_address;
use svc::power_windows::PowerWindowSvc;
use tokio::sync::Mutex;

use crate::app::events::ServerRequest;

mod app;
mod hal;
mod http;
mod svc;

pub const DEBUG: bool = true;

fn main() -> anyhow::Result<()> {
    setup_system()?;

    let peripherals = Peripherals::take().expect("Couldn't take peripherals");

    // Setup WI-FI AP and client connection
    let mut wifi = connect_wifi_sync(peripherals.modem, SYSTEM_AP_SSID, SYSTEM_AP_PASSWORD)?;
    
    let mac_address = get_sta_mac_address(&mut wifi)?;

    log::info!("Mac address: {:?}", mac_address);

    let power_windows_svc = Arc::new(Mutex::new(PowerWindowSvc::new(
        PowerWindowDriverPins {
            adc: peripherals.adc1,
            window_closing_sense_pin: peripherals.pins.gpio2,
            window_opening_sense_pin: peripherals.pins.gpio3,
            window_closing_pin: peripherals.pins.gpio10,
            window_opening_pin: peripherals.pins.gpio11,
        },
        PowerWindowsConfig {
            opening_current_interrupt_threshold_amps: 20,
            closing_current_interrupt_threshold_amps: 20,
            handle_time_threshold_millis: 300,
        },
    )?));

    run_tokio_runtime(async move {
        let (sender, pw_svc_receiver) = tokio::sync::broadcast::channel::<ServerRequest>(8);

        let http_server = prepare_http_server(sender.clone(), Arc::new(Mutex::new(AppState { random_val: "hey" })));

        tokio::spawn(PowerWindowSvc::run_loop(pw_svc_receiver, power_windows_svc)).await.expect("Power window service crashed!");

        drop(http_server);
    })?;

    Ok(())
}
