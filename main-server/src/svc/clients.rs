use std::sync::Arc;

use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use shared_lib::wifi::ext::get_ap_client_infos;
use tokio::sync::{broadcast, Mutex};

use crate::clients::{
    addresses::get_mac_for_client_type,
    list::ClientsList,
    types::CLIENT_TYPES,
};

pub struct ClientsSvc {
    clients: ClientsList,
}

impl ClientsSvc {
    pub fn new() -> ClientsSvc {
        ClientsSvc {
            clients: Default::default(),
        }
    }

    pub async fn run_loop(
        mut wifi: AsyncWifi<EspWifi<'static>>,
        sender: broadcast::Sender<ClientsList>,
        svc_src: Arc<Mutex<ClientsSvc>>,
    ) {
        log::info!("Spawned clients service.");

        let svc = svc_src.clone();
        let new_client_identifier_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                let clients =
                    get_ap_client_infos(&mut wifi).expect("Failed to get AP client infos");
                let client_count = clients.len();

                if client_count > 0 {
                    log::info!("Listing clients...");
                }

                let mut client_list = svc.lock().await.clients;
                let mut any_change = false;
                for client_type in CLIENT_TYPES {
                    let existing_client = client_list.get_client_for_type(client_type);
                    let found_client = match clients.iter().find(|client| match client {
                        Some(client) => client.mac == get_mac_for_client_type(client_type).unwrap(),
                        None => false,
                    }) {
                        Some(client) => *client,
                        None => None,
                    };

                    match (existing_client, found_client) {
                        (Some(existing_client), Some(found_client)) => {
                            if existing_client.ip != found_client.ip {
                                log::info!(
                                    "Client {:?} has changed IP from {:?} to {:?}",
                                    client_type,
                                    existing_client.ip,
                                    found_client.ip
                                );

                                client_list.set_client_for_type(client_type, found_client.clone());
                                any_change = true;
                            } else {
                                log::debug!("Client {:?} has not changed", client_type);
                            }
                        }
                        (None, Some(found_client)) => {
                            log::info!("Client {:?} has connected", client_type);
                            client_list.set_client_for_type(client_type, found_client.clone());
                            any_change = true;
                        }
                        (Some(_), None) => {
                            log::info!("Client {:?} has disconnected", client_type);
                            client_list.remove_client_for_type(client_type);
                            any_change = true;
                        }
                        _ => {}
                    }
                }

                if any_change {
                    log::info!("Notifying of changed client list...");
                    sender.send(client_list.clone()).unwrap();
                }

                if client_count == 0 {
                    log::info!("No clients connected");
                }
            }
        });

        new_client_identifier_task.await.unwrap();
    }
}
