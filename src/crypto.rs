use md5::{Digest, Md5};

pub fn md5(data: impl AsRef<[u8]>) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hash.into()
}
