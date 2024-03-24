use hmac::{Hmac, Mac};
use sha2::Sha512;

pub use self::error::{Error, Result};

mod error;
pub mod pwd;

pub struct EncryptContent {
    pub content: String,
    pub salt: String,
}

pub fn encrypt_into_b64url(key: &[u8], enc_content: &EncryptContent) -> Result<String> {
    let EncryptContent { content, salt } = enc_content;

    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;

    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    let hmac_result = hmac_sha512.finalize();
    let result_bytes = hmac_result.into_bytes();

    let result = base64_url::encode(&result_bytes);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    fn test_encrypt_into_b64url() -> Result<()> {
        let mut fx_key = [0u8; 64]; // INFO: 512 bits => 64 bytes
        rand::thread_rng().fill_bytes(&mut fx_key);

        let fx_enc_content = EncryptContent {
            content: "Hello, World!".to_string(),
            salt: "1234567890".to_string(),
        };

        let fx_res = encrypt_into_b64url(&fx_key, &fx_enc_content)?;
        let res = encrypt_into_b64url(&fx_key, &fx_enc_content)?;

        println!("fx_res: {fx_res}");
        println!("res: {res}");

        assert_eq!(fx_res, res);
        Ok(())
    }
}
