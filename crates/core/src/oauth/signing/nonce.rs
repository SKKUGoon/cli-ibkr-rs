use rand::rngs::OsRng;
use rand::TryRngCore;

pub fn nonce_hex(bytes: usize) -> String {
    let mut data = vec![0_u8; bytes];
    // rand 0.9 surfaces `OsRng` as fallible (`TryRngCore`). In practice the
    // OS CSPRNG only fails if `getrandom` cannot reach the kernel, which is
    // not recoverable for OAuth nonce generation — panic loudly.
    OsRng
        .try_fill_bytes(&mut data)
        .expect("OS CSPRNG unavailable");
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
