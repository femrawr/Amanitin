use hex;
use blake3::derive_key;
use tiger::{Digest, Tiger};

pub fn hash(data: &str) -> String {
    let data = data.as_bytes();

    let mut tiger = Tiger::new();
    tiger.update(data);

    let salt = derive_key("��", data);
    tiger.update(&salt);

    let hash = tiger.finalize();
    hex::encode(hash)
}