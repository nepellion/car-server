use esp_idf_hal::{
    adc::ADC1,
    gpio::{Gpio2, Gpio3, Gpio4, Gpio5},
};

mod power_window_input;
pub mod power_window_controls_driver;

pub type DefaultPowerWindowPeripherals = power_window_input::RequiredPeripherals<ADC1, Gpio2, Gpio3, Gpio4, Gpio5>;

pub type DefaultRightRequiredButtonPins = power_window_input::RequiredButtonPins<Gpio2, Gpio3>;
pub type DefaultLeftRequiredButtonPins = power_window_input::RequiredButtonPins<Gpio4, Gpio5>;
pub type DefaultPowerWindowPins = power_window_input::PowerWindowPins<ADC1, Gpio2, Gpio3, Gpio4, Gpio5>;
