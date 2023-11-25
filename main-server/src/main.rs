use std::sync::Arc;

use bt::server::BluetoothServer;
use clients::list::ClientsList;
use esp_idf_hal::peripherals::Peripherals;
use hal::power_window_controls_driver::PowerWindowDriver;
use hal::{
    DefaultLeftRequiredButtonPins, DefaultPowerWindowPeripherals, DefaultRightRequiredButtonPins,
};
use shared_lib::dto::pw_config::PowerWindowsConfig;
use shared_lib::system::{run_tokio_runtime, setup_system};
use shared_lib::wifi::config::{SYSTEM_AP_PASSWORD, SYSTEM_AP_SSID};
use shared_lib::wifi::server::create_wifi_ap_sync;
use svc::clients::ClientsSvc;
use svc::power_window::PowerWindowsSvc;
use svc::rest_client::RestClientSvc;
use tokio::join;
use tokio::sync::{broadcast, Mutex};

use crate::clients::types::ClientType;

mod app;
mod bt;
mod clients;
mod hal;
mod svc;

fn main() -> anyhow::Result<()> {
    setup_system()?;

    let peripherals = Peripherals::take().expect("Couldn't take peripherals");

    // Setup WI-FI AP and client connection
    // let a = create_wifi_ap_sync(peripherals.modem).expect("Setting up custom wifi AP failed");
    let wifi = create_wifi_ap_sync(peripherals.modem, SYSTEM_AP_SSID, SYSTEM_AP_PASSWORD)
        .expect("Failed to connect to wifi...");

    let bt_server = BluetoothServer::new().expect("Failed to create BT server...");

    let clients_svc = Arc::new(Mutex::new(ClientsSvc::new()));

    let power_window_controls_driver = Arc::new(Mutex::new(PowerWindowDriver::new(
        DefaultPowerWindowPeripherals {
            adc: peripherals.adc1,
            left_pins: DefaultLeftRequiredButtonPins {
                open_pin: peripherals.pins.gpio4,
                close_pin: peripherals.pins.gpio5,
            },
            right_pins: DefaultRightRequiredButtonPins {
                open_pin: peripherals.pins.gpio2,
                close_pin: peripherals.pins.gpio3,
            },
        },
    )?));

    let rest_svc = Arc::new(Mutex::new(RestClientSvc::new()));

    run_tokio_runtime(async move {
        let (clients_sender, clients_receiver) = broadcast::channel::<ClientsList>(8);
        let (http_sender, http_receiver) = broadcast::channel::<(ClientType, &'static str, [u8; 8])>(8);
        let (pw_cfg_sender, pw_cfg_receiver) = broadcast::channel::<PowerWindowsConfig>(8);

        let clients_svc_task = ClientsSvc::run_loop(wifi, clients_sender, clients_svc);

        let pw_svc_task = PowerWindowsSvc::run_loop(
            power_window_controls_driver,
            http_sender
        );

        bt_server.setup(pw_cfg_sender);

        let rest_svc_task = RestClientSvc::run_loop(clients_receiver, pw_cfg_receiver, http_receiver, rest_svc);

        join!(clients_svc_task, pw_svc_task, rest_svc_task);
    })?;

    Ok(())
}
