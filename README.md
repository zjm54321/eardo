<div align="center">
	<img src="public/logo.png" alt="EarDo Logo" width="140" />
	<h1>耳朵 · EarDo</h1>
	<p>参数化 + 可视化的 AI 声音创作平台（文本 → 调声 → 生成 → 播放/分享）</p>
</div>

---

## 目录
- [目录](#目录)
- [项目概览](#项目概览)
- [产品亮点](#产品亮点)
- [典型用户场景](#典型用户场景)
- [核心功能](#核心功能)
- [系统架构](#系统架构)
- [交互与页面导览](#交互与页面导览)
- [端到端链路](#端到端链路)
- [数据与接口总览](#数据与接口总览)
- [性能与容量预估](#性能与容量预估)
- [部署与环境](#部署与环境)
- [安全与合规](#安全与合规)
- [测试与验收](#测试与验收)
- [路线图](#路线图)
- [常见问题 FAQ](#常见问题-faq)
- [源码索引](#源码索引)

## 项目概览
EarDo 希望把专业声学参数“看得懂、调得动、存得下、传得出”。通过滑条和预设，用户可以在几分钟内完成文本转语音，并感受音高、语速的即时变化，再一键发布或分享。当前版本已跑通核心链路，并具备账号体系、作品流展示与音频直链播放能力。

## 产品亮点
1. **低门槛**：默认参数即刻可用，零学习成本完成首次生成。
2. **所见即所得**：滑条调节即时生效，预设帮助非专业用户理解“音色”变化。
3. **闭环完整**：文本 → 生成 → 播放 → 发布/分享，一条链路不中断。
4. **可扩展**：模型、滤镜、存储均做可替换设计，能平滑接入 RVC、DSP、云存储或计费模块。
5. **轻量部署**：PostgreSQL + 云端 TTS，无需本地 GPU，适合演示与小规模试运营。

## 典型用户场景
- **短视频创作者**：快速做差异化配音，提升产出效率。
- **教育讲解**：保持稳定、清晰的讲解音频，便于批量生成课程片段。
- **游戏/二创**：角色感语音生成，用于剧情、MOD 或同人作品。
- **入门爱好者**：不懂声学也能通过预设与滑条听出变化，逐步沉淀个人声线。

## 核心功能
- **AI 文本改写**：集成 DeepSeek-V3.1（OpenAI 兼容接口），支持场景化改写（汇报、讲解、创意等），可控时长与受众。
- **文本转语音（TTS）**：调用阿里云 CosyVoice v3 flash，WebSocket 流式接收 MP3；支持参数控制与指令控制两种模式。
- **参数化调声与指令模式**：参数模式精确调节 `pitch`/`speed`/`volume`；指令模式支持自然语言输入（如"温柔点、快一些"）。
- **声线能力标记**：每个声线模型具备 `VoiceModelAbility`，标明是否支持 voice_clone、voice_design、SSML、LaTeX、参数控制、指令控制。
- **播放与分发**：生成音频直接 `<audio>` 播放；作品随帖子发布，支持搜索与详情。
- **账号与会话**：多方式注册（邮箱/手机号），前端 SHA256 哈希传输 + 后端 bcrypt 存储，HttpOnly Cookie 会话；OAuth（GitHub/WeChat）与 Passkey 接口已备。
- **双类型作品流**：`VoiceMetaPost`（分享声线配置）和 `VoicePost`（分享实际音频），各自独立支持列表、搜索、点赞/评论。
- **渐进式配置**：专用 Setup 页面可视化构建声线预设，支持快速切换与分享链接。

## 系统架构
```
浏览器 (Leptos 0.8 / WASM)
		│  SSR 首屏 + CSR Hydration / 参数滑条 / 播放控件
		▼
Axum 0.8 + Leptos Server Functions
		│  鉴权 / 元数据 / 生成调度 / 帖子
		├──────────────────────────┐
		▼                         ▼
CosyVoice TTS (阿里云)    AI 改写 (DeepSeek-V3.1)
WebSocket 流式 MP3         reqwest + OpenAI 兼容接口
		▼
PostgreSQL (用户 / 会话 / 声线模型 / 帖子 / 音频)
```
- 前端：Leptos 0.8 路由与组件，SSR 首屏 + CSR Hydration，样式走 Tailwind CSS v4。
- 后端：Axum 0.8 + Leptos Server Functions，Service Provider 模式（`Arc<dyn XxxService>`）统一注入各服务。
- 存储：PostgreSQL（sqlx 0.8），音频/头像以二进制（BYTEA）存入 DB，读取失败时回退 DiceBear API。

## 交互与页面导览
- **欢迎页**：品牌展示与引导 CTA，默认入口。
- **首页 /home**：主要调声与生成入口，支持双模式：
	- **参数模式**：文本输入 + 声线选择 + 参数滑条（pitch/speed/volume）+ 生成。
	- **指令模式**：文本输入 + 自然语言指令 + 生成（如"温柔、放慢速度"）。
	- 集成 AI 文本改写：场景化改写（汇报/讲解/创意等）可控时长与受众。
	- 快速导航到渐进式配置页。
- **渐进式配置 /setup**：可视化构建声线预设，即时预听，支持保存为链接分享。
- **声音广场 /voice**：浏览作品卡片，播放、查看描述与互动计数。
- **声音滤镜 /filters**：预设声线/滤镜列表（数据结构已备，后端可扩展）。
- **帮助 /help**：操作指引与常见问题。
- **登录/注册 /login /register**：账号体系入口，支持邮箱/手机号注册，未登录时 Header 显示"登录"。
- **个人资料 /profile**：头像、昵称、简介；支持图片裁剪上传（webp 格式存储）。
- **账号设置 /settings**：修改密码、绑定/解绑邮箱或手机号、管理第三方登录（OAuth）。

## 端到端链路
1. **输入阶段**：用户输入文本，可选 AI 改写（选择场景、受众、时长）。
2. **调声阶段**：选择声线，支持两种调声方式：
	- **参数模式**：精确调节 `pitch/speed/volume` 滑条，默认值保证开箱即用。
	- **指令模式**：输入自然语言指令，后端解析并转化为参数。
3. **生成阶段**：生成 VoiceMeta 元数据 → 通过 WebSocket 调用 CosyVoice → 流式累积 MP3。
4. **存储阶段**：音频存入数据库或作为帖子内容的一部分；接口 `/api/audio/{id}`、`/api/post/audio/{post_id}` 暴露流式播放。
5. **分发阶段**：作品流展示作者、时间、描述、点赞/评论计数和音频链接；支持搜索与详情；支持分享链接预填参数（`?meta_id=xxx`）。

## 数据与接口总览
- **数据模型（摘要）**
	- `Parametic`：`pitch`/`speed`/`volume` 三参数，精确控制音频输出。
	- `VoiceMeta`：声线配置，`parametric: Option<Parametic>` 与 `instruction: Option<String>` 二选一或组合使用。
	- `VoiceModel`：声线模型，含 `VoiceModelInfo`（名称/描述）、`VoiceModelCategory`（Official/User）、`VoiceModelAbility`（voice_clone/voice_design/ssml/latex/parametric_control/instruction_control）。
	- `VoiceLibrary`：生成的音频记录，关联 `meta_id` 与原始文本。
	- `VoiceMetaPost`：分享声线配置的帖子，关联 `meta_id`，含状态与互动计数。
	- `VoicePost`：分享实际音频的帖子，关联 `library_id`，含状态与互动计数。
	- `User` / `UserMeta`：用户资料，`UserRole` = Admin/User/FakeUser/BannedUser/DeletedUser/Bot。
	- `UserAuth`：认证枚举，支持 `PasswordAuth`（`AuthID` = Email/Phone）、`OAuthProvider`（GitHub/WeChat）、Passkey。
- **接口分组（Server Functions）**
	- 认证：`register`、`login`、`logout`、`get_authinfo`、`update_authinfo`。
	- 用户：`get_user_profile`、`update_user_profile`、`update_user_avatar`。
	- 声线：`list_voice_models`、`search_voice_model`、`generate_voice_model`、`update_voice_model`；`generate_meta`、`get_meta`、`generate_voice`。
	- AI 改写：`ai_rewrite_text`（调用 `OPENAI_API_BASE` + DeepSeek-V3.1）。
	- 声线配置帖：`create/get/update/delete/search/list_voice_meta_post`、`action_voice_meta_post`、`get_voice_meta_post_comments`。
	- 音频帖：`create/get/update/delete/search/list_voice_post`、`action_voice_post`、`get_voice_post_comments`。
	- 媒体流：`GET /api/audio/{id}`、`GET /api/avatar/{user_id}`（失败重定向 DiceBear）、`GET /api/post/audio/{post_id}`、`GET /api/voice_avatar/{voice_id}`（失败重定向 DiceBear）。

## 性能与容量预估
- **推理延迟**：CosyVoice v3 flash 典型 1–3 秒（视网络与文本长度而定）；AI 改写通常 1–2 秒。
- **并发**：PostgreSQL 连接池默认上限 5（`PgPoolOptions::max_connections`），并发上升时建议扩容并增加缓存层。
- **带宽**：MP3 输出，码率较小；前端直接流式播放，减少下载等待。
- **缓存策略（可选）**：可对重复文本+参数做缓存或预生成，当前版本未开启以保持简单。

## 部署与环境
- **依赖**：Rust nightly + `wasm32-unknown-unknown` target；`cargo-leptos`；PostgreSQL 实例；端到端测试需 Node 与 Playwright。
- **关键环境变量**：

 | 变量              | 说明                         | 默认值                                              |
 | ----------------- | ---------------------------- | --------------------------------------------------- |
 | `ALIYUN_API_KEY`  | 阿里云 CosyVoice TTS（必须） | —                                                   |
 | `PG_DATABASE_URL` | PostgreSQL 连接串            | `postgres://postgres:postgres@localhost:5432/eardo` |
 | `OPENAI_API_BASE` | AI 改写接口（OpenAI 兼容）   | —                                                   |
 | `OPENAI_API_KEY`  | AI 改写 API Key              | —                                                   |

- **本地快速启动**：
	1. 启动 PostgreSQL 并执行 `sql/` 目录下的 Schema
	2. 创建 `.env` 文件配置上述环境变量（dotenv 自动加载）
	3. `rustup target add wasm32-unknown-unknown && cargo install cargo-leptos`
	4. 开发模式（热重载）：`cargo leptos watch`
	5. 生产构建：`cargo leptos build --release && cargo run -p server`
	6. 端到端测试：`cd end2end && npm install && npx playwright test`
- **扩展建议**：
	- 对象存储：头像/音频迁移至 OSS/S3，减轻 DB 压力；
	- 缓存：加入 Redis 做 session/生成结果缓存；
	- 限流：在 Axum 层增加请求限速中间件。

## 安全与合规
- **密码安全**：前端用 SHA256 对密码哈希后传输，后端再用 bcrypt 二次哈希存储，双层保护。
- **会话**：登录成功后写入 HttpOnly + SameSite=Lax Cookie；存储在数据库的 `user_sessions` 表。
- **权限与状态**：`UserRole` 含 Admin/User/FakeUser/BannedUser/DeletedUser/Bot；帖子 `PostStatus` 含 Normal/Deleted/Banned/Recommended，支持软删除与推荐标记。
- **头像回退**：用户/声线头像读取失败时重定向 DiceBear API 生成默认头像，不暴露内部错误。
- **隐私与版权**：当前只采集账号基础信息与头像；建议上线时增加音频水印、用户声明与内容合规检测。
- **错误处理**：缺少环境变量、数据库连接失败、未登录等均返回清晰 `ServerFnError`，前端可提示用户。

## 测试与验收
- **单元/集成**：Rust 层可用 `cargo test`（未在此仓库预置测试样例）。
- **端到端**：`end2end` 目录提供 Playwright 脚手架，可覆盖登录、生成、播放、发帖主链路。
- **手动验收清单**：
	- 登录/注册：输入合法凭据后成功创建会话；登出清除 Cookie。
	- 生成：填写文本，调节参数后成功生成并播放音频。
	- 帖子：创建帖子可上传音频（或使用生成结果），列表与详情可见，音频可播。
	- 头像：上传 base64 头像后 `/api/avatar/{user_id}` 正常返回。

## 路线图
- **近期**：
	- 补足前端错误提示与生成进度；
	- 加入缓存与并发限流；
	- 上线声线库与基础 DSP 滤镜；
	- 完善移动端样式与可访问性；
	- **AI 改写与指令模式全量上线**（已集成 OpenAI/DeepSeek）。
- **中期**：
	- 接入 RVC 变声与更多官方声线预设；
	- 作品分享卡片、榜单、评论/点赞全量上线；
	- 对象存储/缓存层替换，支持更大流量。
- **远期**：
	- 商业化（订阅/按量计费/声线市场分成）；
	- 国际化与多语言支持；
	- 多模型编排、移动端/边缘推理。

## 常见问题 FAQ
- **需要 GPU 吗？** 不需要，TTS 走云端 CosyVoice；AI 改写调用云端 DeepSeek（OpenAI 兼容接口）。
- **为什么生成失败？** 常见原因：未配置 `ALIYUN_API_KEY`、`PG_DATABASE_URL` 连接失败、网络不可达、输入文本过长或空白。
- **为什么 AI 改写失败？** 检查 `OPENAI_API_BASE` 和 `OPENAI_API_KEY` 是否正确配置。
- **参数模式与指令模式有什么区别？** 参数模式精确调节 pitch/speed/volume 三个滑条；指令模式输入自然语言（如"温柔点"），后端直接传给 CosyVoice 指令接口。
- **音频在哪存？** 存入 PostgreSQL `voice_library` 表（BYTEA），可按需迁移到 OSS/S3。
- **VoiceMetaPost 和 VoicePost 有什么区别？** `VoiceMetaPost` 分享声线配置（可复用），`VoicePost` 分享已生成的具体音频。
- **头像显示异常怎么办？** 头像接口在 DB 中无数据时自动重定向 DiceBear API 生成默认头像。
- **并发会不会顶不住？** 小规模 OK，若上量需调大 `max_connections`、增加缓存与限流。
- **如何分享声线配置？** 通过 `?meta_id=xxx` 链接分享，他人打开 /setup 时会自动加载对应配置。

## 源码索引
- 前端壳与路由：[app/src/lib.rs](app/src/lib.rs)、[app/src/pages.rs](app/src/pages.rs)
- 首页（双模式调声 + `ai_rewrite_text` server fn）：[app/src/pages/homepage.rs](app/src/pages/homepage.rs)
- 渐进式配置页（含 AI 改写）：[app/src/pages/voicesetup.rs](app/src/pages/voicesetup.rs)
- 认证 / 资料 / 设置页面：[app/src/pages/auth.rs](app/src/pages/auth.rs)
- 数据模型 + Server Functions 定义：[app/src/api.rs](app/src/api.rs)
- 用户认证 & 资料实现（PostgreSQL/bcrypt）：[app/src/api/userimpl.rs](app/src/api/userimpl.rs)
- 语音服务实现（PostgreSQL）：[app/src/api/voiceimpl.rs](app/src/api/voiceimpl.rs)
- CosyVoice WebSocket 流式调用：[app/src/api/voice_backend.rs](app/src/api/voice_backend.rs)
- 帖子与作品流实现（PostgreSQL）：[app/src/api/postimpl.rs](app/src/api/postimpl.rs)
- 服务端入口、AppState 与 Provider 注入：[server/src/main.rs](server/src/main.rs)
- 数据库 Schema：[sql/](sql/)
