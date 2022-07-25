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
/// let key = String::from(
///     "606edace3053c4e9222515b7ba0e16e41648c40c56860edb464f813cd53c5726"
/// );
///
/// // When request is made with key in header, server will do something along
/// // these lines. Apply the same idea to decryption as well.
/// let data = String::from("super secret data");
/// let encrypted_data = encrypt(&data, &key);
/// ```
pub fn generate_key() {
    let rng = fastrand::Rng::new();

    let key: Vec<u8> = repeat_with(|| rng.u8(..)).take(32).collect();

    println!(
        "
Save this key and keep it secret! It \
cannot and will not be regenerated.\n{}",
        hex::encode(key)
    );
}

    let encrypted_data = crypt(&key, &data, Crypt::Encrypt);
    let decrypted_data =
        crypt(&key, &hex::encode(encrypted_data), Crypt::Decrypt);

    assert_eq!(hex::encode(&decrypted_data), data);

    Ok(hex::encode(decrypted_data))
}

pub enum Crypt {
    Encrypt,
    Decrypt,
}

/// Encrypt or decrypt text.
///
/// Specify whether encryption or decryption by passing Crypt::Encrypt/Decrypt
/// as the direction parameter.
pub fn crypt(key: &Vec<u8>, data: &String, direction: Crypt) -> Vec<u8> {
    let key = Key::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    let rng = fastrand::Rng::new();

    let nonce: Vec<u8> = repeat_with(|| rng.u8(..)).take(12).collect();
    let nonce = Nonce::from_slice(&nonce);

    let mut crypted_text: Vec<u8> = Vec::new();
    match direction {
        Crypt::Encrypt => {
            crypted_text = cipher
                .encrypt(nonce, data.as_ref())
                .expect("encryption failure")
        }
        Crypt::Decrypt => {
            crypted_text = cipher
                .decrypt(nonce, data.as_ref())
                .expect("decryption failure!");
        }
    }
    crypted_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        generate_key();
    }
}
