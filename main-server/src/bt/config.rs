use esp32_nimble::{utilities::BleUuid, uuid128};

pub const DEBUG_NOTIFYING_UUID: BleUuid = uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93");
pub const PW_CFG_UUID: BleUuid = uuid128!("82abaa9d-850d-46a1-87a6-88d4facf293b");