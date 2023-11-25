use esp_idf_hal::{
    adc::{
        attenuation::{self},
        config::Config,
        Adc, AdcChannelDriver, AdcDriver,
    },
    gpio::ADCPin,
};
use esp_idf_sys::EspError;
use shared_lib::hal::adc::create_adc_pin_driver_atten_db11;

use super::{DefaultPowerWindowPeripherals, DefaultPowerWindowPins};

pub struct RequiredButtonPins<TOpenPin, TClosePin>
where
    TOpenPin: ADCPin,
    TClosePin: ADCPin,
{
    pub open_pin: TOpenPin,
    pub close_pin: TClosePin,
}

pub struct RequiredPeripherals<TADC, TROpenPin, TRClosePin, TLOpenPin, TLClosePin>
where
    TLOpenPin: ADCPin,
    TLClosePin: ADCPin,
    TROpenPin: ADCPin,
    TRClosePin: ADCPin,
{
    pub adc: TADC,
    pub left_pins: RequiredButtonPins<TLOpenPin, TLClosePin>,
    pub right_pins: RequiredButtonPins<TROpenPin, TRClosePin>,
}

pub struct PowerWindowButtonPins<TOpenPin, TClosePin>
where
    TOpenPin: ADCPin,
    TClosePin: ADCPin,
{
    open_pin: AdcChannelDriver<'static, { attenuation::DB_11 }, TOpenPin>,
    close_pin: AdcChannelDriver<'static, { attenuation::DB_11 }, TClosePin>,
}

pub struct PowerWindowPins<TADC, TROpenPin, TRClosePin, TLOpenPin, TLClosePin>
where
    TADC: Adc + 'static,
    TLOpenPin: ADCPin<Adc = TADC>,
    TLClosePin: ADCPin<Adc = TADC>,
    TROpenPin: ADCPin<Adc = TADC>,
    TRClosePin: ADCPin<Adc = TADC>,
{
    adc_driver: AdcDriver<'static, TADC>,
    left_pw: PowerWindowButtonPins<TLOpenPin, TLClosePin>,
    right_pw: PowerWindowButtonPins<TROpenPin, TRClosePin>,
}

impl<TADC, TROpenPin, TRClosePin, TLOpenPin, TLClosePin>
    PowerWindowPins<TADC, TROpenPin, TRClosePin, TLOpenPin, TLClosePin>
where
    TADC: Adc + 'static,
    TLOpenPin: ADCPin<Adc = TADC>,
    TLClosePin: ADCPin<Adc = TADC>,
    TROpenPin: ADCPin<Adc = TADC>,
    TRClosePin: ADCPin<Adc = TADC>,
{
    /// Reads the voltage of left open button output
    pub fn read_pw_l_open(&mut self) -> Result<u16, EspError> {
        let reading = self.adc_driver.read(&mut self.left_pw.open_pin)?;

        Ok(reading)
    }

    /// Reads the voltage of left close button output
    pub fn read_pw_l_close(&mut self) -> Result<u16, EspError> {
        let reading = self.adc_driver.read(&mut self.left_pw.close_pin)?;

        Ok(reading)
    }

    /// Reads the voltage of right open button output
    pub fn read_pw_r_open(&mut self) -> Result<u16, EspError> {
        let reading = self.adc_driver.read(&mut self.right_pw.open_pin)?;

        Ok(reading)
    }

    /// Reads the voltage of right close button output
    pub fn read_pw_r_close(&mut self) -> Result<u16, EspError> {
        let reading = self.adc_driver.read(&mut self.right_pw.close_pin)?;

        Ok(reading)
    }
}

pub fn prepare_input_pins(
    peripherals: DefaultPowerWindowPeripherals,
) -> Result<DefaultPowerWindowPins, EspError> {
    let adc_driver = match AdcDriver::new(peripherals.adc, &Config::new().calibration(true)) {
        Ok(adc) => adc,
        Err(e) => {
            log::error!("Couldn't initialize ADC driver: {:?}", e);
            panic!("Couldn't initialize ADC driver: {:?}", e);
        }
    };

    Ok(PowerWindowPins {
        adc_driver,
        left_pw: PowerWindowButtonPins {
            open_pin: create_adc_pin_driver_atten_db11(peripherals.left_pins.open_pin),
            close_pin: create_adc_pin_driver_atten_db11(peripherals.left_pins.close_pin),
        },
        right_pw: PowerWindowButtonPins {
            open_pin: create_adc_pin_driver_atten_db11(peripherals.right_pins.open_pin),
            close_pin: create_adc_pin_driver_atten_db11(peripherals.right_pins.close_pin),
        },
    })
}
