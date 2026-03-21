# ah.sh

## 为什么是 ah.sh

如果你需要精细化管理项目级的开发环境请使用 mise, asdf, 或 nix 生态的 nix-shell / devenv

如果你仅仅需要快速进入一个临时的开发环境，请使用 ah.sh

`ah.sh` 使用最贴近人脑的表达方式，没有任何负担：

临时使用 rust go js

```bash
ah use rust go js
```

---

## 核心特性

<details>
<summary><b>极简入口，隐式调用子命令</b></summary>

`ah use <LANGUAGES...>` = `ah <LANGUAGES...>`

`ah restore <KEY>` = `ah session restore <KEY>`

</details>

<details>
<summary><b>支持多语言组合</b></summary>

`ah use <LANGUAGES...>` = `ah <LANGUAGES...>`

`ah restore <KEY>` = `ah session restore <KEY>`

</details>

<details>
<summary><b>环境会话管理</b></summary>

支持列出 / 恢复 / 删除 / 清空历史会话，常用开发环境组合一键召回，不再重复搭建。

</details>

<details>
<summary><b>使用社区最新配置</b></summary>

提供两个社区开源项目的配置，安全，透明，没有维护负担

dev-templates

devenv

</details>

<details>
<summary><b>语言缩写</b></summary>

智能映射 `js`、`ts` 等语言缩写，输入更顺手。

</details>

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

## 查看支持语言

```bash
ah provider list
ah provider show dev-templates
ah provider show devenv
ah provider show all
```

````

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
````

---

## Roadmap（克制版）

- 管理当前会话
- 重构cli.rs, provider/mod.rs
- 更友好的输出（更清晰的错误提示、可复制的建议命令）
- 为脚本/CI 提供机器可读输出（如 `--json`）

---

## License

MIT License，见 [LICENSE](./LICENSE)。
