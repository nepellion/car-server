use std::sync::Arc;

use shared_lib::http::endpoints;
use tokio::sync::{broadcast, Mutex};

use crate::{
    clients::types::ClientType,
    hal::power_window_controls_driver::{PowerWindowButtonState, PowerWindowDriver},
};

pub struct PowerWindowsSvc {
}

impl PowerWindowsSvc {
    pub async fn run_loop(
        power_window_controls_driver: Arc<Mutex<PowerWindowDriver>>,
        http_sender: broadcast::Sender<(ClientType, &'static str, [u8; 8])>,
    ) {
        log::info!("Spawned clients service.");


        let button_handling_task = tokio::spawn(async move {
            loop {
                let mut power_window_controls_driver = power_window_controls_driver.lock().await;

                let left_button_state = match power_window_controls_driver.read_left_button_state()
                {
                    Ok(state) => state,
                    Err(err) => {
                        log::error!("Couldn't read left button state: {:?}", err);
                        continue;
                    }
                };

                Self::send_command_to_client(http_sender.clone(), ClientType::LeftDoor, left_button_state);

                let right_button_state =
                    match power_window_controls_driver.read_right_button_state() {
                        Ok(state) => state,
                        Err(err) => {
                            log::error!("Couldn't read right button state: {:?}", err);
                            continue;
                        }
                    };

                Self::send_command_to_client(http_sender.clone(), ClientType::RightDoor, right_button_state);
            }
        });

        button_handling_task.await.unwrap();
    }

    fn send_command_to_client(
        http_sender: broadcast::Sender<(ClientType, &'static str, [u8; 8])>,
        client_type: ClientType,
        button_state: PowerWindowButtonState,
    ) {
        match button_state {
            PowerWindowButtonState::CloseContinuous => {
                http_sender
                    .send((client_type, endpoints::CLOSE_WINDOWS_CONTINUOUS_PATH, [0; 8]))
                    .unwrap();
            }
            PowerWindowButtonState::CloseFully => {
                http_sender
                    .send((client_type, endpoints::CLOSE_WINDOWS_FULLY_PATH, [0; 8]))
                    .unwrap();
            }
            PowerWindowButtonState::OpenContinuous => {
                http_sender
                    .send((client_type, endpoints::OPEN_WINDOWS_CONTINUOUS_PATH, [0; 8]))
                    .unwrap();
            }
            PowerWindowButtonState::OpenFully => {
                http_sender
                    .send((client_type, endpoints::OPEN_WINDOWS_FULLY_PATH, [0; 8]))
                    .unwrap();
            }
            PowerWindowButtonState::None => {
                http_sender
                    .send((client_type, endpoints::STOP_WINDOWS_PATH, [0; 8]))
                    .unwrap();
            }
        }
    }
}
