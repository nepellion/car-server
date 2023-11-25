use std::sync::Arc;

use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::EspHttpConnection;
use shared_lib::{dto::pw_config::{PowerWindowsConfig, Serialize}, http::endpoints};
use tokio::{
    join,
    sync::{broadcast, Mutex},
};

use crate::clients::{list::ClientsList, types::ClientType};

#[derive(Debug, Clone)]
pub struct RestClientSvc {
    clients: ClientsList,
}

impl RestClientSvc {
    pub fn new() -> RestClientSvc {
        RestClientSvc {
            clients: Default::default(),
        }
    }

    pub async fn run_loop(
        mut clients_receiver: broadcast::Receiver<ClientsList>,
        mut pw_cfg_receiver: broadcast::Receiver<PowerWindowsConfig>,
        mut http_receiver: broadcast::Receiver<(ClientType, &'static str, [u8; 8])>,
        svc_src: Arc<Mutex<Self>>,
    ) {
        let svc = svc_src.clone();
        let clients_listener_task = tokio::spawn(async move {
            loop {
                let new_client_list = match clients_receiver.recv().await {
                    Ok(new_client_list) => new_client_list,
                    Err(err) => match err {
                        broadcast::error::RecvError::Closed => panic!("Event channel closed!"),
                        broadcast::error::RecvError::Lagged(count) => {
                            log::warn!("Event channel lagged by {} events, skipping...", count);
                            continue;
                        }
                    },
                };

                let mut svc = svc.lock().await;
                svc.clients = new_client_list;
            }
        });

        let svc = svc_src.clone();
        let pw_cfg_listener_task = tokio::spawn(async move {
            loop {
                let pw_cfg = match pw_cfg_receiver.recv().await {
                    Ok(request) => request,
                    Err(err) => match err {
                        broadcast::error::RecvError::Closed => {
                            log::error!("HTTP channel closed!");
                            break;
                        }
                        broadcast::error::RecvError::Lagged(count) => {
                            log::warn!("HTTP channel lagged by {} events, skipping...", count);
                            continue;
                        }
                    },
                };

                let svc = svc.lock().await;

                let pw_cfg_raw = pw_cfg.serialize();
                Self::call_for_client(
                    svc.clients,
                    ClientType::LeftDoor,
                    endpoints::CONFIGURE_WINDOWS_CURRENT_THRESHOLDS_PATH,
                    pw_cfg_raw,
                )
                .unwrap();
                
                Self::call_for_client(
                    svc.clients,
                    ClientType::RightDoor,
                    endpoints::CONFIGURE_WINDOWS_CURRENT_THRESHOLDS_PATH,
                    pw_cfg_raw,
                )
                .unwrap();
            }
        });

        let svc = svc_src.clone();
        let http_request_handling_task = tokio::spawn(async move {
            loop {
                let (client_type, endpoint, buffer) = match http_receiver.recv().await {
                    Ok(request) => request,
                    Err(err) => match err {
                        broadcast::error::RecvError::Closed => {
                            log::error!("HTTP channel closed!");
                            break;
                        }
                        broadcast::error::RecvError::Lagged(count) => {
                            log::warn!("HTTP channel lagged by {} events, skipping...", count);
                            continue;
                        }
                    },
                };

                let svc = svc.lock().await;
                Self::call_for_client(svc.clients, client_type, endpoint, buffer).unwrap();
            }
        });

        join!(
            clients_listener_task,
            pw_cfg_listener_task,
            http_request_handling_task
        );
    }

    fn get_client() -> anyhow::Result<Client<EspHttpConnection>> {
        Ok(Client::wrap(EspHttpConnection::new(&Default::default())?))
    }

    fn get_url(
        clients: ClientsList,
        client_type: ClientType,
        endpoint: &'static str,
    ) -> Option<String> {
        let ap_client_info_mutex = match client_type {
            ClientType::LeftDoor => clients.left_door,
            ClientType::RightDoor => clients.right_door,
        };

        return match ap_client_info_mutex {
            Some(client_info) => Some(format!("http://{}/{}", client_info.ip, endpoint)),
            None => return None,
        };
    }

    fn call_for_client(
        clients: ClientsList,
        client_type: ClientType,
        endpoint: &'static str,
        buffer: [u8; 8],
    ) -> anyhow::Result<()> {
        let endpoint_url = match Self::get_url(clients, client_type, endpoint) {
            Some(url) => url,
            None => return Err(anyhow::anyhow!("Couldn't get client URL")),
        };

        match Self::get_client().unwrap().post(&endpoint_url, &[]) {
            Ok(mut req) => {
                req.write(&buffer).unwrap();
                req.submit().unwrap();
            }
            Err(err) => {
                log::error!("Couldn't send request to client: {:?}", err);
            }
        };

        Ok(())
    }
}
