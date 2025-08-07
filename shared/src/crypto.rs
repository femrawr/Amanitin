use std::error::Error;

use blake3::derive_key;

use cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Encryptor, Decryptor};
use twofish::Twofish;
use rand::RngCore;

use crate::hash;

type Encrypt = Encryptor<Twofish>;
type Decrypt = Decryptor<Twofish>;

const IV_SIZE: usize = 16;

pub fn encrypt(data: &str, key: &str) -> Result<String, Box<dyn Error>> {
    let data: &[u8] = data.as_bytes();

    let key: [u8; 32] = derive_key("���I�O�H�T", hash::hash(key).as_bytes());

    let mut iv: [u8; 16] = [0u8; IV_SIZE];
    rand::thread_rng().fill_bytes(&mut iv);

    let mut buffer: Vec<u8> = data.to_vec();
    buffer.resize(buffer.len() + IV_SIZE, 0);

    let encrypt: Encryptor<Twofish> = Encrypt::new_from_slices(&key, &iv[..16])?;
    let cipher: &[u8] = encrypt
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, data.len())
        .map_err(|e| e.to_string())?;

    let mut result: Vec<u8> = iv.to_vec();
    result.extend_from_slice(cipher);

    Ok(hex::encode(result))
}

pub fn decrypt(data: &str, key: &str) -> Result<String, Box<dyn Error>> {
    let data: Vec<u8> = hex::decode(data)?;
    let (iv, cipher) = data.split_at(IV_SIZE);

    let key: [u8; 32] = derive_key("���I�O�H�T", hash::hash(key).as_bytes());

    let mut buffer: Vec<u8> = cipher.to_vec();
    let decryptor = Decrypt::new_from_slices(&key, iv)?;
    let decrypted: &[u8] = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| format!("{e}"))?;

    let result: String = String::from_utf8(decrypted.to_vec())?;
    Ok(result)
}