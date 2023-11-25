use shared_lib::wifi::ext::ApClientInfo;

use super::types::ClientType;

#[derive(Debug, Clone, Copy, Default)]
pub struct ClientsList {
  pub left_door: Option<ApClientInfo>,
  pub right_door: Option<ApClientInfo>,
}

impl ClientsList {
  pub fn get_client_for_type(self, client_type: ClientType) -> Option<ApClientInfo> {
    match client_type {
      ClientType::LeftDoor => self.left_door,
      ClientType::RightDoor => self.right_door,
      _ => None,
    }
  }

  pub fn set_client_for_type(&mut self, client_type: ClientType, client: ApClientInfo) {
    match client_type {
      ClientType::LeftDoor => self.left_door = Some(client),
      ClientType::RightDoor => self.right_door = Some(client),
      _ => (),
    }
  }

  pub fn remove_client_for_type(&mut self, client_type: ClientType) {
    match client_type {
      ClientType::LeftDoor => self.left_door = None,
      ClientType::RightDoor => self.right_door = None,
      _ => (),
    }
  }
}