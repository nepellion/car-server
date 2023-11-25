use shared_lib::{mac, wifi::mac::MacAddress};

use super::types::ClientType;

const RIGHT_DOOR_MAC: MacAddress = mac!("40:4c:ca:43:8a:64");

pub fn get_mac_for_client_type(client_type: ClientType) -> Option<MacAddress> {
    match client_type {
        ClientType::LeftDoor => None,
        ClientType::RightDoor => Some(RIGHT_DOOR_MAC),
        _ => None,
    }
}

pub fn get_client_type_for_mac(mac: MacAddress) -> Option<ClientType> {
    match mac {
        RIGHT_DOOR_MAC => Some(ClientType::RightDoor),
        _ => None,
    }
}
