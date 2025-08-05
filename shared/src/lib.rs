use rand::{distributions::Alphanumeric, Rng};

pub mod crypto;
pub mod hash;

pub fn gen_str(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}