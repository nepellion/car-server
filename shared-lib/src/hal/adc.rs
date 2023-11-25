use esp_idf_hal::{
    adc::{AdcChannelDriver, attenuation},
    gpio::ADCPin,
};

pub fn create_adc_pin_driver_atten_db11<TADC, TPin>(
    pin: TPin,
) -> AdcChannelDriver<'static, { attenuation::DB_11 }, TPin>
where
    TPin: ADCPin<Adc = TADC>,
{
    match AdcChannelDriver::new(pin) {
        Ok(pin) => pin,
        Err(e) => {
            log::error!("Couldn't initialize power window close pin: {:?}", e);
            panic!("Couldn't initialize power window close pin: {:?}", e);
        }
    }
}

pub fn create_adc_pin_driver_atten_none<TADC, TPin>(
    pin: TPin,
) -> AdcChannelDriver<'static, { attenuation::NONE }, TPin>
where
    TPin: ADCPin<Adc = TADC>,
{
    match AdcChannelDriver::new(pin) {
        Ok(pin) => pin,
        Err(e) => {
            log::error!("Couldn't initialize power window close pin: {:?}", e);
            panic!("Couldn't initialize power window close pin: {:?}", e);
        }
    }
}
