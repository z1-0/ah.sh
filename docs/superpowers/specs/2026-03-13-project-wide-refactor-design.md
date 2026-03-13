# 全项目优化重构设计（Project-wide Refactor）

日期：2026-03-13

## 目标

在保证功能不异常的前提下，对整个项目进行结构与可靠性优先的优化重构，并在每个阶段保持可验证、可回滚。

优先级（用户确认）：

1. **代码结构/模块边界**：职责单一、依赖方向清晰、公开 API 收敛
2. **可靠性**：错误处理一致、边界条件覆盖、关键流程有测试兜底
3. **开发体验（DX）**：CLI 帮助/错误信息清晰一致，输出更可预测
4. **性能**：仅做“确定收益且不扩大范围”的小优化

## 允许变更范围

用户确认允许 **破坏性变更**（C），但要求 **功能不能出现异常**。

- 允许：重命名/移动模块、调整 CLI 输出与文案、调整内部调用链、收敛或重排 public API
- 不允许：panic、明显错误结果、关键命令不可用、数据破坏（例如错误清理/覆盖 session 数据）

## 验收标准（硬闸门）

任意阶段完成都必须通过：

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`

这些命令的通过是 Ralph Loop 允许输出完成承诺的基础证据。

## 现状概览（基于代码阅读）

当前关键路径（简化）：

- `main` → `cli::run()`
- `cli` 解析参数后分派到 `Manager::*`
- `Manager` 编排 session 的 list/restore/remove/clear/create
- providers (`providers/*`) 负责生成 flake/拉取模板
- `executor::execute_nix_develop` 使用 `exec()` 替换进程进入 `nix develop`
- `error::AppError` 为统一错误入口

## 已识别的关键痛点

1. **边界混杂**：`Manager` 同时承担交互输出（print/confirm）、业务编排、并触发 `exec()` 边界。
2. **库层直接输出 warning**：`providers/dev_templates` 中存在 `eprintln!`，污染测试输出且不利于统一 DX。
3. **`exec()` 失败路径不可观测**：当前实现丢弃 `cmd.exec()` 的错误。
4. **静默降级**：例如 language aliases 的 JSON 解析失败会 `unwrap_or_default()`，导致行为变化但不显式报错/告警。

## 目标架构分层（最终形态）

以“可测试的纯逻辑”与“有副作用的系统边界”分离为核心：

- **cli（表现层）**
  - clap 解析、交互确认、输出格式
  - 不做文件 IO / 网络 / exec / 业务规则

- **app / services（用例层）**
  - 业务编排：创建/恢复/清理/移除 session、选择 provider、聚合 warnings
  - 返回结构化结果，不直接打印

- **domain（领域层）**
  - 类型与不变量：`SessionKey` 解析规则、provider/语言等概念
  - 尽量无 IO，便于单测

- **infra（基础设施层）**
  - fs/network/process：session storage、模板 fetch、`nix develop` exec 等
  - 错误与告警必须可观测、可向上冒泡

依赖方向约束：

`cli -> app/services -> domain`

`app/services -> infra`

并且：`infra` 不反向依赖 `cli`（杜绝库里 println/eprintln）。

## 关键边界决策：nix exec 语义

用户确认：`execute_nix_develop` 继续采用 `exec()`（成功时替换当前进程）。因此：

- **成功路径：函数不返回**
- **失败路径：返回 Err，错误必须可观测**

建议用类型表达“成功不返回”的事实：

- `fn execute_nix_develop(...) -> Result<!>`

## CLI 策略（路线 A）

用户确认选择路线 A：

- 尽量保持现有命令结构，不进行“CLI 重塑”
- session 被定位为 **落盘 artifact/cache**（为了复用开发环境），不是核心叙事
- 用户确认：直接运行 `ah`（无 languages 且无子命令）时，行为为：
  - **显示 help** 并 **退出非 0**

## 破坏性变更的管理方式（Changelog-as-Contract）

由于允许破坏性变更，为避免功能异常与回归不透明：

- 在本 spec 内维护“对外可见变更清单”（单一事实来源）
- 每次涉及 CLI 命令/参数/输出/语义变化必须追加记录

建议结构：

- Added / Changed / Removed
- Output（脚本依赖风险点）
- Behavior（语义变化与返回码）

## 分阶段实施（4 阶段）

### 阶段 1：系统边界清晰化（可靠性地基）

- `executor::execute_nix_develop`：保留 `exec()`，但在类型/错误路径上表达“成功不返回、失败返回 Err”
- providers：库层不直接 `eprintln!`，改为返回结构化 warnings，由 CLI 统一展示
- 处理静默降级：把 `unwrap_or_default()` 之类变成显式错误或可观测 warning

产出：错误/告警可观测、边界语义明确。

### 阶段 2：按层重排与收敛依赖方向（结构主目标）

- 将代码组织为 `cli` / `app` / `domain` / `infra`
- 使依赖方向稳定，避免 infra 反向依赖 cli
- 收敛 public API（`lib.rs` 统一导出策略）

产出：模块职责清晰，便于后续测试与维护。

### 阶段 3：可靠性补强（测试兜底）

- 为关键不变量/关键路径补单元测试
  - `SessionKey` 解析
  - language normalize / aliases / validate
  - provider 支持语言加载等错误路径
- 对 `exec()` 边界：测试覆盖到“参数拼装与错误能冒泡”，但不在测试中真正 exec 替换进程

产出：关键行为被测试锁住，减少回归风险。

### 阶段 4：DX 统一 + 确定性性能小优化

- 统一错误信息/帮助文案/输出格式
- 只做确定收益的小性能优化（避免引入大规模性能工程）

产出：用户体验更一致，项目整体更“可维护”。

## 对外可见变更清单（待实施时维护）

- TBD（随每次变更追加）
