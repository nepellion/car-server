/// Creates a MacAddress struct from raw string

const fn next_hex_char(string: &[u8], mut pos: usize) -> Option<(u8, usize)> {
    while pos < string.len() {
        let raw_val = string[pos];
        pos += 1;
        let val = match raw_val {
            b'0'..=b'9' => raw_val - 48,
            b'A'..=b'F' => raw_val - 55,
            b'a'..=b'f' => raw_val - 87,
            b':' => continue,
            0..=127 => panic!("Encountered invalid ASCII character"),
            _ => panic!("Encountered non-ASCII character"),
        };
        return Some((val, pos));
    }
    None
}

const fn next_byte(string: &[u8], pos: usize) -> Option<(u8, usize)> {
    let (half1, pos) = match next_hex_char(string, pos) {
        Some(v) => v,
        None => return None,
    };
    let (half2, pos) = match next_hex_char(string, pos) {
        Some(v) => v,
        None => panic!("Odd number of hex characters"),
    };
    Some(((half1 << 4) + half2, pos))
}

/// Compute length of a byte array which will be decoded from the strings.
///
/// This function is an implementation detail and SHOULD NOT be called directly!
#[doc(hidden)]
pub const fn len(strings: &[&[u8]]) -> usize {
    let mut i = 0;
    let mut len = 0;
    while i < strings.len() {
        let mut pos = 0;
        while let Some((_, new_pos)) = next_byte(strings[i], pos) {
            len += 1;
            pos = new_pos;
        }
        i += 1;
    }
    len
}

/// Decode hex strings into a byte array of pre-computed length.
///
/// This function is an implementation detail and SHOULD NOT be called directly!
#[doc(hidden)]
pub const fn decode<const LEN: usize>(strings: &[&[u8]]) -> [u8; LEN] {
    let mut i = 0;
    let mut buf = [0u8; LEN];
    let mut buf_pos = 0;
    while i < strings.len() {
        let mut pos = 0;
        while let Some((byte, new_pos)) = next_byte(strings[i], pos) {
            buf[buf_pos] = byte;
            buf_pos += 1;
            pos = new_pos;
        }
        i += 1;
    }
    if LEN != buf_pos {
        panic!("Length mismatch. Please report this bug.");
    }
    buf
}

#[macro_export]
macro_rules! mac {
    ($($s:literal)*) => {{
        const STRINGS: &[&'static [u8]] = &[$($s.as_bytes(),)*];
        const LEN: usize = $crate::wifi::mac::len(STRINGS);
        const RES: [u8; LEN] = $crate::wifi::mac::decode(STRINGS);

        MacAddress::new(RES)
    }};
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MacAddress {
    pub raw: [u8; 6],
}

impl MacAddress {
    pub const fn new(mac: [u8; 6]) -> Self {
        Self { raw: mac }
    }
}

impl std::fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:x}:{:x}:{:x}:{:x}:{:x}:{:x}]",
            self.raw[0], self.raw[1], self.raw[2], self.raw[3], self.raw[4], self.raw[5]
        )
    }
}
