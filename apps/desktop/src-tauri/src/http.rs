//! HTTP/TLS helper for remote inference.
//!
//! Uses `ureq` with rustls and the platform certificate verifier (Security
//! framework on macOS, SChannel roots on Windows, the system trust store on
//! Linux). The API key only lives in memory as an `Authorization` header; there
//! is no temporary curl config file or system-binary dependency.

use std::time::Duration;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub trait HttpTransport: Send + Sync {
    fn post_json(&self, url: &str, api_key: &str, body: &str) -> Result<HttpResponse, String>;
}

pub struct NativeHttpTransport;

static INSTALL_PROVIDER: std::sync::Once = std::sync::Once::new();

fn ensure_crypto_provider() {
    INSTALL_PROVIDER.call_once(|| {
        // A process-level provider is required by ureq's rustls-no-provider build.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

impl NativeHttpTransport {
    fn agent(&self) -> ureq::Agent {
        // ureq defaults to the WebPki root set, which is disabled in this
        // feature set. Use the platform verifier (system trust store) instead.
        let tls = ureq::tls::TlsConfig::builder().root_certs(ureq::tls::RootCerts::PlatformVerifier).build();
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .http_status_as_error(false)
            .tls_config(tls)
            .build()
            .into()
    }
}

impl HttpTransport for NativeHttpTransport {
    fn post_json(&self, url: &str, api_key: &str, body: &str) -> Result<HttpResponse, String> {
        ensure_crypto_provider();
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

#[cfg(test)]
mod tls_smoke {
    use super::*;

    #[test]
    #[ignore]
    fn https_endpoint_does_not_panic() {
        let result = NativeHttpTransport.post_json(
            "https://api.deepseek.com/v1/chat/completions",
            "invalid-key",
            r#"{"model":"deepseek-flash","messages":[{"role":"user","content":"ping"}]}"#,
        );
        println!("result: {result:?}");
    }
}
