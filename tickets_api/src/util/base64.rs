use crate::{Error, Result};

pub fn b64u_encode(content: &str) -> String {
    base64_url::encode(content)
}

pub fn b64u_decode(content: &str) -> Result<String> {
    let decoded_string = base64_url::decode(content)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or(Error::FailtToB643UrlDecode);

    Ok(decoded_string.unwrap())
}
