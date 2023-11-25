use embedded_svc::http::Method;
use esp_idf_hal::task::block_on;
use esp_idf_svc::http::server::EspHttpServer;
use shared_lib::http::endpoints;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use crate::app::{
    events::{ServerRequest, ServerRequestType},
    state::AppState,
};

const STACK_SIZE: usize = 10240;

pub fn prepare_http_server<'a>(
    sender: broadcast::Sender<ServerRequest>,
    app_state: Arc<Mutex<AppState>>,
) -> EspHttpServer<'a> {
    log::info!("Spawned HTTP server task.");

    let mut http_server = EspHttpServer::new(&esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    })
    .unwrap();

    http_server
        .fn_handler("/state", Method::Get, move |req| {
            let app_state = app_state.clone();

            block_on(async move {
                let app_state = app_state.lock().await;

                req.into_ok_response()
                    .expect("hey")
                    .write(format!("App state: {}", app_state.random_val).as_bytes())
                    .expect("heya");
            });

            Ok(())
        })
        .unwrap();

    let _sender = sender.clone();
    http_server
        .fn_handler(endpoints::OPEN_WINDOWS_CONTINUOUS_PATH, Method::Post, move |req| {
            _sender.send(ServerRequest {
                request_type: ServerRequestType::Open,
                request_data: Default::default(),
            })?;

            req.into_ok_response()?;

            Ok(())
        })
        .unwrap();

    let _sender = sender.clone();
    http_server
        .fn_handler(endpoints::CLOSE_WINDOWS_CONTINUOUS_PATH, Method::Post, move |req| {
            _sender.send(ServerRequest {
                request_type: ServerRequestType::Close,
                request_data: Default::default(),
            })?;

            req.into_ok_response()?;

            Ok(())
        })
        .unwrap();

    let _sender = sender.clone();
    http_server
        .fn_handler(endpoints::OPEN_WINDOWS_FULLY_PATH, Method::Post, move |req| {
            _sender.send(ServerRequest {
                request_type: ServerRequestType::OpenFully,
                request_data: Default::default(),
            })?;

            req.into_ok_response()?;

            Ok(())
        })
        .unwrap();

    let _sender = sender.clone();
    http_server
        .fn_handler(endpoints::CLOSE_WINDOWS_FULLY_PATH, Method::Post, move |req| {
            _sender.send(ServerRequest {
                request_type: ServerRequestType::CloseFully,
                request_data: Default::default(),
            })?;

            req.into_ok_response()?;

            Ok(())
        })
        .unwrap();

    let _sender = sender.clone();
    http_server
        .fn_handler(endpoints::STOP_WINDOWS_PATH, Method::Post, move |req| {
            _sender.send(ServerRequest {
                request_type: ServerRequestType::Stop,
                request_data: Default::default(),
            })?;

            req.into_ok_response()?;

            Ok(())
        })
        .unwrap();

    let _sender = sender.clone();
    http_server
        .fn_handler(endpoints::CONFIGURE_WINDOWS_CURRENT_THRESHOLDS_PATH, Method::Post, move |mut req| {
            let mut buffer: [u8; 8] = [0; 8];

            req.read(&mut buffer)?;

            _sender.send(ServerRequest {
                request_type: ServerRequestType::ConfigureCurrentThresholds,
                request_data: buffer,
            })?;

            req.into_ok_response()?;

            Ok(())
        })
        .unwrap();

    return http_server;
}
