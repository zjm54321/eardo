# 耳朵网站功能演示录屏

仅访问预发布环境 `https://dev.eardoai.com`。脚本会拒绝访问生产域名，所有注册、发布、点赞、评论、头像修改都发生在预发布数据库。不会修改网站代码，也不会伪造生成或分享结果。

## 环境与运行

适用于 NixOS / WSL2；使用 PATH 中的 `google-chrome`，通过 Playwright Core 连接 Chrome CDP，不需要下载 Playwright 浏览器。

```bash
mkdir -p /tmp/opencode/demo-video
timeout 120 nix shell nixpkgs#nodejs_22 --command npm install \
  --prefix /tmp/opencode/demo-video playwright-core@1.64.1 --no-audit --no-fund

timeout 2100 nix shell nixpkgs#ffmpeg nixpkgs#nodejs_22 --command \
  node ~/projects/eardo/scripts/demo-video/record.mjs
```

启动后最多等待 20 分钟，每 30 秒检查 `/generate` 是否出现 `/pkg/eardo.<hash>.js`。随后创建全新 Chrome 用户目录，按 `sections/` 的文件名顺序操作。每次运行会创建一个随机 `139000xxxxx` 手机号账号，密码 `Demo12345!`。请勿指定已有手机号：流程包含注册。

默认输出目录 `/tmp/opencode/demo-video/`，可以用 `DEMO_OUT` 改为其他目录，但需要先将同版本 `playwright-core` 安装到该目录。不要并行运行：CDP 与控制端口固定为 9451 / 9452。

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

- `eardo-demo.mp4`：1440×900、15 fps、H.264、yuv420p、faststart；无系统声音轨。
- `clips/`：各分段 MP4；`manifest.json`：分段起始秒数和时长。
- `frames/live-*.png`：运行时关键状态截图。
- `account.json`：本次测试账号；`release.txt`：部署资源版本。
- `browser-errors.json`：HTTP 4xx/5xx、控制台错误与 JS 异常。
- `tts_audio.mp3`：点击下载按钮保存的生成音频。

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

本次交付在分段衔接处去掉了两处各 10 秒的自动化定位等待，保留正常的真实操作和站点响应；完整时间线与问题记录见输出目录 `REPORT.md`。后续重跑已更新相关定位和保存后跳转处理，无须这些剪切。生成服务变慢时，成片时长可能变化。

`--serve` 是分段录制调试模式，只在本机 `127.0.0.1:9452` 接收受信任的 JavaScript 操作指令；正常重跑不需要此参数。
