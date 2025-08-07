use hex;
use blake3::derive_key;
use tiger::{Digest, Tiger};

pub fn hash(data: &str) -> String {
    let data: &[u8] = data.as_bytes();

    let mut tiger = Tiger::new();
    tiger.update(data);

    let salt: [u8; 32] = derive_key("��", data);
    tiger.update(&salt);

    let hash = tiger.finalize();
    hex::encode(hash)
}