# Session 模块重构设计（统一 Session 类型）

日期：2026-03-21
范围：`src/session/*`（最小改动，不改变外部行为）

## 1. 背景与问题

当前 `session` 子模块存在两个 `Session` 类型：

- 持久化模型（metadata 语义）
- service 运行时模型（含 `session_dir`）

导致：

- 类型语义重复，命名不一致（`id` vs `session_id`）
- 跨层边界不清晰，storage 细节向上泄漏
- 维护成本增高（字段调整需双点同步）

## 2. 目标与非目标

### 目标

1. 统一为一个公开 `Session` 类型
2. 删除 `created_at`
3. `session list` 改为按 `session_dir` 的 `mtime` 降序（最新在前）
4. 所有**非私有类型**集中在 `src/session/types.rs`
5. 保持现有 CLI 外部行为与命令语义不变

### 非目标

- 不调整 provider 行为
- 不修改 CLI 参数协议
- 本次不做测试改动

## 3. 分层设计

### 3.1 types 层（公开类型唯一出口）

文件：`src/session/types.rs`

保留/定义公开类型：

- `pub struct Session { id, session_dir, provider, languages }`
- `pub enum SessionKey`
- `pub enum SessionError`
- `pub const SESSION_ID_LEN`

说明：

- `created_at` 从公开模型中移除
- 以后 `session` 子模块所有公开类型均从 `types.rs` 导出

### 3.2 storage 层（私有持久化实现）

文件：`src/session/storage.rs`

新增私有 DTO（不导出）：

- `struct SessionMetadata { id, provider, languages }`

职责：

- metadata JSON 读写
- session 目录扫描
- mtime 获取与排序
- 对外仅返回 `types::Session` 或最小 primitive

边界规则：

- storage 不向外暴露 metadata 结构
- service 不关心 JSON 结构细节

### 3.3 service 层（业务编排）

文件：`src/session/service.rs`

- 删除本地重复 `Session` 定义
- 全部改用 `types::Session`
- 保持现有业务流程：find/create/list/remove/resolve
- 只做业务编排，不承载持久化细节

### 3.4 模块导出

文件：`src/session/mod.rs`

- 从 `types` 导出 `Session` 与其它公开类型
- `service` 仅导出服务对象 `SessionService`
- `mod.rs` 只做 re-export，不新增任何类型定义

### 3.5 非私有类型硬约束

- `src/session/service.rs` 不定义任何新的非私有类型（不新增 `pub struct` / `pub enum` / `pub type`）
- `src/session/storage.rs` 不导出持久化 DTO（`SessionMetadata` 必须保持私有）
- `src/session` 子模块所有 `pub` 类型统一定义在 `src/session/types.rs`
- `CreateSessionResult`、`SessionRemoveResult` 迁移到 `src/session/types.rs`

## 4. 排序设计（替代 created_at）

排序输入：每个 `session_dir` **目录本身** 的 `std::fs::metadata(...).modified()`（不读取 `metadata.json` 或其他子文件时间）

排序规则：

1. 先按 `mtime` 降序（最新在前）
2. 同时间戳时按 `id` 升序稳定打散
3. 无法读取 `mtime` 的项视为最旧，排在末尾

该排序由 `storage::list_sessions()` 完成，`SessionService::list_sessions()` 不再重复排序。

## 5. 兼容性与行为保证

- `ah session list`：仍输出 Index/ID/Provider/Languages，仅排序依据变更为目录 mtime
- `ah use` / `restore` / `remove` / `clear`：外部行为保持
- 个别会话目录 `mtime` 不可读时，仅影响该会话的排序位置，不影响其展示与恢复能力
- `id` 生成规则不变（仍由 provider + sorted languages hash）

## 6. 变更清单（最小改动）

1. `src/session/types.rs`
   - 合并公开 `Session`
   - 移除 `created_at`
2. `src/session/service.rs`
   - 删除本地 `Session` 结构
   - 改用 `types::Session`
   - 修正字段访问（`session_id` -> `id`）
3. `src/session/storage.rs`
   - 引入私有 `SessionMetadata`
   - `list_sessions` 改按 `session_dir.mtime` 排序
4. `src/session/mod.rs`
   - 统一 re-export 来源
5. `src/manager.rs`
   - 兼容字段名调整（若有引用）

## 7. 风险与缓解

风险：

- 文件时间在不同文件系统精度差异可能影响同秒内顺序
- 历史目录/权限异常导致个别目录无 mtime

缓解：

- 引入 `id` 次排序保证稳定性
- mtime 读取失败降级为“最旧”而非整体失败

## 8. 实施验收标准

1. 代码中仅存在一个公开 `Session` 类型（位于 `types.rs`）
2. `created_at` 字段与相关依赖彻底移除
3. `session list` 按 `session_dir` mtime 降序
4. storage 私有 metadata 类型不向上层泄漏
5. manager/cli 行为对用户保持一致
