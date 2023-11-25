use esp_idf_hal::{
    adc::ADC1,
    gpio::{Gpio10, Gpio11, Gpio2, Gpio3}, delay::FreeRtos,
};
use esp_idf_sys::EspError;

use super::{
    input::{prepare_input_pins, InputPins},
    output::{prepare_output_pins, OutputPins},
};

pub enum WindowDriverState {
    INTERRUPTED = 0,

    OPENING = 0b01,
    CLOSING = 0b10,
}

pub struct PowerWindowDriverPins {
    pub adc: ADC1,
    pub window_closing_sense_pin: Gpio2,
    pub window_opening_sense_pin: Gpio3,
    pub window_closing_pin: Gpio10,
    pub window_opening_pin: Gpio11,
}

pub struct PowerWindowDriver {
    input: InputPins,
    output: OutputPins,

    pub state: WindowDriverState,
}

pub struct WindowCurrentState {
    pub closing_current: u16,
    pub opening_current: u16,
}

impl PowerWindowDriver {
    pub fn new(pins: PowerWindowDriverPins) -> Result<PowerWindowDriver, EspError> {
        Ok(PowerWindowDriver {
            input: prepare_input_pins(
                pins.adc,
                pins.window_closing_sense_pin,
                pins.window_opening_sense_pin,
            )?,
            output: prepare_output_pins(pins.window_closing_pin, pins.window_opening_pin)?,
            state: WindowDriverState::INTERRUPTED,
        })
    }

    /// Reads the current in Amps
    pub fn read_current(&mut self) -> Result<WindowCurrentState, EspError> {
        let closing_millivolts = self.input.read_closing_voltage_drop()?;
        let opening_millivolts = self.input.read_opening_voltage_drop()?;

        // Should be 80mV/A
        let closing_current = closing_millivolts * 80;
        let opening_current = opening_millivolts * 80;

        Ok(WindowCurrentState {
            closing_current,
            opening_current
        })
    }

    pub fn start_opening(&mut self) -> Result<(), EspError> {
        if matches!(self.state, WindowDriverState::OPENING) {
            return Ok(());
        }

        log::info!("Setting relays to opening mode...");

        self.output.set_close_low()?;
        FreeRtos::delay_ms(50);
        self.output.set_open_high()?;

        self.state = WindowDriverState::OPENING;

        Ok(())
    }

    pub fn start_closing(&mut self) -> Result<(), EspError> {
        if matches!(self.state, WindowDriverState::CLOSING) {
            return Ok(());
        }

        log::info!("Setting relays to closing mode...");

        self.output.set_open_low()?;
        FreeRtos::delay_ms(50);
        self.output.set_close_high()?;

        self.state = WindowDriverState::CLOSING;

        Ok(())
    }


    pub fn interrupt(&mut self) -> Result<(), EspError> {
        log::info!("Setting relays into stopped mode...");

        self.output.set_close_low()?;
        self.output.set_open_low()?;

        self.state = WindowDriverState::INTERRUPTED;

        Ok(())
    }
}
