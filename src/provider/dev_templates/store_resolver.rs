use crate::error::{AppError, Result};
use serde_json::Value;

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStoreSource {
    pub locked_key: String,
    pub flake_source: String,
}

pub fn extract_locked_key(raw: &str) -> Result<String> {
    let parsed: Value = serde_json::from_str(raw)?;

    parsed
        .get("narHash")
        .and_then(Value::as_str)
        .or_else(|| parsed.get("rev").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::Provider("lock key missing".to_string()))
}

pub fn resolve_store_source(lang: &str, runner: &dyn CommandRunner) -> Result<ResolvedStoreSource> {
    let lock_raw = query_lock_data(lang, runner)?;

    let (locked_key, store_path) = match extract_lock_and_store_path(&lock_raw) {
        Ok(parts) => parts,
        Err(_) => {
            prefetch(lang, runner)?;
            let retried_lock_raw = query_lock_data(lang, runner)?;
            extract_lock_and_store_path(&retried_lock_raw)?
        }
    };

    let flake_path = format!("{store_path}/flake.nix");
    let flake_source = runner.run("cat", &[flake_path.as_str()])?;

    Ok(ResolvedStoreSource {
        locked_key,
        flake_source,
    })
}

fn query_lock_data(lang: &str, runner: &dyn CommandRunner) -> Result<String> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");
    runner.run("nix", &["flake", "prefetch", "--json", flake_ref.as_str()])
}

fn prefetch(lang: &str, runner: &dyn CommandRunner) -> Result<()> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");
    let _ = runner.run("nix", &["prefetch", flake_ref.as_str()])?;
    Ok(())
}

fn extract_lock_and_store_path(raw: &str) -> Result<(String, String)> {
    let parsed: Value = serde_json::from_str(raw)?;

    let key = parsed
        .get("locked")
        .and_then(Value::as_object)
        .and_then(|locked| {
            locked
                .get("narHash")
                .and_then(Value::as_str)
                .or_else(|| locked.get("rev").and_then(Value::as_str))
        })
        .or_else(|| parsed.get("narHash").and_then(Value::as_str))
        .or_else(|| parsed.get("rev").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::Provider("lock key missing".to_string()))?;

    let path = parsed
        .get("path")
        .and_then(Value::as_str)
        .or_else(|| parsed.get("storePath").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::Provider("store path missing".to_string()))?;

    Ok((key, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Debug)]
    struct CommandReply {
        output: std::result::Result<String, String>,
    }

    #[derive(Clone, Debug)]
    struct FakeRunner {
        replies: Arc<Mutex<VecDeque<CommandReply>>>,
        calls: Arc<Mutex<Vec<(String, Vec<String>)>>>,
    }

    impl FakeRunner {
        fn with_outputs(outputs: Vec<Result<String>>) -> Self {
            let replies = outputs
                .into_iter()
                .map(|output| CommandReply {
                    output: output.map_err(|err| err.to_string()),
                })
                .collect();

            Self {
                replies: Arc::new(Mutex::new(replies)),
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().expect("calls lock poisoned").clone()
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<String> {
            self.calls.lock().expect("calls lock poisoned").push((
                program.to_string(),
                args.iter().map(|arg| arg.to_string()).collect(),
            ));

            self.replies
                .lock()
                .expect("replies lock poisoned")
                .pop_front()
                .expect("missing fake reply")
                .output
                .clone()
                .map_err(AppError::Provider)
        }
    }

    #[test]
    fn resolver_supports_injected_command_runner() {
        let runner =
            FakeRunner::with_outputs(vec![Err(AppError::Provider("expected".to_string()))]);

        let result = resolve_store_source("rust", &runner);

        assert!(result.is_err());
    }

    #[test]
    fn extract_locked_key_prefers_nar_hash() {
        let raw = r#"{"narHash":"sha256-abc","rev":"deadbeef"}"#;

        assert_eq!(
            extract_locked_key(raw).expect("must extract key"),
            "sha256-abc"
        );
    }

    #[test]
    fn extract_locked_key_falls_back_to_rev() {
        let raw = r#"{"rev":"deadbeef"}"#;

        assert_eq!(
            extract_locked_key(raw).expect("must extract key"),
            "deadbeef"
        );
    }

    #[test]
    fn resolve_retries_after_prefetch_when_lock_data_missing() {
        let runner = FakeRunner::with_outputs(vec![
            Ok(r#"{"locked":{}}"#.to_string()),
            Ok(r#"{"path":"/nix/store/prefetched-source"}"#.to_string()),
            Ok(r#"{"locked":{"narHash":"sha256-prefetched"},"path":"/nix/store/prefetched-source"}"#.to_string()),
            Ok("# prefetched flake".to_string()),
        ]);

        let resolved = resolve_store_source("rust", &runner).expect("must resolve after prefetch");

        assert_eq!(resolved.locked_key, "sha256-prefetched");
        assert_eq!(resolved.flake_source, "# prefetched flake");

        let calls = runner.calls();
        assert_eq!(
            calls.len(),
            4,
            "resolver should query, prefetch, retry, read"
        );
        assert!(
            calls
                .iter()
                .any(|(program, args)| program == "nix" && args.iter().any(|arg| arg == "prefetch")),
            "expected prefetch command to be invoked"
        );
    }
}
