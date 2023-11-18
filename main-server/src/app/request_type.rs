
use std::fmt::Display;

pub enum RequestType {
  KeepClosingWindow,
  KeepOpeningWindow,
  OpenWindow,
  CloseWindow
}

impl Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
          Self::KeepClosingWindow => "KeepClosingWindow",
          Self::KeepOpeningWindow => "KeepOpeningWindow",
          Self::OpenWindow => "OpenWindow",
          Self::CloseWindow => "CloseWindow"
        })
    }
}