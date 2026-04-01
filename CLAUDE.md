# ah.sh

## 项目概述

## Quick Start

```bash
# 快速创建并进入开发环境
ah use rust go nodejs
# 直接使用隐式命令
ah rust go nodejs
```

`ah` 是一个基于 Rust 开发的 CLI 工具，旨在通过 Nix 为开发者提供简便的临时开发环境。用户可以以最自然的方式（如 `ah use rust go`）按需构建和切换开发环境。

- **语言**: Rust (Edition 2024)
- **核心底座**: Nix (Flakes)
- **关键库**: clap、anyhow、serde、rnix、rowan、rayon、fp-core

## 常用命令

### 核心命令

````bash
# 创建并进入开发环境
ah use rust go nodejs
# 或更短的写法（隐式 use）：
ah rust go nodejs

# 会话管理

```bash
# 初始化会话并生成 flake.nix
ah init rust go

# 更新当前会话的依赖（如果未指定会话则使用当前会话）
ah update          # 使用当前会话
ah update <key>    # 指定会话 ID 或索引
````

ah session list # 列出历史会话
ah session restore <key> # 按索引恢复会话
ah session restore 1 # 按索引恢复
ah session remove <ID> # 删除指定会话
ah session clear # 清空所有会话

# Provider 管理

ah provider list # 列出可用 Provider
ah provider show all # 显示所有支持的语言
ah provider show dev-templates
ah provider show devenv

````

### 开发调试

```bash
# 进入本项目自身的开发环境
nix develop

# 构建项目
cargo build

# 本地运行
cargo run -- --help
cargo run -- use rust  # 测试特定功能

# 格式化代码
nix run .#format  # 使用 treefmt 全量格式化

# 静态检查
cargo clippy

# 运行测试
# 可使用 --test <test_name> 过滤单个测试
# 如需在 release 模式下运行: cargo test --release
cargo test
````

## 架构

### 模块结构

````
src/
├── cli/           # 命令行解析，隐式 use 命令处理
├── cmd.rs         # 执行 nix develop
├── manager.rs     # 核心业务编排，用于给cli的命令行提供实现
├── paths.rs       # 路径工具函数
├── provider/      # Provider 抽象和实现
│   ├── devenv/    # devenv provider - 生成 devenv flake 配置
│   ├── dev_templates/  # dev-templates provider
│   ├── language.rs     # 语言标准化、验证、别名映射
│   └── mod.rs          # ProviderType 枚举，provider 加载逻辑
└── session/       # 会话管理

```bash
# 初始化会话并生成 flake.nix
ah init rust go

# 更新当前会话的依赖（如果未指定会话则使用当前会话）
ah update          # 使用当前会话
ah update <key>    # 指定会话 ID 或索引
````

（创建、恢复、列表、删除）
├── storage.rs # 会话持久化（JSON 文件）
├── service.rs # 会话业务逻辑
└── types.rs # 会话数据结构

````

### Provider 系统

两个 provider 负责将语言列表转换为 Nix flake 配置：

- **dev-templates**（默认）：使用社区 dev-templates 项目
- **devenv**：使用 devenv 项目

每个 provider 在 `src/assets/providers/{provider}/` 目录下有 JSON 配置文件：

- `supported_languages.json` - 支持的语言列表
- `language_aliases.json` - 语言缩写映射（例如 "js" → "nodejs"）

### 隐式 Use

CLI 支持隐式 use 命令：`ah rust go` 等同于 `ah use rust go`。实现位于 `cli/implicit_use.rs`。

### 会话管理

```bash
# 初始化会话并生成 flake.nix
ah init rust go

# 更新当前会话的依赖（如果未指定会话则使用当前会话）
ah update          # 使用当前会话
ah update <key>    # 指定会话 ID 或索引
````

会话存储在数据目录中，使用 8 位十六进制 ID 标识。系统可以：

- 通过 provider + 语言列表查找已存在的会话
- 使用生成的 flake 配置创建新会话
- 通过 `nix develop` 恢复会话

## 开发规范

- 使用 `haksell` `FP` 的编程范式去组织代码，包括架构，结构，实现逻辑。
- 仅使用 `rust` 语言自身提供的 `Option` `Result` 等函数式特性以及 `fp-core` 提供的函数式封装，不另外做封装。
