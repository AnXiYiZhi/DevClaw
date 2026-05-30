<div align="center">

# DevClaw

### AI コーディングツール統合管理クライアント

[![Version](https://img.shields.io/github/v/release/AnXiYiZhi/DevCLaw?color=blue&label=version)](https://github.com/AnXiYiZhi/DevCLaw/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey.svg)](https://github.com/AnXiYiZhi/DevCLaw/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)

### 公式サイト：**[devclaw.cc.cd](https://devclaw.cc.cd)**

[English](README_EN.md) | [中文](README.md) | 日本語 | [変更履歴](CHANGELOG.md)

</div>

## 概要

DevClaw は Tauri 2 (Rust) で構築された軽量高速デスクトップアプリで、7 つの AI コーディングツールを一元管理できます。50 以上のプロバイダープリセット、ローカルプロキシと自動フェイルオーバー、MCP/Skills/Prompts の統合管理、使用量・コスト追跡、100%ローカルデータ保存を備えています。

## 対応ツール

| ツール | 説明 |
|--------|------|
| Claude Code | Anthropic 公式 CLI |
| Claude Desktop | Anthropic デスクトップアプリ |
| Codex | OpenAI CLI |
| Gemini CLI | Google CLI |
| OpenCode | オープンソース AI コーディングツール |
| OpenClaw | オープンソース AI コーディングツール |
| Hermes Agent | オープンソース AI Agent |

## 主な機能

### プロバイダー管理

- **7 ツール、50 以上のプリセット** — 公式 API、AWS Bedrock、各種中継サービス対応、ワンクリックでインポート・切り替え
- **ユニバーサルプロバイダー** — 1 回の設定で複数ツールに同期（OpenCode、OpenClaw）
- システムトレイからのクイック切替、ドラッグ＆ドロートで並べ替え、インポート/エクスポート

### ローカルプロキシとフェイルオーバー

- **内蔵リバースプロキシ** — フォーマット変換、自動フェイルオーバー、サーキットブレーカー、プロバイダー健全性監視
- **アプリレベルのテイクオーバー** — Claude、Codex、Gemini を個別にプロキシ可能

### MCP、Prompts、Skills

- **MCP 管理** — 4 つのアプリの MCP サーバー設定を統合、双方向同期
- **Prompts 管理** — Markdown エディタ、クロスアプリ同期（CLAUDE.md / AGENTS.md / GEMINI.md）
- **Skills 管理** — GitHub リポジトリからワンクリックインストール、カスタムリポジトリ対応

### 使用量・コスト追跡

- **使用量ダッシュボード** — モデル・プロバイダー別のコストとトークン使用量を追跡、カスタム価格設定対応

### セッション管理

- 全アプリの会話履歴を閲覧・検索・復元

### システムとプラットフォーム

- **クラウド同期** — Dropbox、OneDrive、iCloud、WebDAV 対応
- **Deep Link** — `devclaw://` URL で設定をインポート
- ダーク / ライト / システムテーマ、自動起動、自動アップデート、多言語対応（日/英/中）

## スクリーンショット

|                  メイン画面                  |                  プロバイダー追加                  |
| :-----------------------------------------------: | :--------------------------------------------: |
| ![メイン画面](assets/screenshots/main-en.png) | ![プロバイダー追加](assets/screenshots/add-en.png) |

## ダウンロード

### システム要件

- **Windows**: Windows 10 以降
- **macOS**: macOS 12 (Monterey) 以降

### インストール

**[ダウンロードページ](https://devclaw.cc.cd/download)** または [GitHub Releases](https://github.com/AnXiYiZhi/DevCLaw/releases/latest) から最新版を入手してください。

**Windows**:
- `DevClaw-v{version}-Windows-Portable.zip` — ポータブル版、展開して実行
- `DevClaw-v{version}-Windows.msi` — インストーラー

**macOS**:
- `DevClaw-v{version}-macOS.dmg` — 推奨、アプリケーションにドラッグ

> macOS 版は Apple によって公証されています。そのままインストール・起動できます。

## クイックスタート

1. **プロバイダー追加**: 「プロバイダー追加」をクリック → プリセットを選択またはカスタム設定を作成
2. **プロバイダー切替**: プロバイダーを選択 → 「有効にする」をクリック、またはシステムトレイから切替
3. **反映**: ターミナルまたは CLI ツールを再起動（Claude Code はホット切替対応）
4. **公式に戻す**: 「公式ログイン」プリセットを追加、再起動後に OAuth フローに従う

### MCP、Prompts、Skills

- **MCP**: 「MCP」をクリック → テンプレートまたはカスタム設定でサーバー追加
- **Prompts**: 「Prompts」をクリック → プリセット作成 → 有効化で同期
- **Skills**: 「Skills」をクリック → GitHub リポジトリを閲覧 → ワンクリックインストール

## 開発

### 必要要件

- Node.js 18+
- pnpm 8+
- Rust 1.85+
- Tauri CLI 2.8+

### コマンド

```bash
pnpm install          # 依存関係のインストール
pnpm dev              # 開発モード（ホットリロード）
pnpm typecheck        # 型チェック
pnpm format           # コードフォーマット
pnpm test:unit        # ユニットテスト実行
pnpm build            # アプリケーションビルド
```

### 技術スタック

**フロントエンド**: React 18 · TypeScript · Vite · TailwindCSS · TanStack Query · react-i18next · shadcn/ui

**バックエンド**: Tauri 2 · Rust · serde · tokio · SQLite

## 貢献

Issue と提案を歓迎します！

PR を提出する前に以下を確認してください：
- `pnpm typecheck`
- `pnpm format:check`
- `pnpm test:unit`

## ライセンス

MIT
