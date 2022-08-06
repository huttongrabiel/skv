use crate::crypto::{decrypt, encrypt, generate_key};
use regex::Regex;
use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

pub struct KeyValueStore {
    key_value_store: HashMap<String, Box<dyn Display + Send>>,
    encryption_key: String,
}

impl KeyValueStore {
    /// Create a new key-value store and generate user encryption key.
    ///
    /// Uses std::Collections::HashMap as the backing data structure at the
    /// moment.
    pub fn new() -> Self {
        let key_value_store = HashMap::new();
        let encryption_key = generate_key();
        Self {
            key_value_store,
            encryption_key,
        }
    }

    /// Handle a TCP Request to the key-value store.
    ///
    /// Errors are propagated back to the caller.
    pub fn handle_request(
        &mut self,
        mut stream: TcpStream,
    ) -> Result<(), &'static str> {
        let mut buf = [0; 1024];
        match stream.read(&mut buf) {
            Ok(_) => (),
            Err(_) => return Err("Failed to read stream to buffer"),
        }

        // Verify request has valid HTTP header.
        let buf_string = String::from_utf8_lossy(&buf);
        let pattern =
            Regex::new(r"\w{3,6}\s/\w*\sHTTP/1.1\r\n(\w*\r\n)*").unwrap();

        // FIXME: When hitting localhost:3400 in browser, this message is
        // displayed in the terminal, yet the output in the browser is fine. Has
        // to do with the additional information included in the request that
        // the browser sends.
        if !pattern.is_match(&buf_string) {
            return Err("Invalid HTTP request received.");
        }

        let get_request = b"GET";
        let put_request = b"PUT";
        let delete_request = b"DELETE";

        // Default response if request is anything but get, put, or delete.
        let unknown_request = (
            "HTTP/1.1 400 BAD REQUEST".to_string(),
            "This key-value store does not support \
            the HTTP request you are trying to make."
                .to_string(),
        );

        let (status_line, body) = if buf.starts_with(get_request) {
            self.handle_get_request(&buf)
        } else if buf.starts_with(put_request) {
            self.handle_put_request(&buf)
        } else if buf.starts_with(delete_request) {
            self.handle_delete_request(&buf)
        } else {
            unknown_request
        };

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            body.len(),
            body
        );

        match stream.write(response.as_bytes()) {
            Ok(_) => (),
            Err(_) => return Err("Failed to write to stream."),
        }
        match stream.flush() {
            Ok(_) => (),
            Err(_) => return Err("Failed to flush stream."),
        };

        Ok(())
    }

    fn handle_get_request(&mut self, buf: &[u8; 1024]) -> (String, String) {
        let key = match parse_key_from_request(buf) {
            Ok(key) => key,
            Err(_) => {
                return (
                    "HTTP/1.1 400 Bad Request".to_string(),
                    "Key for key-value store not provided!".to_string(),
                )
            }
        };

        let encryption_key = match parse_encryption_key_from_headers(buf) {
            Ok(key) => key,
            Err(_) => {
                return (
                    "HTTP/1.1 400 Bad Request".to_string(),
                    "User encryption key not provided in request headers."
                        .to_string(),
                )
            }
        };

        if key == "ls" {
            return (
                "HTTP/1.1 200 OK".to_string(),
                self.list_keys(encryption_key),
            );
        }

        let encrypted_key = match encrypt(&key, &encryption_key) {
            Ok(ek) => ek,
            Err(_) => return (
                "HTTP/1.1 400 Bad Request".to_string(),
                "Failed to encrypt key for lookup, check your encryption key!"
                    .to_string(),
            ),
        };

        let value: String = match self.key_value_store.get(&encrypted_key) {
            Some(val) => val.to_string(),
            None => {
                return (
                    "HTTP/1.1 404 NOT FOUND".to_string(),
                    format!(
                        "\
Key '{}' not found in key-value store. Either the data \
does not exist or you have an invalid key. Try using the ls command.",
                        key
                    ),
                )
            }
        };

        let mut value = decrypt(&value, &self.encryption_key).unwrap();

        // If the value that corresponds to the given key is a file, read the
        // file contents and print that to the stream.
        if Path::exists(Path::new(&value))
            && fs::metadata(&value).unwrap().is_file()
        {
            value = fs::read_to_string(&value).unwrap();
        }

        ("HTTP/1.1 200 OK".to_string(), value)
    }

    fn handle_put_request(&mut self, buf: &[u8; 1024]) -> (String, String) {
        let key = match parse_key_from_request(buf) {
            Ok(key) => key,
            Err(_) => {
                return (
                    "HTTP/1.1 400 Bad Request".to_string(),
                    "Key for key-value store not provided!".to_string(),
                )
            }
        };

        let mut value = match parse_body_from_request(buf) {
            Ok(value) => value,
            Err(_) => {
                return (
                    "HTTP/1.1 400 Bad Request".to_string(),
                    format!(
                        "No value provided to store with the key {}!",
                        &key
                    ),
                )
            }
        };

        if Path::exists(Path::new(&value))
            && fs::metadata(&value).unwrap().is_file()
        {
            value = fs::read_to_string(&value).expect("Failed to read file.");
        }

        let encrypted_key = encrypt(&key, &self.encryption_key).unwrap();
        let encrypted_value = encrypt(&value, &self.encryption_key).unwrap();

        match self
            .key_value_store
            .insert(encrypted_key, Box::new(encrypted_value))
        {
            Some(_) => (
                "HTTP/1.1 200 OK".to_string(),
                format!(
                    "Value associated with key, \"{}\", \
                            updated to \"{}\", in key-value store.",
                    key, &value
                ),
            ),
            None => (
                "HTTP/1.1 200 OK".to_string(),
                format!(
                    "[\"{}\", \"{}\"], \n200 - Success: \
                Entry inserted into key-value store.",
                    key, &value
                ),
            ),
        }
    }

    fn handle_delete_request(&mut self, buf: &[u8; 1024]) -> (String, String) {
        let key = match parse_key_from_request(buf) {
            Ok(key) => key,
            Err(_) => {
                return (
                    "HTTP/1.1 400 Bad Request".to_string(),
                    "Key for key-value store not provided!".to_string(),
                )
            }
        };

        let encryption_key = match parse_encryption_key_from_headers(buf) {
            Ok(key) => key,
            Err(_) => {
                return (
                    "HTTP/1.1 400 Bad Request".to_string(),
                    "User encryption key not provided in request headers."
                        .to_string(),
                )
            }
        };

        let encrypted_key = match encrypt(&key, &encryption_key) {
            Ok(ek) => ek,
            Err(_) => return (
                "HTTP/1.1 400 Bad Request".to_string(),
                "Failed to encrypt key for lookup, check your encryption key!"
                    .to_string(),
            ),
        };

        match self.key_value_store.remove(&encrypted_key) {
            // FIXME: Print the non-encrypted value to stream.
            Some(val) => (
                "HTTP/1.1 200 OK".to_string(),
                format!(
                    "Key-value pair [\"{}\", \"{}\"], removed from key-value store.",
                    key, val
                ),
            ),
            None => (
                "HTTP/1.1 404 NOT FOUND".to_string(),
                format!("Key '{}' not found in key-value store.", key),
            ),
        }
    }

    fn list_keys(&self, user_provided_encryption_key: String) -> String {
        let mut keys = String::new();
        for key in self.key_value_store.keys() {
            let key = match decrypt(key, &user_provided_encryption_key) {
                Ok(key) => key,
                Err(_) => {
                    return "Decryption failed due to invalid encryption key."
                        .to_string()
                }
            };
            keys.push_str(format!("{}\n", key).as_str());
        }
        keys
    }
}

impl Default for KeyValueStore {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_body_from_request(buf: &[u8; 1024]) -> Result<String, &'static str> {
    let body = match buf.split(|byte| *byte == b'\n').last() {
        Some(body) => body,
        None => return Err("Failed to parse body out of request"),
    };

    // Body is everything after the final \r\n which means, depending on the
    // size of the buffer used, it can have a lot of garbage values that look
    // like [0,0,0,0,...]. This essentially trims all those off.
    let body = match body.split(|byte| *byte == 0).next() {
        Some(body) => body,
        None => return Err(
            "Failed to trim off rest of body, or possiblity of unread bytes.",
        ),
    };

    let body_str = String::from_utf8_lossy(body);
    Ok(body_str.to_string())
}

fn parse_key_from_request(buf: &[u8; 1024]) -> Result<String, &'static str> {
    let mut key = buf.split(|byte| *byte == b' ');

    key.next();

    // In an HTTP header the URI is the second element:
    //  <request> <URI>.
    //
    //  So we move the iterator past the request to the URI.
    let key = match key.next() {
        Some(key) => key,
        None => return Err("Failed to parse key out of request"),
    };

    let key = &key[1..];
    let key = String::from_utf8_lossy(key).to_string();

    Ok(key)
}

fn parse_encryption_key_from_headers(
    buf: &[u8; 1024],
) -> Result<String, &'static str> {
    let mut headers = buf.split(|byte| *byte == b'\n');

    // Move to header section of HTTP request.
    headers.next();

    let headers: Vec<&[u8]> = buf.split(|byte| *byte == b'\n').collect();

    // FIXME: I assume there is a much more rust-y way of doing this using
    // filter().
    let mut key_header = String::new();
    for header in &headers {
        let text = String::from_utf8_lossy(header);
        let mut text = text.split(|ch| ch == ':');
        let cur = text.next().unwrap();
        if cur == "key" {
            key_header = text.next().unwrap().trim().to_string();
        }
    }

    if key_header.is_empty() {
        return Err("Please provide key header in request!");
    }

    Ok(key_header)
}

#[cfg(test)]
mod test {
    use super::*;

    // Yes, this buffer is big and ugly, but that is how our TCP stream
    // comes in.
    pub const BUF: [u8; 1024] = [
        80, 85, 84, 32, 47, 83, 97, 109, 112, 108, 101, 75, 101, 121, 32, 72,
        84, 84, 80, 47, 49, 46, 49, 13, 10, 72, 111, 115, 116, 58, 32, 108,
        111, 99, 97, 108, 104, 111, 115, 116, 58, 51, 52, 48, 48, 13, 10, 85,
        115, 101, 114, 45, 65, 103, 101, 110, 116, 58, 32, 99, 117, 114, 108,
        47, 55, 46, 56, 49, 46, 48, 13, 10, 65, 99, 99, 101, 112, 116, 58, 32,
        42, 47, 42, 13, 10, 107, 101, 121, 58, 32, 54, 48, 54, 101, 100, 97,
        99, 101, 51, 48, 53, 51, 99, 52, 101, 57, 50, 50, 50, 53, 49, 53, 98,
        55, 98, 97, 48, 101, 49, 54, 101, 52, 49, 54, 52, 56, 99, 52, 48, 99,
        53, 54, 56, 54, 48, 101, 100, 98, 52, 54, 52, 102, 56, 49, 51, 99, 100,
        53, 51, 99, 53, 55, 50, 54, 13, 10, 67, 111, 110, 116, 101, 110, 116,
        45, 76, 101, 110, 103, 116, 104, 58, 32, 49, 49, 13, 10, 67, 111, 110,
        116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108,
        105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 119, 119, 119, 45, 102,
        111, 114, 109, 45, 117, 114, 108, 101, 110, 99, 111, 100, 101, 100, 13,
        10, 13, 10, 83, 97, 109, 112, 108, 101, 86, 97, 108, 117, 101, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[test]
    fn parse_body() {
        assert_eq!(parse_body_from_request(&BUF).unwrap(), "SampleValue");
    }

    #[test]
    fn parse_key() {
        assert_eq!(parse_key_from_request(&BUF).unwrap(), "SampleKey");
    }

    #[test]
    fn test_parse_encryption_key_from_headers() {
        let encryption_key = parse_encryption_key_from_headers(&BUF).unwrap();
        assert_eq!(
            encryption_key,
            "606edace3053c4e9222515b7ba0e16e41648c40c56860edb464f813cd53c5726"
        );
    }
}
