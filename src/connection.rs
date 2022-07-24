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
        let pattern = Regex::new(r"\w{3,6}\s/\w*\sHTTP/1.1\r\n").unwrap();

        // FIXME: When hitting localhost:3400/ls in browser, this message is
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

    fn handle_get_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let key = parse_key_from_request(buf)?;

        if key == "ls" {
            return Ok(self.list_keys());
        }

        let mut value: String = match self.key_value_store.get(&key) {
            Some(val) => format!(
                "{} \n200 - Success: Value retrieved from key-value store.",
                val.to_string()
            ),
            None => format!("Key '{}' not found in key-value store.", key),
        };

        // If the value that corresponds to the given key is a file, read the
        // file contents and print that to the stream.
        if Path::exists(Path::new(&value))
            && fs::metadata(&value).unwrap().is_file()
        {
            value = fs::read_to_string(&value).unwrap();
        }

        Ok(value)
    }

    fn handle_put_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let key = parse_key_from_request(buf)?;
        let mut value = parse_body_from_request(buf)?;

        // If path is a file, read its contents into a string and store as value
        if Path::exists(Path::new(&value))
            && fs::metadata(&value).unwrap().is_file()
        {
            value = fs::read_to_string(&value).expect("Failed to read file.");
        }

        match self
            .key_value_store
            .insert(key.clone(), Box::new(value.clone()))
        {
            Some(_) => Ok(format!(
                "Value associated with key, \"{}\", \
                        updated to \"{}\", in key-value store.
                        200 - Success: Value updated.",
                key, &value
            )),
            None => Ok(format!(
                "[\"{}\", \"{}\"], \n200 - Success: \
                Entry inserted into key-value store.",
                key, &value
            )),
        }
    }

    fn handle_delete_request(
        &mut self,
        buf: &[u8; 1024],
    ) -> Result<String, &'static str> {
        let key = parse_key_from_request(buf)?;

        match self.key_value_store.remove(&key) {
            Some(val) => Ok(format!(
                "Key-value pair, [{}, {}], removed from key-value store.\
                \n200 - Success: Entry deleted from key-value store.",
                key, val
            )),
            None => Ok(format!("Key '{}' not found in key-value store.", key)),
        }
    }

    fn list_keys(&self) -> String {
        let mut keys = String::new();
        for (key, _) in &self.key_value_store {
            keys.push_str(format!("{}\n", key).as_str());
        }
        keys
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

    Ok(String::from_utf8_lossy(key).to_string())
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
