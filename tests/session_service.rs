use ah::providers::ProviderType;
use ah::session::SessionService;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

#[test]
fn create_session_returns_dir() {
    let cache_root = unique_temp_dir("ah-session-service");
    fs::create_dir_all(&cache_root).expect("cache root should be created");

    let original_cache = std::env::var("XDG_CACHE_HOME").ok();
    unsafe {
        std::env::set_var("XDG_CACHE_HOME", &cache_root);
    }

    let result = SessionService::create_session(ProviderType::Devenv, vec!["rust".to_string()])
        .expect("session should be created");

    assert!(result.session_dir.is_dir());
    assert!(result.warnings.is_empty());

    let _ = fs::remove_dir_all(&cache_root);

    match original_cache {
        Some(value) => unsafe {
            std::env::set_var("XDG_CACHE_HOME", value);
        },
        None => unsafe {
            std::env::remove_var("XDG_CACHE_HOME");
        },
    }
}
