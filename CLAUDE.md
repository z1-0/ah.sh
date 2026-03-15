# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

- 中文沟通 / English comments：与用户对话用中文，代码注释使用英文

## 项目概览

`ah` 是一个 Rust 编写的 CLI，用“语言列表”快速创建/恢复可复用的 Nix dev shell 会话。

- 入口命令：`ah use <languages...>`
- 会话：落盘到 XDG cache 目录，可 `list/restore/remove/clear`
- Provider：决定如何把语言列表生成 `flake.nix`
  - `dev-templates`（默认）：基于 `the-nix-way/dev-templates` 拼装；并尝试解析各模板的 `mkShell` 属性
  - `devenv`：生成 `devenv` 风格 flake，按语言写 `languages.<lang>.enable = true`

## 常用开发命令

> 推荐先进入本仓库的 devShell（包含 rust toolchain、treefmt、pre-commit hooks）。

### 进入开发环境（Nix flake）

```bash
nix develop
```

### 构建 / 运行

```bash
cargo build
cargo run -- --help

# 直接跑二进制（需要先 build）
./target/debug/ah --help
```

本仓库 `flake.nix` 也定义了 flake app：

```bash
nix run . -- --help
```

### 测试

```bash
cargo test

# 只跑一个测试（按名字过滤）
cargo test parse_flake_shell_minimal_extracts_env_and_extras
```

### 格式化（treefmt）

本仓库使用 `treefmt-nix` 统一格式化（nixfmt / prettier / rustfmt）。在 `nix develop` 环境中：

```bash
treefmt
```

### Lint（clippy）

```bash
cargo clippy --all-targets -- -D warnings
```

## 代码结构与数据流（大图）

### CLI 分层

- `src/main.rs`：进程入口；打印错误并用 `exit_code()` 退出
- `src/cli.rs`：clap 定义命令与参数，并把请求分发到 `Manager`
- `src/manager.rs`：面向终端输出/交互（例如 `session clear` 在 TTY 下二次确认），并在创建会话后 `exec nix develop`
- `src/app/session_app.rs`：很薄的一层“应用层”包装
- `src/session/service.rs`：核心业务逻辑（创建/列出/删除/解析会话目录）
- `src/session/storage.rs`：落盘与索引（XDG cache 目录、metadata.json）

典型调用链（创建会话）：

`cli::run` → `Manager::use_languages` → `SessionApp::prepare_create_session` → `SessionService::create_session` → `provider.ensure_files(...)` + `storage::save_session(...)` → `executor::execute_nix_develop(..., new_session=true)`（最终 `exec` 替换当前进程）

### Session 存储模型

- `src/session/types.rs`：
  - `Session { id, languages, provider, created_at }`
  - `SessionKey`：既支持序号（列表中的 1,2,...）也支持 8 位十六进制 id
- 会话目录（XDG cache）：
  - 基路径：`$XDG_CACHE_HOME/ah` 或 `~/.cache/ah`（见 `src/paths.rs`）
  - 会话集合：`.../sessions/<session_id>/`
  - 元数据：`.../sessions/<session_id>/metadata.json`
  - 创建会话时会写入 `flake.nix`；进入 shell 后 `nix develop --profile <dir>/nix-profile ...` 会生成/更新 profile
- `session_id`：`blake3(provider + ":" + sorted_langs.join(","))` 截断为 8 位（`src/session/storage.rs`）

### Provider 体系

- `src/providers/mod.rs`：
  - `ProviderType`（clap value enum）：`devenv` / `dev-templates`
  - `ShellProvider` trait：`ensure_files`（生成会话目录文件）、`get_supported_languages`、`normalize_language`
  - 语言别名：`src/assets/language_aliases.json`（按 provider 维度映射，例如 `js` → `javascript`）
  - 校验：`validate_languages(languages, supported)`

#### Provider: devenv

- `src/providers/devenv/mod.rs` + `flake_generator.rs`
- 逻辑：按语言生成 `devenv.lib.mkShell` 模块片段（`languages.<lang>.enable = true;`），写入会话目录 `flake.nix`
- 支持语言列表：编译期 `include_str!("src/assets/providers/devenv/supported_langs.json")`

#### Provider: dev-templates

- `src/providers/dev_templates/mod.rs`：
  - 会并发 fetch 各语言模板 flake（最多 8 并发，依据 CPU 核数 clamp）
  - fetch 失败不会中断创建：会产出 `AppWarning`，并继续生成 flake（依赖 `inputsFrom` 仍可用）
- `fetcher.rs`：从 GitHub raw 拉取 `the-nix-way/dev-templates/main/<lang>/flake.nix`，并在 XDG cache 里做 24h TTL 缓存
- `nix_parser.rs`：用 `rnix/rowan` 解析模板 flake，提取 `mkShell` attrset 中的 `env` 与“非标准 extra attrs”（跳过 `packages/buildInputs/...` 等）
- `flake_generator.rs`：生成最终 flake：
  - `inputsFrom` 来自每个模板 devShell
  - extra attrs/env 会引用 `shells."<lang>".<attr>`，并合并冲突（hook 类 attr 会用 `+ "\n" +` 连接）
- 支持语言列表：编译期 `include_str!("src/assets/providers/dev-templates/supported_langs.json")`

### 执行 nix develop 的方式

- `src/executor.rs`：使用 `CommandExt::exec()` 直接替换进程（不会 return）。
  - 新会话：`nix develop --profile <session_dir>/nix-profile --no-pure-eval <session_dir>`
  - 恢复会话：`nix develop --profile <session_dir>/nix-profile`（不传 session_dir）

## CI / 自动化

- `.github/workflows/update-langs.yml`：每周一自动生成并 PR 更新两份静态语言列表：
  - `src/assets/providers/devenv/supported_langs.json`
  - `src/assets/providers/dev-templates/supported_langs.json`

## 重要实现提示（易踩点）

- 该工具会落盘到用户 XDG cache 目录；测试/调试涉及文件系统时，优先关注 `src/paths.rs` 与 `src/session/storage.rs`。
- `executor::execute_nix_develop` 通过 `exec` 替换进程：在调用链上它之后的代码不会执行（返回类型为 `Infallible`）。
