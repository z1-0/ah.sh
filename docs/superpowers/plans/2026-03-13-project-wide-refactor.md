# Project-wide Refactor Implementation Plan

> **For agentic workers:** REQUIRED: Use @superpowers:subagent-driven-development
> (if subagents available) or @superpowers:executing-plans to implement this plan.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在不改变“核心用例：进入/管理开发环境”的前提下，完成全项目的
分层重构、可靠性补强与 DX 统一，并将对外行为变化显式记录与可回归验证。

**Architecture:** 采用 `cli / app / domain / infra` 分层，集中系统边界
（fs/network/process）并让 warnings/错误可观测、可测试；每阶段以
`cargo fmt/clippy/test/build` + CLI contract 作为闸门。

**Tech Stack:** Rust 2024、clap 4、thiserror、serde/serde_json、ureq、
rnix/rowan、blake3

---

## Scope note（范围说明）

本计划覆盖 spec 的 4 阶段，但优先完成“阶段 1：合同与边界”。

若阶段 2（分层重排）工作量过大，应拆出独立计划文件，不在同一分支堆积
长期未验证的改动。

---

## Pre-flight（执行前准备）

- [ ] **Step: 建议在隔离环境执行**

建议在独立分支或 worktree 中执行（避免与 `main` 混杂）。

- [ ] **Step: 确认代码结构与关键路径存在（避免路径不一致）**

Run:

- `ls -la src`
- `ls -la src/session || true`
- `ls -la src/providers || true`

Expected:

- 存在 `src/cli.rs`、`src/manager.rs`、`src/executor.rs`、`src/error.rs`
- 存在 `src/session/`（`mod.rs/types.rs/service.rs/storage.rs`）

- [ ] **Step: 记录当前基线（仅一次）**

Run:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`

Expected: 全部 PASS。

---

## Files map（计划会涉及的文件）

### 阶段 1 预计修改

- `src/main.rs`（按错误类型映射退出码）
- `src/cli.rs`（`ah` 无参数：打印 help + 退出码 2）
- `src/manager.rs`（CLI 层统一展示 warnings 到 stderr）
- `src/error.rs`（新增 CLI usage 错误 + `exit_code()`）
- `src/executor.rs`（保留 `exec()`，失败返回 Err；建议 `Result<!>`）
- `src/providers/mod.rs`（去除 language aliases 静默降级）
- `src/providers/dev_templates/mod.rs`（库层 warning 改为结构化 warnings）
- `src/providers/dev_templates/fetcher.rs`（去除 `eprintln!`）
- `src/session/service.rs`（create 透传 warnings）

### 阶段 1 预计新增

- `src/warning.rs`（最小 warnings 协议）
- `tests/cli_contract.rs`（CLI contract：exit code + stdout/stderr + 不阻塞）

### 阶段 1 依赖调整

- `Cargo.toml`：新增 dev-deps（`assert_cmd`、`predicates`、`tempfile`）

---

## Chunk 1: 阶段 1（边界可观测性 + CLI 合同）

> 目标：先把“对外合同”与“边界错误/告警”做成可测试、可回归的形态。

### Task 1: 建立 CLI contract 测试（TDD 入口）

**Files:**

- Modify: `Cargo.toml`
- Create: `tests/cli_contract.rs`

- [ ] **Step 1: 添加 dev-dependencies（先让测试能写）**

Edit `Cargo.toml` 新增：

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

- [ ] **Step 2: 写失败的 CLI contract 测试（先写测试）**

Create `tests/cli_contract.rs`：

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;

#[test]
fn help_exits_zero() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.arg("--help").assert().success().code(0);
}

#[test]
fn no_args_prints_help_and_exits_2() {
    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("USAGE").or(predicate::str::contains("Usage")));
}

#[test]
fn session_list_empty_is_ok() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.env("XDG_CACHE_HOME", tmp.path())
        .args(["session", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No sessions found."));
}

#[test]
fn session_clear_non_tty_does_not_block() {
    let tmp = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("ah").unwrap();
    cmd.env("XDG_CACHE_HOME", tmp.path())
        .args(["session", "clear"])
        .write_stdin("")
        .timeout(Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared"));
}
```

- [ ] **Step 3: 运行测试，确认至少 1 个失败**

Run: `cargo test -q`

Expected: `no_args_prints_help_and_exits_2` FAIL。

- [ ] **Step 4: Commit（只包含测试与 dev-deps）**

```bash
git add Cargo.toml tests/cli_contract.rs
git commit -m "test: add CLI contract"
```

---

### Task 2: 实现 `ah` 无参数显示 help 并退出码为 2（含 Changelog gate）

**Files:**

- Modify: `docs/superpowers/specs/2026-03-13-project-wide-refactor-design.md`
- Modify: `src/cli.rs`
- Modify: `src/error.rs`
- Modify: `src/main.rs`
- Test: `tests/cli_contract.rs`

- [ ] **Step 1:（合同）先更新 spec 的“对外可见变更清单”**

在 spec 中确认已记录：

- `ah` 无参数：显示 help，退出码 2

- [ ] **Step 2: 运行测试确认仍失败（锁定红灯）**

Run: `cargo test -q`

Expected: FAIL。

- [ ] **Step 3: 在 `AppError` 增加 CLI usage 错误类型 + `exit_code()`**

Edit `src/error.rs`：

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // ... existing variants ...

    #[error("{0}")]
    CliUsage(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::CliUsage(_) => 2,
            _ => 1,
        }
    }
}
```

- [ ] **Step 4: `main` 使用 `exit_code()`**

Edit `src/main.rs`：

```rust
fn main() {
    if let Err(e) = ah::cli::run() {
        eprintln!("\x1b[1;31merror:\x1b[0m {e}");
        std::process::exit(e.exit_code());
    }
}
```

- [ ] **Step 5: `cli::run()` 无 args 时打印 help 并返回 `CliUsage`**

Edit `src/cli.rs`：

```rust
use clap::{CommandFactory, Parser, Subcommand};

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Session { action }) = &cli.command {
        match action {
            None | Some(SessionCommands::List) => Manager::list_sessions()?,
            Some(SessionCommands::Restore { key }) => Manager::restore_session(key)?,
            Some(SessionCommands::Clear) => Manager::clear_sessions()?,
            Some(SessionCommands::Remove { keys }) => Manager::remove_sessions(keys)?,
        }
        return Ok(());
    }

    if cli.languages.is_empty() {
        let mut cmd = Cli::command();
        cmd.print_help()
            .map_err(|e| crate::error::AppError::CliUsage(e.to_string()))?;
        println!();
        return Err(crate::error::AppError::CliUsage(
            "Missing LANG arguments".to_string(),
        ));
    }

    Manager::create_session(cli.provider, cli.languages)?;
    Ok(())
}
```

- [ ] **Step 6: 运行测试确认通过**

Run: `cargo test -q`

Expected: PASS。

- [ ] **Step 7: 运行硬闸门**

Run:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`

Expected: 全部 PASS。

- [ ] **Step 8: Commit**

```bash
git add docs/superpowers/specs/2026-03-13-project-wide-refactor-design.md \
  src/main.rs src/cli.rs src/error.rs

git commit -m "feat(cli): show help and exit 2 on missing args"
```

---

### Task 3: `execute_nix_develop`：成功不返回、失败返回 Err（保留 exec 语义）

**Decision（本计划定案）：**

- `execute_nix_develop(...) -> Result<!>`
- `Manager::{restore_session, create_session} -> Result<!>`
- `cli::run() -> Result<()>` 保持不变（`!` 可协变为 `()`）

**Files:**

- Modify: `src/executor.rs`
- Modify: `src/manager.rs`

- [ ] **Step 1: 先写 exec 失败路径单测（不会替换进程）**

在 `src/executor.rs`：

```rust
#[cfg(test)]
fn exec_missing_command_for_test() -> std::io::Error {
    use std::os::unix::process::CommandExt;
    std::process::Command::new("__definitely_missing__").exec()
}

#[cfg(test)]
mod tests {
    #[test]
    fn exec_returns_not_found_when_command_missing() {
        let err = super::exec_missing_command_for_test();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
```

- [ ] **Step 2: 运行测试（先确保 scaffold 正确）**

Run: `cargo test -q`

Expected: PASS。

- [ ] **Step 3: `execute_nix_develop` 返回 `Result<!>` 且不吞错误**

Edit `src/executor.rs`（要点：不要 `let _ = cmd.exec()`）：

```rust
pub fn execute_nix_develop(
    session_dir: PathBuf,
    new_session: bool,
) -> Result<!> {
    // ... build cmd ...
    let err = cmd.exec();
    Err(AppError::Io(err))
}
```

- [ ] **Step 4: 更新 `Manager::{restore_session, create_session}` 返回 `Result<!>`**

Edit `src/manager.rs`：

- `restore_session`：解析后直接 `execute_nix_develop(...)?;`
- `create_session`：创建后直接 `execute_nix_develop(...)?;`

- [ ] **Step 5: 硬闸门 + CLI contract**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo build`

Expected: PASS。

- [ ] **Step 6: Commit**

```bash
git add src/executor.rs src/manager.rs
git commit -m "refactor(executor): surface exec errors and mark success diverging"
```

---

### Task 4: 结构化 warnings + 去除库层 `eprintln!`（含 Changelog gate）

**Files:**

- Modify: `docs/superpowers/specs/2026-03-13-project-wide-refactor-design.md`
- Create: `src/warning.rs`
- Modify: `src/providers/mod.rs`
- Modify: `src/providers/dev_templates/mod.rs`
- Modify: `src/providers/dev_templates/fetcher.rs`
- Modify: `src/session/service.rs`
- Modify: `src/manager.rs`

- [ ] **Step 1:（合同）先更新 spec 的变更清单（Output/Behavior）**

记录：warnings 统一由 CLI 输出到 stderr，格式稳定：

- `warning[code]: message`

- [ ] **Step 2: 先写失败测试（引用不存在类型/方法）**

Create `src/warning.rs` 仅测试：

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn warning_can_attach_context() {
        let w = crate::warning::AppWarning::new("x", "m").with_context("k", "v");
        assert_eq!(w.context.len(), 1);
    }
}
```

- [ ] **Step 3: 写最小 `AppWarning` 实现让测试通过**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppWarning {
    pub code: &'static str,
    pub message: String,
    pub context: Vec<(String, String)>,
}

impl AppWarning {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), context: Vec::new() }
    }

    pub fn with_context(
        mut self,
        k: impl Into<String>,
        v: impl Into<String>,
    ) -> Self {
        self.context.push((k.into(), v.into()));
        self
    }
}
```

- [ ] **Step 4: `write_cache_best_effort` 返回 warning（确定性失败测试）**

Edit `src/providers/dev_templates/fetcher.rs`：

测试用例（用 `tempfile`，向目录路径写入会稳定失败）：

```rust
#[test]
fn cache_write_best_effort_returns_warning_on_failure() {
    let dir = tempfile::tempdir().unwrap();
    let w = write_cache_best_effort(dir.path(), "hello").expect("should warn");
    assert_eq!(w.code, "dev_templates.cache_write_failed");
}
```

实现：

```rust
fn write_cache_best_effort(
    cache_file: &Path,
    body: &str,
) -> Option<AppWarning> {
    match fs::write(cache_file, body) {
        Ok(()) => None,
        Err(e) => Some(
            AppWarning::new("dev_templates.cache_write_failed", e.to_string())
                .with_context("path", cache_file.display().to_string()),
        ),
    }
}
```

- [ ] **Step 5: provider 接口：`ensure_files` 返回 warnings**

Edit `src/providers/mod.rs`：

```rust
pub struct EnsureFilesResult {
    pub warnings: Vec<AppWarning>,
}

pub trait ShellProvider {
    fn name(&self) -> &str;
    fn ensure_files(&self, languages: &[String], target_dir: &Path) -> Result<EnsureFilesResult>;
    fn get_supported_languages(&self) -> Result<Vec<String>>;
}
```

同步更新：

- `src/providers/devenv/mod.rs`：返回空 warnings
- `src/providers/dev_templates/mod.rs`：把并发 fetch 失败等转成 warnings（不打印）

- [ ] **Step 6: `SessionService::create_session` 透传 warnings**

Edit `src/session/service.rs`：

- 新增 `CreateSessionResult { session_dir, warnings }`
- create 结束返回 `CreateSessionResult`

- [ ] **Step 7: CLI 层统一展示 warnings 到 stderr**

Edit `src/manager.rs`：

- create 流程中将 warnings 输出到 stderr（格式稳定）

- [ ] **Step 8: 硬闸门 + CLI contract**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo build`

Expected: PASS。

- [ ] **Step 9: Commit**

```bash
git add docs/superpowers/specs/2026-03-13-project-wide-refactor-design.md \
  src/warning.rs src/providers/mod.rs src/providers/devenv/mod.rs \
  src/providers/dev_templates/mod.rs src/providers/dev_templates/fetcher.rs \
  src/session/service.rs src/manager.rs

git commit -m "refactor: plumb structured warnings and remove library eprintln"
```

---

### Task 5: 去除 language aliases 的静默降级（fail-fast）

**Files:**

- Modify: `src/providers/mod.rs`
- Modify: `src/session/service.rs`

- [ ] **Step 1: 先写失败测试：坏 JSON 必须报错**

在 `src/providers/mod.rs` 内引入：

- `parse_aliases(json: &str) -> Result<LanguageAliases>`

并对 `"not json"` 断言返回 Err（错误信息包含 `language aliases`）。

- [ ] **Step 2: OnceLock 保存 `Result<...>`，去掉 `unwrap_or_default()`**

把静默降级改为显式错误或 warning（按 spec）。

- [ ] **Step 3: create_session normalize 阶段遇错直接 `?` 返回**

- [ ] **Step 4: 硬闸门 + Commit**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo build`

---

### Checkpoint: 阶段 1 Review

- [ ] **Step: 对照 spec 验收阶段 1 合同**

必须满足：

- `ah --help` exit 0
- `ah` 无参数：stdout 打 help，exit 2
- `ah session clear` 非 TTY 不阻塞
- warnings 不在库层直接输出
- `exec()` 失败路径可观测（exit 1）
- `cargo fmt/clippy/test/build` 全部 PASS

---

## Chunk 2: 阶段 2（分层重排：cli/app/domain/infra）

> 注意：当前仓库已有 `src/cli.rs`，本阶段不创建 `src/cli/mod.rs`。

### Task 6: 增加 app/domain/infra 的最小骨架（不触发文件搬家）

**Files:**

- Create: `src/app/mod.rs`
- Create: `src/domain/mod.rs`
- Create: `src/infra/mod.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: 创建空模块文件**

Create:

- `src/app/mod.rs`
- `src/domain/mod.rs`
- `src/infra/mod.rs`

- [ ] **Step 2: 在 `src/lib.rs` 引入新模块（不动现有导出）**

```rust
pub mod app;
pub mod domain;
pub mod infra;
```

- [ ] **Step 3: `cargo test -q`**

Expected: PASS。

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs src/app/mod.rs src/domain/mod.rs src/infra/mod.rs
git commit -m "refactor: add layering module skeleton"
```

---

### Task 7: 把业务编排从 `Manager` 迁到 app 层（小步迁移）

**Files:**

- Create: `src/app/session_app.rs`
- Modify: `src/manager.rs`

- [ ] **Step 1: 先迁移 list_sessions（不涉及 exec）**

- 新增 `SessionApp::list_sessions()` 调用 `SessionService::list_sessions()`
- `Manager::list_sessions` 改为调用 `SessionApp` 并继续负责输出

Run: `cargo test -q`
Expected: PASS。

- [ ] **Step 2: Commit（仅 list 迁移）**

```bash
git add src/app/session_app.rs src/manager.rs
git commit -m "refactor(app): route session list through SessionApp"
```

- [ ] **Step 3: 再迁移 remove/clear（仍不涉及 exec）**

同样方式迁移 `remove_sessions` / `clear_sessions`。

Run: `cargo test -q`
Expected: PASS。

---

### Task 8: create/restore：业务归 app，exec 边界留给 executor

- [ ] **Step 1: app 层提供 create/restore 的“准备结果”**

app 层返回：session_dir + warnings。

- [ ] **Step 2: CLI/Manager 触发 exec**

- [ ] **Step 3: 硬闸门 + CLI contract**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo build`

Expected: PASS。

---

## Chunk 3: 阶段 3（测试兜底）

### Task 9: 为关键不变量补齐测试（不引入真实网络/真实 exec）

- [ ] **Step 1: `SessionKey` 解析测试**
- [ ] **Step 2: language normalize / aliases / validate 测试**
- [ ] **Step 3: providers 纯函数测试（`nix_parser`、flake generator）**
- [ ] **Step 4: 硬闸门**

---

## Chunk 4: 阶段 4（DX + 小性能优化）

### Task 10: DX 统一（输出通道/颜色/提示信息）

- [ ] **Step 1: ANSI 上色仅在 TTY 启用（或提供开关）**
- [ ] **Step 2: warnings 输出格式与顺序稳定**
- [ ] **Step 3: 硬闸门 + CLI contract**

### Task 11: 小性能优化（仅确定性、低风险）

- [ ] **Step 1: 只修 clippy 指出的明显 clone/alloc**
- [ ] **Step 2: 不更换并发模型/网络栈，不引入复杂缓存**
- [ ] **Step 3: 硬闸门**

## Expected: PASS

## Plan complete

Plan complete and saved to `docs/superpowers/plans/2026-03-13-project-wide-refactor.md`.
Ready to execute?
