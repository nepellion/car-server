use std::net::Ipv4Addr;

use esp_idf_svc::{
    handle::RawHandle,
    wifi::{AsyncWifi, EspWifi},
};
use esp_idf_sys::{
    esp, esp_netif_dhcps_get_clients_by_mac, esp_netif_pair_mac_ip_t,
    esp_wifi_ap_get_sta_list, wifi_sta_list_t,
};

pub struct ApClientInfo {
    pub mac: [u8; 6],
    pub ip: Ipv4Addr,
}

pub fn get_ap_client_infos(
    wifi: &mut AsyncWifi<EspWifi<'static>>,
) -> anyhow::Result<Vec<ApClientInfo>> {
    let mut ap_info_raw = wifi_sta_list_t::default();

    esp!(unsafe { esp_wifi_ap_get_sta_list(&mut ap_info_raw) })?;

    let ap_netif = wifi.wifi().ap_netif();
    let esp_netif = ap_netif.handle();

    let mut vec: Vec<ApClientInfo> = Vec::<ApClientInfo>::new();

    for info in ap_info_raw.sta {
        let mut esp_netif_mac_pair = esp_netif_pair_mac_ip_t {
            mac: info.mac,
            ..Default::default()
        };

        esp!(unsafe { esp_netif_dhcps_get_clients_by_mac(esp_netif, 1, &mut esp_netif_mac_pair) })?;

        if esp_netif_mac_pair.ip.addr != 0 {
            vec.push(ApClientInfo {
                mac: info.mac,
                ip: Ipv4Addr::from(esp_netif_mac_pair.ip.addr.to_be()),
            });
        }
    }

    Ok(vec)
}
