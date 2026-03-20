# Session 模块统一类型 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 统一 `src/session` 子模块为单一公开 `Session` 类型（定义在 `types.rs`），删除 `created_at`，并将会话列表排序改为按 `session_dir` 目录 `mtime` 降序，同时保持外部行为不变。

**Architecture:** 采用“类型集中 + 存储私有 DTO + 服务编排”方案：所有 `pub` 类型收敛到 `src/session/types.rs`；`storage.rs` 内部使用私有 `SessionMetadata` 负责 JSON 与文件系统细节；`service.rs` 仅编排流程并复用 `types::Session`。排序逻辑下沉到 `storage::list_sessions()`，基于目录本身 `mtime`。

**Tech Stack:** Rust, serde/serde_json, std::fs metadata API, existing session/provider architecture

---

## 文件结构与职责

- Modify: `src/session/types.rs`
  - 定义唯一公开 `Session`
  - 保留 `SessionKey` / `SessionError` / `SESSION_ID_LEN`
  - 收敛 `CreateSessionResult` / `SessionRemoveResult` 等公开结果类型
- Modify: `src/session/service.rs`
  - 删除本地 `Session` 与其它公开类型定义
  - 全部改用 `types.rs` 中类型
  - 保持业务流程与外部行为
- Modify: `src/session/storage.rs`
  - 新增私有 `SessionMetadata`
  - metadata 读写只在 storage 内部
  - Task 1 阶段：`list_sessions` 保持旧返回签名以确保可编译推进，仅替换排序为 `session_dir.mtime`
  - Task 2 阶段：在类型统一时一次性切换为返回新的公开 `types::Session`
- Modify: `src/session/mod.rs`
  - 调整 re-export，确保公开类型来源统一到 `types.rs`
- Modify: `src/manager.rs`（按编译报错最小改动，可能必改）
  - 修复被统一类型变更影响的字段引用，不做额外重构或清理
- Reference: `docs/superpowers/specs/2026-03-21-session-module-unification-design.md`

> 约束：本次不做测试改动（不新增/不修改测试文件）。验证仅使用编译与静态检查。

### Task 1: 先在 `storage.rs` 落地私有 metadata 与 mtime 排序（解除 created_at 依赖）

**Files:**

- Modify: `src/session/storage.rs`

- [ ] **Step 1: 在 `storage.rs` 引入私有 `SessionMetadata`（过渡阶段）**

实现要点：

- 定义非 `pub` 的 `SessionMetadata { id, provider, languages }`
- metadata JSON 反序列化使用 `SessionMetadata`
- Task 1 保持现有函数返回签名不变，确保迁移期间不中断编译
- 禁止引入新的时间字段回填语义（不通过 `Session::new` 等方式注入当前时间）
- Task 1 仅替换存储内部表示，不改变对外类型语义

- [ ] **Step 2: 将 `list_sessions` 排序改为目录 `mtime`（过渡不改返回类型）**

实现要点：

- 对每个会话目录读取目录本身 `metadata().modified()`
- 排序规则：`mtime` 降序；同值按 `id` 升序；无法读取 `mtime` 视为最旧
- 不依赖 `metadata.json` 中时间字段（即不再依赖 `created_at`）
- Task 1 阶段仅变更排序依据，不在此步切换到新的统一 `types::Session` 返回

- [ ] **Step 3: 编译检查 storage 改动可独立通过**

Run: `cargo check`
Expected: 编译通过，且不再有对 `created_at` 排序的依赖。

- [ ] **Step 4: 提交本任务**

```bash
git add src/session/storage.rs
git commit -m "refactor(session): move session ordering to directory mtime"
```

### Task 2: 收敛公开类型并在同阶段完成调用点最小兼容

**Files:**

- Modify: `src/session/types.rs`
- Modify: `src/session/service.rs`
- Modify: `src/session/mod.rs`
- Modify: `src/manager.rs`（若编译报错指向）

- [ ] **Step 1: 在 `types.rs` 定义唯一公开 Session 与公开结果类型**

实现要点：

- 新增/调整 `pub struct Session { id, session_dir, provider, languages }`
- 删除旧模型中的 `created_at` 字段及构造逻辑
- 将 `CreateSessionResult`、`SessionRemoveResult` 迁移到 `types.rs`
- 保留 `SessionKey`、`SessionError`、`SESSION_ID_LEN`

- [ ] **Step 2: 在 `service.rs` 删除重复公开类型定义并改用 `types` 导入**

实现要点：

- 删除 `service.rs` 中 `pub struct Session` 等公开类型定义
- 替换所有类型引用为 `crate::session::types::{...}`
- 统一字段访问为 `session.id`

- [ ] **Step 3: 在 `mod.rs` 统一 re-export 来源**

实现要点：

- 从 `types` 导出全部公开类型（含 `Session`、结果类型、错误与 key）
- `service` 仅导出 `SessionService`

- [ ] **Step 4: 按编译器报错最小修复调用点（同阶段完成）**

实现要点：

- 在 `service.rs`/`manager.rs` 按报错最小修复字段引用（例如 `session_id` -> `id`）
- 仅修复编译所需改动，不做无关清理

- [ ] **Step 5: 编译检查类型迁移与调用点兼容完整性**

Run: `cargo check`
Expected: 编译通过，无重复公开类型冲突、无 `created_at` 相关编译错误。

- [ ] **Step 6: 提交本任务**

```bash
git add src/session/types.rs src/session/service.rs src/session/mod.rs src/manager.rs
git commit -m "refactor(session): centralize public types and update call sites"
```

(若 `src/manager.rs` 无改动，提交时不要包含该文件)

### Task 3: 调用方最小兼容收尾

**Files:**

- Optional Modify: `src/manager.rs`（仅当编译错误指向时）
- Modify: `src/session/service.rs`（仅必要收尾）

- [ ] **Step 1: 按编译器报错最小修复调用点**

实现要点：

- 仅在报错指向时修复字段引用（如 `session_id` -> `id`）
- 不做无报错文件的主动改动

- [ ] **Step 2: 全量静态验证**

Run: `cargo check && cargo clippy --all-targets -- -D warnings`
Expected: 全部通过，无新增告警。

- [ ] **Step 3: 提交本任务**

```bash
git add src/session/service.rs src/manager.rs
git commit -m "refactor(session): update call sites for unified session type"
```

(若 `src/manager.rs` 无改动，提交时不要包含该文件)

### Task 4: 最终回归核对（无测试改动）

**Files:**

- Modify: (none required; only if发现小修复)

- [ ] **Step 1: 核对验收标准**

检查清单：

- 仅一个公开 `Session` 类型，且位于 `src/session/types.rs`
- `created_at` 已完全移除
- `list_sessions` 按 `session_dir` 目录 `mtime` 降序
- `storage` 未泄漏 `SessionMetadata`
- CLI 外部行为（use/restore/list/remove/clear）未改语义

- [ ] **Step 2: 运行最终命令验证**

Run: `cargo check && cargo clippy --all-targets -- -D warnings`
Expected: 通过。

- [ ] **Step 3: 汇总变更并提交最终修复（如有）**

```bash
git add <only-if-needed-files>
git commit -m "chore(session): finalize session type unification"
```

(若无额外改动，则跳过该提交)
