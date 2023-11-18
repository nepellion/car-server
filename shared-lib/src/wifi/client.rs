use anyhow::{Ok, Result};
use embedded_svc::wifi::{Configuration, AuthMethod, ClientConfiguration};

use esp_idf_hal::modem::Modem;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{
    AsyncWifi, 
    EspWifi
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use futures::executor::block_on;
use log::info;

pub fn connect_wifi_sync(modem: Modem, ssid: &'static str, password: &'static str) -> Result<AsyncWifi<EspWifi<'static>>> {
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
        timer_service,
    )?;

    block_on(connect_wifi(&mut wifi, ssid, password))?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi Client DHCP info: {:?}", ip_info);

    Ok(wifi)
}

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>, ssid: &'static str, password: &'static str) -> Result<()> {
    let wifi_configuration = Configuration::Client(get_client_config(ssid, password));

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start().await?;
    info!("Wifi started");

    wifi.connect().await?;
    info!("Wifi connected");

    wifi.wait_netif_up().await?;
    info!("Wifi netif up");

    Ok(())
}

fn get_client_config(ssid: &'static str, password: &'static str) -> ClientConfiguration {
    ClientConfiguration {
        ssid: ssid.into(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.into(),
        channel: Some(1)
    }
}