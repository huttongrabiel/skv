use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use std::iter::repeat_with;

pub fn generate_key() -> Result<String, &'static str> {
    let rng = fastrand::Rng::new();

    let key: Vec<u8> = repeat_with(|| rng.u8(..)).take(32).collect();
    let key = Key::from_slice(&key);

    let cipher = Aes256Gcm::new(key);

    let nonce: Vec<u8> = repeat_with(|| rng.u8(..)).take(12).collect();
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce, b"key data".as_ref())
        .expect("encryption failure!");

    decrypt(key.to_vec(), ciphertext);

    Ok(hex::encode(nonce))
}

fn decrypt(key: Vec<u8>, ciphertext: Vec<u8>) {
    let key = Key::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    let rng = fastrand::Rng::new();

    let nonce: Vec<u8> = repeat_with(|| rng.u8(..)).take(12).collect();
    let nonce = Nonce::from_slice(&nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("decryption failure!");

    println!("Plaintext: {}", hex::encode(&plaintext));
}

//fn generate_nonce() -> Vec<u8> {
//    let nonce: Vec<u8> = repeat_with(|| rng.u8(..)).take(12).collect();
//    let nonce = Nonce::from_slice(&nonce);
//    return nonce.to_vec();
//}

//pub fn encrypt(value: String) -> Result<String, &'static str> {
//    Ok()
//}

//pub fn decrypt(String) -> String {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        generate_key();
    }
}
