use crate::error::{AppError, Result};
use serde_json::Value;
use std::fs;

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<String>;
}

trait FlakeReader {
    fn read_to_string(&self, path: &str) -> std::io::Result<String>;
}

struct FsFlakeReader;

impl FlakeReader for FsFlakeReader {
    fn read_to_string(&self, path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
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
    resolve_store_source_with_reader(lang, runner, &FsFlakeReader)
}

fn resolve_store_source_with_reader(
    lang: &str,
    runner: &dyn CommandRunner,
    reader: &dyn FlakeReader,
) -> Result<ResolvedStoreSource> {
    let lock_raw = query_lock_data(lang, runner)?;

    let (locked_key, store_path) = match extract_lock_and_store_path(&lock_raw) {
        Ok(parts) => parts,
        Err(_) => {
            prefetch(lang, runner)?;
            let retried_lock_raw = query_lock_data(lang, runner)?;
            extract_lock_and_store_path(&retried_lock_raw)?
        }
    };

    let flake_source = read_store_flake_with_reader(&store_path, reader)?;

    Ok(ResolvedStoreSource {
        locked_key,
        flake_source,
    })
}

fn query_lock_data(lang: &str, runner: &dyn CommandRunner) -> Result<String> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");
    runner
        .run("nix", &["flake", "prefetch", "--json", flake_ref.as_str()])
        .map_err(|err| map_command_failure(lang, "query lock data", err))
}

fn prefetch(lang: &str, runner: &dyn CommandRunner) -> Result<()> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");
    runner
        .run("nix", &["prefetch", flake_ref.as_str()])
        .map_err(|err| map_command_failure(lang, "prefetch flake input", err))?;
    Ok(())
}

fn read_store_flake_with_reader(store_path: &str, reader: &dyn FlakeReader) -> Result<String> {
    let flake_path = format!("{store_path}/flake.nix");
    reader
        .read_to_string(&flake_path)
        .map_err(|err| AppError::Provider(format!("failed to read {flake_path}: {err}")))
}

fn map_command_failure(lang: &str, action: &str, err: AppError) -> AppError {
    let summary = summarize_error(&err);
    AppError::Provider(format!(
        "failed to {action} for language `{lang}`: {summary}"
    ))
}

fn summarize_error(err: &AppError) -> String {
    match err {
        AppError::Provider(message) => message
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .unwrap_or_else(|| "unknown provider error".to_string()),
        _ => err.to_string(),
    }
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
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Debug)]
    struct CommandReply {
        output: std::result::Result<String, String>,
    }

    type ProgramArgsCall = (String, Vec<String>);

    #[derive(Clone, Debug)]
    struct FakeRunner {
        replies: Arc<Mutex<VecDeque<CommandReply>>>,
        calls: Arc<Mutex<Vec<ProgramArgsCall>>>,
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

    #[derive(Clone, Debug)]
    struct FakeFlakeReader {
        files: Arc<Mutex<HashMap<String, std::io::Result<String>>>>,
    }

    impl FakeFlakeReader {
        fn with_files(files: Vec<(String, std::io::Result<String>)>) -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::from_iter(files))),
            }
        }
    }

    impl FlakeReader for FakeFlakeReader {
        fn read_to_string(&self, path: &str) -> std::io::Result<String> {
            self.files
                .lock()
                .expect("files lock poisoned")
                .remove(path)
                .unwrap_or_else(|| {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("missing fake file: {path}"),
                    ))
                })
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
        ]);
        let reader = FakeFlakeReader::with_files(vec![(
            "/nix/store/prefetched-source/flake.nix".to_string(),
            Ok("{ description = \"prefetched\"; }".to_string()),
        )]);

        let resolved = resolve_store_source_with_reader("rust", &runner, &reader)
            .expect("resolver should complete after retry and fake read");

        assert_eq!(resolved.locked_key, "sha256-prefetched");
        assert_eq!(resolved.flake_source, "{ description = \"prefetched\"; }");

        let calls = runner.calls();
        assert_eq!(
            calls.len(),
            3,
            "resolver should query, prefetch, retry before store read"
        );
        assert!(
            calls
                .iter()
                .any(|(program, args)| program == "nix" && args.iter().any(|arg| arg == "prefetch")),
            "expected prefetch command to be invoked"
        );
    }

    #[test]
    fn read_store_flake_returns_provider_error_when_missing() {
        let reader = FakeFlakeReader::with_files(vec![]);
        let err = read_store_flake_with_reader("/nix/store/ah-missing-store-source", &reader)
            .unwrap_err();

        assert!(matches!(err, AppError::Provider(_)));
        assert!(err.to_string().contains("flake.nix"));
    }

    #[test]
    fn command_failure_maps_to_provider_error_with_context() {
        let runner = FakeRunner::with_outputs(vec![Err(AppError::Provider(
            "error: unable to download source\nexit status: 1".to_string(),
        ))]);

        let err = resolve_store_source("rust", &runner).unwrap_err();

        assert!(matches!(err, AppError::Provider(_)));
        let rendered = err.to_string();
        assert!(rendered.contains("rust"));
        assert!(rendered.contains("unable to download source"));
    }
}
