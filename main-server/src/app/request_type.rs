
use std::fmt::Display;

pub enum RequestType {
  PowerWindowsStop = 0b0001,
  PowerWindowsOpen = 0b0010,
  PowerWindowsClose = 0b0110,
  PowerWindowsOpenFully = 0b1010,
  PowerWindowsCloseFully = 0b1110,
}

impl Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
          Self::PowerWindowsStop => "PowerWindowsStop",
          Self::PowerWindowsOpen => "PowerWindowsOpen",
          Self::PowerWindowsClose => "PowerWindowsClose",
          Self::PowerWindowsOpenFully => "PowerWindowsOpenFully",
          Self::PowerWindowsCloseFully => "PowerWindowsCloseFully",
        })
    }
}