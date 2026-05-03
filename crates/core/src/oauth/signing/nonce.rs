use rand::{rngs::OsRng, RngCore};

pub fn nonce_hex(bytes: usize) -> String {
    let mut data = vec![0_u8; bytes];
    OsRng.fill_bytes(&mut data);
    hex::encode(data)
}

pub fn timestamp_seconds() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is before UNIX_EPOCH")
        .as_secs();
    secs.to_string()
}
