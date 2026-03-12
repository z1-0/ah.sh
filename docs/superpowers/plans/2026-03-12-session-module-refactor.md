# Session Module Refactor Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development
> (if subagents available) or superpowers:executing-plans to implement this plan.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将会话模块重构为 `src/session/` 功能分组目录结构，并保持
对外 API 与调用链不变。

**Architecture:** 使用 `session/mod.rs` 统一声明并 re-export 类型与服务，
`types` 提供数据结构与解析，`storage` 负责文件系统读写，`service`
承担业务流程。外部引用统一走 `crate::session::*`。

**Tech Stack:** Rust、Cargo、serde_json、blake3、thiserror

---

## Chunk 1: 文件结构与模块拆分

### Task 1: 新建会话模块目录结构

**Files:**

- Create: `src/session/mod.rs`
- Create: `src/session/types.rs`
- Modify: `src/lib.rs`
- Create: `tests/session_types.rs`

- [ ] **Step 1: 写失败测试（SessionKey 解析）**

```rust
#[test]
fn session_key_parses_id() {
    let key: crate::session::SessionKey = "abcdef12".parse().unwrap();
    assert!(matches!(key, crate::session::SessionKey::Id(_)));
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test session_key_parses_id`
Expected: FAIL（`SessionKey` 尚未实现）

- [ ] **Step 3: 创建最小模块骨架并导出**

```rust
// src/session/mod.rs
mod types;

pub use types::{Session, SessionError, SessionKey, SESSION_ID_LEN};
```

```rust
// src/lib.rs
pub mod session;
```

- [ ] **Step 4: 实现 `types.rs`（仅满足测试）**

```rust
// src/session/types.rs
pub const SESSION_ID_LEN: usize = 8;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("invalid session key: {0}")]
    InvalidSelector(String),
    #[error("session '{0}' not found")]
    NotFound(String),
}

#[derive(Debug, Clone)]
pub enum SessionKey {
    Index(usize),
    Id(String),
}

impl std::fmt::Display for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionKey::Index(i) => write!(f, "{i}"),
            SessionKey::Id(id) => write!(f, "{id}"),
        }
    }
}

impl std::str::FromStr for SessionKey {
    type Err = crate::error::AppError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(SessionError::InvalidSelector(
                "session target cannot be empty".to_string(),
            )
            .into());
        }

        if input.chars().all(|c| c.is_ascii_digit()) {
            let index = input
                .parse::<usize>()
                .map_err(|_| SessionError::InvalidSelector("invalid session index".to_string()))?;
            if index == 0 {
                return Err(SessionError::InvalidSelector(
                    "session index must be greater than 0".to_string(),
                )
                .into());
            }
            return Ok(SessionKey::Index(index));
        }

        if !input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SessionError::InvalidSelector(
                "session id must contain only hexadecimal characters".to_string(),
            )
            .into());
        }

        if input.len() != SESSION_ID_LEN {
            return Err(SessionError::InvalidSelector(format!(
                "session id must be exactly {} hexadecimal characters",
                SESSION_ID_LEN
            ))
            .into());
        }

        Ok(SessionKey::Id(input.to_string()))
    }
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test session_key_parses_id`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add src/lib.rs src/session/mod.rs src/session/types.rs tests/session_types.rs

git commit -m "refactor: add session module types"
```

### Task 2: 扩展 types 与迁移 storage 逻辑

**Files:**

- Modify: `src/session/types.rs`
- Create: `src/session/storage.rs`
- Modify: `src/session/mod.rs`

- [ ] **Step 1: 写失败测试（Session::new 设置 created_at）**

```rust
#[test]
fn session_new_sets_created_at() {
    let session = crate::session::Session::new(
        "id".to_string(),
        vec!["rust".to_string()],
        "provider".to_string(),
    );
    assert!(session.created_at > 0);
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test session_new_sets_created_at`
Expected: FAIL（`Session` 尚未实现）

- [ ] **Step 3: 实现 `Session` 结构体与构造**

```rust
// src/session/types.rs
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub languages: Vec<String>,
    pub provider: String,
    pub created_at: u64,
}

impl Session {
    pub fn new(id: String, languages: Vec<String>, provider: String) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id,
            languages,
            provider,
            created_at,
        }
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test session_new_sets_created_at`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
git add src/session/types.rs tests/session_types.rs

git commit -m "refactor: add session struct"
```

- [ ] **Step 6: 写失败测试（storage 不对外暴露）**

- 由于 `storage` 仅 `pub(crate)`，不新增对外测试。
- 改为新增 `tests/session_service_smoke.rs`，在后续通过 `SessionService`
  行为覆盖。

- [ ] **Step 7: 迁移 storage 实现并设为 `pub(crate)`**

```rust
// src/session/storage.rs
use crate::error::Result;
use crate::paths::{get_xdg_dir, XdgDir};
use crate::session::{Session, SessionError, SessionKey, SESSION_ID_LEN};
use std::fs;
use std::path::PathBuf;

pub(crate) fn get_session_dir() -> Result<PathBuf> {
    let dir = get_xdg_dir(XdgDir::Cache)?.join("sessions");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub(crate) fn generate_id(provider: &str, languages: &[String]) -> String {
    let mut sorted_langs = languages.to_vec();
    sorted_langs.sort();

    let input = format!("{}:{}", provider, sorted_langs.join(","));
    let digest = blake3::hash(input.as_bytes());
    digest.to_hex().to_string()[..SESSION_ID_LEN].to_string()
}

pub(crate) fn list_sessions() -> Result<Vec<Session>> {
    let session_dir = get_session_dir()?;
    let mut sessions = Vec::new();

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let meta_path = path.join("metadata.json");
            if meta_path.exists() {
                let content = fs::read_to_string(&meta_path)?;
                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                    sessions.push(session);
                }
            }
        }
    }

    sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(sessions)
}

pub(crate) fn save_session(session: &Session) -> Result<()> {
    let session_path = get_session_dir()?.join(&session.id);
    if !session_path.exists() {
        fs::create_dir_all(&session_path)?;
    }
    let meta_path = session_path.join("metadata.json");
    let content = serde_json::to_string_pretty(session)?;
    fs::write(&meta_path, content)?;
    Ok(())
}

pub(crate) fn resolve_session(
    sessions: &[Session],
    key: &SessionKey,
) -> Result<Session> {
    match key {
        SessionKey::Index(idx) => {
            if *idx > 0 && *idx <= sessions.len() {
                Ok(sessions[idx - 1].clone())
            } else {
                Err(SessionError::NotFound(key.to_string()).into())
            }
        }
        SessionKey::Id(id) => sessions
            .iter()
            .find(|s| s.id == *id)
            .cloned()
            .ok_or_else(|| SessionError::NotFound(id.clone()).into()),
    }
}

pub(crate) fn find_session(key: &SessionKey) -> Result<Session> {
    let sessions = list_sessions()?;
    resolve_session(&sessions, key)
}

pub(crate) fn delete_session(session_id: &str) -> Result<bool> {
    let session_path = get_session_dir()?.join(session_id);
    if !session_path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(session_path)?;
    Ok(true)
}

pub(crate) fn clear_sessions() -> Result<usize> {
    let session_dir = get_session_dir()?;
    let mut removed = 0usize;

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
            removed += 1;
        }
    }

    Ok(removed)
}
```

```rust
// src/session/mod.rs
mod storage;
```

- [ ] **Step 8: 运行类型测试回归**

Run: `cargo test session_key_parses_id session_new_sets_created_at`
Expected: PASS

- [ ] **Step 9: 提交**

```bash
git add src/session/storage.rs src/session/mod.rs

git commit -m "refactor: add session storage"
```

## Chunk 2: Service 迁移与引用更新

### Task 3: 迁移 SessionService

**Files:**

- Create: `src/session/service.rs`
- Modify: `src/session/mod.rs`
- Create: `tests/session_service.rs`

- [ ] **Step 1: 写失败测试（创建会话）**

```rust
#[test]
fn create_session_returns_dir() {
    let dir = crate::session::SessionService::create_session(
        crate::providers::ProviderType::DevTemplates,
        vec!["rust".to_string()],
    );
    assert!(dir.is_ok());
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test create_session_returns_dir`
Expected: FAIL（`SessionService` 尚未存在）

- [ ] **Step 3: 迁移 `SessionService` 实现**

```rust
// src/session/service.rs
use crate::error::{AppError, Result};
use crate::providers::{validate_languages, ProviderType};
use crate::session::storage;
use crate::session::{Session, SessionError, SessionKey};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct SessionService;

pub struct SessionRemoveResult {
    pub removed_ids: Vec<String>,
    pub missing_keys: Vec<String>,
}

impl SessionService {
    pub fn list_sessions() -> Result<Vec<Session>> {
        storage::list_sessions()
    }

    pub fn resolve_session_dir(key: &SessionKey) -> Result<PathBuf> {
        let session = storage::find_session(key)?;
        Ok(storage::get_session_dir()?.join(&session.id))
    }

    pub fn clear_sessions() -> Result<usize> {
        storage::clear_sessions()
    }

    pub fn remove_sessions(keys: &[SessionKey])
        -> Result<Option<SessionRemoveResult>> {
        if keys.is_empty() {
            return Ok(None);
        }

        let sessions = storage::list_sessions()?;
        if sessions.is_empty() {
            return Ok(None);
        }

        let mut removed_ids = Vec::new();
        let mut missing_keys = Vec::new();
        let mut deduped_session_ids = HashSet::new();

        for key in keys {
            match storage::resolve_session(&sessions, key) {
                Ok(session) => {
                    if deduped_session_ids.insert(session.id.clone()) {
                        let session_id = session.id;
                        if storage::delete_session(&session_id)? {
                            removed_ids.push(session_id);
                        } else {
                            missing_keys.push(session_id);
                        }
                    }
                }
                Err(
                    AppError::Session(SessionError::NotFound(missing_input)),
                ) => {
                    missing_keys.push(missing_input)
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Some(SessionRemoveResult {
            removed_ids,
            missing_keys,
        }))
    }

    pub fn create_session(
        provider_type: ProviderType,
        languages: Vec<String>,
    ) -> Result<PathBuf> {
        let provider = provider_type.into_shell_provider();

        let mut normalized_langs = languages
            .iter()
            .map(|language| provider.normalize_language(language))
            .collect::<Vec<_>>();

        let mut seen = HashSet::new();
        normalized_langs.retain(|language| seen.insert(language.clone()));

        if normalized_langs.is_empty() {
            return Err(AppError::Generic(
                "No languages specified. Use 'ah <langs>' or 'ah session list'".to_string(),
            ));
        }

        let supported_langs = provider.get_supported_languages()?;
        validate_languages(&normalized_langs, &supported_langs)?;

        let session_id = storage::generate_id(provider.name(), &normalized_langs);
        let session_dir = storage::get_session_dir()?.join(&session_id);
        std::fs::create_dir_all(&session_dir)?;

        provider.ensure_files(&normalized_langs, &session_dir)?;

        let session = Session::new(session_id, normalized_langs, provider.name().to_string());
        storage::save_session(&session)?;

        Ok(session_dir)
    }
}
```

- [ ] **Step 4: 更新 `mod.rs` 导出**

```rust
// src/session/mod.rs
mod service;

pub use service::{SessionRemoveResult, SessionService};
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test create_session_returns_dir`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add src/session/service.rs src/session/mod.rs tests/session_service.rs

git commit -m "refactor: move session service"
```

### Task 4: 更新引用并清理旧文件

**Files:**

- Modify: `src/lib.rs`
- Modify: `src/manager.rs`
- Modify: `src/cli.rs`
- Delete: `src/session.rs`
- Delete: `src/session_service.rs`
- Create: `tests/session_module_paths.rs`

- [ ] **Step 1: 写失败测试（编译期导出检查）**

```rust
#[test]
fn session_module_exports_types_and_service() {
    let _ = crate::session::SessionKey::Index(1);
    let _ = crate::session::SessionService::list_sessions;
    let _ = crate::session::SessionRemoveResult {
        removed_ids: Vec::new(),
        missing_keys: Vec::new(),
    };
    let _ = crate::session::Session::new(
        "id".to_string(),
        Vec::new(),
        "provider".to_string(),
    );
    let _ = crate::session::SessionError::NotFound("id".to_string());
    let _ = crate::session::SESSION_ID_LEN;
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test session_module_exports_types_and_service`
Expected: FAIL（旧路径未更新）

- [ ] **Step 3: 更新引用并删除旧文件**

```rust
// src/lib.rs
pub mod session;
// 删除 pub mod session_service;

// src/manager.rs
use crate::session::SessionService;

// src/cli.rs
use crate::session::SessionKey;

// 删除 src/session.rs 与 src/session_service.rs
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test session_module_exports_types_and_service`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
git add src/lib.rs src/manager.rs src/cli.rs src/session.rs
src/session_service.rs tests/session_module_paths.rs

git commit -m "refactor: align session module exports"
```

## Chunk 3: 全量验证与收尾

### Task 5: 完整测试

**Files:**

- Test: 全部测试

- [ ] **Step 1: 运行全量测试**

Run: `cargo test`
Expected: PASS

- [ ] **Step 2: 运行静态检查**

Run: `cargo check`
Expected: PASS

- [ ] **Step 3: 提交（如有必要）**

```bash
# 若前面步骤有未提交更改
git status
```
