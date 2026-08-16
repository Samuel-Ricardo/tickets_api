use crate::crypt::Result;
use rand::RngCore;

pub async fn gen_random() -> Result<()> {
    let mut key = [0u8; 64]; // INFO: 512 bits => 64 bytes
    rand::thread_rng().fill_bytes(&mut key);

    println!("key genereted for HMAC: {key:?}");

    let b64u = base64_url::encode(&key);
    println!("Key b64u encoded: {b64u}");

    Ok(())
}
