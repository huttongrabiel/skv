use regex::Regex;
use std::{
    collections::HashMap,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
    str,
};

pub struct KeyValueStore {
    key_value_store: HashMap<String, Box<dyn Display>>,
}

impl KeyValueStore {
    /// Create a new key-value store.
    ///
    /// Uses std::Collections::HashMap as the backing data structure at the
    /// moment.
    pub fn new() -> Self {
        let key_value_store = HashMap::new();
        Self { key_value_store }
    }

    /// Handle a TCP Request to the key-value store.
    ///
    /// Errors are propagated back to the caller.
    ///
    /// # Panics
    ///
    /// * Reading from the stream could panic.
    /// * Writing to the stream could panic.
    /// * Flushing the stream could panic.
    pub fn handle_request(
        &mut self,
        mut stream: TcpStream,
    ) -> Result<(), &'static str> {
        let mut buf = [0; 1024];
        stream
            .read(&mut buf)
            .expect("Failed to read stream into byte buffer");

        // Verify request has valid HTTP header.
        let buf_string = str::from_utf8(&buf).unwrap();
        let pattern = Regex::new(r"\w{3,6}\s/\w*\sHTTP/1.1\r\n").unwrap();
        // FIXME: We don't want to just shut off the server if we receive an
        // invalid request.
        assert!(pattern.is_match(buf_string));

        let get_request = b"GET";
        let put_request = b"PUT";
        let delete_request = b"DELETE";

        // Default response if request is anything but get, put, or delete.
        let unknown_request = "This key-value store does not support \
            the HTTP request you are trying to make.";

        let (status_line, body) = if buf.starts_with(get_request) {
            ("HTTP/1.1 200 OK", self.handle_get_request(&buf)?)
        } else if buf.starts_with(put_request) {
            ("HTTP/1.1 200 OK", self.handle_put_request(&buf)?)
        } else if buf.starts_with(delete_request) {
            ("HTTP/1.1 200 OK", self.handle_delete_request(&buf)?)
        } else {
            ("HTTP/1.1 404 NOT FOUND", unknown_request.to_string())
        };

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            body.len(),
            body
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();

        Ok(())
    }

    fn handle_get_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let key = parse_key_from_request(buf)
            .expect("Failed to parse key from request.");

        let value: String = match self.key_value_store.get(&key) {
            Some(val) => val.to_string(),
            None => format!("Key '{}' not found in key-value store.", key),
        };

        Ok(value)
    }

    fn handle_put_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let key = parse_key_from_request(buf)
            .expect("Failed to parse key data from request.");
        let value = parse_body_from_request(buf)
            .expect("Failed to parse body data from request.");

        match self
            .key_value_store
            .insert(key.clone(), Box::new(value.clone()))
        {
            Some(val) => Ok(format!(
                "Value associated with key, {}, \
                        updated to {}, in key-value store.",
                key, val
            )),
            None => Ok(format!(
                "[{}, {}], successfully inserted into key-value store",
                key, value
            )),
        }
    }

    fn handle_delete_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let key = parse_key_from_request(buf)
            .expect("Failed to parse key data from request.");

        match self.key_value_store.remove(&key) {
            Some(val) => Ok(format!(
                "Key-value pair, [{}, {}], removed from key-value store.",
                key, val
            )),
            None => Ok(format!("Key '{}' not found in key-value store.", key)),
        }
    }
}

fn parse_body_from_request(buf: &[u8; 1024]) -> Result<String, ()> {
    let body = buf.split(|byte| *byte == b'\n').last().unwrap();
    let body = body.split(|byte| *byte == 0).next().unwrap();
    Ok(str::from_utf8(body).unwrap().to_string())
}

fn parse_key_from_request(buf: &[u8; 1024]) -> Result<String, ()> {
    let mut key = buf.split(|byte| *byte == b' ');
    key.next();
    let key = &key.next().unwrap()[1..];
    Ok(str::from_utf8(key).unwrap().to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    // Yes, this buffer is big and ugly, but that is how our TCP stream
    // comes in.
    static BUF: [u8; 1024] = [
        80, 85, 84, 32, 47, 107, 101, 121, 100, 111, 111, 100, 108, 101, 32,
        72, 84, 84, 80, 47, 49, 46, 49, 13, 10, 72, 111, 115, 116, 58, 32, 108,
        111, 99, 97, 108, 104, 111, 115, 116, 58, 51, 52, 48, 48, 13, 10, 85,
        115, 101, 114, 45, 65, 103, 101, 110, 116, 58, 32, 99, 117, 114, 108,
        47, 55, 46, 56, 49, 46, 48, 13, 10, 65, 99, 99, 101, 112, 116, 58, 32,
        42, 47, 42, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110,
        103, 116, 104, 58, 32, 49, 53, 13, 10, 67, 111, 110, 116, 101, 110,
        116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97,
        116, 105, 111, 110, 47, 120, 45, 119, 119, 119, 45, 102, 111, 114, 109,
        45, 117, 114, 108, 101, 110, 99, 111, 100, 101, 100, 13, 10, 13, 10,
        115, 111, 32, 109, 117, 99, 104, 32, 110, 105, 99, 101, 114, 33, 33, 0,
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
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    #[test]
    fn parse_body() {
        assert_eq!(parse_body_from_request(&BUF).unwrap(), "so much nicer!!");
    }

    #[test]
    fn parse_key() {
        assert_eq!(parse_key_from_request(&BUF).unwrap(), "keydoodle");
    }
}
