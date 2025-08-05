use std::error::Error;

use blake3::derive_key;

use cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Encryptor, Decryptor};
use twofish::Twofish;
use rand::RngCore;

type Encrypt = Encryptor<Twofish>;
type Decrypt = Decryptor<Twofish>;

const IV_SIZE: usize = 16;

pub fn encrypt(data: &str) -> Result<String, Box<dyn Error>> {
    let data = data.as_bytes();

    let key = derive_key("���I�O�H�T", b"E1E1LHH");

    let mut iv = [0u8; IV_SIZE];
    rand::thread_rng().fill_bytes(&mut iv);

    let mut buffer = data.to_vec();
    buffer.resize(buffer.len() + IV_SIZE, 0);

    let encrypt = Encrypt::new_from_slices(&key, &iv[..16])?;
    let cipher = encrypt
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
        .map_err(|e| e.to_string())?;

    let mut result = iv.to_vec();
    result.extend_from_slice(cipher);

    Ok(hex::encode(result))
}

pub fn decrypt(data: &str) -> Result<String, Box<dyn Error>> {
    let data = hex::decode(data)?;
    let (iv, cipher) = data.split_at(IV_SIZE);

    let key = derive_key("���I�O�H�T", b"E1E1LHH");

    let mut buffer = cipher.to_vec();
    let decryptor = Decrypt::new_from_slices(&key, iv)?;
    let decrypted = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| format!("{e}"))?;

    let result = String::from_utf8(decrypted.to_vec())?;
    Ok(result)
}