use std::collections::HashMap;

pub struct KeyValueRequest<'a> {
    request: &'a [u8; 1024],
}

type KeyValueResponse = String;

impl<'a> KeyValueRequest<'a> {
    pub fn new(request: &'a [u8; 1024]) -> Self {
        Self { request }
    }

    pub fn handle_request(&self) -> Result<String, &'static str> {
        let get_request = b"GET / HTTP/1.1\r\n";
        let put_request = b"PUT / HTTP/1.1\r\n";
        let delete_request = b"DELETE / HTTP/1.1\r\n";

        let (status_line, body) = if self.request.starts_with(get_request) {
            ("HTTP/1.1 200 OK", Self::handle_get_request()?)
        } else if self.request.starts_with(put_request) {
            ("HTTP/1.1 200 OK", Self::handle_put_request()?)
        } else if self.request.starts_with(delete_request) {
            ("HTTP/1.1 200 OK", Self::handle_delete_request()?)
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

        Ok(response)
    }

    fn handle_get_request() -> Result<String, &'static str> {
        Ok("value".to_string())
    }
    fn handle_put_request() -> Result<String, &'static str> {
        Ok("value".to_string())
    }
    fn handle_delete_request() -> Result<String, &'static str> {
        Ok("value".to_string())
    }
}

pub struct KeyValueStore {
    key_value_store: HashMap<String, Box<()>>,
}
