use std::sync::Arc;

use embedded_svc::utils::mutex::Mutex;
use esp32_nimble::{
    utilities::mutex::RawMutex, uuid128, BLECharacteristic, BLEDevice, BLEService, NimbleProperties, enums::{AuthReq, SecurityIOCap},
};
use shared_lib::dto::pw_config::{PowerWindowsConfig, Deserialize};
use tokio::sync::broadcast;

use super::config::{DEBUG_NOTIFYING_UUID, PW_CFG_UUID};

pub struct BluetoothServer {
    pub service: Arc<Mutex<RawMutex, BLEService>>,
    pub debug_characteristic: Arc<Mutex<RawMutex, BLECharacteristic>>,
    pub pw_cfg_characteristic: Arc<Mutex<RawMutex, BLECharacteristic>>,
}

impl BluetoothServer {
    pub fn new() -> anyhow::Result<BluetoothServer> {
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
            Ok(_) => log::info!("Advertising started"),
            Err(e) => {
                log::error!("Advertising failed to start: {:?}", e);
                return Err(anyhow::anyhow!("Advertising failed to start: {:?}", e));
            }
        }
    
        let debug_characteristic = service.lock().create_characteristic(
            DEBUG_NOTIFYING_UUID,
            NimbleProperties::READ | NimbleProperties::NOTIFY | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
        );
    
        let pw_cfg_characteristic = service.lock().create_characteristic(
            PW_CFG_UUID,
            NimbleProperties::READ | NimbleProperties::WRITE | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
        );
    
        Ok(BluetoothServer {
            service,
            debug_characteristic,
            pw_cfg_characteristic
        })
    }

    pub fn setup(self, config_sender: broadcast::Sender<PowerWindowsConfig>) {
        let mut pw_cfg = self.pw_cfg_characteristic.lock();
        
        pw_cfg.on_write(move |value| {
            if value.recv_data.len() != 8 {
                log::error!("Invalid config size: {}", value.recv_data.len());
                return;
            }

            let pw_cfg_raw: [u8; 8] = match value.recv_data.try_into() {
                Ok(pw_cfg_raw) => pw_cfg_raw,
                Err(e) => {
                    log::error!("Error: {:?}", e);
                    return;
                }
            };

            match config_sender.send(PowerWindowsConfig::deserialize(pw_cfg_raw)) {
                Ok(_) => log::info!("Sent config to clients"),
                Err(e) => log::error!("Error: {:?}", e)
            }
        });

        drop(pw_cfg);
    }
}
