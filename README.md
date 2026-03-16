# ah.sh（命令：`ah`）

把「临时使用哪些语言」变成一条极短的命令：**然后你就得到了你想要的开发环境**。

```bash
ah use rust go js
```

对吧，它真的很简单，短且快!

- 可多语言组合
- 自动管理版本
- 具备缓存及提供会话恢复
- 使用社区最新的配置

`ah` 基于 nix/flake 实现，目标很也简单：
为nix用户提供另外一个 devshells 的快速逃生方式。

- 不再反复手写/复制 `flake.nix`
- 不再专门维护一份临时开发环境的nix template
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

## 核心特性

- **极简入口**：`ah use <languages...>` 直接进入 `nix develop`。
- **会话管理**：列出 / 恢复 / 删除 / 清空，常用组合不再重复搭。
- **Provider 可切换**：同样语言列表，用不同方案生成 dev shell。
- **语言别名**：如 `js`、`ts` 等别名按 provider 维度映射，输入更顺手。
- **Nix 原生**：你仍然在用 `nix develop`；`ah` 只负责把“语言列表 → 可进入的 dev shell”这件事做顺。

## 安装

### 先决条件

你需要安装 **Nix** 作为底座。
请参考：[Nix 安装说明](https://nix.dev/install-nix)

> **给非 Nix 用户的建议**：你 **不需要学习** Nix 复杂的表达式和底层原理，你只需要安装它。`ah` 会为你屏蔽掉所有复杂的 Nix 命令细节。

### 快速体验

无需安装，直接试用：

```bash
nix run github:z1-0/ah.sh -- --help
```

### Flake 安装方式（推荐）

如果你使用 Nix Flake 管理系统，在 `flake.nix` 的 `inputs` 中添加：

```nix
inputs = {
    ah = {
      url = "github:z1-0/ah.sh";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        treefmt-nix.follows = "treefmt-nix";
        git-hooks-nix.follows = "git-hooks-nix";
      };
    };
    # ... 其他 inputs
};
```

然后在你的配置中引用：

```nix
{
  environment.systemPackages = [ inputs.ah.packages.${system}.default ];
}
```

### 添加 Binary Cache（可选，加速安装）

为了避免在本地从源码编译，建议添加二进制缓存。

**方式 A：在 Flake 中配置（局部有效）**
在你的 `flake.nix` 中添加 `nixConfig`：

```nix
{
  nixConfig = {
    extra-substituters = [ "https://z1-0.cachix.org" ];
    extra-trusted-public-keys = [ "z1-0.cachix.org-1:mAd5hSyjiIzSLbMFGFaI3Xhb1GhkEm7Q+ITqTO5gxVw=" ];
  };
}
```

**方式 B：在 NixOS 系统配置中设置（全局生效）**
在 `configuration.nix` 中添加：

```nix
nix.settings = {
  extra-substituters = [ "https://z1-0.cachix.org" ];
  extra-trusted-public-keys = [
    "z1-0.cachix.org-1:mAd5hSyjiIzSLbMFGFaI3Xhb1GhkEm7Q+ITqTO5gxVw="
  ];
};
```

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

- 管理当前会话
- 更友好的输出（更清晰的错误提示、可复制的建议命令）
- 为脚本/CI 提供机器可读输出（如 `--json`）

---

## License

MIT License，见 [LICENSE](./LICENSE)。
