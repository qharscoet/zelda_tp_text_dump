pub fn get_u32_be(data: &[u8], idx: usize) -> u32 {
    u32::from_be_bytes(data[idx..idx + 4].try_into().unwrap())
}

pub fn get_u16_be(data: &[u8], idx: usize) -> u16 {
    u16::from_be_bytes(data[idx..idx + 2].try_into().unwrap())
}

pub fn get_u16_le(data: &[u8], idx: usize) -> u16 {
    u16::from_le_bytes(data[idx..idx + 2].try_into().unwrap())
}


pub fn get_u32_le(data: &[u8], idx: usize) -> u32 {
    u32::from_le_bytes(data[idx..idx + 4].try_into().unwrap())
}

pub fn unpack_u16(v:u16) -> (u8,u8) {
    (((v & 0xFF00) >> 8) as u8, (v & 0x00FF) as u8)
}
