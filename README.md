<div align="center">

# DevClaw

### AI 编程工具统一管理客户端

[![Version](https://img.shields.io/github/v/release/AnXiYiZhi/DevCLaw?color=blue&label=version)](https://github.com/AnXiYiZhi/DevCLaw/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey.svg)](https://github.com/AnXiYiZhi/DevCLaw/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)

### 唯一官方网站：**[devclaw.cc.cd](https://devclaw.cc.cd)**

[English](README_EN.md) | 中文 | [日本語](README_JA.md) | [更新日志](CHANGELOG.md)

</div>

## 简介

DevClaw 是一款基于 Tauri 2 (Rust) 构建的轻量极速桌面客户端，一站式管理 7 款 AI 编程工具。内置 50+ 服务商预设、本地代理与自动故障转移、统一 MCP/Skills/Prompts 管理、费用追踪与 Token 统计，数据全部本地存储，100% 离线可用。

## 支持的工具

| 工具 | 说明 |
|------|------|
| Claude Code | Anthropic 官方 CLI |
| Claude Desktop | Anthropic 桌面客户端 |
| Codex | OpenAI CLI |
| Gemini CLI | Google CLI |
| OpenCode | 开源 AI 编程工具 |
| OpenClaw | 开源 AI 编程工具 |
| Hermes Agent | 开源 AI Agent |

## 核心功能

### 服务商管理

- **7 款工具，50+ 预设** — 涵盖官方 API、AWS Bedrock、各大中转平台，一键导入切换
- **通用服务商** — 一次配置同步到多个工具（OpenCode、OpenClaw）
- 系统托盘快切、拖拽排序、导入导出

### 本地代理与故障转移

- **内置反向代理** — 格式转换、自动故障转移、熔断保护、服务商健康监测
- **应用级接管** — 可独立代理 Claude、Codex 或 Gemini，精确到单个服务商

### MCP、Prompts 与 Skills

- **MCP 管理** — 统一管理 4 款应用的 MCP 服务器配置，双向同步
- **Prompts 管理** — Markdown 编辑器，跨应用同步（CLAUDE.md / AGENTS.md / GEMINI.md）
- **Skills 管理** — 从 GitHub 一键安装，支持自定义仓库

### 费用追踪

- **用量仪表盘** — 按模型、服务商统计花费和 Token 用量，支持自定义定价

### 会话管理

- 跨应用浏览、搜索、恢复对话历史

### 系统与平台

- **云端同步** — 支持 Dropbox、OneDrive、iCloud、WebDAV
- **Deep Link** — 通过 `devclaw://` 链接一键导入配置
- 深色 / 浅色 / 跟随系统主题、开机自启、自动更新、国际化（中/英/日）

## 截图

|                  主界面                  |                  添加服务商                  |
| :-----------------------------------------------: | :--------------------------------------------: |
| ![主界面](assets/screenshots/main-en.png) | ![添加服务商](assets/screenshots/add-en.png) |

## 下载

### 系统要求

- **Windows**：Windows 10 及以上
- **macOS**：macOS 12 (Monterey) 及以上

### 下载安装

前往 **[下载页](https://devclaw.cc.cd/download)** 获取最新版本，或从 [GitHub Releases](https://github.com/AnXiYiZhi/DevCLaw/releases/latest) 下载。

**Windows**：
- `DevClaw-v{version}-Windows-Portable.zip` — 解压即用
- `DevClaw-v{version}-Windows.msi` — 安装版

**macOS**：
- `DevClaw-v{version}-macOS.dmg` — 推荐，拖入应用程序

> macOS 版本已通过 Apple 公证，可直接安装使用。

## 快速上手

1. **添加服务商**：点击「添加服务商」→ 选择预设或自定义配置
2. **切换服务商**：主界面选择后点击「启用」，或从系统托盘直接切换
3. **生效**：重启终端或对应 CLI 工具（Claude Code 支持热切换，无需重启）
4. **恢复官方登录**：添加「官方登录」预设，重启后按提示完成 OAuth

### MCP、Prompts、Skills

- **MCP**：点击「MCP」按钮 → 通过模板或自定义配置添加服务器
- **Prompts**：点击「Prompts」→ 创建预设 → 激活后同步到对应文件
- **Skills**：点击「Skills」→ 浏览 GitHub 仓库 → 一键安装

## 开发

### 环境要求

- Node.js 18+
- pnpm 8+
- Rust 1.85+
- Tauri CLI 2.8+

### 常用命令

```bash
pnpm install          # 安装依赖
pnpm dev              # 开发模式（热重载）
pnpm typecheck        # 类型检查
pnpm format           # 格式化代码
pnpm test:unit        # 运行单元测试
pnpm build            # 构建应用
```

### 技术栈

**前端**：React 18 · TypeScript · Vite · TailwindCSS · TanStack Query · react-i18next · shadcn/ui

**后端**：Tauri 2 · Rust · serde · tokio · SQLite

## 贡献

欢迎提交 Issue 和建议！

提交 PR 前请确保通过：
- `pnpm typecheck`
- `pnpm format:check`
- `pnpm test:unit`

## 许可证

MIT
