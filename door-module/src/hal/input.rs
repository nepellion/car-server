use esp_idf_hal::{
    adc::{
      attenuation::{self}, 
      config::Config, 
      AdcChannelDriver, 
      AdcDriver, 
      ADC1
    },
    gpio::{Gpio2, Gpio3},
};
use esp_idf_sys::EspError;

pub struct InputPins {
    adc_driver: AdcDriver<'static, ADC1>,
    closing_current_sense_pin: AdcChannelDriver<'static, { attenuation::NONE }, Gpio2>,
    opening_current_sense_pin: AdcChannelDriver<'static, { attenuation::NONE }, Gpio3>,
}

impl InputPins {
    /// Reads the voltage drop across the closing current sense resistor in mV
    pub fn read_closing_voltage_drop(&mut self) -> Result<u16, EspError> {
        let reading = self.adc_driver.read(&mut self.closing_current_sense_pin)?;

        Ok(reading)
    }

    /// Reads the voltage drop across the opening current sense resistor in mV
    pub fn read_opening_voltage_drop(&mut self) -> Result<u16, EspError> {
        let reading = self.adc_driver.read(&mut self.opening_current_sense_pin)?;

        Ok(reading)
    }
}

pub fn prepare_input_pins(adc: ADC1, closing_sense_pin: Gpio2, opening_sense_pin: Gpio3) -> Result<InputPins, EspError> {
    let adc_driver = match AdcDriver::new(adc, &Config::new().calibration(true)) {
        Ok(adc) => adc,
        Err(e) => {
            log::error!("Couldn't initialize ADC driver: {:?}", e);
            panic!("Couldn't initialize ADC driver: {:?}", e);
        }
    };

    let closing_current_sense_pin: AdcChannelDriver<{ attenuation::NONE }, Gpio2> = match AdcChannelDriver::new(closing_sense_pin) {
        Ok(pin) => pin,
        Err(e) => {
            log::error!("Couldn't initialize current sense pin: {:?}", e);
            panic!("Couldn't initialize current sense pin: {:?}", e);
        }
    };

    let opening_current_sense_pin: AdcChannelDriver<{ attenuation::NONE }, Gpio3> = match AdcChannelDriver::new(opening_sense_pin) {
        Ok(pin) => pin,
        Err(e) => {
            log::error!("Couldn't initialize current sense pin: {:?}", e);
            panic!("Couldn't initialize current sense pin: {:?}", e);
        }
    };

    Ok(InputPins {
        adc_driver,
        closing_current_sense_pin,
        opening_current_sense_pin,
    })
}
