use std::net::Ipv4Addr;

use esp_idf_svc::{
    handle::RawHandle,
    wifi::{AsyncWifi, EspWifi},
};
use esp_idf_sys::{
    esp, esp_netif_dhcps_get_clients_by_mac, esp_netif_pair_mac_ip_t,
    esp_wifi_ap_get_sta_list, wifi_sta_list_t, EspError,
};

use super::mac::MacAddress;

#[derive(Debug, Clone, Copy)]
pub struct ApClientInfo {
    pub mac: MacAddress,
    pub ip: Ipv4Addr,
}

pub fn get_sta_mac_address(
    wifi: &mut AsyncWifi<EspWifi<'static>>,
) -> Result<MacAddress, EspError> {
    match wifi.wifi().driver().get_mac(esp_idf_svc::wifi::WifiDeviceId::Sta) {
        Ok(mac) => Ok(MacAddress::new(mac)),
        Err(e) => Err(e),
    }
}

pub fn get_ap_mac_address(
    wifi: &mut AsyncWifi<EspWifi<'static>>,
) -> Result<MacAddress, EspError> {
    match wifi.wifi().driver().get_mac(esp_idf_svc::wifi::WifiDeviceId::Ap) {
        Ok(mac) => Ok(MacAddress::new(mac)),
        Err(e) => Err(e),
    }
}

pub fn get_ap_client_infos(
    wifi: &mut AsyncWifi<EspWifi<'static>>,
) -> anyhow::Result<[Option<ApClientInfo>; 10]> {
    let mut ap_info_raw = wifi_sta_list_t::default();

    esp!(unsafe { esp_wifi_ap_get_sta_list(&mut ap_info_raw) })?;

    let ap_netif = wifi.wifi().ap_netif();
    let esp_netif = ap_netif.handle();

    let mut result: [Option<ApClientInfo>; 10] = [None; 10];

    for (i, info) in ap_info_raw.sta.iter().enumerate() {
        let mut esp_netif_mac_pair = esp_netif_pair_mac_ip_t {
            mac: info.mac,
            ..Default::default()
        };

        esp!(unsafe { esp_netif_dhcps_get_clients_by_mac(esp_netif, 1, &mut esp_netif_mac_pair) })?;

        if esp_netif_mac_pair.ip.addr != 0 {
            result[i] = Some(ApClientInfo {
                mac: MacAddress::new(info.mac),
                ip: Ipv4Addr::from(esp_netif_mac_pair.ip.addr.to_be()),
            });
        }
    }

    Ok(result)
}
