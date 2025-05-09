use base64::prelude::*;
use md5::{Digest, Md5};

pub fn compute_signature(api_key: &str, payload: impl AsRef<[u8]>) -> String {
    let data = format!("{}{}", BASE64_STANDARD.encode(payload), api_key);
    let signature = md5(data);
    hex::encode(signature)
}

fn md5(data: impl AsRef<[u8]>) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hash.into()
}
