# AGENTS.md

This file provides guidance for agentic coding assistants working in this repository.

## 交互要求

- Thinking 思考过程用中文表述
- Reply 回答也要用中文回复

## Project Overview

This is a Rust CLI tool (ah.sh) that manages development environments using Nix flakes. It supports multiple programming languages by configuring appropriate toolchains and formatting tools dynamically.

## Build Commands

### Standard Cargo Commands

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the CLI
cargo run -- <language>...

# Format code
cargo fmt

# Check code without building
cargo check

# Run linter (clippy)
cargo clippy
```

### Running Tests

```bash
# Run all tests
cargo test

# Run a single test
cargo test <test_name>

# Run tests in a specific module
cargo test <module_name>

# Run tests with output
cargo test -- --nocapture

# Run tests with specific filter
cargo test <pattern>
```

### Nix/Devenv Commands

```bash
# Enter development environment
nix develop

# Format Nix files
nixfmt <file.nix>
```

## Code Style Guidelines

### Imports

- Standard library imports first (`std::*`)
- External crate imports second
- Internal module imports last (`crate::*`)
- Group related imports using `{...}` for multiple items from the same module
- Use `use std::{path::Path, process::Command};` pattern

```rust
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use clap::{Parser, command};
use serde_json::from_str;

use crate::{command::exec_nix_develop, env};
```

### Naming Conventions

- Functions: `snake_case` (e.g., `ensure_languages`, `flatten_pkgs`)
- Structs/Enums: `PascalCase` (e.g., `Cli`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `FLAKE_URL`)
- Module names: `snake_case` (e.g., `cli`, `command`)
- Variables: `snake_case` (e.g., `supported`, `invalids`)
- Acronyms in types: preserve case consistency (e.g., `Json` not `JSON` in type names)

### Types and Generics

- Use `Vec<T>` for collections
- Use `HashMap<K, V>` for key-value mappings
- Use `HashSet<T>` for unique collections
- Use `&str` for borrowed string references when possible
- Use `String` for owned strings
- Generic type parameters use single uppercase letters: `T`, `K`, `V`
- Use type annotations when inference is ambiguous

### Error Handling

- Return `Result<T, E>` for fallible operations
- Use `String` for simple error messages: `Result<Vec<String>, String>`
- Use `unwrap_or_else()` for user-facing error messages with `eprintln!`
- Use `std::process::exit(1)` to terminate on errors
- Use `unwrap()` only when the error is truly unreachable (internal invariants)
- Use `expect()` for internal errors with descriptive messages

```rust
fn ensure_languages(langs: Vec<String>, supported_langs: Vec<String>) -> Result<Vec<String>, String> {
    let supported: HashSet<String> = supported_langs.into_iter().collect();

    let invalids: Vec<String> = langs
        .iter()
        .filter(|&l| !supported.contains(l))
        .cloned()
        .collect();

    if invalids.is_empty() {
        Ok(langs)
    } else {
        Err(format!("Languages {:?} are not supported", invalids))
    }
}

// Usage with user-friendly error
let ensures = ensure_languages(cli.language, supported).unwrap_or_else(|e| {
    eprintln!("{e}");
    std::process::exit(1);
});
```

### Clap CLI Definitions

- Use `#[derive(Parser)]` for CLI structures
- Add `#[command(version, about, long_about = None)]` at struct level
- Use `Vec<String>` for multiple arguments
- Parse with `Cli::parse()` in entry functions

### Constants

- Define module-level constants with `const`
- Use descriptive names in SCREAMING_SNAKE_CASE
- Place constants near the top of their module

### Serde/JSON Handling

- Use `serde_json::to_string()` for serialization (returns `Result<String, Error>`)
- Use `serde_json::from_str()` for deserialization (returns `Result<T, Error>`)
- Use `include_str!()` to bundle static JSON assets at compile time
- Use `.expect()` for deserialization of bundled assets (internal errors)

### File I/O

- Use `Path` for path manipulation
- Use `Path::new()` to create paths
- Use `.join()` for path concatenation
- Use `.read_dir()` with `.ok().unwrap()` for directory reading
- Chain `.filter_map(Result::ok)` to handle directory entry errors

### Iterators

- Prefer functional style with iterators (`.map()`, `.filter()`, `.collect()`)
- Use `.flatten()` to flatten nested iterables
- Use `.filter_map()` to combine filtering and mapping
- Use `.cloned()` to convert `&T` to `T` in iterators

### Mod Organization

- Keep modules focused on single responsibilities
- Declare modules in `lib.rs`
- Use `pub mod` for public modules
- Functions are `pub` by default for public APIs

### File Structure

- `main.rs`: Entry point only, calls into library modules
- `lib.rs`: Module declarations
- `cli.rs`: CLI argument parsing and validation
- `command.rs`: Command execution (e.g., spawning subprocesses)
- `env.rs`: Environment constants
- `assets/`: Static data files (JSON, etc.)

### Environment Variables

- Define compile-time env vars using `env!("VAR_NAME")` in `env.rs`
- Pass env vars to subprocesses using `.env()` on `Command`

### Process Spawning

- Use `std::process::Command` for subprocess execution
- Use `.args(["arg1", "arg2"])` for multiple arguments
- Use `.env("KEY", "value")` to set environment variables
- Use `.exec()` to replace current process

### No Comments

- Do not add comments to code
- Keep code self-documenting through clear naming
- Comment only when explicitly requested
