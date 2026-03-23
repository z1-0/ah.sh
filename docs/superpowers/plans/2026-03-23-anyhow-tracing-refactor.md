# Anyhow + Tracing Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace thiserror with anyhow for errors, and replace println!/eprintln! with tracing. Delete error.rs and warning.rs.

**Architecture:** All errors use `anyhow::Error` with `anyhow!` macro. All warnings and logging use `tracing`. CLI output (tables, prompts) stays as println!.

**Tech Stack:** anyhow, tracing, tracing-subscriber

---

## File Changes Overview

| File                              | Change                                                    |
| --------------------------------- | --------------------------------------------------------- |
| Cargo.toml                        | Add anyhow, tracing, tracing-subscriber; Remove thiserror |
| src/main.rs                       | Add tracing init, update error handling                   |
| src/lib.rs                        | Remove error/warning module exports                       |
| src/error.rs                      | DELETE                                                    |
| src/warning.rs                    | DELETE                                                    |
| src/manager.rs                    | Update imports, tracing for progress/warnings             |
| src/cmd.rs                        | Use anyhow instead of CommandError                        |
| src/session/types.rs              | Remove SessionError, AppWarning usage                     |
| src/session/service.rs            | Update to use anyhow + tracing                            |
| src/session/storage.rs            | Update imports to anyhow                                  |
| src/cli/mod.rs                    | Update imports to anyhow                                  |
| src/paths.rs                      | Update imports to anyhow                                  |
| src/provider/registry.rs          | Update imports to anyhow                                  |
| src/provider/types.rs             | Remove AppWarning                                         |
| src/provider/dev_templates/mod.rs | Update to use anyhow + tracing                            |
| src/provider/devenv/mod.rs        | Update imports to anyhow                                  |
| src/provider/language_maps.rs     | Update to use anyhow                                      |

---

## Task 1: Update Cargo.toml dependencies

**Files:** `Cargo.toml`

- [ ] **Step 1: Modify Cargo.toml**

```toml
[dependencies]
# Remove
thiserror = "2.0"

# Add after blake3
blake3 = "1.8"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Keep existing
clap = { version = "4.6.0", features = ["derive"] }
rnix = "0.14"
rowan = "0.16"
serde = { version = "1.0.228" ,features = ["derive"]}
serde_json = "1.0.149"
directories = "6"
rayon = "1.10"
```

- [ ] **Step 2: Run cargo check to verify**

```bash
cargo check 2>&1 | head -30
```

Expected: Should fail on missing modules (expected, we'll fix next)

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "refactor: add anyhow/tracing, remove thiserror"
```

---

## Task 2: Update src/main.rs - tracing init + error handling

**Files:** `src/main.rs`

- [ ] **Step 1: Replace main.rs content**

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match ah::cli::run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            // Detect CLI usage error for exit code 2
            let exit_code = if e.to_string().contains("Usage:") || e.to_string().contains("error:") {
                2
            } else {
                1
            };
            tracing::error!("{:#}", e);
            ExitCode::from(exit_code as u8)
        }
    }
}
```

- [ ] **Step 2: Run cargo check**

```bash
cargo check 2>&1 | head -50
```

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "refactor: add tracing init in main.rs"
```

---

## Task 3: Update src/lib.rs - remove module exports

**Files:** `src/lib.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/lib.rs
```

- [ ] **Step 2: Remove error and warning module lines**

Remove:

```rust
pub mod error;
pub mod warning;
```

- [ ] **Step 3: Run cargo check**

```bash
cargo check 2>&1 | head -50
```

Expected: Multiple errors about missing modules (we'll fix in subsequent tasks)

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs
git commit -m "refactor: remove error/warning module exports from lib.rs"
```

---

## Task 4: Delete src/error.rs

**Files:** `src/error.rs`

- [ ] **Step 1: Delete the file**

```bash
rm src/error.rs
```

- [ ] **Step 2: Commit**

```bash
git rm src/error.rs
git commit -m "refactor: delete error.rs - using anyhow instead"
```

---

## Task 5: Delete src/warning.rs

**Files:** `src/warning.rs`

- [ ] **Step 1: Delete the file**

```bash
rm src/warning.rs
```

- [ ] **Step 2: Commit**

```bash
git rm src/warning.rs
git commit -m "refactor: delete warning.rs - using tracing instead"
```

---

## Task 6: Update src/paths.rs

**Files:** `src/paths.rs:1`

- [ ] **Step 1: Change import**

Change:

```rust
use crate::error::Result;
```

To:

```rust
use anyhow::Result;
```

- [ ] **Step 2: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|paths" | head -20
```

- [ ] **Step 3: Commit**

```bash
git add src/paths.rs
git commit -m "refactor: paths.rs use anyhow instead of crate::error"
```

---

## Task 7: Update src/cli/mod.rs

**Files:** `src/cli/mod.rs:5`

- [ ] **Step 1: Change import**

Change:

```rust
use crate::error::Result;
```

To:

```rust
use anyhow::Result;
```

- [ ] **Step 2: Check if there are other error-related imports**

```bash
grep -n "error" src/cli/mod.rs
```

- [ ] **Step 3: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|cli" | head -20
```

- [ ] **Step 4: Commit**

```bash
git add src/cli/mod.rs
git commit -m "refactor: cli/mod.rs use anyhow instead of crate::error"
```

---

## Task 8: Update src/session/storage.rs

**Files:** `src/session/storage.rs:1`

- [ ] **Step 1: Change import**

Change:

```rust
use crate::error::Result;
```

To:

```rust
use anyhow::Result;
```

- [ ] **Step 2: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|storage" | head -20
```

- [ ] **Step 3: Commit**

```bash
git add src/session/storage.rs
git commit -m "refactor: storage.rs use anyhow instead of crate::error"
```

---

## Task 9: Update src/session/types.rs

**Files:** `src/session/types.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/session/types.rs
```

- [ ] **Step 2: Remove error imports**

Remove:

```rust
use crate::error::AppError;
use crate::warning::AppWarning;
```

Add:

```rust
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
```

- [ ] **Step 3: Remove SessionError enum**

Delete:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("invalid session key: {0}")]
    InvalidSelector(String),

    #[error("session '{0}' not found")]
    NotFound(String),
}
```

- [ ] **Step 4: Update CreateSessionResult - remove warnings**

Change:

```rust
pub struct CreateSessionResult {
    pub session: Session,
    pub warnings: Vec<AppWarning>,
}
```

To:

```rust
pub struct CreateSessionResult {
    pub session: Session,
}
```

- [ ] **Step 5: Update FromStr implementation**

Change from:

```rust
impl FromStr for SessionKey {
    type Err = AppError;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(SessionError::InvalidSelector(
                "session target cannot be empty".to_string(),
            ).into());
        }
        // ... rest of implementation
    }
}
```

To:

```rust
impl FromStr for SessionKey {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("session target cannot be empty"));
        }
        // ... rest of implementation
    }
}
```

Update all error returns in this function to use `anyhow::anyhow!()`.

- [ ] **Step 6: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|types" | head -30
```

- [ ] **Step 7: Commit**

```bash
git add src/session/types.rs
git commit -m "refactor: session/types.rs remove SessionError, use anyhow"
```

---

## Task 10: Update src/cmd.rs

**Files:** `src/cmd.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/cmd.rs
```

- [ ] **Step 2: Replace imports**

Change:

```rust
use crate::error::{AppError, Result};
```

To:

```rust
use anyhow::{Context, Result};
```

- [ ] **Step 3: Remove CommandError enum and replace with anyhow**

Remove:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Failed to start command `{command}`: {source}")]
    Io {
        command: String,
        source: std::io::Error,
    },

    #[error("Command `{command}` failed: {details}")]
    Failed { command: String, details: String },
}
```

- [ ] **Step 4: Update run() function**

Change:

```rust
fn run(mut cmd: Command) -> Result<String> {
    let command = command_to_string(&cmd);
    let output = cmd.output().map_err(|source| CommandError::Io {
        command: command.clone(),
        source,
    })?;

    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let details = if details.is_empty() {
            format!("exit status {}", output.status)
        } else {
            details
        };

        return Err(CommandError::Failed { command, details }.into());
    }
    // ...
}
```

To:

```rust
fn run(mut cmd: Command) -> Result<String> {
    let command = command_to_string(&cmd);
    let output = cmd.output()
        .context(format!("failed to start command: {}", command))?;

    if !output.status.success() {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let details = if details.is_empty() {
            format!("exit status {}", output.status)
        } else {
            details
        };

        anyhow::bail!("command `{}` failed: {}", command, details);
    }
    // ...
}
```

- [ ] **Step 5: Update exec() function**

Change:

```rust
fn exec(mut cmd: Command) -> Result<Infallible> {
    if cfg!(debug_assertions) {
        eprintln!("exec: {}", command_to_string(&cmd));
    }

    let command = command_to_string(&cmd);
    let source = cmd.exec();
    Err(CommandError::Io { command, source }.into())
}
```

To:

```rust
fn exec(mut cmd: Command) -> Result<Infallible> {
    if cfg!(debug_assertions) {
        tracing::debug!(exec = %command_to_string(&cmd), "executing command");
    }

    let command = command_to_string(&cmd);
    let source = cmd.exec();
    Err(anyhow::anyhow!("failed to exec: {}: {}", command, source))
}
```

- [ ] **Step 6: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|cmd" | head -20
```

- [ ] **Step 7: Commit**

```bash
git add src/cmd.rs
git commit -m "refactor: cmd.rs use anyhow, replace eprintln with tracing"
```

---

## Task 11: Update src/session/service.rs

**Files:** `src/session/service.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/session/service.rs
```

- [ ] **Step 2: Update imports**

Change:

```rust
use crate::error::{AppError, Result};
use crate::warning::AppWarning;
```

To:

```rust
use anyhow::{Context, Result};
```

- [ ] **Step 3: Find all error creation points**

```bash
grep -n "anyhow\|bail\|Context" src/session/service.rs
```

Update all to use `anyhow::anyhow!()` or `anyhow::bail!()`.

- [ ] **Step 4: Find warning handling**

```bash
grep -n "AppWarning\|warnings" src/session/service.rs
```

- Replace warning creation with `tracing::warn!()` calls directly.

- [ ] **Step 5: Update create_session return type**

If it returns `Result<CreateSessionResult>`, update to handle no warnings:

```rust
// Instead of returning warnings in CreateSessionResult,
// emit warnings directly with tracing::warn!()
```

- [ ] **Step 6: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|service" | head -30
```

- [ ] **Step 7: Commit**

```bash
git add src/session/service.rs
git commit -m "refactor: session/service.rs use anyhow + tracing"
```

---

## Task 12: Update src/manager.rs

**Files:** `src/manager.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/manager.rs
```

- [ ] **Step 2: Update imports**

Change:

```rust
use crate::error::Result;
use crate::warning::AppWarning;
```

To:

```rust
use anyhow::Result;
```

- [ ] **Step 3: Keep CLI output (println!) as-is:**
- Line 35: `println!("No sessions found.")` - KEEP
- Lines 38-46: Session table - KEEP
- Line 59: Interactive prompt - KEEP
- Line 66: `println!("Cancelled.")` - KEEP
- Line 72: `println!("Cleared...")` - KEEP
- Line 78: `println!("No sessions found.")` - KEEP
- Lines 83-90: Removed sessions output - KEEP
- Lines 116-124: Provider list - KEEP
- Line 137: Blank line - KEEP

- [ ] **Step 4: Replace with tracing (info!):**

- Line 103: `println!("Restoring develop shell...")` → `tracing::info!("Restoring develop shell...")`
- Line 107: `println!("Creating develop shell...")` → `tracing::info!("Creating develop shell...")`

- [ ] **Step 5: Replace with tracing (warn!):**

- Line 90: `println!("Not found: {}", ...)` → `tracing::warn!("Not found: {}", keys)`

- [ ] **Step 6: Remove warning formatting functions**

Delete `format_warning_line()`, `sorted_warnings_for_print()`, `print_warnings()` (lines 183-204).

- [ ] **Step 7: Update use_languages to emit warnings directly**

Instead of:

```rust
print_warnings(&result.warnings);
```

Use:

```rust
for w in &warnings {
    tracing::warn!(code = %w.code, "{}", w.message);
}
```

- [ ] **Step 8: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|manager" | head -30
```

- [ ] **Step 9: Commit**

```bash
git add src/manager.rs
git commit -m "refactor: manager.rs use tracing for logs, keep println for output"
```

---

## Task 13: Update src/provider/registry.rs

**Files:** `src/provider/registry.rs:1`

- [ ] **Step 1: Change import**

Change:

```rust
use crate::error::Result;
```

To:

```rust
use anyhow::Result;
```

- [ ] **Step 2: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|registry" | head -20
```

- [ ] **Step 3: Commit**

```bash
git add src/provider/registry.rs
git commit -m "refactor: provider/registry.rs use anyhow"
```

---

## Task 14: Update src/provider/types.rs

**Files:** `src/provider/types.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/provider/types.rs
```

- [ ] **Step 2: Update imports**

Remove:

```rust
use crate::error::Result;
use crate::warning::AppWarning;
```

Add:

```rust
use anyhow::Result;
```

- [ ] **Step 3: Find and remove AppWarning usage**

```bash
grep -n "AppWarning" src/provider/types.rs
```

- [ ] **Step 4: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|types" | head -20
```

- [ ] **Step 5: Commit**

```bash
git add src/provider/types.rs
git commit -m "refactor: provider/types.rs remove AppWarning, use anyhow"
```

---

## Task 15: Update src/provider/devenv/mod.rs

**Files:** `src/provider/devenv/mod.rs:3`

- [ ] **Step 1: Change import**

Change:

```rust
use crate::error::Result;
```

To:

```rust
use anyhow::Result;
```

- [ ] **Step 2: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|devenv" | head -20
```

- [ ] **Step 3: Commit**

```bash
git add src/provider/devenv/mod.rs
git commit -m "refactor: provider/devenv/mod.rs use anyhow"
```

---

## Task 16: Update src/provider/language_maps.rs

**Files:** `src/provider/language_maps.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/provider/language_maps.rs
```

- [ ] **Step 2: Update imports**

Change:

```rust
use crate::error::{AppError, Result};
```

To:

```rust
use anyhow::{Context, Result};
```

- [ ] **Step 3: Find all error creation points**

```bash
grep -n "AppError\|anyhow\|bail" src/provider/language_maps.rs
```

Update all to use `anyhow::anyhow!()`.

- [ ] **Step 4: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|language_maps" | head -20
```

- [ ] **Step 5: Commit**

```bash
git add src/provider/language_maps.rs
git commit -m "refactor: provider/language_maps.rs use anyhow"
```

---

## Task 17: Update src/provider/dev_templates/mod.rs

**Files:** `src/provider/dev_templates/mod.rs`

- [ ] **Step 1: Read current content**

```bash
cat src/provider/dev_templates/mod.rs
```

- [ ] **Step 2: Update imports**

Change:

```rust
use crate::error::{AppError, Result};
use crate::warning::AppWarning;
```

To:

```rust
use anyhow::{Context, Result};
```

- [ ] **Step 3: Find all error/warning creation points**

```bash
grep -n "AppError\|AppWarning\|anyhow" src/provider/dev_templates/mod.rs
```

Update all to use `anyhow::anyhow!()` and `tracing::warn!()`.

- [ ] **Step 4: Run cargo check**

```bash
cargo check 2>&1 | grep -E "error|dev_templates" | head -30
```

- [ ] **Step 5: Commit**

```bash
git add src/provider/dev_templates/mod.rs
git commit -m "refactor: provider/dev_templates/mod.rs use anyhow + tracing"
```

---

## Task 18: Final verification

**Files:** All

- [ ] **Step 1: Run cargo check**

```bash
cargo check 2>&1 | head -50
```

Expected: No errors

- [ ] **Step 2: Run cargo clippy**

```bash
cargo clippy --all-targets -- -D warnings 2>&1 | head -50
```

Expected: No warnings (or minimal, fix if needed)

- [ ] **Step 3: Run cargo build**

```bash
cargo build 2>&1 | head -30
```

Expected: Successful build

- [ ] **Step 4: Test basic functionality**

```bash
cargo run -- --help
```

Expected: Help output shown correctly

- [ ] **Step 5: Commit final changes**

```bash
git status
git add -A
git commit -m "refactor: complete anyhow + tracing migration"
```

---

## Summary

This plan consists of 18 tasks:

1. Update Cargo.toml
2. Update src/main.rs
3. Update src/lib.rs
4. Delete src/error.rs
5. Delete src/warning.rs
6. Update src/paths.rs
7. Update src/cli/mod.rs
8. Update src/session/storage.rs
9. Update src/session/types.rs
10. Update src/cmd.rs
11. Update src/session/service.rs
12. Update src/manager.rs
13. Update src/provider/registry.rs
14. Update src/provider/types.rs
15. Update src/provider/devenv/mod.rs
16. Update src/provider/language_maps.rs
17. Update src/provider/dev_templates/mod.rs
18. Final verification
