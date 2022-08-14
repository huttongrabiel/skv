use crate::connection::DataObject;
use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use std::iter::repeat_with;

/// Generate key for AEAD encryption of data.
///
/// On creation of the server, a one-time key is generated.
/// This key is printed out to the screen and will be used in the curl request
/// to encrypt and decrypt data.
///
/// # Examples
///
/// ```
/// use skv::crypto::*;
///
/// // Key is generated upon start of the server and given to user. For example,
/// // this will use a hardcoded key.
/// let key = generate_key();
///
/// // When request is made with key in header, server will do something along
/// // these lines. Apply the same idea to decryption as well.
/// let data = String::from("super secret data");
/// let encrypted_data = encrypt(&data, &key);
/// ```
pub fn generate_key() -> String {
    let rng = fastrand::Rng::new();

    let key: Vec<u8> = repeat_with(|| rng.u8(..)).take(32).collect();

    println!(
        "
Save this key and keep it secret! It \
cannot and will not be regenerated.\n{}",
        hex::encode(&key)
    );

    hex::encode(key)
}

/// Returns ciphertext.
///
/// # Panics
///
/// ```encrypt()``` can panic if passed bad data.
pub fn encrypt(
    plaintext: &String,
    key: &String,
) -> Result<DataObject, &'static str> {
    let key = match hex::decode(&key) {
        Ok(key) => key,
        Err(_) => return Err("Hex decode failed to decode key."),
    };

    let key = Key::from_slice(&key);

    let cipher = Aes256Gcm::new(key);

    let rng = fastrand::Rng::new();

    let nonce_buf: Vec<u8> = repeat_with(|| rng.u8(..)).take(12).collect();
    let nonce = Nonce::from_slice(&nonce_buf);

    assert_eq!(nonce.len(), nonce_buf.len());

    let mut ciphertext = match cipher.encrypt(nonce, plaintext.as_ref()) {
        Ok(et) => et,
        Err(_) => return Err("Failed to encrypt data."),
    };

    ciphertext.append(&mut nonce_buf.clone());

    Ok(DataObject::new(hex::encode(ciphertext), nonce.len()))
}

/// Returns plaintext given encrypted text and encryption key.
pub fn decrypt(
    data_object: &DataObject,
    encryption_key: &String,
) -> Result<String, &'static str> {
    let encryption_key = match hex::decode(encryption_key) {
        Ok(key) => key,
        Err(_) => return Err("Invalid key format!"),
    };

    let ciphertext_bytes = match hex::decode(&data_object.ciphertext) {
        Ok(ct) => ct,
        Err(_) => return Err("Hex decode failed to decode ciphertext."),
    };

    let encryption_key = Key::from_slice(&encryption_key);
    let cipher = Aes256Gcm::new(encryption_key);

    let nonce_start_pos = ciphertext_bytes.len() - data_object.nonce_size;
    assert!(nonce_start_pos < ciphertext_bytes.len());

    let nonce = &ciphertext_bytes[nonce_start_pos..];
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = &ciphertext_bytes[..nonce_start_pos];

    let decrypted_text = match cipher.decrypt(nonce, ciphertext.as_ref()) {
        Ok(dt) => dt,
        Err(_) => return Err("Failed to decrypt data! Check your key."),
    };

    let decrypted_text: String =
        decrypted_text.iter().map(|val| *val as char).collect();

    Ok(decrypted_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        generate_key();
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = String::from("plaintext test");
        let encrypted_data = encrypt(&plaintext, &key).unwrap();
        eprintln!("ciphertext: {}", encrypted_data.ciphertext);
        let decrypted_text = decrypt(&encrypted_data, &key).unwrap();
        eprintln!("decrypted_text: {}", decrypted_text);
        assert_eq!(decrypted_text, plaintext);
    }
}
