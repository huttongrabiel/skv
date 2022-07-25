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

/// Returns ciphertext.
///
/// # Panics
///
/// ```encrypt()``` can panic if passed bad data.
///
/// ```hex::decode()``` can panic if unable to convert key string to byte array.
pub fn encrypt(
    plaintext: &String,
    key: &String,
) -> Result<String, &'static str> {
    let key = hex::decode(&key).expect("Failed to decode key to byte array");
    let key = Key::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    // I don't want the user to have to have a new nonce (authentication tag)
    // for every request they make so we are just using a part of their key.
    // #SecurityExpert
    let nonce = Nonce::from_slice(&key[4..16]);

    let encrypted_text = match cipher.encrypt(nonce, plaintext.as_ref()) {
        Ok(et) => et,
        Err(_) => return Err("Failed to encrypt data."),
    };

    Ok(hex::encode(encrypted_text))
}

/// Returns plaintext.
///
/// # Panics
///
/// ```decrypt()``` will panic if passed bad/wrong data.
pub fn decrypt(
    ciphertext: String,
    key: &Vec<u8>,
) -> Result<String, &'static str> {
    let key = Key::from_slice(&key);
    let cipher = Aes256Gcm::new(key);

    // I don't want the user to have to have a new nonce (authentication tag)
    // for every request they make so we are just using a part of their key.
    let nonce = Nonce::from_slice(&key[4..16]);

    let decrypted_text = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("decryption failure");

    Ok(hex::encode(decrypted_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        generate_key();
    }
}
