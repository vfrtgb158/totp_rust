use std::time::SystemTime;
use base32::Alphabet;
use ring::hmac;

pub fn generate_totp(secret_key: &str) -> Option<String> {
    totp(&secret_key, 30, 6)
}

fn totp(secret_key: &str, period: u64, digits: usize) -> Option<String> {
    let decoded_secret = base32::decode(Alphabet::Rfc4648 { padding: false }, secret_key)?;
    let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, &decoded_secret);
    let ts =SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let current_time = ts / period;
    let message = current_time.to_be_bytes();
    let tag = hmac::sign(&key, &message);
    let hash = tag.as_ref();

    let offset = hash[hash.len() - 1] & 0xf;
    let code = ((hash[offset as usize] as u32 & 0x7f) << 24 |
        (hash[offset as usize + 1] as u32 & 0xff) << 16 |
        (hash[offset as usize + 2] as u32 & 0xff) << 8 |
        (hash[offset as usize + 3] as u32 & 0xff)) % 10_u32.pow(digits as u32);

    Some(format!("{:0>width$}", code, width = digits))
}