# Error Handling & Logging Refactoring Design

## Overview

Refactor the project's error handling and logging to enterprise-grade using thiserror, anyhow, and tracing.

## Goals

1. Replace all `println!`/`eprintln!` with structured logging
2. Add anyhow for CLI entry point error context
3. Maintain thiserror for library-level errors (keep existing `AppError`, `CommandError`, `SessionError`)
4. Dependency audit and cleanup

---

## 1. Error Handling Architecture

### Layered Approach

| Layer              | Technology | Purpose                      |
| ------------------ | ---------- | ---------------------------- |
| Library (internal) | thiserror  | Strongly-typed error enums   |
| CLI entry point    | anyhow     | Add context to errors easily |
| Logging            | tracing    | Structured logging output    |

### Existing Error Types (保留不变)

```rust
// src/error.rs
pub enum AppError {
    EnvVarNotFound(VarError),
    Io(IoError),
    Json(serde_json::Error),
    InvalidPath(PathBuf),
    UnsupportedLanguages(Vec<String>),
    Provider(String),
    Command(CommandError),
    Session(SessionError),
    CliUsage(String),
    Generic(String),
}

// src/cmd.rs
pub enum CommandError {
    Io { command: String, source: std::io::Error },
    Failed { command: String, details: String },
}

// src/session/types.rs
pub enum SessionError {
    InvalidSelector(String),
    NotFound(String),
}
```

### CLI Entry Point: anyhow Wrapper

In `src/main.rs`:

```rust
fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(e) = ah::cli::run() {
        let ctx = anyhow::anyhow!("CLI execution failed").context(e);
        tracing::error!("{:#}", ctx);
        std::process::exit(e.exit_code());
    }
}
```

---

## 2. Logging Design

### Log Level Mapping

| Original Code        | tracing Level | Output Target                     |
| -------------------- | ------------- | --------------------------------- |
| `println!(...)`      | `info!`       | stdout                            |
| `eprintln!(warning)` | `warn!`       | stderr                            |
| `eprintln!(error)`   | `error!`      | stderr                            |
| `eprintln!(debug)`   | `debug!`      | stderr (only when RUST_LOG=debug) |

### Interactive Input (保留不变)

```rust
// Keep print! + flush() for interactive confirmation
print!("This will remove all sessions. Continue? [y/N]: ");
io::stdout().flush()?;
let mut input = String::new();
io::stdin().read_line(&mut input)?;
```

### Changes by File

#### src/manager.rs (21 changes)

| Line  | Original                                      | Replace With                                        |
| ----- | --------------------------------------------- | --------------------------------------------------- |
| 35    | `println!("No sessions found.")`              | `tracing::info!("No sessions found.")`              |
| 38    | `println!("{:<5} {:<10} ...", ...)`           | `tracing::info!("{:<5} {:<10} ...", ...)`           |
| 40-46 | `println!(...)`                               | `tracing::info!(...)`                               |
| 66    | `println!("Cancelled.")`                      | `tracing::info!("Cancelled.")`                      |
| 72    | `println!("Cleared {} session(s).", removed)` | `tracing::info!("Cleared {} session(s).", removed)` |
| 78    | `println!("No sessions found.")`              | `tracing::info!("No sessions found.")`              |
| 83    | `println!("Removed {}...", ...)`              | `tracing::info!("Removed {}...", ...)`              |
| 90    | `println!("Not found: {}", ...)`              | `tracing::warn!("Not found: {}", ...)`              |
| 103   | `println!("Restoring develop shell...")`      | `tracing::info!("Restoring develop shell...")`      |
| 107   | `println!("Creating develop shell...")`       | `tracing::info!("Creating develop shell...")`       |
| 116   | `println!("{:<5} {:<width$}", ...)`           | `tracing::info!(...)`                               |
| 124   | `println!("{}", format_provider_row(...))`    | `tracing::info!("{}", format_provider_row(...))`    |
| 137   | `println!()`                                  | Keep (provider separation)                          |
| 202   | `eprintln!(warning)`                          | `tracing::warn!(...)`                               |

#### src/cmd.rs (2 changes)

| Line   | Original                     | Replace With                       |
| ------ | ---------------------------- | ---------------------------------- |
| 47     | `eprintln!("exec: {}", ...)` | `tracing::debug!("exec: {}", ...)` |
| (none) | (add init check)             | Add tracing initialization         |

#### src/main.rs (changes)

- Add tracing_subscriber initialization
- Replace manual error output with tracing

#### src/warning.rs (changes)

- Remove `format_warning_line()` and `print_warnings()`
- Return `Vec<AppWarning>` from functions
- Let caller decide how to log with `tracing::warn!()`

---

## 3. Dependencies

### Add to Cargo.toml

```toml
[dependencies]
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Keep Existing

```toml
thiserror = "2.0"
```

### Dependency Audit

| Package            | Status | Reason               |
| ------------------ | ------ | -------------------- |
| thiserror          | Keep   | Library-level errors |
| anyhow             | Add    | CLI context wrapper  |
| tracing            | Add    | Logging              |
| tracing-subscriber | Add    | Log subscriber       |
| blake3             | Keep   | Hashing              |
| clap               | Keep   | CLI parsing          |
| rnix, rowan        | Keep   | Nix parsing          |
| serde, serde_json  | Keep   | Serialization        |
| directories        | Keep   | XDG paths            |
| rayon              | Keep   | Parallel processing  |

---

## 4. Key Design Decisions

### Q1: How to handle interactive input?

**Decision:** Keep `print!` + `io::stdout().flush()` for the single interactive confirmation prompt in `session clear`. This is user interaction, not logging.

### Q2: How to handle warnings?

**Decision:** Warnings are returned as `Vec<AppWarning>` from functions. The calling code (Manager) logs them with `tracing::warn!()`. This separates concerns: warnings are data, logging is output.

### Q3: How to handle debug output?

**Decision:** `cmd.rs:47` `eprintln!` becomes `tracing::debug!()`, only output when `RUST_LOG=debug ah ...`.

### Q4: Anyhow vs thiserror for library errors?

**Decision:** Keep thiserror for library. Anyhow is only used at CLI entry point (main.rs) to add context before logging.

---

## 5. Backward Compatibility

- Exit codes remain unchanged
- All existing error types preserved
- No breaking API changes
- Output format changes from plain text to structured logging
