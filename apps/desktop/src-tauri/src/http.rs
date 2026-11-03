//! HTTP/TLS helper for remote inference.
//!
//! Uses `ureq` with rustls and the platform certificate verifier (Security
//! framework on macOS, SChannel roots on Windows, the system trust store on
//! Linux). The API key only lives in memory as an `Authorization` header; there
//! is no temporary curl config file or system-binary dependency.

use std::time::Duration;

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub trait HttpTransport: Send + Sync {
    fn post_json(&self, url: &str, api_key: &str, body: &str) -> Result<HttpResponse, String>;
}

pub struct NativeHttpTransport;

impl NativeHttpTransport {
    fn agent(&self) -> ureq::Agent {
        ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(30))).build().into()
    }
}

impl HttpTransport for NativeHttpTransport {
    fn post_json(&self, url: &str, api_key: &str, body: &str) -> Result<HttpResponse, String> {
        let mut response = self
            .agent()
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {api_key}"))
            .send(body)
            .map_err(|error| format!("HTTP request failed: {error}"))?;
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string().map_err(|error| format!("HTTP response body: {error}"))?;
        Ok(HttpResponse { status, body })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn read_request(stream: &mut std::net::TcpStream) -> String {
        let mut data = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let count = stream.read(&mut buffer).unwrap_or(0);
            if count == 0 {
                break;
            }
            data.extend_from_slice(&buffer[..count]);
            if let Some(head_end) = data.windows(4).position(|window| window == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&data[..head_end + 4]).to_lowercase();
                let content_length = head
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .and_then(|value| value.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                if data.len() >= head_end + 4 + content_length {
                    break;
                }
            }
        }
        String::from_utf8_lossy(&data).to_string()
    }

    #[test]
    fn http_posts_json_with_bearer_header() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_request(&mut stream);
            assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"), "{request}");
            assert!(request.to_lowercase().contains("authorization: bearer test-key"), "{request}");
            assert!(request.contains("\"model\":\"test-model\""), "{request}");
            let body = r#"{"choices":[{"message":{"content":"pong"}}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        let transport = NativeHttpTransport;
        let response = transport
            .post_json(
                &format!("http://{address}/v1/chat/completions"),
                "test-key",
                r#"{"model":"test-model","messages":[{"role":"user","content":"ping"}]}"#,
            )
            .unwrap();
        server.join().unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("pong"));
    }
}
