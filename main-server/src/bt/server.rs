use std::sync::Arc;

use embedded_svc::utils::mutex::Mutex;
use esp32_nimble::{
    utilities::mutex::RawMutex, uuid128, BLECharacteristic, BLEDevice, BLEService, NimbleProperties, enums::{AuthReq, SecurityIOCap},
};

use super::config::DEBUG_NOTIFYING;

pub struct BluetoothServer {
    pub service: Arc<Mutex<RawMutex, BLEService>>,
    pub debug_characteristic: Arc<Mutex<RawMutex, BLECharacteristic>>,
}

pub fn create_bt_server() -> Result<BluetoothServer, String> {
    let ble_device = BLEDevice::take();
    ble_device
      .security()
      .set_auth(AuthReq::all())
      .set_passkey(423156)
      .set_io_cap(SecurityIOCap::DisplayOnly);

    let server = ble_device.get_server();

    server.on_connect(|server, desc| {
        ::log::info!("Client connected");

        server
            .update_conn_params(desc.conn_handle, 24, 48, 0, 60)
            .unwrap();

        ::log::info!("Multi-connect support: start advertising");
        ble_device.get_advertising().start().unwrap();
    });

    server.on_disconnect(|_desc, reason| {
        ::log::info!("Client disconnected ({:X})", reason);
    });

    let service = server.create_service(uuid128!("82abaa9d-850d-46a1-87a6-88d4facf293a"));

    let ble_advertising = ble_device.get_advertising();
    ble_advertising
        .name("Nepellion CAR Server - E36")
        .add_service_uuid(uuid128!("82abaa9d-850d-46a1-87a6-88d4facf293a"));

    match ble_advertising.start() {
        Ok(_) => {},
        Err(e) => {
            return Err(format!("Failed to start advertising, code: {}", e.0));
        }
    }

    let debug_characteristic = service.lock().create_characteristic(
        DEBUG_NOTIFYING,
        NimbleProperties::READ | NimbleProperties::NOTIFY | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
    );

    Ok(BluetoothServer {
        service,
        debug_characteristic
    })
}
