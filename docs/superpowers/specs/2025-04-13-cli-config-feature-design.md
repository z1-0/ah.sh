# CLI 配置文件功能设计

**日期**: 2025-04-13
**作者**: Claude Code
**状态**: 设计完成

---

## 1. 项目概述

### 1.1 背景
`ah` 是一个 Nix 驱动的开发环境管理 CLI 工具。当前项目使用编译时嵌入的 JSON 文件定义 provider 的语言支持配置。本项目旨在添加**用户级配置文件**支持，允许用户自定义默认行为。

### 1.2 目标
- 引入用户配置文件（`~/.config/ah/config.toml`）
- 首次使用时自动创建默认配置（从 `src/assets/default_config.toml` 复制）
- 使用现代 Rust 生态（config-rs + toml + schemars）
- 配置文件错误时直接退出并提示查看 schema

### 1.3 范围
**本阶段实现**：
- ✅ 配置文件路径管理
- ✅ 默认配置文件的自动创建
- ✅ 配置加载和验证（使用 config-rs）
- ✅ CLI 启动时预加载配置（但不实际使用）

**暂缓实现**（后续任务）：
- ❌ 配置文件中 `provider` 值覆盖 CLI 默认值
- ❌ 配置文件中 `shell` 值的实际使用
- ❌ CLI 参数与配置的合并逻辑

---

## 2. 技术方案

### 2.1 依赖栈
```toml
[dependencies]
config = "0.15"      # 配置加载和合并
toml = "0.8"         # TOML 格式支持
schemars = "0.8"     # JSON Schema 生成（文档和验证）
serde = { version = "1.0", features = ["derive"] }
```

### 2.2 配置结构定义
```rust
// src/config.rs
use crate::provider::ProviderType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    /// 提供商类型: "devenv" 或 "dev-templates"
    pub provider: ProviderType,

    /// 自定义 shell 路径，留空则使用 $SHELL 环境变量
    #[serde(default)]
    pub shell: Option<String>,
}
```

**重要实现细节**：
- `provider` 字段**不**使用 `#[serde(default)]`，要求必须存在（强制用户显式设置或使用默认文件）
- `shell` 字段使用 `#[serde(default)]`，表示可选，缺失时默认为 `None`
- `JsonSchema` derive 用于后续生成 `config.schema.json` 文档

### 2.3 配置文件路径
遵循 XDG 规范：

```rust
// src/paths.rs
pub mod config {
    use super::*;
    use anyhow::Result;

    /// 配置文件文件名
    pub const CONFIG_FILE: &str = "config.toml";

    /// 获取完整配置文件路径: ~/.config/ah/config.toml
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = get_config_dir()?;
        Ok(config_dir.join(CONFIG_FILE))
    }

    /// 获取配置目录: ~/.config/ah/
    pub fn get_config_dir() -> Result<PathBuf> {
        let project_dirs = get_project_dirs()?;
        Ok(project_dirs.config_dir().to_pathbuf())
    }
}
```

### 2.4 配置加载流程
```rust
// src/config.rs
use anyhow::{Context, Result};
use config::{Config, File, FileFormat};

pub fn load_config() -> Result<Config> {
    let config_path = paths::get_config_path()?;

    // 首次使用：复制默认配置文件
    if !config_path.exists() {
        create_default_config(&config_path)
            .context("Failed to create default config")?;
    }

    // 构建配置加载器
    let config = Config::builder()
        .add_source(
            File::from(&config_path)
                .format(FileFormat::Toml)
                .required(true)
        )
        .build()
        .context("Failed to build config loader")?
        .try_deserialize()
        .context(
            "Failed to parse config.toml. \
             Check syntax at: https://github.com/z1-0/ah.sh/blob/main/config.schema.json"
        )?;

    Ok(config)
}

/// 从 assets 复制默认配置到用户目录
fn create_default_config(dest_path: &Path) -> Result<()> {
    use std::fs;

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create config directory")?;
    }

    // 嵌入默认配置
    let default_config = include_str!("../assets/default_config.toml");
    fs::write(dest_path, default_config)
        .context("Failed to write default config")?;

    Ok(())
}
```

### 2.5 默认配置文件
**位置**: `src/assets/default_config.toml`

```toml
provider = "dev-templates"
shell = ""
```

- 这是**首次使用时自动复制**的模板文件
- `shell = ""` 表示使用环境变量 `$SHELL`

---

## 3. 集成方案

### 3.1 CLI 入口修改
```rust
// src/cli/mod.rs
pub fn run() -> Result<()> {
    // 预加载配置（首次运行会创建默认文件）
    // ⚠️ 注意：此时配置值尚未实际使用，只是确保文件存在
    let _config = crate::config::load_config()
        .context("Failed to load configuration")?;

    let args = preprocess_args();
    let cli = Cli::try_parse_from(args)?;
    handle_command(cli.command)
}
```

### 3.2 后续使用点（未来任务）
配置文件中的值将在以下位置生效：
1. `manager::use_languages()` - 使用 `config.provider` 作为默认值
2. `cmd::nix_develop_of_session()` - 使用 `config.shell` 替代 `std::env::var("SHELL")`

---

## 4. 文件修改清单

| 文件 | 类型 | 说明 |
|------|------|------|
| `Cargo.toml` | 修改 | 添加 `config`, `toml`, `schemars` 依赖 |
| `src/config.rs` | 新建 | `Config` 结构体 + `load_config()` |
| `src/paths.rs` | 修改 | 添加 `config` 模块，包含 `get_config_path()` |
| `src/cli/mod.rs` | 修改 | `run()` 开头调用 `load_config()?` |
| `src/assets/default_config.toml` | 新建 | 默认配置文件模板 |

---

## 5. 边界情况处理

| 场景 | 行为 | 用户可见信息 |
|------|------|--------------|
| 配置文件不存在 | 自动复制默认配置 | 静默 |
| 配置目录无法创建（权限问题） | `load_config()` 返回 `Err` | "Failed to create config directory" |
| TOML 语法错误 | `load_config()` 返回 `Err` | "Failed to parse config.toml" + schema 链接提示 |
| 字段缺失（如 `provider`） | `load_config()` 返回 `Err`（deserialize 失败） | "missing field `provider`" |
| `provider` 值无效（如 "foo"） | `load_config()` 返回 `Err` | "invalid value: string `foo`, expected `devenv` or `dev-templates`" |
| `shell` 字段缺失 | 使用 `None`（默认） | 不影响 |
| `shell` 字段为空字符串 (`""`) | 解析为 `None`（Option 会自动处理） | 使用 `$SHELL` |

---

## 6. 设计原则

1. **零配置启动**：首次运行自动创建配置文件，无需用户干预
2. **明确的错误提示**：配置问题直接退出，引导用户查看 schema
3. **XDG 合规**：配置文件遵循 `$XDG_CONFIG_HOME/ah/config.toml`
4. **向后兼容**：通过 `#[serde(default)]` 支持字段缺失
5. **单一职责**：`load_config()` 只负责加载和验证，不处理业务逻辑

---

## 7. 未来扩展

### 7.1 Schema 生成
在 `build.rs` 中生成 `config.schema.json`：
```rust
// build.rs
use schemars::schema_for;
let schema = schema_for!(crate::config::Config);
std::fs::write("config.schema.json", serde_json::to_string_pretty(&schema)?)?;
```

### 7.2 配置热重载（可选）
未来可在 `src/config.rs` 添加 `watch_config()` 功能，监听文件变化并重新加载。

### 7.3 provider 配置迁移
下一个任务将实现：
- 修改 `Use.provider` 的默认值从配置文件读取
- 修改 `cmd.rs` 的 shell 命令使用配置的 `shell` 值

---

## 8. 验收标准

- [ ] 首次运行 `ah` 命令时，自动创建 `~/.config/ah/config.toml`
- [ ] 配置文件内容与 `src/assets/default_config.toml` 一致
- [ ] `cargo build` 成功，无 warning
- [ ] 故意破坏配置文件（如删除 `provider` 行），运行 `ah use rust` 应报错并提示查看 schema
- [ ] 配置文件语法正确时，`ah` 可以正常启动（即使 config 值暂未使用）
- [ ] `cargo clippy` 通过（`-D warnings`）
- [ ] `cargo fmt` 格式化后无变化

---

## 9. 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| XDG 目录在某些系统上不存在/无法访问 | 中 | 高 | 使用 `directories` crate 处理，自动创建目录 |
| config-rs 与现有代码冲突 | 低 | 中 | 将在独立模块 `src/config.rs` 中封装，避免污染 |
| 配置文件权限问题 | 低 | 中 | 错误信息明确提示权限问题 |
| 用户修改默认配置后升级导致字段不兼容 | 低 | 中 | 使用 `#[serde(default)]` 保证向后兼容 |

---

## 10. 测试建议

**手动测试步骤**：
1. 删除 `~/.config/ah/config.toml`（如果存在）
2. 运行 `cargo run -- use rust`，检查是否自动创建配置
3. 检查配置内容是否正确
4. 修改配置文件的 `provider = "invalid"`，运行命令验证错误提示
5. 删除 `provider` 行，验证错误提示

**单元测试建议**（后续）：
- `config::load_config()` 测试：
  - 首次创建文件
  - 加载有效配置
  - 处理无效配置（语法错误、缺失字段、无效值）

---

## 11. 术语表

| 术语 | 定义 |
|------|------|
| XDG | X Desktop Group 基础目录规范，定义 `$XDG_CONFIG_HOME` 等环境变量 |
| config-rs | Rust 配置加载库，支持多源合并（文件、环境、内存） |
| TOML | Tom's Obvious, Minimal Language - 配置文件格式 |
| JsonSchema | JSON Schema 规范，用于描述和验证 JSON/TOML 数据结构 |
| provider | 底层环境管理后端（devenv 或 dev-templates） |
