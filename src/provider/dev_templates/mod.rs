pub mod attrs_cache;
pub mod flake_generator;
pub mod nix_parser;
pub mod store_resolver;

use crate::error::Result;
use crate::provider::dev_templates::nix_parser::ShellAttrs;
use crate::provider::{EnsureFilesResult, ShellProvider};
use crate::warning::AppWarning;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::thread;

pub struct DevTemplatesProvider;

const MAX_PIPELINE_CONCURRENCY: usize = 8;

type ResolveStoreSourceFn =
    dyn Fn(&str) -> Result<store_resolver::ResolvedStoreSource> + Send + Sync;
type LoadCachedAttrsFn =
    dyn Fn(&str, &str) -> Result<(Option<ShellAttrs>, Option<AppWarning>)> + Send + Sync;
type SaveCachedAttrsFn = dyn Fn(&str, &str, &ShellAttrs) -> Option<AppWarning> + Send + Sync;
type ParseFlakeFn = dyn Fn(&str) -> ShellAttrs + Send + Sync;
type WriteFlakeFn = dyn Fn(&Path, &str) -> Result<()> + Send + Sync;

#[derive(Clone)]
struct PipelineDeps {
    resolve_store_source: Arc<ResolveStoreSourceFn>,
    load_cached_attrs: Arc<LoadCachedAttrsFn>,
    save_cached_attrs: Arc<SaveCachedAttrsFn>,
    parse_flake_shell: Arc<ParseFlakeFn>,
    write_flake: Arc<WriteFlakeFn>,
}

impl PipelineDeps {
    fn production() -> Self {
        Self {
            resolve_store_source: Arc::new(store_resolver::resolve_store_source),
            load_cached_attrs: Arc::new(attrs_cache::load_cached_attrs),
            save_cached_attrs: Arc::new(attrs_cache::save_cached_attrs),
            parse_flake_shell: Arc::new(nix_parser::parse_flake_shell),
            write_flake: Arc::new(write_flake_to_target),
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

    let resolved = match (deps.resolve_store_source)(language) {
        Ok(resolved) => resolved,
        Err(err) => {
            warnings.push(
                AppWarning::new("dev_templates.resolve_failed", err.to_string())
                    .with_context("language", language.to_string()),
            );

            return LanguageOutcome {
                language: language.to_string(),
                attrs: None,
                warnings,
            };
        }
    };

    match (deps.load_cached_attrs)(language, &resolved.locked_key) {
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

    let attrs = (deps.parse_flake_shell)(&resolved.flake_source);
    if let Some(warning) = (deps.save_cached_attrs)(language, &resolved.locked_key, &attrs) {
        warnings.push(warning);
    }

    LanguageOutcome {
        language: language.to_string(),
        attrs: Some(attrs),
        warnings,
    }
}

fn write_flake_to_target(target_dir: &Path, content: &str) -> Result<()> {
    std::fs::write(target_dir.join("flake.nix"), content)?;
    Ok(())
}
