# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目简介

EarDo（耳朵）是一个参数化 + 可视化的 AI 声音创作平台，基于 Rust 全栈框架 Leptos 构建。用户可完成"文本 → 调声 → 生成 → 播放/分享"全链路操作。后端调用阿里云 CosyVoice v3 flash 进行 TTS 语音合成。

## 常用命令

```bash
# 开发（热重载）
cargo leptos watch

# 构建（release）
cargo leptos build --release

# 运行服务端（不使用 cargo-leptos）
cargo run -p server

# 格式化
cargo fmt

# 端到端测试（Playwright）
cd end2end && npm install && npx playwright test
```

需要安装 `cargo-leptos`（`cargo install cargo-leptos`）和 Rust nightly 工具链（target: `wasm32-unknown-unknown`）。

## 环境变量

- `ALIYUN_API_KEY`：阿里云 CosyVoice TTS API Key（必需）
- `PG_DATABASE_URL`：PostgreSQL 连接串，默认 `postgres://postgres:postgres@localhost:5432/eardo`

## 架构概览

Cargo workspace 包含三个 crate：

```
app/       → 共享 crate：API 定义（trait + server function）、页面组件、数据模型
frontend/  → WASM 入口（cdylib），仅调用 hydrate()，编译为浏览器端代码
server/    → 服务端入口（Axum binary），组装 AppState、注册路由、启动 HTTP 服务器
```

### 前后端同构模式

项目使用 Leptos 0.8 的 SSR + CSR Hydration 模式：
- `app` crate 通过 Cargo feature 区分编译目标：`ssr`（服务端）和 `hydrate`（浏览器 WASM）
- `#[server]` 宏标记的函数在服务端执行，前端通过自动生成的 RPC 调用
- `#[cfg(feature = "ssr")]` 守护服务端专属代码（数据库、WebSocket 等）
- `frontend` crate 仅在 `hydrate` feature 下编译，输出 WASM

### Service Provider 模式

业务逻辑通过 trait 抽象 + Leptos Context 注入：

1. `app/src/api.rs` 定义了三组 trait：
   - `AuthService` / `UserService`（用户认证与资料）
   - `VoiceService`（声音模型、元数据、语音生成）
   - `VoiceMetaPostService` / `VoicePostService`（帖子/作品流）
2. `app/src/api/` 下的 `*impl.rs` 文件实现这些 trait（基于 PostgreSQL / sqlx）
3. `ServiceProvider<PgPool>` 统一实现所有 trait，在 `server/src/main.rs` 中实例化并包装为 `Arc<dyn XxxService>`
4. 通过 `provide_context()` 注入到 Leptos 的 server function 和 SSR handler 中

### 路由

- 页面路由定义在 `app/src/lib.rs`（Leptos Router）
- API 路由在 `server/src/main.rs`（Axum Router），包括：
  - `/api/{*fn_name}` → server function handler
  - `/api/audio/{id}`、`/api/avatar/{user_id}`、`/api/post/audio/{post_id}`、`/api/voice_avatar/{voice_id}` → 媒体流端点

### TTS 生成链路

`app/src/api/voice_backend.rs`（SSR-only）通过 WebSocket 调用阿里云 CosyVoice API，流式接收 MP3 数据，累积后存入 PostgreSQL 的 `voice_library` 表。

## 关键代码位置

| 功能 | 文件 |
|------|------|
| 路由 & App 壳 | `app/src/lib.rs` |
| API trait 定义 + server function | `app/src/api.rs` |
| 用户认证/资料实现 | `app/src/api/userimpl.rs` |
| 语音服务实现 | `app/src/api/voiceimpl.rs` |
| CosyVoice WebSocket 调用 | `app/src/api/voice_backend.rs` |
| 帖子/作品流实现 | `app/src/api/postimpl.rs` |
| 页面组件 | `app/src/pages/*.rs` |
| 服务端入口 & AppState | `server/src/main.rs` |
| WASM hydrate 入口 | `frontend/src/lib.rs` |
| Tailwind 主题配置 | `style/tailwind.css` |
| CI 构建 | `.github/workflows/build.yml` |

## 开发注意事项

- 数据库使用 PostgreSQL（sqlx），非 README 中提到的 SQLite（已迁移）
- 新增 server function 时，若需要数据库或 Provider，通过 `use_context::<XxxProvider>()` 获取
- 新增 Provider 需要在 `server/src/main.rs` 的 `server_fn_handler` 和 `ssr_handler` 两处同时注入 `provide_context()`
- 样式使用 Tailwind CSS v4（`@import 'tailwindcss'` 语法），自定义主题色定义在 `style/tailwind.css` 的 `@theme` 块中
- Rust edition 2024，使用 nightly 工具链
