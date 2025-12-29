# dot

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/username/dot)

一个用 Rust 编写的 Git 代理工具，用于管理隐藏目录的版本控制。

[English Documentation](../README.md) | [文档](.) | [示例](examples.md)

## 概述

`dot` 是一个强大的命令行工具，允许您管理隐藏目录（如 `.kiro`、`.config` 等）的版本控制，同时保持它们在公开的 GitHub 页面上不可见。它通过为每个隐藏目录维护独立的 Git 仓库，并原子性地同步所有仓库的操作来实现这一功能。

### 🚀 核心特性

- **多仓库管理**：管理多个隐藏目录，每个目录都有自己的 Git 仓库
- **原子性操作**：默认所有操作都是原子性的 - 要么所有仓库都成功，要么全部回滚
- **GitHub 集成**：自动在指定的 GitHub 组织中创建和管理隐藏仓库
- **全局索引**：维护所有项目及其关联隐藏仓库的全局索引
- **透明克隆**：克隆项目时，自动发现并克隆所有关联的隐藏仓库
- **智能密钥生成**：基于项目 URL 和目录路径生成唯一的仓库密钥
- **灵活配置**：基于 JSON 的配置，支持组织授权

## 📦 安装

### 快速安装（推荐）

```bash
git clone https://github.com/username/dot.git
cd dot
make install
```

### 手动安装

#### 前置要求

- [Rust](https://rustup.rs/) (1.70+)
- [Git](https://git-scm.com/)
- 具有仓库权限的 GitHub 个人访问令牌

#### 从源码安装

```bash
git clone https://github.com/username/dot.git
cd dot
cargo install --path .
```

#### 使用 Homebrew (macOS)

```bash
brew tap username/dot
brew install dot
```

#### 开发环境设置

```bash
git clone https://github.com/username/dot.git
cd dot
make setup  # 安装依赖并运行测试
```

## ⚙️ 设置

### 1. 设置 GitHub 令牌

创建一个具有仓库权限的 [GitHub 个人访问令牌](https://github.com/settings/tokens)：

```bash
export GITHUB_TOKEN="your_github_token_here"

# 永久设置
echo 'export GITHUB_TOKEN="your_github_token_here"' >> ~/.bashrc
# 或者对于 zsh
echo 'export GITHUB_TOKEN="your_github_token_here"' >> ~/.zshrc
```

### 2. 配置组织

首次使用时，`dot` 会创建 `~/.dot/dot.conf`。编辑此文件以添加授权的 GitHub 组织：

```bash
# 创建示例配置
make create-config

# 编辑配置
nano ~/.dot/dot.conf
```

配置格式：
```json
{
  "authorized_organizations": ["your-org", "another-org"],
  "default_organization": "your-org"
}
```

### 3. 验证安装

```bash
dot --version
dot --help
make check-install
```

## 🎯 快速开始

```bash
# 1. 使用隐藏目录初始化项目
cd your-project
dot init .kiro .config

# 2. 添加和提交文件
echo "secret config" > .kiro/settings.json
dot add .
dot commit -m "添加隐藏配置"

# 3. 推送到所有仓库
dot push

# 4. 在其他地方克隆项目（获取所有内容）
cd /tmp
dot clone git@github.com:user/your-project.git
```

## 📖 使用方法

### 初始化项目

在您的项目中使用一个或多个隐藏目录初始化 dot：

```bash
# 单个隐藏目录
dot init .kiro

# 多个隐藏目录
dot init .kiro .config .secrets

# 使用全局标志
dot init .kiro --no-atomic  # 禁用原子性操作
```

**发生的操作：**
- 检查 git 是否已初始化（如果没有则初始化）
- 验证 git remote origin 是否已设置
- 为每个隐藏目录创建独立的 Git 仓库
- 在全局索引中注册项目
- 将隐藏仓库发布到您配置的 GitHub 组织

### 检查状态

查看所有仓库的状态：

```bash
dot status

# 跳过隐藏仓库
dot status --skip-hidden
```

### 添加文件

将文件添加到所有相关仓库：

```bash
# 添加特定文件
dot add file1.txt .kiro/config.json

# 添加所有更改
dot add .

# 跳过隐藏仓库
dot add . --skip-hidden
```

### 提交更改

使用相同的消息提交所有仓库的更改：

```bash
dot commit -m "更新配置并添加新功能"

# 非原子模式（即使某些失败也继续）
dot commit -m "更新" --no-atomic
```

### 推送更改

将所有仓库推送到它们的远程仓库：

```bash
dot push

# 跳过隐藏仓库
dot push --skip-hidden

# 非原子模式
dot push --no-atomic
```

### 克隆项目

克隆项目并自动获取所有隐藏仓库：

```bash
# 克隆到默认目录名
dot clone git@github.com:user/project.git

# 克隆到特定目录
dot clone git@github.com:user/project.git my-project
```

### 全局标志

所有命令都支持这些标志：

| 标志 | 描述 |
|------|------|
| `--skip-hidden` | 跳过隐藏仓库的操作 |
| `--no-atomic` | 禁用原子性行为（即使某些操作失败也继续） |
| `--help` | 显示帮助信息 |

## 🔧 工作原理

### 仓库密钥

每个隐藏目录都有一个唯一的仓库密钥：

```
格式：{base_key}/{directory_path}

示例：
- 主仓库：git@github.com:user/project.git
- 隐藏目录：.kiro
- 仓库密钥：github.com/user/project/.kiro
```

### 全局索引

`dot` 在您的 GitHub 组织中维护一个全局 `.index` 仓库，用于跟踪：
- 所有注册的项目
- 关联的隐藏仓库
- 元数据（创建时间、git 用户、路径等）

### 原子性操作

默认情况下，所有多仓库操作都是原子性的：

1. **执行阶段**：首先在隐藏仓库上执行操作，然后在主仓库上执行
2. **回滚阶段**：如果任何操作失败，所有已完成的操作都会被回滚
3. **成功**：所有操作都成功完成

### 架构

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   主仓库        │    │  隐藏仓库 1      │    │  隐藏仓库 2     │
│  (公开)         │    │   (.kiro)        │    │   (.config)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌──────────────────┐
                    │  全局索引        │
                    │   (.index)       │
                    └──────────────────┘
```

## 📋 配置

### 配置文件

**位置**：`~/.dot/dot.conf`

**格式**：
```json
{
  "authorized_organizations": [
    "my-personal-org",
    "my-company-org"
  ],
  "default_organization": "my-personal-org"
}
```

### 环境变量

| 变量 | 描述 | 必需 |
|------|------|------|
| `GITHUB_TOKEN` | GitHub 个人访问令牌 | 是 |
| `GH_TOKEN` | GITHUB_TOKEN 的替代 | 是（如果未设置 GITHUB_TOKEN） |

### Makefile 命令

| 命令 | 描述 |
|------|------|
| `make install` | 构建并全局安装 |
| `make build` | 构建调试版本 |
| `make release` | 构建发布版本 |
| `make test` | 运行测试 |
| `make clean` | 清理构建产物 |
| `make setup` | 初始开发环境设置 |
| `make check-install` | 验证安装 |

## 📚 示例

### 完整工作流程

```bash
# 1. 设置新项目
mkdir my-project && cd my-project
git init
git remote add origin git@github.com:user/my-project.git

# 2. 使用隐藏目录初始化
dot init .kiro .config

# 3. 创建内容
echo "# My Project" > README.md
echo '{"theme": "dark"}' > .kiro/settings.json
echo 'debug=true' > .config/app.conf

# 4. 提交所有内容
dot add .
dot commit -m "初始项目设置"
dot push

# 5. 在其他地方克隆（获取所有内容）
cd /tmp
dot clone git@github.com:user/my-project.git
cd my-project
ls -la  # 显示 README.md, .kiro/, .config/
```

### 处理现有项目

```bash
# 检查项目是否已初始化 dot
dot status

# 初始化现有项目
dot init .kiro

# 克隆带有隐藏仓库的现有项目
dot clone git@github.com:user/existing-project.git
```

### 高级用法

```bash
# 非原子性操作（失败时继续）
dot add . --no-atomic
dot commit -m "部分更新" --no-atomic
dot push --no-atomic

# 跳过隐藏仓库
dot status --skip-hidden
dot push --skip-hidden

# 检查配置
make show-config
```

## 🐛 故障排除

### 常见问题

| 问题 | 解决方案 |
|------|----------|
| "组织未授权" | 将组织添加到 `~/.dot/dot.conf` |
| "GitHub API 错误" | 检查 `GITHUB_TOKEN` 权限 |
| "git 未安装" | 安装 Git 并添加到 PATH |
| "无效的 git remote origin URL" | 设置远程：`git remote add origin <url>` |

### 调试命令

```bash
# 检查安装
make check-install

# 显示配置
make show-config

# 验证 GitHub 令牌
echo $GITHUB_TOKEN

# 测试基本功能
dot --version
dot --help
```

### 获取帮助

1. 查看[文档](.)
2. 查看[示例](examples.md)
3. 阅读[故障排除指南](troubleshooting.md)
4. 提交[问题](https://github.com/username/dot/issues)

## 🤝 贡献

我们欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 开发

```bash
# 设置开发环境
git clone https://github.com/username/dot.git
cd dot
make setup

# 运行测试
make test

# 格式化和检查
make fmt
make clippy

# 构建
make build
```

### 项目结构

```
dot/
├── src/                 # 源代码
│   ├── main.rs         # CLI 入口点
│   ├── config.rs       # 配置管理
│   ├── index.rs        # 全局索引管理
│   ├── repository.rs   # 仓库操作
│   ├── atomic.rs       # 原子性操作
│   └── ...
├── docs/               # 文档
├── Formula/            # Homebrew 公式
├── Makefile           # 构建自动化
└── README.md          # 主文档
```

## 📄 许可证

本项目采用 MIT 许可证 - 详情请查看 [LICENSE](../LICENSE) 文件。

## 🙏 致谢

- 使用 [Rust](https://www.rust-lang.org/) 构建
- 使用 [clap](https://clap.rs/) 进行 CLI 解析
- 通过 [octocrab](https://github.com/XAMPPRocky/octocrab) 集成 GitHub
- 使用 [git2](https://github.com/rust-lang/git2-rs) 进行 Git 操作

---

**用 ❤️ 和 Rust 制作**