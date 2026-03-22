# Error Handling & Logging Refactoring Design

## Overview

Refactor the project's error handling and logging to enterprise-grade using only anyhow and tracing (no thiserror).

## Goals

1. Delete `src/error.rs` and `src/warning.rs`
2. Replace all error types with `anyhow::Error`
3. Replace all warnings with `tracing::warn!()`
4. Replace most `println!`/`eprintln!` with tracing, but keep user-facing CLI output

---

## 1. Architecture

### New Error Handling

All errors use `anyhow::Error` + `anyhow!` macro:

```rust
// Before
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    // ...
}

// After
use anyhow::{Context, Result};
fn foo() -> Result<()> {
    anyhow::anyhow!("some error").context("additional context")
}
```

### New Warning Handling

All warnings use `tracing::warn!()` directly:

```rust
// Before
fn print_warnings(warnings: &[AppWarning]) {
    for w in warnings {
        eprintln!("warning[{}]: {}", w.code, w.message);
    }
}

// After
for w in warnings {
    tracing::warn!(code = %w.code, "{}", w.message);
}
```

---

## 2. CLI Output vs Logging

### Keep as println! (User-facing output)

These are terminal output for users, not logs:

| Location           | Content                      | Reason                             |
| ------------------ | ---------------------------- | ---------------------------------- |
| manager.rs:38      | Session table header         | User wants to see column headers   |
| manager.rs:40-46   | Session list                 | User wants to see session list     |
| manager.rs:59      | Confirmation prompt          | Interactive user input             |
| manager.rs:116-124 | Provider list                | User wants to see provider options |
| manager.rs:137     | Blank line between providers | Visual separation                  |

### Replace with tracing (Developer/ops output)

| Original                                           | tracing Level | Rationale                             |
| -------------------------------------------------- | ------------- | ------------------------------------- |
| `println!("Restoring...")`                         | `info!`       | Operations log                        |
| `println!("Creating...")`                          | `info!`       | Operations log                        |
| `println!("No sessions found.")` (when empty list) | `info!`       | Operations log                        |
| `println!("Cancelled.")`                           | `info!`       | Operations log                        |
| `println!("Cleared {} session(s)."`                | `info!`       | Operations log                        |
| `eprintln!(warning)`                               | `warn!`       | Warning messages                      |
| `eprintln!(debug)`                                 | `debug!`      | Debug info (only with RUST_LOG=debug) |

---

## 3. Files to Delete

### src/error.rs

- Delete entirely
- Replace all `use crate::error::{AppError, Result}` with `use anyhow::{Context, Result}`

### src/warning.rs

- Delete entirely
- Replace `print_warnings()` calls with `tracing::warn!()`

---

## 4. Changes by File

### Cargo.toml

```toml
[dependencies]
# Remove
thiserror = "2.0"

# Add
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### src/main.rs

```rust
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(e) = ah::cli::run() {
        // Detect CLI usage error (clap errors) for exit code 2
        let exit_code = if e.downcast_ref::<clap::Error>().is_some() {
            2
        } else {
            1
        };
        tracing::error!("{:#}", e);
        std::process::exit(exit_code);
    }
}
```

### src/error.rs

**DELETE** - Replace with anyhow in all importing files

### src/warning.rs

**DELETE** - Replace with tracing::warn! in calling code

### src/manager.rs

| Line    | Original                                | Replace With                                   |
| ------- | --------------------------------------- | ---------------------------------------------- |
| 2       | `use crate::error::Result;`             | `use anyhow::Result;`                          |
| 6       | `use crate::warning::AppWarning;`       | Remove                                         |
| 35      | `println!("No sessions found.")`        | Keep (CLI output)                              |
| 38-46   | `println!(...)` table                   | Keep (CLI output)                              |
| 59      | `print!(...)` prompt                    | Keep (interactive)                             |
| 66      | `println!("Cancelled.")`                | Keep (CLI output)                              |
| 72      | `println!("Cleared...")`                | Keep (CLI output)                              |
| 78      | `println!("No sessions found.")`        | Keep (CLI output)                              |
| 83      | `println!("Removed...")`                | Keep (CLI output)                              |
| 90      | `println!("Not found: {}")`             | `tracing::warn!("Not found: {}", ...)`         |
| 103     | `println!("Restoring...")`              | `tracing::info!("Restoring develop shell...")` |
| 107     | `println!("Creating...")`               | `tracing::info!("Creating develop shell...")`  |
| 116-124 | Provider list                           | Keep (CLI output)                              |
| 137     | Blank line                              | Keep (visual)                                  |
| 183-203 | `format_warning_line`, `print_warnings` | Delete - replace with `tracing::warn!`         |

### src/cmd.rs

| Line  | Change                                                                           |
| ----- | -------------------------------------------------------------------------------- |
| 1     | Change `use crate::error::{AppError, Result}` to `use anyhow::{Context, Result}` |
| 7-17  | Delete `CommandError` enum                                                       |
| 21-34 | Use `anyhow::anyhow!` instead of `CommandError`                                  |
| 47    | `eprintln!(debug)` → `tracing::debug!(exec: {})`                                 |

### src/session/types.rs

| Line  | Change                                               |
| ----- | ---------------------------------------------------- |
| 1     | Remove `use crate::error::AppError`                  |
| 2     | Remove `use crate::warning::AppWarning`              |
| 9-16  | Delete `SessionError` enum                           |
| 26-29 | Delete `CreateSessionResult.warnings` field          |
| 51-60 | Remove `FromStr` error handling using `SessionError` |

### src/session/service.rs

- Change error types to `anyhow::Result<T>`
- Change return type from `Result<CreateSessionResult>` to `Result<(Session, Vec<Warning>)>`

### src/cli/mod.rs

- Change error types to `anyhow::Result<T>`

### src/lib.rs

- Remove `pub mod error;` and `pub mod warning;`

### src/provider/\* (multiple files)

- Change error types to `anyhow::Result<T>`

---

## 5. Exit Code Handling

Original behavior:

- `AppError::CliUsage(_)` → exit code 2
- All other errors → exit code 1

New behavior with anyhow:

```rust
fn main() {
    if let Err(e) = ah::cli::run() {
        let exit_code = if e.downcast_ref::<clap::Error>().is_some() {
            2
        } else {
            1
        };
        tracing::error!("{:#}", e);
        std::process::exit(exit_code);
    }
}
```

---

## 6. Dependency Audit

| Package            | Status     | Reason              |
| ------------------ | ---------- | ------------------- |
| thiserror          | **Remove** | No longer needed    |
| anyhow             | **Add**    | All errors          |
| tracing            | **Add**    | Logging             |
| tracing-subscriber | **Add**    | Log subscriber      |
| blake3             | Keep       | Hashing             |
| clap               | Keep       | CLI parsing         |
| rnix, rowan        | Keep       | Nix parsing         |
| serde, serde_json  | Keep       | Serialization       |
| directories        | Keep       | XDG paths           |
| rayon              | Keep       | Parallel processing |

---

## 7. Key Design Decisions

### Q1: CLI output vs logging

**Decision:** Keep `println!` for:

- Table headers and data (user wants to see it)
- Interactive prompts (user must respond)
- Final results shown to user

Use `tracing` for:

- Progress messages (can be suppressed)
- Warnings and errors
- Debug information

### Q2: How to handle missing exit_code() from AppError?

**Decision:** Check if error is `clap::Error` for exit code 2, otherwise exit code 1. This covers most CLI usage errors.

### Q3: Warnings without AppWarning struct?

**Decision:** Just use `tracing::warn!()` directly in code, no struct needed. Warning is an event, not data.

---

## 8. Backward Compatibility

- Exit codes: 2 for clap errors, 1 for others (same as before)
- Output format: changes for logging, stays same for CLI output
- API: Breaking changes (remove error types), but internal to crate
