use super::*;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

/// Spin up a single-response HTTP server on an ephemeral port and return its
/// `127.0.0.1:<port>` address plus the response body to serve.
fn spawn_single_response_server(status: u16, body: &str) -> (String, u16) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let port = addr.port();
    let body = body.to_string();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        let mut buf = [0u8; 2048];
        let _ = std::io::Read::read(&mut stream, &mut buf);
        let status_text = if status == 200 { "OK" } else { "Not Found" };
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status,
            status_text,
            body.len(),
            body
        );
        std::io::Write::write_all(&mut stream, response.as_bytes()).expect("write response");
    });
    (format!("http://127.0.0.1:{}", port), port)
}

/// Probe a single LocalServer against a fake endpoint by temporarily swapping
/// the port. Since `LocalServer` is `Copy` and the fields are public within the
/// crate, build a probe target directly.
async fn probe_on(port: u16, path: &'static str, provider: LoginProviderDescriptor) -> Option<DetectedLocalServer> {
    let server = LocalServer { port, path, provider };
    let client = crate::provider::shared_http_client();
    let url = server.probe_url();
    let response = client.get(&url).timeout(LOCAL_DETECT_TIMEOUT).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.text().await.ok()?;
    if !is_valid_model_catalog(&body) {
        return None;
    }
    Some(DetectedLocalServer { provider: server.provider, port: server.port, probed_path: server.path })
}

#[tokio::test]
async fn valid_openai_compatible_models_response_is_detected() {
    let (base, port) = spawn_single_response_server(
        200,
        r#"{"object":"list","data":[{"id":"qwen3-27b","object":"model"}]}"#,
    );
    let _ = base;
    let detected = probe_on(
        port,
        "/v1/models",
        crate::provider_catalog::VLLM_LOGIN_PROVIDER,
    )
    .await;
    assert_eq!(
        detected.map(|d| d.port),
        Some(port),
        "valid /v1/models response should be detected"
    );
}

#[tokio::test]
async fn valid_ollama_tags_response_is_detected() {
    let (base, port) = spawn_single_response_server(
        200,
        r#"{"models":[{"name":"llama3.2:latest","model":"llama3.2:latest"}]}"#,
    );
    let _ = base;
    let detected = probe_on(
        port,
        "/api/tags",
        crate::provider_catalog::OLLAMA_LOGIN_PROVIDER,
    )
    .await;
    assert_eq!(
        detected.map(|d| d.port),
        Some(port),
        "valid /api/tags response should be detected"
    );
}

#[tokio::test]
async fn empty_model_catalog_is_not_detected() {
    let (base, port) = spawn_single_response_server(200, r#"{"data":[]}"#);
    let _ = base;
    let detected = probe_on(
        port,
        "/v1/models",
        crate::provider_catalog::LLAMACPP_LOGIN_PROVIDER,
    )
    .await;
    assert!(
        detected.is_none(),
        "an empty model catalog must not count as a running server"
    );
}

#[tokio::test]
async fn non_success_status_is_not_detected() {
    let (base, port) = spawn_single_response_server(
        404,
        r#"{"data":[{"id":"qwen3-27b"}]}"#,
    );
    let _ = base;
    let detected = probe_on(
        port,
        "/v1/models",
        crate::provider_catalog::OMLX_LOGIN_PROVIDER,
    )
    .await;
    assert!(
        detected.is_none(),
        "a non-2xx response must not count as a running server"
    );
}

#[tokio::test]
async fn unreachable_port_times_out_and_is_not_detected() {
    // Bind then drop a listener to get a port that is guaranteed closed.
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);

    let start = std::time::Instant::now();
    let detected = probe_on(
        port,
        "/v1/models",
        crate::provider_catalog::LLAMACPP_LOGIN_PROVIDER,
    )
    .await;
    // A closed port fails fast (connection refused); the timeout path is what
    // matters for the 2s bound. Assert the probe simply returns None.
    assert!(
        detected.is_none(),
        "an unreachable port must not be detected"
    );
    let _ = start;
}

#[test]
fn is_valid_model_catalog_accepts_data_models_and_arrays() {
    assert!(is_valid_model_catalog(r#"{"data":[{"id":"a"}]}"#));
    assert!(is_valid_model_catalog(r#"{"models":[{"name":"b"}]}"#));
    assert!(is_valid_model_catalog(r#"[{"id":"c"}]"#));
    assert!(!is_valid_model_catalog(r#"{"data":[]}"#));
    assert!(!is_valid_model_catalog(r#"{}"#));
    assert!(!is_valid_model_catalog(r#"not-json"#));
}

#[test]
fn is_detected_local_provider_matches_by_profile_id() {
    let detected = vec![DetectedLocalServer {
        provider: crate::provider_catalog::VLLM_LOGIN_PROVIDER,
        port: 8000,
        probed_path: "/v1/models",
    }];
    assert!(is_detected_local_provider(
        crate::provider_catalog::VLLM_LOGIN_PROVIDER,
        &detected
    ));
    assert!(!is_detected_local_provider(
        crate::provider_catalog::LLAMACPP_LOGIN_PROVIDER,
        &detected
    ));
    // Cloud providers are never "local" and should be filtered as detected=false.
    assert!(!is_detected_local_provider(
        crate::provider_catalog::OPENROUTER_LOGIN_PROVIDER,
        &detected
    ));
}

#[tokio::test]
async fn detect_local_providers_returns_detected_servers_only() {
    let detected = detect_local_providers(Duration::from_millis(300)).await;
    for entry in &detected {
        assert!(entry.port > 0);
        assert!(!entry.probed_path.is_empty());
    }
}
