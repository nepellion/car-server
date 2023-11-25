use std::{sync::Arc, time::Duration};

use esp_idf_svc::systime::EspSystemTime;
use esp_idf_sys::EspError;
use shared_lib::dto::pw_config::{PowerWindowsConfig, Deserialize};
use tokio::{
    join,
    sync::{broadcast, Mutex},
};

use crate::{
    app::events::{ServerRequest, ServerRequestType},
    hal::power_window_driver::{PowerWindowDriver, PowerWindowDriverPins},
};

#[derive(Debug, Clone, Copy)]
pub enum State {
    None = 0,
    Stopped = 0b0001,

    OpeningContinuous = 0b0010,
    OpeningFully = 0b0110,
    OpeningInterrupted = 0b0011,
    OpeningFinished = 0b0111,

    ClosingContinuous = 0b1010,
    ClosingFully = 0b1110,
    ClosingInterrupted = 0b1011,
    ClosingFinished = 0b1111,
}

pub struct PowerWindowSvc {
    window_driver: PowerWindowDriver,

    last_handle_time_millis: u128,
    state: State,
    config: PowerWindowsConfig,
}

impl PowerWindowSvc {
    pub fn new(
        pins: PowerWindowDriverPins,
        config: PowerWindowsConfig,
    ) -> Result<PowerWindowSvc, EspError> {
        Ok(PowerWindowSvc {
            window_driver: PowerWindowDriver::new(pins)?,
            last_handle_time_millis: 0,
            state: State::None,
            config: config,
        })
    }

    pub async fn run_loop(
        mut receiver: broadcast::Receiver<ServerRequest>,
        svc: Arc<Mutex<PowerWindowSvc>>,
    ) {
        log::info!("Spawned power window service.");

        let svc_src = svc.clone();

        const ERROR_THRESHOLD: u8 = 3;
        let error_count = Arc::new(Mutex::<u8>::new(0));

        async fn handle_error(error_count: Arc<Mutex<u8>>) {
            let mut error_count = error_count.lock().await;
            *error_count += 1;

            if *error_count > ERROR_THRESHOLD {
                panic!("Too many errors, shutting down!");
            }
        }

        async fn handle_result(result: Result<(), EspError>, error_count: Arc<Mutex<u8>>) {
            match result {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Error: {:?}", err);
                    handle_error(error_count.clone()).await;
                }
            }
        }

        let svc = svc_src.clone();
        let server_listener_task = tokio::spawn(async move {
            loop {
                let server_request = match receiver.recv().await {
                    Ok(event) => event,
                    Err(err) => match err {
                        broadcast::error::RecvError::Closed => panic!("Event channel closed!"),
                        broadcast::error::RecvError::Lagged(count) => {
                            log::warn!("Event channel lagged by {} events, skipping...", count);
                            continue;
                        }
                    },
                };

                match server_request.request_type {
                    ServerRequestType::Open => {
                        let mut svc = svc.lock().await;
                        handle_result(svc.handle_opening(true), error_count.clone()).await;
                    }
                    ServerRequestType::Close => {
                        let mut svc = svc.lock().await;
                        handle_result(svc.handle_closing(true), error_count.clone()).await;
                    }
                    ServerRequestType::OpenFully => {
                        let mut svc = svc.lock().await;
                        handle_result(svc.handle_opening(false), error_count.clone()).await;
                    }
                    ServerRequestType::CloseFully => {
                        let mut svc = svc.lock().await;
                        handle_result(svc.handle_closing(false), error_count.clone()).await;
                    }
                    ServerRequestType::Stop => {
                        let mut svc = svc.lock().await;
                        handle_result(svc.handle_stop(), error_count.clone()).await;
                    }
                    ServerRequestType::ConfigureCurrentThresholds => {
                        let mut svc = svc.lock().await;
                        handle_result(svc.configure(server_request.request_data), error_count.clone()).await;
                    },
                }
            }
        });

        let svc = svc_src.clone();
        let interrupt_handler_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));

            let mut _svc = svc.lock().await;

            let handle_time_threshold: Duration = Duration::from_millis(_svc.config.handle_time_threshold_millis.into());

            drop(_svc);

            loop {
                interval.tick().await;

                let mut svc = svc.lock().await;

                if get_time_as_millis() - svc.last_handle_time_millis >= handle_time_threshold.as_millis() {
                    match svc.state {
                        State::ClosingContinuous => {
                            log::info!("Closing continuous timeout, stopping...");
                            svc.handle_stop().unwrap();
                            continue;
                        }
                        State::OpeningContinuous => {
                            log::info!("Opening continuous timeout, stopping...");
                            svc.handle_stop().unwrap();
                            continue;
                        }
                        _ => {}
                    }
                }

                if crate::DEBUG {
                    let current_state = svc.window_driver.read_current().unwrap();
                    log::info!(
                        "Current: closing_{}mA, opening_{}mA",
                        current_state.closing_current,
                        current_state.opening_current
                    );

                    if get_time_as_millis() - svc.last_handle_time_millis >= 4000 {
                        match svc.state {
                            State::ClosingFully => {
                                log::info!("DEBUG MODE: Closed fully, stopping...");
                                svc.handle_close_interrupt().unwrap();
                                continue;
                            }
                            State::OpeningFully => {
                                log::info!("DEBUG MODE: Opened fully, stopping...");
                                svc.handle_open_interrupt().unwrap();
                                continue;
                            }
                            _ => {}
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }

                let current_state = svc.window_driver.read_current().unwrap();
                log::debug!(
                    "Current: closing_{}mA, opening_{}mA",
                    current_state.closing_current,
                    current_state.opening_current
                );

                if current_state.closing_current
                    > svc.config.closing_current_interrupt_threshold_amps
                {
                    svc.handle_close_interrupt().unwrap();
                }

                if current_state.opening_current
                    > svc.config.opening_current_interrupt_threshold_amps
                {
                    svc.handle_open_interrupt().unwrap();
                }
            }
        });

        let (server_listener_task_result, interrupt_handler_task_result) =
            join!(server_listener_task, interrupt_handler_task);
        server_listener_task_result.unwrap();
        interrupt_handler_task_result.unwrap();
    }

    fn handle_opening(&mut self, continuous: bool) -> Result<(), EspError> {
        self.last_handle_time_millis = get_time_as_millis();

        match self.state {
            State::OpeningContinuous => {
                if continuous {
                    log::debug!("Already opening, timeout slid forward.");
                    return Ok(());
                } else {
                    log::info!(
                        "Already opening, but upgrading from OPENING_CONTINUOUS to OPENING_FULLY."
                    );

                    self.state = State::OpeningFully;
                    return Ok(());
                }
            }
            State::OpeningFinished => {
                log::info!("Tried opening more when already fully open, ignoring...");
                return Ok(());
            }
            State::OpeningFully => {
                log::info!("Tried opening when open fully already running, ignoring...");
                return Ok(());
            }
            State::OpeningInterrupted => {
                log::info!("Tried opening when interrupted, ignoring...");
                return Ok(());
            }
            _ => {
                log::info!("Starting opening...");
                self.state = match continuous {
                    true => State::OpeningContinuous,
                    false => State::OpeningFully,
                };

                self.window_driver.start_opening()?;
            }
        }

        return Ok(());
    }

    fn handle_closing(&mut self, continuous: bool) -> Result<(), EspError> {
        self.last_handle_time_millis = get_time_as_millis();

        match self.state {
            State::ClosingContinuous => {
                if continuous {
                    log::info!("Already closing, timeout slid forward.");
                    return Ok(());
                } else {
                    log::info!(
                        "Already closing, but upgrading from CLOSING_CONTINUOUS to CLOSING_FULLY."
                    );

                    self.state = State::ClosingFully;
                    return Ok(());
                }
            }
            State::ClosingFinished => {
                log::info!("Tried closing more when already fully closed, ignoring...");
                return Ok(());
            }
            State::ClosingFully => {
                log::info!("Tried closing when closed fully already running, ignoring...");
                return Ok(());
            }
            State::ClosingInterrupted => {
                log::info!("Tried closing when interrupted, ignoring...");
                return Ok(());
            }
            _ => {
                log::info!("Starting closing...");
                self.state = match continuous {
                    true => State::ClosingContinuous,
                    false => State::ClosingFully,
                };

                self.window_driver.start_closing()?;
            }
        }

        return Ok(());
    }

    fn handle_close_interrupt(&mut self)-> Result<(), EspError> {
        self.window_driver.interrupt()?;

        match self.state {
            State::ClosingContinuous => {
                log::info!("Interrupted closing.");
                self.state = State::ClosingInterrupted;
            }
            State::ClosingFully => {
                log::debug!("Finished closing.");
                self.state = State::ClosingFinished;
            }
            _ => {
                log::error!("Interrupt requested but not closing!");
                panic!("Interrupt requested but not closing!");
            }
        }

        Ok(())
    }

    fn handle_open_interrupt(&mut self) -> Result<(), EspError> {
        self.window_driver.interrupt()?;

        match self.state {
            State::OpeningContinuous => {
                log::info!("Interrupted opening.");
                self.state = State::OpeningInterrupted;
            }
            State::OpeningFully => {
                log::debug!("Finished opening.");
                self.state = State::OpeningFinished;
            }
            _ => {
                log::error!("Interrupt requested but not opening!");
                panic!("Interrupt requested but not opening!");
            }
        }

        Ok(())
    }

    fn handle_stop(&mut self) -> Result<(), EspError> {
        match self.state {
            State::OpeningFully => {
                log::info!("Opening fully, therefore ignoring soft stop...");
            }
            State::ClosingFully => {
                log::info!("Closing fully, therefore ignoring soft stop...");
            }
            _ => {
                log::debug!("Stopping operation...");
                self.state = State::Stopped;
                self.window_driver.interrupt()?;
            }
        }

        return Ok(());
    }

    fn configure(&mut self, data: [u8; 8]) -> Result<(), EspError> {
        let pw_cfg = PowerWindowsConfig::deserialize(data);

        log::info!("Configuring power window service with:");
        log::info!("Opening current interrupt threshold: {}mA", pw_cfg.opening_current_interrupt_threshold_amps);
        log::info!("Closing current interrupt threshold: {}mA", pw_cfg.closing_current_interrupt_threshold_amps);
        log::info!("Handle time threshold: {}ms", pw_cfg.handle_time_threshold_millis);

        self.config = pw_cfg;

        Ok(())
    }
}

fn get_time_as_millis() -> u128 {
    return EspSystemTime::now(&EspSystemTime {}).as_millis();
}
