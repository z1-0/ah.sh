# ah.sh（命令：`ah`）

把「临时使用哪些语言」变成一条极短的命令：**然后你就得到了你想要的开发环境**。

```bash
ah use rust go js
```

对吧，它真的很简单，也很短，但是它短小精悍：

- 可多语言组合
- 自动管理版本
- 具备缓存及提供会话恢复

`ah` 基于 nix/flake 实现，目标很也简单：
为nix用户提供一个更友好的 devshells 逃生方式。

- 不再反复手写/复制 `flake.nix`
- 不再记一堆目录、脚本或历史命令
- 用同一套语言组合，稳定、快速地重复进入开发环境

---

## 为什么是 ah.sh（设计理念）

Nix 很强，但“临时组一个 dev shell”往往成本不低：你要写/改 flake、找模板、或者维护一堆项目目录。

`ah` 选择把入口降到**语言列表**这个最贴近人脑的表达方式：

- **语言组合**就是你的意图（`rust + go + nodejs`）
- **会话**就是可复用的结果（下次一键恢复）
- **provider**决定你偏好的生成方式（模板拼装 or `devenv` 风格声明）

---

## 典型使用场景

- **频繁切换语言栈**：在 `rust + go`、`nodejs + python` 等组合之间来回。
- **多项目同栈复用**：同一套语言组合在不同仓库里保持一致。
- **临时试验 / 教学演示**：不写配置，直接开一个可用的 dev shell。
- **团队约定**：用固定 provider + 语言列表减少环境差异。

---

## 核心特性

- **极简入口**：`ah use <languages...>` 直接进入 `nix develop`。
- **会话管理**：列出 / 恢复 / 删除 / 清空，常用组合不再重复搭。
- **Provider 可切换**：同样语言列表，用不同方案生成 dev shell。
- **语言别名**：如 `js`、`ts` 等别名按 provider 维度映射，输入更顺手。
- **Nix 原生**：你仍然在用 `nix develop`；`ah` 只负责把“语言列表 → 可进入的 dev shell”这件事做顺。

> 你需要可用的 `nix` 命令，并建议启用 Flakes。

---

## 快速上手（3 分钟）

### 1) 创建并进入一个语言组合

```bash
ah use rust go
```

更多组合示例：

```bash
ah use nodejs python
ah use rust go nodejs
```

### 2) 查看已有会话并恢复

```bash
ah session list

# 用序号恢复
ah session restore 1

# 或用会话 ID（8 位十六进制）恢复
ah session restore deadbeef
```

### 3) 删除 / 清空会话

```bash
# 删除一个或多个
ah session remove 1 2
ah session remove deadbeef cafebabe

# 清空全部（交互式终端会二次确认）
ah session clear
```

---

## Provider：两种风格，一样的入口

`provider` 决定“语言列表如何变成 dev shell”。你可以按偏好选择：

| Provider                | 适合你如果…                    | 关键词           |
| ----------------------- | ------------------------------ | ---------------- |
| `dev-templates`（默认） | 想用现成模板快速拼装语言环境   | 上手快、组合直观 |
| `devenv`                | 偏好 `devenv` 风格声明语言能力 | 语义清晰、可扩展 |

切换 provider：

```bash
ah -p dev-templates use rust go
ah -p devenv       use rust go
```

---

## 支持语言与别名（快速查询）

```bash
ah provider list
ah provider show dev-templates
ah provider show devenv
ah provider show all
```

示例输出：

```text
$ ah provider list
Index Provider
1     devenv
2     dev-templates
```

`provider show` 会输出支持的语言，并在有别名时以括号展示：

```text
$ ah provider show devenv
cplusplus (c++,cpp)
go
java
javascript
...
```

```text
$ ah provider show dev-templates
c-cpp (c,c++,cpp)
go
node (javascript,typescript)
...
```

> 注：示例仅展示片段，你的输出会随版本更新而变化。

---

## 与“直接用 Nix”相比，ah.sh 帮你省掉什么？

| 方式                | 你需要做的事             | ah.sh 的优势                              |
| ------------------- | ------------------------ | ----------------------------------------- |
| 手写/维护 flake     | 写配置、改配置、复制粘贴 | `ah use ...` 用语言列表表达意图，速度更快 |
| 直接 `nix develop`  | 记住路径/模板/组合方式   | `ah` 用会话把常用组合固化成“一键恢复”     |
| 现成模板 + 手动组合 | 自己拼装、处理冲突       | `provider` 把“组合方式”变成可切换策略     |

`ah` 并不替代 Nix：它只是把你每天都在做的那一步变得更短、更稳定。

---

## 使用建议（Best Practices）

- **先选 provider，再固定团队习惯**：
  - 团队偏“模板即实践”→ 用 `dev-templates`
  - 团队偏“声明式能力开关”→ 用 `devenv`
- **把语言当作接口**：尽量用 canonical 名称（或项目约定的别名），减少歧义。
- **小步组合**：先 `ah use rust` 再加 `go/nodejs`，更容易定位不支持的语言。
- **用 `provider show` 做事实来源**：当你不确定语言名/别名时，先查再输。

---

## 命令速查（Cheat Sheet）

```text
ah [OPTIONS] <COMMAND>

Commands:
  use       Create a development session
  session   Manage development sessions
  provider  Inspect available providers
  help      Print this message or the help of the given subcommand(s)

Options:
  -p, --provider <PROVIDER>  [default: dev-templates] [possible values: devenv, dev-templates]
  -h, --help
  -V, --version
```

---

## Roadmap（克制版）

- 更丰富的语言别名与 provider 支持
- 更友好的输出（更清晰的错误提示、可复制的建议命令）
- 为脚本/CI 提供机器可读输出（如 `--json`）

---

## Contributing

欢迎 Issue / PR（越可复现越好）：

- Bug：请附上你的命令、期望行为与实际输出
- 新语言/别名：请说明目标 provider、语言名以及期望映射规则

---

## License

MIT License，见 [LICENSE](./LICENSE)。
