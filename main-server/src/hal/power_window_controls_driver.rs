use esp_idf_sys::EspError;

use super::{
    power_window_input::prepare_input_pins, DefaultPowerWindowPeripherals, DefaultPowerWindowPins,
};

pub struct PowerWindowDriver {
    input: DefaultPowerWindowPins,
}

const VOLTAGE_CONTINOUS_THRESHOLD: u16 = 300;
const VOLTAGE_FULL_THRESHOLD: u16 = 700;

pub enum PowerWindowButtonState {
    None = 0,
    OpenContinuous = 0b001,
    CloseContinuous = 0b010,
    OpenFully = 0b101,
    CloseFully = 0b110,
}

impl PowerWindowDriver {
    pub fn new(pins: DefaultPowerWindowPeripherals) -> Result<PowerWindowDriver, EspError> {
        Ok(PowerWindowDriver {
            input: prepare_input_pins(pins)?,
        })
    }

    /// Reads the current state of left button
    pub fn read_left_button_state(&mut self) -> Result<PowerWindowButtonState, EspError> {
        let open = self.input.read_pw_l_open()?;
        let close = self.input.read_pw_l_close()?;

        Ok(Self::get_state_for_voltages(open, close))
    }

    /// Reads the current state of right button
    pub fn read_right_button_state(&mut self) -> Result<PowerWindowButtonState, EspError> {
      let open = self.input.read_pw_r_open()?;
      let close = self.input.read_pw_r_close()?;

      Ok(Self::get_state_for_voltages(open, close))
    }

    fn get_state_for_voltages(open: u16, close: u16) -> PowerWindowButtonState {
        if open > VOLTAGE_CONTINOUS_THRESHOLD && close > VOLTAGE_CONTINOUS_THRESHOLD {
            log::warn!("Both buttons are pressed at the same time!");
            return PowerWindowButtonState::None;
        }

        if open > VOLTAGE_CONTINOUS_THRESHOLD {
            if open > VOLTAGE_FULL_THRESHOLD {
                return PowerWindowButtonState::OpenFully;
            } else {
                return PowerWindowButtonState::OpenContinuous;
            }
        }

        if close > VOLTAGE_CONTINOUS_THRESHOLD {
            if close > VOLTAGE_FULL_THRESHOLD {
                return PowerWindowButtonState::CloseFully;
            } else {
                return PowerWindowButtonState::CloseContinuous;
            }
        }

        PowerWindowButtonState::None
    }
}
