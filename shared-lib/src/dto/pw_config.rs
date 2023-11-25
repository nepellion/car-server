pub trait Serialize {
    fn serialize(&self) -> [u8; 8];
}

pub trait Deserialize {
    fn deserialize(buffer: [u8; 8]) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub struct PowerWindowsConfig {
    pub opening_current_interrupt_threshold_amps: u16,
    pub closing_current_interrupt_threshold_amps: u16,
    pub handle_time_threshold_millis: u16,
}

impl Serialize for PowerWindowsConfig {
    fn serialize(&self) -> [u8; 8] {
        let mut buffer: [u8; 8] = [0; 8];

        buffer[0..2].copy_from_slice(&self.opening_current_interrupt_threshold_amps.to_be_bytes());
        buffer[2..4].copy_from_slice(&self.closing_current_interrupt_threshold_amps.to_be_bytes());
        buffer[4..6].copy_from_slice(&self.handle_time_threshold_millis.to_be_bytes());

        buffer
    }
}

impl Deserialize for PowerWindowsConfig {
    fn deserialize(buffer: [u8; 8]) -> Self {
        let opening_current_interrupt_threshold_amps = u16::from_be_bytes([buffer[0], buffer[1]]);
        let closing_current_interrupt_threshold_amps = u16::from_be_bytes([buffer[2], buffer[3]]);
        let handle_time_threshold_millis = u16::from_be_bytes([buffer[4], buffer[5]]);

        PowerWindowsConfig {
            opening_current_interrupt_threshold_amps,
            closing_current_interrupt_threshold_amps,
            handle_time_threshold_millis,
        }
    }
}
