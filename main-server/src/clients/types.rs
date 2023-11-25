#[derive(Debug, Clone, Copy)]
pub enum ClientType {
  RightDoor,
  LeftDoor,
}

pub const CLIENT_TYPES: [ClientType; 2] = [
  ClientType::RightDoor,
  ClientType::LeftDoor,
];