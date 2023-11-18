use anyhow::{Ok, Result};
use embedded_svc::wifi::{AccessPointConfiguration, AuthMethod, Configuration, Protocol};

use esp_idf_hal::modem::Modem;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use futures::executor::block_on;
use log::info;

pub fn create_wifi_ap_sync(
    modem: Modem,
    ssid: &'static str,
    password: &'static str,
) -> Result<AsyncWifi<EspWifi<'static>>> {
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
        timer_service,
    )?;

    block_on(connect_wifi(&mut wifi, ssid, password))?;

    let ip_info = wifi.wifi().ap_netif().get_ip_info()?;

    info!("Wifi Server DHCP info: {:?}", ip_info);

    Ok(wifi)
}

async fn connect_wifi(
    wifi: &mut AsyncWifi<EspWifi<'static>>,
    ssid: &'static str,
    password: &'static str,
) -> Result<()> {
    let wifi_configuration = Configuration::AccessPoint(get_ap_config(ssid, password));

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start().await?;
    info!("Wifi started");

    wifi.wait_netif_up().await?;
    info!("Wifi netif up");

    Ok(())
}

fn get_ap_config(ssid: &'static str, password: &'static str) -> AccessPointConfiguration {
    AccessPointConfiguration {
        ssid: ssid.into(),
        ssid_hidden: true,
        auth_method: AuthMethod::WPA3Personal,
        password: password.into(),
        channel: 2,
        secondary_channel: Some(3),
        max_connections: 10,
        protocols: Protocol::P802D11BGN | Protocol::P802D11BGNLR,
    }
}
