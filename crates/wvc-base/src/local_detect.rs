//! Local model-server auto-detection for `wvc login`.
//!
//! Probes the well-known local OpenAI-compatible endpoints (and Ollama's native
//! `/api/tags`) with a short per-port timeout and reports which servers are
//! reachable with a valid model catalog. The login picker uses this to surface
//! only the local servers that are actually running, instead of always listing
//! every local option.

use std::time::Duration;

use wvc_provider_metadata::{LoginProviderDescriptor, LoginProviderTarget};

/// Per-port probe timeout (spec NRA-721 S1T4: 2 seconds per port).
pub const LOCAL_DETECT_TIMEOUT: Duration = Duration::from_secs(2);

/// A single local server that responds with a valid model catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetectedLocalServer {
    /// Login provider descriptor that should be surfaced in the picker.
    pub provider: LoginProviderDescriptor,
    /// Port the server was detected on.
    pub port: u16,
    /// The endpoint that was probed successfully (`/v1/models` or `/api/tags`).
    pub probed_path: &'static str,
}

/// The well-known local servers wvc can auto-detect.
///
/// Order matters: it defines both the probe order and the display order in the
/// login picker.
pub(crate) const LOCAL_SERVERS: &[LocalServer] = &[
    LocalServer {
        port: 11434,
        path: "/api/tags",
        provider: crate::provider_catalog::OLLAMA_LOGIN_PROVIDER,
    },
    LocalServer {
        port: 1234,
        path: "/v1/models",
        provider: crate::provider_catalog::LMSTUDIO_LOGIN_PROVIDER,
    },
    LocalServer {
        port: 8080,
        path: "/v1/models",
        provider: crate::provider_catalog::LLAMACPP_LOGIN_PROVIDER,
    },
    LocalServer {
        port: 8000,
        path: "/v1/models",
        provider: crate::provider_catalog::VLLM_LOGIN_PROVIDER,
    },
    LocalServer {
        port: 8000,
        path: "/v1/models",
        provider: crate::provider_catalog::OMLX_LOGIN_PROVIDER,
    },
];

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalServer {
    pub port: u16,
    pub path: &'static str,
    pub provider: LoginProviderDescriptor,
}

impl LocalServer {
    fn probe_url(&self) -> String {
        format!("http://127.0.0.1:{}{}", self.port, self.path)
    }
}

/// Probe every known local server and return the ones that respond with a
/// valid model catalog. Probes run concurrently; each has a 2s timeout, so the
/// worst-case wall time stays near one timeout, not one per server.
pub async fn detect_local_providers(
    timeout: Duration,
) -> Vec<DetectedLocalServer> {
    let client = crate::provider::shared_http_client();

    let futures = LOCAL_SERVERS.iter().map(|server| {
        let client = client.clone();
        async move {
            let url = server.probe_url();
            let probe = async {
                let response = client
                    .get(&url)
                    .timeout(timeout)
                    .send()
                    .await
                    .map_err(|_| ())?;
                if !response.status().is_success() {
                    return Err(());
                }
                let body = response.text().await.map_err(|_| ())?;
                if !is_valid_model_catalog(&body) {
                    return Err(());
                }
                Ok(DetectedLocalServer {
                    provider: server.provider,
                    port: server.port,
                    probed_path: server.path,
                })
            };
            probe.await.ok()
        }
    });

    futures::future::join_all(futures).await.into_iter().flatten().collect()
}

/// Lightweight validation that an OpenAI-compatible `/v1/models` (or Ollama
/// `/api/tags`) response carries a non-empty model array. Ollama's native
/// `/api/tags` returns `{"models": [...]}`, while OpenAI-compatible servers
/// return `{"data": [...]}` (and occasionally a top-level `models` array).
fn is_valid_model_catalog(raw_body: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw_body) else {
        return false;
    };
    match &value {
        serde_json::Value::Array(items) => !items.is_empty(),
        serde_json::Value::Object(object) => object
            .get("data")
            .or_else(|| object.get("models"))
            .and_then(serde_json::Value::as_array)
            .map(|items| !items.is_empty())
            .unwrap_or(false),
        _ => false,
    }
}

/// Convenience for the login picker: does the given provider's target point at
/// a local server that was detected? Used to decide whether a local provider
/// should be filtered out of the picker when nothing is running.
pub fn is_detected_local_provider(
    provider: LoginProviderDescriptor,
    detected: &[DetectedLocalServer],
) -> bool {
    let LoginProviderTarget::OpenAiCompatible(profile) = provider.target else {
        return false;
    };
    detected.iter().any(|found| {
        let LoginProviderTarget::OpenAiCompatible(found_profile) = found.provider.target else {
            return false;
        };
        found_profile.id == profile.id
    })
}

/// Whether a provider targets a local (localhost/127.0.0.1) endpoint that is
/// subject to auto-detection. Cloud and non-OpenAI-compatible providers are
/// never gated behind local detection.
pub fn is_local_probe_target(provider: LoginProviderDescriptor) -> bool {
    let LoginProviderTarget::OpenAiCompatible(profile) = provider.target else {
        return false;
    };
    crate::provider_catalog::api_base_uses_localhost(profile.api_base)
}

/// Filter a login-provider list so cloud providers are always kept and local
/// providers are kept only when they were detected running. If `detected` is
/// empty, all local providers are dropped and the list is effectively
/// cloud-only.
pub fn filter_login_providers_by_local_detection(
    providers: &[LoginProviderDescriptor],
    detected: &[DetectedLocalServer],
) -> Vec<LoginProviderDescriptor> {
    providers
        .iter()
        .copied()
        .filter(|provider| {
            if !is_local_probe_target(*provider) {
                return true;
            }
            is_detected_local_provider(*provider, detected)
        })
        .collect()
}

#[cfg(test)]
#[path = "local_detect_tests.rs"]
mod local_detect_tests;
