use bt::server::create_bt_server;
use shared_lib::wifi::config::{SYSTEM_AP_PASSWORD, SYSTEM_AP_SSID};
use shared_lib::wifi::ext::get_ap_client_infos;
use shared_lib::wifi::server::create_wifi_ap_sync;
use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};

mod app;
mod bt;
mod mqtt;

fn main() {
    // setup
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("Couldn't take peripherals");

    // Setup WI-FI AP and client connection
    // let a = create_wifi_ap_sync(peripherals.modem).expect("Setting up custom wifi AP failed");
    let mut wifi = create_wifi_ap_sync(peripherals.modem, SYSTEM_AP_SSID, SYSTEM_AP_PASSWORD)
        .expect("Failed to connect to wifi...");

    let _bt_server = create_bt_server().expect("Failed to create BT server...");

    loop {
        let clients = get_ap_client_infos(&mut wifi).expect("Failed to get AP client infos");
        let client_count = clients.len();

        if client_count > 0 {
            log::info!("Listing clients...");
        }

        for (i, client) in clients.iter().enumerate() {
            log::info!("Client {:?}: {:?}", i, client.ip);
        }

        if client_count == 0 {
            log::info!("No clients connected");
        }

        FreeRtos::delay_ms(30000);
    }
}
