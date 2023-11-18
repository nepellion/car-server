use std::fmt::Display;

use super::request_type::RequestType;

pub struct AppState {
  pub last_request: RequestType,
  pub is_current: bool,
  pub last_debug: u128,
}

impl Display for AppState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "(last_request: {}, is_current: {})", self.last_request, self.is_current)
  }
}