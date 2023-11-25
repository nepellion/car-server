use esp_idf_hal::gpio::{Gpio10, Gpio11, Output, PinDriver};
use esp_idf_sys::EspError;

pub struct OutputPins {
    closing_pin: PinDriver<'static, Gpio10, Output>,
    opening_pin: PinDriver<'static, Gpio11, Output>,
}

impl OutputPins {
    pub fn set_open_high(&mut self) -> Result<(), EspError> {
        self.opening_pin.set_high()
    }

    pub fn set_open_low(&mut self) -> Result<(), EspError> {
        self.opening_pin.set_low()
    }

    pub fn set_close_high(&mut self) -> Result<(), EspError> {
        self.closing_pin.set_high()
    }

    pub fn set_close_low(&mut self) -> Result<(), EspError> {
        self.closing_pin.set_low()
    }
}

pub fn prepare_output_pins(closing_pin: Gpio10, opening_pin: Gpio11) -> Result<OutputPins, EspError> {
    Ok(OutputPins {
        closing_pin: PinDriver::output(closing_pin)?,
        opening_pin: PinDriver::output(opening_pin)?,
    })
}
