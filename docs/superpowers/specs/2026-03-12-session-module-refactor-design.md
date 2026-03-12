# 会话模块功能分组重构设计

日期：2026-03-12

## 目标

将会话相关逻辑从 `session.rs` 与 `session_service.rs` 的平级结构，
调整为 Rust 风格的功能分组目录结构，保持对外调用链不变
（CLI → Manager → SessionService）。

## 方案概述

采用 `src/session/` 目录进行功能分组，通过 `session/mod.rs` 统一
声明与 re-export，对外仍使用 `crate::session::*`。

## 模块划分

- `types.rs`
  - `Session`
  - `SessionKey`
  - `SessionError`
  - `SESSION_ID_LEN`
- `storage.rs`
  - 会话目录、ID 生成、metadata 读写与 CRUD（list/save/find/delete/clear）
  - **可见性**：`pub(crate)`，仅供 `session` 模块内部使用
- `service.rs`
  - `SessionService`
  - `SessionRemoveResult`
- `mod.rs`
  - `mod types; mod storage; mod service;`
  - `pub use` 对外统一导出：
    - `SessionService`
    - `SessionRemoveResult`
    - `Session`
    - `SessionKey`
    - `SessionError`
    - `SESSION_ID_LEN`

## 组件与数据流

- 组件职责
  - `types`: 数据结构与解析（`SessionKey`、`Session`）
  - `storage`: 仅文件系统读写与查询
  - `service`: 业务流程（创建、移除、解析目录）
- 数据流
  - CLI → `Manager` → `SessionService`
  - `SessionService` 调用 `storage` 完成读写
  - `SessionKey` 解析仍在 CLI 输入阶段完成

## 对外接口保持不变

- 通过 `crate::session::*` 继续使用：
  - `SessionService`、`SessionKey`、`SessionError` 等
- 调用链不变，仅更新引用路径：
  - `crate::session_service::SessionService` → `crate::session::SessionService`

## 错误处理

- `SessionError` 仍由 `session` 模块对外 re-export
- `AppError::Session` 保持不变（`error.rs` 不需改动）
- `storage` 继续通过 `AppError` 上抛错误

## 实施要点

- 新增：`src/session/mod.rs`、`types.rs`、`storage.rs`、`service.rs`
- 删除：`src/session.rs`、`src/session_service.rs`
- 更新：`src/lib.rs`、`src/manager.rs` 等引用路径

## 验证方式

- `cargo check`
- 若有测试：`cargo test`
- 手动验证（可选）：`ah session list / clear`
