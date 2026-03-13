# 全项目优化重构设计（Project-wide Refactor）

日期：2026-03-13

## 目标

在保证功能不异常的前提下，对整个项目进行结构与可靠性优先的优化重构，并在每个阶段保持可验证、可回滚。

优先级（用户确认）：

- **代码结构/模块边界**：职责单一、依赖方向清晰、公开 API 收敛
- **可靠性**：错误处理一致、边界条件覆盖、关键流程有测试兜底
- **开发体验（DX）**：CLI 帮助/错误信息清晰一致，输出更可预测
- **性能**：仅做“确定收益且不扩大范围”的小优化

### 可验证目标（Definition of Done）

- 每个阶段结束时：通过本 spec 的“硬闸门”检查（见下文）。
- 重构过程中：不引入 panic；错误路径必须可观测并能定位来源。
- 对外可见行为变化：必须记录在“对外可见变更清单”，并提供最小迁移说明。

## 允许变更范围

用户确认允许 **破坏性变更**（C），但要求 **功能不能出现异常**。

- 允许：重命名/移动模块、调整 CLI 输出与文案、调整内部调用链、收敛或重排 public API
- 不允许：panic、明显错误结果、关键命令不可用、数据破坏（例如错误清理/覆盖 session 数据）

## 非目标（Non-goals）

- 不进行“CLI 重塑”（保持现有命令结构，见“CLI 策略”）。
- 不引入新的 provider 类型（仅允许对现有 provider 的边界/错误/告警可观测性做调整）。
- 不做大规模性能工程（例如更换并发模型、引入复杂缓存系统、重写网络层）；性能优化仅限于确定性的小改动。

## 验收标准（硬闸门）

任意阶段完成都必须通过：

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build`

此外必须通过最小 CLI 行为验收（命令与预期）：

- `ah --help`：退出码 0，显示 help
- `ah`（无 languages 且无子命令）：显示 help，退出码非 0（建议 2）
- `ah session list`：空环境下退出码 0，输出 `No sessions found.`
- `ah session clear`：在非交互环境（stdin 非 TTY）不应阻塞等待确认（当前行为为仅在 TTY 才确认；该语义应保持或在变更清单中明确）

## 错误、返回码与输出通道约定

为降低脚本依赖风险与回归不透明，约定：

- **stdout**：用于“成功且稳定”的业务输出（例如 list 的表格）。
- **stderr**：用于 warnings、错误、诊断信息。
- **退出码**（建议约定，实施时如变更必须记录）：
  - 0：成功
  - 2：CLI 使用错误（例如缺少必须参数时显示 help）
  - 1：运行时失败（IO/网络/provider/exec 等）

## 现状概览（基于代码阅读）

当前关键路径（简化）与对应文件：

- `main` → `cli::run()`（`src/main.rs`、`src/cli.rs`）
- `cli` 解析参数后分派到 `Manager::*`（`src/cli.rs`、`src/manager.rs`）
- `Manager` 编排 session 的 list/restore/remove/clear/create（`src/manager.rs`）
- providers（`src/providers/mod.rs`、`src/providers/devenv/*`、
  `src/providers/dev_templates/*`）负责生成 flake/拉取模板
- `executor::execute_nix_develop` 使用 `exec()` 替换进程进入 `nix develop`（`src/executor.rs`）
- `error::AppError` 为统一错误入口（`src/error.rs`）

## 已识别的关键痛点

- **边界混杂**：`Manager` 同时承担交互输出（print/confirm）、业务编排、并触发 `exec()` 边界。
- **库层直接输出 warning**：`providers/dev_templates` 中存在 `eprintln!`，污染测试输出且不利于统一 DX。
- **`exec()` 失败路径不可观测**：当前实现丢弃 `cmd.exec()` 的错误。
- **静默降级**：例如 language aliases 的 JSON 解析失败会 `unwrap_or_default()`，导致行为变化但不显式报错/告警。

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

- `cli -> app/services -> domain`
- `app/services -> infra`
- `infra` 不反向依赖 `cli`（杜绝库里 println/eprintln）

## warnings 最小协议（库层到 CLI 的结构化告警）

阶段 1 起，库层不得直接输出 warning，而是返回结构化 warnings。最小协议：

- 字段建议：
  - `code`：稳定标识（例如 `dev_templates.fetch_failed`）
  - `message`：人类可读信息
  - `context`：可选上下文（例如 provider 名称、language）
- 聚合规则：app/services 汇总 warnings；cli 统一格式化输出到 stderr。
- 顺序规则：输出顺序应稳定（避免并发导致 nondeterministic 输出顺序）。

## 关键边界决策：nix exec 语义

用户确认：`execute_nix_develop` 继续采用 `exec()`（成功时替换当前进程）。因此：

- **成功路径：函数不返回**
- **失败路径：返回 Err，错误必须可观测**

建议用类型表达“成功不返回”的事实：

- `fn execute_nix_develop(...) -> Result<!>`

exec 失败时的对外合同（建议）：

- 错误输出到 stderr
- 退出码为 1（运行时失败）
- 错误信息至少包含：执行目标（nix develop）与底层 IO 错误

## CLI 策略（路线 A）

用户确认选择路线 A：

- 尽量保持现有命令结构，不进行“CLI 重塑”
- session 被定位为 **落盘 artifact/cache**（为了复用开发环境），不是核心叙事

用户确认的对外行为：

- 直接运行 `ah`（无 languages 且无子命令）时：显示 help 并退出非 0（建议 2）

## 破坏性变更的管理方式（Changelog-as-Contract）

由于允许破坏性变更，为避免功能异常与回归不透明：

- 在本 spec 内维护“对外可见变更清单”（单一事实来源）
- 每次涉及 CLI 命令/参数/输出/语义变化必须追加记录
- 若存在脚本兼容风险：必须给出最小迁移建议（例如替代命令、输出变化说明）

## 分阶段实施（4 阶段）

每个阶段都采用同一模板描述，以便实现与评审：

- Deliverables：本阶段必须产出什么
- Blast radius：可能影响哪些模块/命令
- Rollback：如何小步回滚
- Stage gate：除硬闸门外，本阶段特有的验收点

### 阶段 1：系统边界清晰化（可靠性地基）

Deliverables：

- `executor::execute_nix_develop`：保留 `exec()`，但表达“成功不返回、失败返回 Err”，且不再吞掉错误
- providers：库层不直接 `eprintln!`，改为返回结构化 warnings，由 CLI 统一展示
- 处理静默降级：把 `unwrap_or_default()` 之类变成显式错误或可观测 warning

Blast radius：

- `src/executor.rs`、`src/providers/**`、CLI 输出（warning 展示位置/格式可能变化）

Rollback：

- 将 warning 的展示逻辑封装在 CLI 层，允许阶段内回滚实现而不影响对外接口

Stage gate：

- `exec()` 失败时能看到明确错误；不得静默失败
- warnings 不得在库层直接输出

### 阶段 2：按层重排与收敛依赖方向（结构主目标）

Deliverables：

- 将代码组织为 `cli` / `app` / `domain` / `infra`（以最小可行拆分为准）
- 使依赖方向稳定，避免 infra 反向依赖 cli
- 收敛 public API（`src/lib.rs` 统一导出策略）

Blast radius：

- 模块路径与导出变化（破坏性变更），需要同步更新“对外可见变更清单”（如果对外 API/模块路径可见）

Rollback：

- 按模块逐步移动，保持每次移动后编译/测试通过；必要时使用临时 re-export（仅阶段内过渡，阶段结束前移除）

Stage gate：

- 依赖方向审查：infra 不得引用 cli（包括打印、clap 类型等）

### 阶段 3：可靠性补强（测试兜底）

Deliverables：

- 为关键不变量/关键路径补测试
  - `SessionKey` 解析
  - language normalize / aliases / validate
  - provider 支持语言加载与错误路径
- 对 `exec()` 边界：测试覆盖到“参数拼装与错误能冒泡”，但不在测试中真正 exec 替换进程
- provider 网络路径：测试不依赖真实网络（需要通过抽象/注入或 fixtures 达成）

Blast radius：

- `tests/*` 与相关模块的可测试性改造（接口拆分/依赖注入可能影响 app/infra 边界）

Rollback：

- 优先在 domain/app 层增加纯逻辑测试；对 infra 的可测试性改造小步引入

Stage gate：

- `cargo test` 覆盖关键契约：`ah` 无参数时退出非 0 且显示 help（可用集成测试或最小 CLI smoke 测试实现）

### 阶段 4：DX 统一 + 确定性性能小优化

Deliverables：

- 统一错误信息/帮助文案/输出格式
- 只做确定收益的小性能优化（避免引入大规模性能工程）

Blast radius：

- CLI 输出变化概率较高，必须逐条记录到“对外可见变更清单”

Rollback：

- 输出格式变更必须可回滚（避免与内部结构变更强耦合）

Stage gate：

- 输出通道约定（stdout/stderr）保持一致，不在库层直接输出诊断信息

## 对外可见变更清单

### Changed

- 直接运行 `ah`（无 languages 且无子命令）时：显示 help 并退出非 0（建议 2）

## 约定

- 本 spec 是“合同”：所有对外可见行为变化都必须先更新“对外可见变更清单”。
- 任何阶段完成都必须满足“硬闸门”。
