use embedded_svc::http::Method;
use esp_idf_svc::http::server::EspHttpServer;
use std::sync::{Arc, Mutex};

use crate::io::input::InputPins;

const STACK_SIZE: usize = 10240;

pub fn create_http_server<'a>(
    input_pins: Arc<Mutex<InputPins>>,
) -> EspHttpServer<'a> {
    let input_pins = input_pins.clone();

    log::info!("Starting HTTP server on...");

    let mut http_server = EspHttpServer::new(&esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    }).expect("Couldn't create HTTP server");

    http_server.fn_handler("/", Method::Get, move |req| {
        let current = input_pins.lock().unwrap().read_closing_current().unwrap();
        
        req.into_ok_response()?.write(format!("Current: {}mA", current).as_bytes())?;
        
        Ok(())
    }).expect("Couldn't register handler");

    log::info!("Starting HTTP server started");

    return http_server;
}