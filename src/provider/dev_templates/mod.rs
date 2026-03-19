pub mod attrs_cache;
pub mod fetcher;
pub mod flake_generator;
pub mod nix_parser;
pub mod store_resolver;

use crate::error::{AppError, Result};
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use crate::provider::{EnsureFilesResult, ShellProvider};
use crate::warning::AppWarning;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread;

pub struct DevTemplatesProvider;

const MAX_PIPELINE_CONCURRENCY: usize = 8;

type ProbeLockedKeyFn = dyn Fn(&str) -> Result<String> + Send + Sync;
type ResolveStoreSourceFn =
    dyn Fn(&str) -> Result<store_resolver::ResolvedStoreSource> + Send + Sync;
type LoadCachedAttrsFn =
    dyn Fn(&str, &str) -> Result<(Option<ShellAttrs>, Option<AppWarning>)> + Send + Sync;
type SaveCachedAttrsFn = dyn Fn(&str, &str, &ShellAttrs) -> Option<AppWarning> + Send + Sync;
type ParseFlakeFn = dyn Fn(&str) -> ShellAttrs + Send + Sync;
type WriteFlakeFn = dyn Fn(&Path, &str) -> Result<()> + Send + Sync;

#[derive(Clone)]
struct PipelineDeps {
    probe_locked_key: Arc<ProbeLockedKeyFn>,
    resolve_store_source: Arc<ResolveStoreSourceFn>,
    load_cached_attrs: Arc<LoadCachedAttrsFn>,
    save_cached_attrs: Arc<SaveCachedAttrsFn>,
    parse_flake_shell: Arc<ParseFlakeFn>,
    write_flake: Arc<WriteFlakeFn>,
}

impl PipelineDeps {
    fn production() -> Self {
        let runner = Arc::new(SystemCommandRunner);
        Self {
            probe_locked_key: Arc::new(probe_locked_key),
            resolve_store_source: Arc::new({
                let runner = Arc::clone(&runner);
                move |lang| store_resolver::resolve_store_source(lang, runner.as_ref())
            }),
            load_cached_attrs: Arc::new(attrs_cache::load_cached_attrs),
            save_cached_attrs: Arc::new(attrs_cache::save_cached_attrs),
            parse_flake_shell: Arc::new(nix_parser::parse_flake_shell),
            write_flake: Arc::new(write_flake_to_target),
        }
    }
}

struct SystemCommandRunner;

impl store_resolver::CommandRunner for SystemCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|err| AppError::Provider(format!("failed to execute `{program}`: {err}")))?;

        if output.status.success() {
            String::from_utf8(output.stdout).map_err(|err| {
                AppError::Provider(format!("invalid UTF-8 in `{program}` output: {err}"))
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::Provider(stderr.trim().to_string()))
        }
    }
}

impl ShellProvider for DevTemplatesProvider {
    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<EnsureFilesResult> {
        ensure_files_with_deps(languages, target_dir, &PipelineDeps::production())
    }
}

fn ensure_files_with_deps(
    languages: &[String],
    target_dir: &Path,
    deps: &PipelineDeps,
) -> Result<EnsureFilesResult> {
    let mut seen = HashSet::new();
    let deduped_languages: Vec<String> = languages
        .iter()
        .filter(|lang| seen.insert((*lang).clone()))
        .cloned()
        .collect();
    let pipeline_languages: Vec<String> = deduped_languages
        .iter()
        .filter(|lang| *lang != "empty")
        .cloned()
        .collect();

    let pipeline_concurrency = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, MAX_PIPELINE_CONCURRENCY);

    let mut parsed_attrs = Vec::new();
    let mut warnings: Vec<AppWarning> = Vec::new();

    for chunk in pipeline_languages.chunks(pipeline_concurrency) {
        let mut tasks = Vec::with_capacity(chunk.len());

        for lang in chunk {
            let lang = lang.clone();
            let deps = deps.clone();
            let task = thread::spawn(move || ensure_language_attrs(&lang, &deps));
            tasks.push(task);
        }

        for task in tasks {
            match task.join() {
                Ok(outcome) => {
                    if let Some(attrs) = outcome.attrs {
                        parsed_attrs.push((outcome.language, attrs));
                    }
                    warnings.extend(outcome.warnings);
                }
                Err(_) => {
                    warnings.push(AppWarning::new(
                        "dev_templates.pipeline_panicked",
                        "Template pipeline task panicked unexpectedly",
                    ));
                }
            }
        }
    }

    let flake_content =
        flake_generator::generate_dev_templates_flake(&deduped_languages, &parsed_attrs);
    (deps.write_flake)(target_dir, &flake_content)?;

    Ok(EnsureFilesResult { warnings })
}

struct LanguageOutcome {
    language: String,
    attrs: Option<ShellAttrs>,
    warnings: Vec<AppWarning>,
}

fn ensure_language_attrs(language: &str, deps: &PipelineDeps) -> LanguageOutcome {
    let mut warnings = Vec::new();

    let probed_locked_key = match (deps.probe_locked_key)(language) {
        Ok(key) => Some(key),
        Err(err) => {
            warnings.push(
                AppWarning::new("dev_templates.lock_probe_failed", err.to_string())
                    .with_context("language", language.to_string()),
            );
            None
        }
    };

    if let Some(locked_key) = probed_locked_key {
        match (deps.load_cached_attrs)(language, &locked_key) {
            Ok((Some(attrs), maybe_warning)) => {
                if let Some(warning) = maybe_warning {
                    warnings.push(warning);
                }
                return LanguageOutcome {
                    language: language.to_string(),
                    attrs: Some(attrs),
                    warnings,
                };
            }
            Ok((None, maybe_warning)) => {
                if let Some(warning) = maybe_warning {
                    warnings.push(warning);
                }
            }
            Err(err) => {
                warnings.push(
                    AppWarning::new("dev_templates.attrs_cache_load_failed", err.to_string())
                        .with_context("language", language.to_string()),
                );
            }
        }
    }

    match (deps.resolve_store_source)(language) {
        Ok(resolved) => {
            let attrs = (deps.parse_flake_shell)(&resolved.flake_source);
            if let Some(warning) = (deps.save_cached_attrs)(language, &resolved.locked_key, &attrs)
            {
                warnings.push(warning);
            }

            LanguageOutcome {
                language: language.to_string(),
                attrs: Some(attrs),
                warnings,
            }
        }
        Err(err) => {
            warnings.push(
                AppWarning::new("dev_templates.resolve_failed", err.to_string())
                    .with_context("language", language.to_string()),
            );

            LanguageOutcome {
                language: language.to_string(),
                attrs: None,
                warnings,
            }
        }
    }
}

fn probe_locked_key(lang: &str) -> Result<String> {
    let flake_ref = format!("github:the-nix-way/dev-templates?dir={lang}");
    let output = Command::new("nix")
        .args(["flake", "prefetch", "--json", flake_ref.as_str()])
        .output()
        .map_err(|err| {
            AppError::Provider(format!(
                "failed to probe lock key for language `{lang}`: {err}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Provider(format!(
            "failed to probe lock key for language `{lang}`: {}",
            stderr.trim()
        )));
    }

    let raw = String::from_utf8(output.stdout)
        .map_err(|err| AppError::Provider(format!("invalid UTF-8 in lock probe output: {err}")))?;

    extract_locked_key_from_prefetch_json(&raw)
}

fn extract_locked_key_from_prefetch_json(raw: &str) -> Result<String> {
    let parsed: Value = serde_json::from_str(raw)?;

    parsed
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
        .ok_or_else(|| AppError::Provider("lock key missing".to_string()))
}

fn write_flake_to_target(target_dir: &Path, content: &str) -> Result<()> {
    std::fs::write(target_dir.join("flake.nix"), content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Default)]
    struct TestState {
        probe_calls: Mutex<Vec<String>>,
        resolve_calls: Mutex<Vec<String>>,
        parse_calls: Mutex<Vec<String>>,
        save_calls: Mutex<Vec<(String, String)>>,
    }

    fn test_target_dir(test_name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ah-dev-templates-{test_name}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("test target directory should be creatable");
        dir
    }

    fn attrs_with_env_key(key: &str) -> ShellAttrs {
        ShellAttrs {
            env: vec![(key.to_string(), "\"1\"".to_string())],
            extra_attrs: Vec::new(),
        }
    }

    fn read_flake(target_dir: &Path) -> String {
        std::fs::read_to_string(target_dir.join("flake.nix")).expect("flake should be written")
    }

    fn build_test_deps(
        state: Arc<TestState>,
        probe_map: HashMap<String, Result<String>>,
        cache_map: HashMap<(String, String), Option<ShellAttrs>>,
        resolve_map: HashMap<String, Result<store_resolver::ResolvedStoreSource>>,
        parse_map: HashMap<String, ShellAttrs>,
    ) -> PipelineDeps {
        let probe_map = Arc::new(Mutex::new(probe_map));
        let cache_map = Arc::new(cache_map);
        let resolve_map = Arc::new(Mutex::new(resolve_map));
        let parse_map = Arc::new(parse_map);

        PipelineDeps {
            probe_locked_key: Arc::new({
                let state = Arc::clone(&state);
                let probe_map = Arc::clone(&probe_map);
                move |lang| {
                    state
                        .probe_calls
                        .lock()
                        .expect("probe calls lock")
                        .push(lang.to_string());
                    probe_map
                        .lock()
                        .expect("probe map lock")
                        .remove(lang)
                        .unwrap_or_else(|| {
                            Err(AppError::Provider(format!("missing probe for {lang}")))
                        })
                }
            }),
            resolve_store_source: Arc::new({
                let state = Arc::clone(&state);
                let resolve_map = Arc::clone(&resolve_map);
                move |lang| {
                    state
                        .resolve_calls
                        .lock()
                        .expect("resolve calls lock")
                        .push(lang.to_string());
                    resolve_map
                        .lock()
                        .expect("resolve map lock")
                        .remove(lang)
                        .unwrap_or_else(|| {
                            Err(AppError::Provider(format!("missing resolve for {lang}")))
                        })
                }
            }),
            load_cached_attrs: Arc::new({
                let cache_map = Arc::clone(&cache_map);
                move |lang, locked_key| {
                    Ok((
                        cache_map
                            .get(&(lang.to_string(), locked_key.to_string()))
                            .cloned()
                            .flatten(),
                        None,
                    ))
                }
            }),
            save_cached_attrs: Arc::new({
                let state = Arc::clone(&state);
                move |lang, locked_key, _attrs| {
                    state
                        .save_calls
                        .lock()
                        .expect("save calls lock")
                        .push((lang.to_string(), locked_key.to_string()));
                    None
                }
            }),
            parse_flake_shell: Arc::new({
                let state = Arc::clone(&state);
                let parse_map = Arc::clone(&parse_map);
                move |source| {
                    state
                        .parse_calls
                        .lock()
                        .expect("parse calls lock")
                        .push(source.to_string());
                    parse_map
                        .get(source)
                        .cloned()
                        .unwrap_or_else(|| attrs_with_env_key("UNMAPPED_SOURCE"))
                }
            }),
            write_flake: Arc::new(write_flake_to_target),
        }
    }

    #[test]
    fn ensure_files_on_full_cache_hit_probes_lock_only_without_reading_store_source() {
        let state = Arc::new(TestState::default());
        let deps = build_test_deps(
            Arc::clone(&state),
            HashMap::from([
                ("rust".to_string(), Ok("key-rust".to_string())),
                ("go".to_string(), Ok("key-go".to_string())),
            ]),
            HashMap::from([
                (
                    ("rust".to_string(), "key-rust".to_string()),
                    Some(attrs_with_env_key("RUST_CACHE_HIT")),
                ),
                (
                    ("go".to_string(), "key-go".to_string()),
                    Some(attrs_with_env_key("GO_CACHE_HIT")),
                ),
            ]),
            HashMap::new(),
            HashMap::new(),
        );

        let target_dir = test_target_dir("full-cache-hit");
        let result =
            ensure_files_with_deps(&["rust".to_string(), "go".to_string()], &target_dir, &deps)
                .expect("ensure_files should succeed");

        assert!(result.warnings.is_empty());
        assert_eq!(state.probe_calls.lock().expect("probe calls lock").len(), 2);
        assert!(
            state
                .resolve_calls
                .lock()
                .expect("resolve calls lock")
                .is_empty(),
            "cache hit should avoid store source resolving"
        );
        assert!(
            state
                .parse_calls
                .lock()
                .expect("parse calls lock")
                .is_empty(),
            "cache hit should avoid store source parsing"
        );

        let flake = read_flake(&target_dir);
        assert!(flake.contains("RUST_CACHE_HIT = shells.\"rust\".RUST_CACHE_HIT;"));
        assert!(flake.contains("GO_CACHE_HIT = shells.\"go\".GO_CACHE_HIT;"));
    }

    #[test]
    fn ensure_files_resolves_missing_languages_via_prefetch_path() {
        let state = Arc::new(TestState::default());
        let deps = build_test_deps(
            Arc::clone(&state),
            HashMap::from([
                ("rust".to_string(), Ok("key-rust".to_string())),
                ("go".to_string(), Ok("key-go-initial".to_string())),
            ]),
            HashMap::from([(
                ("rust".to_string(), "key-rust".to_string()),
                Some(attrs_with_env_key("RUST_CACHE_HIT")),
            )]),
            HashMap::from([(
                "go".to_string(),
                Ok(store_resolver::ResolvedStoreSource {
                    locked_key: "key-go-prefetched".to_string(),
                    flake_source: "go-store-source".to_string(),
                }),
            )]),
            HashMap::from([(
                "go-store-source".to_string(),
                attrs_with_env_key("GO_PREFETCHED"),
            )]),
        );

        let target_dir = test_target_dir("prefetch-miss");
        let result =
            ensure_files_with_deps(&["rust".to_string(), "go".to_string()], &target_dir, &deps)
                .expect("ensure_files should succeed");

        assert!(result.warnings.is_empty());
        assert_eq!(
            state
                .resolve_calls
                .lock()
                .expect("resolve calls lock")
                .as_slice(),
            ["go".to_string()]
        );
        assert_eq!(
            state.save_calls.lock().expect("save calls lock").as_slice(),
            [("go".to_string(), "key-go-prefetched".to_string())]
        );

        let flake = read_flake(&target_dir);
        assert!(flake.contains("RUST_CACHE_HIT = shells.\"rust\".RUST_CACHE_HIT;"));
        assert!(flake.contains("GO_PREFETCHED = shells.\"go\".GO_PREFETCHED;"));
    }

    #[test]
    fn ensure_files_collects_warning_and_still_writes_flake_on_partial_failure() {
        let state = Arc::new(TestState::default());
        let deps = build_test_deps(
            Arc::clone(&state),
            HashMap::from([
                ("rust".to_string(), Ok("key-rust".to_string())),
                ("go".to_string(), Ok("key-go".to_string())),
            ]),
            HashMap::from([(
                ("rust".to_string(), "key-rust".to_string()),
                Some(attrs_with_env_key("RUST_CACHE_HIT")),
            )]),
            HashMap::from([(
                "go".to_string(),
                Err(AppError::Provider("resolver failed".to_string())),
            )]),
            HashMap::new(),
        );

        let target_dir = test_target_dir("partial-failure");
        let result =
            ensure_files_with_deps(&["rust".to_string(), "go".to_string()], &target_dir, &deps)
                .expect("ensure_files should still succeed on partial failure");

        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].code, "dev_templates.resolve_failed");
        assert!(
            result.warnings[0]
                .context
                .contains(&("language".to_string(), "go".to_string()))
        );

        let flake = read_flake(&target_dir);
        assert!(flake.contains("RUST_CACHE_HIT = shells.\"rust\".RUST_CACHE_HIT;"));
    }
}
