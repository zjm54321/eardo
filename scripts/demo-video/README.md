# 耳朵网站功能演示录屏（网站原声 + 自动压低背景音乐）

仅访问预发布环境 `https://dev.eardoai.com`。脚本会拒绝访问生产域名，所有注册、发布、点赞、评论、头像修改都发生在预发布数据库。不会修改网站代码，也不会伪造生成或分享结果。

## 环境与运行

适用于 NixOS / WSL2；使用 PATH 中的 `google-chrome`，通过 Playwright Core 连接 Chrome CDP，不需要下载 Playwright 浏览器。

```bash
mkdir -p /tmp/opencode/demo-video
timeout 120 nix shell nixpkgs#nodejs_22 --command npm install \
  --prefix /tmp/opencode/demo-video playwright-core@1.64.1 --no-audit --no-fund

timeout 2100 nix shell nixpkgs#ffmpeg nixpkgs#nodejs_22 --command \
  node ~/projects/eardo/scripts/demo-video/record.mjs

timeout 600 nix shell nixpkgs#ffmpeg nixpkgs#nodejs_22 --command \
  node ~/projects/eardo/scripts/demo-video/validate-audio.mjs
```

启动后最多等待 20 分钟，每 30 秒检查 `/generate` 是否出现 `/pkg/eardo.<hash>.js`。随后创建全新 Chrome 用户目录，按 `sections/` 的文件名顺序操作。每次运行会创建一个随机 `139000xxxxx` 手机号账号，密码 `Demo12345!`。请勿指定已有手机号：流程包含注册。

默认输出目录 `/tmp/opencode/demo-video/`，可以用 `DEMO_OUT` 改为其他目录，但需要先将同版本 `playwright-core` 安装到该目录。不要并行运行：CDP 与控制端口固定为 9451 / 9452。部署检查和音频下载使用 curl，兼容 WSL 的 HTTPS_PROXY；音频下载异步执行，不阻塞视频帧时钟。

## 内容

1. 欢迎页 → 工作台 → 场景创作 → 声音风格。
2. 手机号注册、登录、头像入口。
3. 文本输入、全屏编辑、表达助手口播改写、原文/助手文本切换、一键填入、多声线选择。
4. 音高/语速/音量滑块、传统参数/语言控制切换、AI 优化指令、生成、环形波形、播放、下载。
5. 配置分享、复制链接、发布声音作品。
6. 渐进式配置四步及再次生成。
7. 广场轮播、搜索/清除、播放、点赞/取消、发表评论。
8. 滤镜搜索、点赞、评论、使用滤镜并生成。
9. 我的作品、上传头像、修改昵称/简介并保存、登录设置和修改密码弹窗（取消，不改密码）。
10. 帮助中心展开问题、退出登录。

私有声音库和声音分析在网站上标记为“功能开发中”，只展示其当前状态。账号绑定/解绑和密码修改不实际提交，避免更改演示账号的登录凭据。页面不存在分页控件，广场展示轮播与搜索。

## 输出与验证

- `eardo-demo.mp4`：1440×900、15 fps、H.264、yuv420p、faststart；AAC 192 kbps、48 kHz、双声道。
- `eardo-demo-silent.mp4`：此前交付的旧静音版，只在 /tmp 保留，不能用新画面替换它。
- `eardo-demo-picture.mp4`：这次重新录制的纯画面，用于重新混音。
- `clips/`：各分段 MP4；`manifest.json`：分段起始秒数和时长。
- `frames/live-*.png`：运行时关键状态截图。
- `account.json`：本次测试账号；`release.txt`：部署资源版本。
- `browser-errors.json`：HTTP 4xx/5xx、控制台错误与 JS 异常。
- `tts_audio.mp3`：点击下载按钮保存的生成音频。
- `media-events.json`：所有 HTMLMediaElement 播放、实际 playing、暂停、结束、缓冲、跳转、速率/音量变化及 AudioContext 状态/连接日志。
- `audio-assets.json`、`audio/*.audio`：原始来源 URL 与精确下载的音频字节，含自动播放和所有被点击的播放器。
- `frame-clock.json`：每个编码帧的 PTS 与原始 CDP 画面时间戳。
- `playback-bookmarks.json`：手动播放动作的前后时间；`playback-timeline.json`：最终原声混音片段及源文件播放偏移。
- `audio/site.wav`：对齐视频时间的网站原声；`audio/music-bed.wav`：未 duck 的音乐；`audio/music-ducked.wav`：按播放时间自动压低后的音乐；`audio/soundtrack.wav`：最终混音。
- `loudness.json`、`music-source.json`：双遍 loudnorm 测量与音乐来源说明。
- `validation/`：音视频流检查、ebur128、3 段播放前后 5 秒样片、1 段空闲 5 秒样片、逐帧按钮变化与音频起点测量。

用以下命令检查成片，并从实际 MP4 抽帧（`-ss` 可按 manifest 调整）：

```bash
timeout 20 nix shell nixpkgs#ffmpeg --command ffprobe -v error \
  -show_entries format=duration,size:stream=codec_name,width,height,pix_fmt \
  -of json /tmp/opencode/demo-video/eardo-demo.mp4
timeout 20 nix shell nixpkgs#ffmpeg --command ffmpeg -y -ss 155 \
  -i /tmp/opencode/demo-video/eardo-demo.mp4 -frames:v 1 \
  /tmp/opencode/demo-video/frames/check.png
cp /tmp/opencode/demo-video/eardo-demo.mp4 \
  '/mnt/c/Users/Zhang/Documents/大学文件/电工/比赛/耳朵/eardo-demo.mp4'
```

## 音画同步与混音

`record.mjs` 在任何站点脚本运行前安装媒体事件监听，同时跟踪用于环形波形的 AudioContext（不会把同一条音频重复混两次）。网站实际 playing / pause / ended / waiting / seeking 等事件映射到当前已编码帧数 ÷ 15；每段起点累加前段真实时长。音视频用同一条帧时钟，不用“运行脚本经历了多少秒”估计。分段结束会截断仍在播放的媒体，下一段继续时记录新的起点。

`mix.mjs` 下载源文件后保留原始字节，根据日志中的 currentTime / playbackRate / volume 截取对应片段，按 48 kHz 采样位置放入视频时间线（等效于精确到采样的 adelay）。用播放起止的媒体时钟跨度校正视频帧调度的毫秒级差异，避免暂停/续播处重复或漏掉音频采样。支持自动播放、重复播放、暂停/结束和跳转，AudioContext 暂停/关闭期间不输出该媒体的声音。原声双遍 loudnorm 到约 -16 LUFS；源文件本来就有的静音不删除。

背景音乐完全由 `mix.mjs` 自行合成：80 BPM、C–G–Am–F（I–V–vi–IV），柔和正弦和声与轻琶音，没有下载任何音乐、采样或第三方录音。没有音乐版权授权依赖。音乐床双遍 loudnorm 到约 -28 LUFS；网站发声时音乐幅度降到 18%（约 -14.9 dB），100 ms 提前平滑压低，停止后 600 ms 恢复。全片开头 2.5 秒淡入、结尾 4 秒淡出。

只重做混音，无须再次操作网站：

```bash
timeout 600 nix shell nixpkgs#ffmpeg nixpkgs#nodejs_22 --command \
  node ~/projects/eardo/scripts/demo-video/mix.mjs
```

`validate-audio.mjs` 校验原声/音乐的 ebur128 响度、AAC 与 H.264 时长，导出 3 个手动播放点的按钮逐帧裁剪，比较播放三角与暂停竖条；检测原声波形的实际起点，要求音画差不超过 150 ms。另导出 3 段播放/1 段空闲样片，用 volumedetect 对照原声、未压低音乐和已压低音乐，要求播放时音乐实际下降约 14.9 dB、空闲时只有正常背景音乐。截图和 JSON 都会保留，供人工复核。

本次完整时间线、新测试账号、音量和同步结果见输出目录 `REPORT.md`。生成服务变慢时，成片时长可能变化；不要剪切成片后继续使用旧的音频事件时间线，须同步重新映射时间。

`--serve` 是分段录制调试模式，只在本机 `127.0.0.1:9452` 接收受信任的 JavaScript 操作指令；正常重跑不需要此参数。

若遇到网络或水合等待超时，可使用 `record-state.json` 里的 profileDir 接续尚未成功的章节（例如 `DEMO_PROFILE=... node record.mjs --resume=06-square`）。它保留此前已完成的画面、同一测试账号、音频日志和帧时钟，覆盖失败章节，不把调试等待拼入成片。必须保留同一输出目录和 Chrome profile；不要借接续模式换账号。完整新录制仍默认使用全新 profile、新测试账号。脚本会等 WASM 初始化完成后再操作，避免点到尚未水合的 SSR 控件。
