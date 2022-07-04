use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
};

pub struct KeyValueStore {
    key_value_store: HashMap<String, Box<()>>,
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
        &self,
        mut stream: TcpStream,
    ) -> Result<(), &'static str> {
        let mut buf = [0; 1024];
        stream
            .read(&mut buf)
            .expect("Failed to read stream into byte buffer");

        let get_request = b"GET / HTTP/1.1\r\n";
        let put_request = b"PUT / HTTP/1.1\r\n";
        let delete_request = b"DELETE / HTTP/1.1\r\n";

        let (status_line, body) = if buf.starts_with(get_request) {
            ("HTTP/1.1 200 OK", self.handle_get_request(&buf)?)
        } else if buf.starts_with(put_request) {
            ("HTTP/1.1 200 OK", self.handle_put_request(&buf)?)
        } else if buf.starts_with(delete_request) {
            ("HTTP/1.1 200 OK", self.handle_delete_request(&buf)?)
        } else {
            (
                "HTTP/1.1 404 NOT FOUND",
                "No key found that matches provided key!".to_string(),
            )
        };

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            body.len(),
            body,
        );

        stream.write(&response.as_bytes()).unwrap();
        stream.flush().unwrap();

        Ok(())
    }

    fn handle_get_request(
        &self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        Ok("get request".to_string())
    }

    fn handle_put_request(
        &self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        Ok("put request".to_string())
    }

    fn handle_delete_request(
        &self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        Ok("delete request".to_string())
    }
}
