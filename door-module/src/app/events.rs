#[derive(Debug, Clone)]
pub enum ServerRequestType {
    Stop = 0b0001,
    Open = 0b0010,
    Close = 0b0110,
    OpenFully = 0b1010,
    CloseFully = 0b1110,

    ConfigureCurrentThresholds = 0x10
}

#[derive(Debug, Clone)]
pub struct ServerRequest {
    pub request_type: ServerRequestType,
    pub request_data: [u8; 8]
}
