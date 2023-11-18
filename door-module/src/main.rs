use std::sync::{Arc, Mutex};

use shared_lib::wifi::config::{SYSTEM_AP_PASSWORD, SYSTEM_AP_SSID};
use shared_lib::wifi::client::connect_wifi_sync;
use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};
use http::server::create_http_server;

use crate::io::input::prepare_input_pins;

mod http;
mod io;

fn main() -> anyhow::Result<()> {
    // setup
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("Couldn't take peripherals");

    // Setup WI-FI AP and client connection
    let _wifi = connect_wifi_sync(peripherals.modem, SYSTEM_AP_SSID, SYSTEM_AP_PASSWORD)
        .expect("Failed to connect to wifi...");

    let raw_input_pins = prepare_input_pins(peripherals.adc1, peripherals.pins.gpio2, peripherals.pins.gpio3)
        .expect("Couldn't initialize input pins");

    let input = Arc::new(Mutex::new(raw_input_pins));

    let _http_server = create_http_server(input.clone());

    loop {
        let val1 = input.lock().unwrap().read_closing_current().unwrap();
        let val2 = input.lock().unwrap().read_opening_current().unwrap();
        log::info!("Current: closing_{}mA, opening_{}mA", val1, val2);

        FreeRtos::delay_ms(5000);
    }
}