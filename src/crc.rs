pub(crate) fn crc(data: &[u8]) -> u8 {
    let mut crc: u8 = 0xFF;
    for byte in data.iter().copied() {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 == 0 {
                crc <<= 1;
            } else {
                crc = (crc << 1) ^ 0x31u8;
            }
        }
    }
    return crc;
}

#[cfg(test)]
mod tests {
    use super::crc;

    #[test]
    fn example() {
        assert_eq!(crc(&[0xbe, 0xef]), 0x92);
    }
}
