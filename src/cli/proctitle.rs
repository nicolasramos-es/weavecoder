//! Mapping from parsed CLI arguments to an initial process title.
//!
//! This logic depends on the clap `Args`/`Command` types defined in `cli`, so
//! it lives in the CLI layer. The low-level title-setting primitives it uses
//! (`compact_process_title`, `session_name`, `set_title`) live in the
//! `process_title` core module.

use crate::cli::args::{AmbientCommand, Args, Command};
use crate::process_title::{compact_process_title, session_name, set_title};

pub(crate) fn initial_title(args: &Args) -> String {
    match &args.command {
        Some(Command::Serve { .. }) => "wvc:server".to_string(),
        Some(Command::Acp) => "wvc acp".to_string(),
        Some(Command::Server { .. }) => "wvc server".to_string(),
        Some(Command::Connect) => "wvc:client".to_string(),
        #[cfg(unix)]
        Some(Command::ApiBridge { .. }) => "wvc api-bridge".to_string(),
        Some(Command::Run { .. }) => "wvc run".to_string(),
        Some(Command::Login { .. }) => "wvc login".to_string(),
        Some(Command::Account { .. }) => "wvc account".to_string(),
        Some(Command::Repl) => "wvc repl".to_string(),
        Some(Command::Update) => "wvc update".to_string(),
        Some(Command::Version { .. }) => "wvc version".to_string(),
        Some(Command::Usage { .. }) => "wvc usage".to_string(),
        Some(Command::SelfDev { .. }) => "wvc:selfdev".to_string(),
        Some(Command::Debug { .. }) => "wvc debug".to_string(),
        Some(Command::Auth(_)) => "wvc auth".to_string(),
        Some(Command::Provider(_)) => "wvc provider".to_string(),
        Some(Command::Memory(_)) => "wvc memory".to_string(),
        Some(Command::Session(_)) => "wvc session".to_string(),
        Some(Command::Ambient(subcommand)) => match subcommand {
            AmbientCommand::RunVisible => "wvc ambient visible".to_string(),
            _ => "wvc ambient".to_string(),
        },
        Some(Command::Cloud(_)) => "wvc cloud".to_string(),
        Some(Command::Pair { .. }) => "wvc pair".to_string(),
        Some(Command::Permissions) => "wvc permissions".to_string(),
        Some(Command::Transcript { .. }) => "wvc transcript".to_string(),
        Some(Command::Dictate { .. }) => "wvc dictate".to_string(),
        Some(Command::SetupHotkey {
            listen_macos_hotkey,
            notify_cli_launch,
            listen_windows_hotkey,
            uninstall,
        }) => {
            if *listen_macos_hotkey || *listen_windows_hotkey {
                "wvc hotkey listener".to_string()
            } else if notify_cli_launch.is_some() {
                "wvc shortcut reminder".to_string()
            } else if *uninstall {
                "wvc hotkey uninstall".to_string()
            } else {
                "wvc hotkey setup".to_string()
            }
        }
        Some(Command::Browser { .. }) => "wvc browser".to_string(),
        Some(Command::Replay { .. }) => "wvc replay".to_string(),
        Some(Command::Model(_)) => "wvc model".to_string(),
        Some(Command::ProviderTestCoverage { .. }) => "wvc provider-test-coverage".to_string(),
        Some(Command::ProviderDoctor { .. }) => "wvc provider-doctor".to_string(),
        Some(Command::Init { .. }) => "wvc init".to_string(),
        Some(Command::AuthTest { .. }) => "wvc auth-test".to_string(),
        Some(Command::Restart { .. }) => "wvc restart".to_string(),
        Some(Command::Menubar { .. }) => "wvc menubar".to_string(),
        Some(Command::SetupLauncher) => "wvc setup-launcher".to_string(),
        None => {
            if let Some(resume) = args.resume.as_deref().filter(|resume| !resume.is_empty()) {
                let prefix = if crate::cli::selfdev::client_selfdev_requested() {
                    "wvc:d:"
                } else {
                    "wvc:c:"
                };
                compact_process_title(prefix, Some(&session_name(resume)))
            } else if crate::cli::selfdev::client_selfdev_requested() {
                "wvc:selfdev".to_string()
            } else {
                "wvc:client".to_string()
            }
        }
    }
}

pub(crate) fn set_initial_title(args: &Args) {
    set_title(initial_title(args));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::lock_test_env;
    use clap::Parser;

    const SELFDEV_ENV: &str = wvc_selfdev_types::CLIENT_SELFDEV_ENV;

    fn with_selfdev_env_removed<T>(f: impl FnOnce() -> T) -> T {
        let _guard = lock_test_env();
        let previous = std::env::var_os(SELFDEV_ENV);
        crate::env::remove_var(SELFDEV_ENV);
        let result = f();
        if let Some(value) = previous {
            crate::env::set_var(SELFDEV_ENV, value);
        }
        result
    }

    #[test]
    fn initial_title_labels_server() {
        with_selfdev_env_removed(|| {
            let args = Args::parse_from(["wvc", "serve"]);
            assert_eq!(initial_title(&args), "wvc:server");
        });
    }

    #[test]
    fn initial_title_labels_resume_client_with_short_name() {
        with_selfdev_env_removed(|| {
            let args = Args::parse_from(["wvc", "--resume", "session_fox_123"]);
            assert_eq!(initial_title(&args), "wvc:c:fox");
        });
    }

    #[test]
    fn initial_title_labels_selfdev_command() {
        with_selfdev_env_removed(|| {
            let args = Args::parse_from(["wvc", "self-dev"]);
            assert_eq!(initial_title(&args), "wvc:selfdev");
        });
    }

    #[test]
    fn initial_title_labels_windows_hotkey_listener() {
        let args = Args::parse_from(["wvc", "setup-hotkey", "--listen-windows-hotkey"]);
        assert_eq!(initial_title(&args), "wvc hotkey listener");
    }

    #[test]
    fn initial_title_labels_hotkey_uninstall() {
        let args = Args::parse_from(["wvc", "setup-hotkey", "--uninstall"]);
        assert_eq!(initial_title(&args), "wvc hotkey uninstall");
    }
}
