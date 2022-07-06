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

        let get_request = b"GET / HTTP/1.1\r\n";
        let put_request = b"PUT / HTTP/1.1\r\n";
        let delete_request = b"DELETE / HTTP/1.1\r\n";
        let key_not_found = "No key found that matches that key!";

        let (status_line, body) = if buf.starts_with(get_request) {
            ("HTTP/1.1 200 OK", self.handle_get_request(&buf)?)
        } else if buf.starts_with(put_request) {
            ("HTTP/1.1 200 OK", self.handle_put_request(&buf)?)
        } else if buf.starts_with(delete_request) {
            ("HTTP/1.1 200 OK", self.handle_delete_request(&buf)?)
        } else {
            ("HTTP/1.1 404 NOT FOUND", key_not_found.to_string())
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
        let body = parse_body_from_request(buf)
            .expect("Failed to parse body data from request");

        Ok(body)
    }

    fn handle_put_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let key = parse_key_from_request(buf)
            .expect("Failed to parse key data from request");
        let value = parse_body_from_request(buf)
            .expect("Failed to parse body data from request");

        eprintln!("Key: {}", key);

        //        self.key_value_store.insert(key, Box::new(value.clone()));

        Ok(value)
    }

    fn handle_delete_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let body = parse_body_from_request(buf)
            .expect("Failed to parse body data from request");

        Ok(body)
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
    Ok(str::from_utf8(key.next().unwrap()).unwrap().to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_body() {
        // Yes, this buffer is big and ugly, but that is how our TCP stream
        // comes in.
        let buf = [
            71, 69, 84, 32, 47, 32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10, 72,
            111, 115, 116, 58, 32, 108, 111, 99, 97, 108, 104, 111, 115, 116,
            58, 51, 52, 48, 48, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101,
            110, 116, 58, 32, 99, 117, 114, 108, 47, 55, 46, 56, 49, 46, 48,
            13, 10, 65, 99, 99, 101, 112, 116, 58, 32, 42, 47, 42, 13, 10, 67,
            111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58,
            32, 49, 53, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121,
            112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111,
            110, 47, 120, 45, 119, 119, 119, 45, 102, 111, 114, 109, 45, 117,
            114, 108, 101, 110, 99, 111, 100, 101, 100, 13, 10, 13, 10, 115,
            111, 32, 109, 117, 99, 104, 32, 110, 105, 99, 101, 114, 33, 33, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        assert_eq!(parse_body_from_request(&buf).unwrap(), "so much nicer!!");
    }
}
