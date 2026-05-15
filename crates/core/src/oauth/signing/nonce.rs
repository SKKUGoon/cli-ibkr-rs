use rand::rngs::SysRng;
use rand::TryRng;

pub fn nonce_hex(bytes: usize) -> String {
    let mut data = vec![0_u8; bytes];
    // `SysRng` is fallible (`TryRngCore`). In practice the OS CSPRNG only
    // fails if `getrandom` cannot reach the kernel, which is not recoverable
    // for OAuth nonce generation — panic loudly.
    SysRng
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
