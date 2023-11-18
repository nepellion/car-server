use std::sync::{Arc, Mutex};

use esp_idf_svc::systime::EspSystemTime;

use crate::app::app_state::AppState;

use super::server::BluetoothServer;

pub trait DebugHandler {
  fn handle_send_debug_info(self: Self, app_state: Arc<Mutex<AppState>>);
}

impl DebugHandler for BluetoothServer {
    fn handle_send_debug_info(self: BluetoothServer, app_state: Arc<Mutex<AppState>>) {
      let now_millis = EspSystemTime::now(&EspSystemTime {}).as_millis();
      
      match app_state.lock() {
        Ok(app_state) => {
          let time_since_last_debug = now_millis - app_state.last_debug;

          if time_since_last_debug < 200 {
            return;
          }

          self.debug_characteristic.lock()
            .set_value(app_state.to_string().as_bytes())
            .notify();

          println!("{}", app_state.is_current);
        },
        Err(e) => {
          println!("Error: {:?}", e);
        }
      }
    }
}
