//! `wvc swarm` — spawn a swarm task with an optional worker profile.
//!
//! Usage: `wvc swarm <message> [--worker-profile <profile>]`
//!
//! Sends the message to the running wvc server as a swarm task. The optional
//! `--worker-profile` flag selects which system-prompt block is injected into
//! the spawned worker (coder, tester, reviewer, researcher).

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::cli::args::WorkerProfileArg;
use crate::cli::provider_init::ProviderChoice;
use crate::server;

/// Run the swarm command: send a message to the server as a swarm task.
pub async fn run_swarm_command(
    provider_choice: &ProviderChoice,
    message: &str,
    worker_profile: Option<WorkerProfileArg>,
) -> Result<()> {
    let debug_socket = server::debug_socket_path();

    if !crate::transport::is_socket_path(&debug_socket) {
        eprintln!("Debug socket not found at {:?}", debug_socket);
        eprintln!("\nMake sure:");
        eprintln!("  1. A wvc server is running (wvc or wvc serve)");
        eprintln!("  2. debug_socket is enabled in ~/.wvc/config.toml");
        eprintln!("     [display]");
        eprintln!("     debug_socket = true");
        anyhow::bail!("Debug socket not available");
    }

    let stream = server::connect_socket(&debug_socket).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Build the swarm task request with optional worker profile
    let profile_str = worker_profile.map(|p| match p {
        WorkerProfileArg::Coder => "coder",
        WorkerProfileArg::Tester => "tester",
        WorkerProfileArg::Reviewer => "reviewer",
        WorkerProfileArg::Researcher => "researcher",
    });

    let request = serde_json::json!({
        "type": "swarm_task",
        "id": 1,
        "message": message,
        "worker_profile": profile_str,
    });

    let mut json = serde_json::to_string(&request)?;
    json.push('\n');
    writer.write_all(json.as_bytes()).await?;

    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        anyhow::bail!("Server disconnected before sending response");
    }

    let response: serde_json::Value = serde_json::from_str(&line)?;

    match response.get("type").and_then(|v| v.as_str()) {
        Some("swarm_response") => {
            let ok = response.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            let output = response.get("output").and_then(|v| v.as_str()).unwrap_or("");

            if ok {
                println!("{}", output);
            } else {
                eprintln!("Error: {}", output);
                std::process::exit(1);
            }
        }
        Some("error") => {
            let message = response.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            eprintln!("Error: {}", message);
            std::process::exit(1);
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}
