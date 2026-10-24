use crate::api::voice::Parametic;
use leptos::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Server function: AI 文本改写（InstructionParams 依赖）
// ═══════════════════════════════════════════════════════════════

#[server]
pub async fn ai_rewrite_text(input: String) -> Result<String, ServerFnError> {
    use crate::api::voice_backend::{http_client, read_limited};
    use std::time::Duration;

    async fn complete(
        input: &str,
        url: &str,
        token: &str,
        model: &str,
        openrouter: bool,
    ) -> anyhow::Result<String> {
        let timeout = Duration::from_secs(
            std::env::var("LLM_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(30)
                .clamp(1, 120),
        );
        let mut request = http_client()?.post(url).bearer_auth(token).timeout(timeout);
        if openrouter {
            request = request
                .header("HTTP-Referer", std::env::var("OPENROUTER_SITE_URL")
                    .unwrap_or_else(|_| "https://eardoai.com".to_string()))
                .header("X-OpenRouter-Title", std::env::var("OPENROUTER_APP_NAME")
                    .unwrap_or_else(|_| "EarDo".to_string()));
        }
        let response = request.json(&serde_json::json!({
            "model": model,
            "messages": [{ "role": "user", "content": input }],
            "max_tokens": 2048,
        })).send().await.map_err(|_| anyhow::anyhow!("AI 请求失败或超时"))?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("AI 服务返回 HTTP {}", response.status().as_u16()));
        }
        let body = read_limited(response, 1024 * 1024).await?;
        let value: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|_| anyhow::anyhow!("AI 响应格式错误"))?;
        let content = value["choices"][0]["message"]["content"]
            .as_str().filter(|s| !s.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("AI 响应为空"))?;
        Ok(content.to_string())
    }

    let mut openrouter_error = None;
    if let Ok(token) = std::env::var("OPENROUTER_API_KEY")
        && !token.trim().is_empty()
    {
        let url = std::env::var("OPENROUTER_API_BASE")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string());
        // Currently listed, low-cost model; free providers were rate limited in live probes.
        let model = std::env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());
        match complete(&input, &url, &token, &model, true).await {
            Ok(content) => return Ok(content),
            Err(error) => openrouter_error = Some(error),
        }
    }

    // Same-region DashScope chat is a verified fallback for unavailable/underfunded OpenRouter.
    if let Ok(token) = std::env::var("ALIYUN_API_KEY")
        .or_else(|_| std::env::var("DASHSCOPE_API_KEY"))
        && !token.trim().is_empty()
    {
        let model = std::env::var("DASHSCOPE_CHAT_MODEL")
            .unwrap_or_else(|_| "qwen-flash".to_string());
        return complete(&input, "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
            &token, &model, false).await
            .map_err(|error| ServerFnError::new(format!("DashScope {error}")));
    }
    Err(ServerFnError::new(match openrouter_error {
        Some(error) => format!("OpenRouter {error}；未配置 DashScope 备用服务。"),
        None => "服务器未配置 AI API Key，请联系管理员。".to_string(),
    }))
}

// ═══════════════════════════════════════════════════════════════
// 自定义音频播放器
// ═══════════════════════════════════════════════════════════════

#[component]
pub fn CustomAudioPlayer(
    /// 音频 URL
    audio_url: String,
    /// 顶部标签文字（默认"作品音频"）
    #[prop(optional)]
    label: Option<String>,
    /// 是否自动播放
    #[prop(optional)]
    autoplay: Option<bool>,
) -> impl IntoView {
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let (is_playing, set_is_playing) = signal(false);
    #[cfg_attr(feature = "ssr", allow(unused_variables))]
    let (current_time, set_current_time) = signal(0.0_f64);
    #[cfg_attr(feature = "ssr", allow(unused_variables))]
    let (duration, set_duration) = signal(0.0_f64);
    let (is_seeking, set_is_seeking) = signal(false);
    let should_autoplay = autoplay.unwrap_or(false);

    let toggle_play = move |_| {
        if let Some(audio) = audio_ref.get() {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                let audio_el: web_sys::HtmlAudioElement = audio.unchecked_into();
                if is_playing.get() {
                    let _ = audio_el.pause();
                    set_is_playing.set(false);
                } else {
                    let _ = audio_el.play();
                    set_is_playing.set(true);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = audio;
        }
    };

    let on_time_update = move |_| {
        if is_seeking.get() {
            return;
        }
        if let Some(audio) = audio_ref.get() {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                let audio_el: web_sys::HtmlAudioElement = audio.unchecked_into();
                set_current_time.set(audio_el.current_time());
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = audio;
        }
    };

    let on_loaded_metadata = move |_| {
        if let Some(audio) = audio_ref.get() {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                let audio_el: web_sys::HtmlAudioElement = audio.unchecked_into();
                set_duration.set(audio_el.duration());
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = audio;
        }
    };

    let on_seek = move |ev: web_sys::Event| {
        if let Some(input) = ev.target() {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                if let Ok(input_el) = input.dyn_into::<web_sys::HtmlInputElement>() {
                    if let Ok(value) = input_el.value().parse::<f64>() {
                        set_current_time.set(value);
                        if let Some(audio) = audio_ref.get() {
                            let audio_el: web_sys::HtmlAudioElement = audio.unchecked_into();
                            audio_el.set_current_time(value);
                        }
                    }
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = input;
        }
        set_is_seeking.set(false);
    };

    let format_time = move |seconds: f64| -> String {
        if seconds.is_nan() || seconds.is_infinite() {
            return "0:00".to_string();
        }
        let mins = (seconds / 60.0).floor() as i32;
        let secs = (seconds % 60.0).floor() as i32;
        format!("{}:{:02}", mins, secs)
    };

    let label_text = label.unwrap_or_else(|| "作品音频".to_string());

    view! {
        <div class="bg-gradient-to-r from-amber-50 to-yellow-50 border border-amber-100 rounded-lg p-4">
            <p class="text-xs text-gray-600 mb-3">{label_text}</p>

            <div class="flex items-center gap-3">
                <button
                    class="w-10 h-10 rounded-full bg-primary hover:bg-primary-focus text-white flex items-center justify-center transition-all duration-200 flex-shrink-0 shadow-sm hover:shadow"
                    on:click=toggle_play
                >
                    {move || {
                        if is_playing.get() {
                            view! { <i class="fa-solid fa-pause"></i> }
                        } else {
                            view! { <i class="fa-solid fa-play ml-0.5"></i> }
                        }
                    }}
                </button>

                <div class="flex-1 flex flex-col gap-1">
                    <input
                        type="range"
                        min="0"
                        max=move || duration.get()
                        step="0.1"
                        class="w-full h-1.5 bg-amber-200 rounded-lg appearance-none cursor-pointer accent-primary"
                        prop:value=move || current_time.get()
                        on:input=move |_| set_is_seeking.set(true)
                        on:change=on_seek
                    />
                    <div class="flex justify-between text-xs text-gray-500">
                        <span>{move || format_time(current_time.get())}</span>
                        <span>{move || format_time(duration.get())}</span>
                    </div>
                </div>
            </div>

            <audio
                node_ref=audio_ref
                src=audio_url
                autoplay=should_autoplay
                on:timeupdate=on_time_update
                on:loadedmetadata=on_loaded_metadata
                on:play=move |_| set_is_playing.set(true)
                on:pause=move |_| set_is_playing.set(false)
                on:ended=move |_| set_is_playing.set(false)
                class="hidden"
            />
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════
// 传统参数（音高 / 语速 / 音量）
// ═══════════════════════════════════════════════════════════════

#[component]
pub fn TraditionalParams(param_signal: RwSignal<Parametic>) -> impl IntoView {
    view! {
        <div class="space-y-8">
            // 音高
            <div>
                <div class="flex justify-between mb-3">
                    <label class="font-semibold text-gray-700 flex items-center">
                        <i class="fa-solid fa-music text-primary mr-2"></i>
                        "音高 (Pitch)"
                    </label>
                    <input
                        type="text"
                        inputmode="decimal"
                        class="w-20 text-sm bg-primary/10 text-primary px-3 py-1 rounded-full font-bold text-center outline-none focus:ring-2 focus:ring-primary/30 hover:bg-primary/20 transition-colors cursor-pointer"
                        prop:value=move || format!("{:.2}", param_signal.get().pitch)
                        on:blur=move |ev| {
                            let val = event_target_value(&ev).parse::<f32>().unwrap_or(1.0).clamp(0.5, 2.0);
                            param_signal.update(|p| p.pitch = val);
                        }
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Enter" {
                                let _ = ev.target()
                                    .and_then(|t| {
                                        use wasm_bindgen::JsCast;
                                        t.dyn_into::<web_sys::HtmlElement>().ok()
                                    })
                                    .map(|el| el.blur());
                            }
                        }
                    />
                </div>
                <input
                    type="range"
                    min="0.5"
                    max="2.0"
                    step="0.01"
                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-primary"
                    prop:value=move || param_signal.get().pitch
                    on:input=move |ev| {
                        let val = event_target_value(&ev).parse::<f32>().unwrap_or(1.0);
                        param_signal.update(|p| p.pitch = val);
                    }
                />
                <div class="relative h-4 text-xs text-gray-400 mt-1">
                    <span class="absolute left-0">"0.5"</span>
                    <span class="absolute" style="left:33.33%;transform:translateX(-50%)">"1.0"</span>
                    <span class="absolute right-0">"2.0"</span>
                </div>
            </div>

            // 语速
            <div>
                <div class="flex justify-between mb-3">
                    <label class="font-semibold text-gray-700 flex items-center">
                        <i class="fa-solid fa-gauge-high text-primary mr-2"></i>
                        "语速 (Speed)"
                    </label>
                    <input
                        type="text"
                        inputmode="decimal"
                        class="w-20 text-sm bg-primary/10 text-primary px-3 py-1 rounded-full font-bold text-center outline-none focus:ring-2 focus:ring-primary/30 hover:bg-primary/20 transition-colors cursor-pointer"
                        prop:value=move || format!("{:.2}x", param_signal.get().speed)
                        on:blur=move |ev| {
                            let raw = event_target_value(&ev);
                            let val = raw.trim_end_matches('x').parse::<f32>().unwrap_or(1.0).clamp(0.5, 2.0);
                            param_signal.update(|p| p.speed = val);
                        }
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Enter" {
                                let _ = ev.target()
                                    .and_then(|t| {
                                        use wasm_bindgen::JsCast;
                                        t.dyn_into::<web_sys::HtmlElement>().ok()
                                    })
                                    .map(|el| el.blur());
                            }
                        }
                    />
                </div>
                <input
                    type="range"
                    min="0.5"
                    max="2.0"
                    step="0.01"
                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-primary"
                    prop:value=move || param_signal.get().speed
                    on:input=move |ev| {
                        let val = event_target_value(&ev).parse::<f32>().unwrap_or(1.0);
                        param_signal.update(|p| p.speed = val);
                    }
                />
                <div class="relative h-4 text-xs text-gray-400 mt-1">
                    <span class="absolute left-0">"0.5x"</span>
                    <span class="absolute" style="left:33.33%;transform:translateX(-50%)">"1.0x"</span>
                    <span class="absolute right-0">"2.0x"</span>
                </div>
            </div>

            // 音量
            <div>
                <div class="flex justify-between mb-3">
                    <label class="font-semibold text-gray-700 flex items-center">
                        <i class="fa-solid fa-volume-high text-primary mr-2"></i>
                        "音量 (Volume)"
                    </label>
                    <input
                        type="text"
                        inputmode="decimal"
                        class="w-20 text-sm bg-primary/10 text-primary px-3 py-1 rounded-full font-bold text-center outline-none focus:ring-2 focus:ring-primary/30 hover:bg-primary/20 transition-colors cursor-pointer"
                        prop:value=move || format!("{:.0}%", param_signal.get().volume * 100.0)
                        on:blur=move |ev| {
                            let raw = event_target_value(&ev);
                            let num = raw.trim_end_matches('%').parse::<f32>().unwrap_or(100.0);
                            let vol = if num > 2.0 { num / 100.0 } else { num }.clamp(0.0, 2.0);
                            param_signal.update(|p| p.volume = vol);
                        }
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Enter" {
                                let _ = ev.target()
                                    .and_then(|t| {
                                        use wasm_bindgen::JsCast;
                                        t.dyn_into::<web_sys::HtmlElement>().ok()
                                    })
                                    .map(|el| el.blur());
                            }
                        }
                    />
                </div>
                <input
                    type="range"
                    min="0.0"
                    max="2.0"
                    step="0.01"
                    class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-primary"
                    prop:value=move || param_signal.get().volume
                    on:input=move |ev| {
                        let val = event_target_value(&ev).parse::<f32>().unwrap_or(1.0);
                        param_signal.update(|p| p.volume = val);
                    }
                />
                <div class="relative h-4 text-xs text-gray-400 mt-1">
                    <span class="absolute left-0">"0%"</span>
                    <span class="absolute left-1/2 -translate-x-1/2">"100%"</span>
                    <span class="absolute right-0">"200%"</span>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════
// AI 语言控制（指令模式）
// ═══════════════════════════════════════════════════════════════

#[component]
pub fn InstructionParams(instruction_text: RwSignal<String>) -> impl IntoView {
    let is_loading = RwSignal::new(false);

    let ai_action = Action::new(move |input: &String| {
        let input = input.clone();
        async move {
            let prompt = format!(
                "你是语音指令优化助手。请根据以下原则，将用户的简单描述改写为高质量的语音合成指令：\n\
                1) 具体而非模糊：使用描绘具体声音特质的词语（如低沉、清脆、语速偏快）。\n\
                2) 多维而非单一：结合音调、语速、情感、特点等多个维度。\n\
                3) 客观而非主观：专注于声音本身的物理和感知特征。\n\
                4) 简洁而非冗余：确保每个词都有意义。\n\
                5) 只输出改写后的指令文本，不要附加任何解释、问候或多余内容。\n\n\
                用户原始描述：\n{}",
                input
            );
            ai_rewrite_text(prompt).await
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(result)) = ai_action.value().get() {
            instruction_text.set(result);
            is_loading.set(false);
        } else if let Some(Err(err)) = ai_action.value().get() {
            leptos::logging::debug_error!("AI 指令优化失败: {}", err);
            is_loading.set(false);
        }
    });

    view! {
        <div class="space-y-4">
            <div class="flex items-center mb-2">
                <i class="fa-solid fa-comment-dots text-primary mr-2"></i>
                <span class="font-semibold text-gray-700">"AI 语言控制"</span>
            </div>
            <p class="text-sm text-gray-500">
                "用自然语言描述你想要的声音效果，AI 会理解并生成对应的语音"
            </p>
            <div class="relative">
                <textarea
                    class="w-full px-4 py-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all resize-none min-h-[120px]"
                    placeholder="例如：用温柔的语气，语速稍微慢一点，像在讲故事一样..."
                    prop:value=move || instruction_text.get()
                    on:input=move |ev| instruction_text.set(event_target_value(&ev))
                    disabled=move || is_loading.get()
                ></textarea>
                <button
                    class="absolute bottom-3 right-3 w-9 h-9 rounded-full bg-primary/10 hover:bg-primary/20 text-primary flex items-center justify-center transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click=move |_| {
                        if !instruction_text.get().trim().is_empty() {
                            is_loading.set(true);
                            ai_action.dispatch(instruction_text.get());
                        }
                    }
                    disabled=move || is_loading.get() || instruction_text.get().trim().is_empty()
                    title="AI 优化指令"
                >
                    <Show
                        when=move || is_loading.get()
                        fallback=move || view! { <i class="fa-solid fa-wand-magic-sparkles"></i> }
                    >
                        <i class="fa-solid fa-spinner fa-spin"></i>
                    </Show>
                </button>
            </div>
            <p class="text-xs text-gray-500">
                <i class="fa-solid fa-circle-info mr-1"></i>
                "使用自然语言描述你想要的声音效果，点击魔法棒可 AI 优化指令。"
            </p>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════
// 圆形频谱可视化（负责 AudioContext / requestAnimationFrame 生命周期）
// ═══════════════════════════════════════════════════════════════

#[cfg(target_arch = "wasm32")]
struct VisualizerState {
    ctx: web_sys::AudioContext,
    alive: std::rc::Rc<std::cell::Cell<bool>>,
    raf_id: std::rc::Rc<std::cell::Cell<i32>>,
    frame: std::rc::Rc<std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>>>,
}

#[cfg(target_arch = "wasm32")]
impl VisualizerState {
    fn stop(self) {
        self.alive.set(false);
        if let Some(window) = web_sys::window() {
            let _ = window.cancel_animation_frame(self.raf_id.get());
        }
        // 断开闭包自引用，释放 analyser / canvas 等资源
        self.frame.borrow_mut().take();
        let _ = self.ctx.close();
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct VisualizerState;

#[cfg(not(target_arch = "wasm32"))]
impl VisualizerState {
    fn stop(self) {}
}

// Browser state holds !Send web-sys handles → LocalStorage. On the server the
// state is a unit struct; LocalStorage there panics when the reactive owner is
// dropped on another tokio worker thread, so use thread-safe storage instead.
#[cfg(target_arch = "wasm32")]
type VisualizerStorage = LocalStorage;
#[cfg(not(target_arch = "wasm32"))]
type VisualizerStorage = SyncStorage;

/// 绑定到组件生命周期的圆形频谱可视化器。
///
/// - `start` 会先释放旧的 AudioContext / 动画帧，再为新的 audio 元素建立分析器；
/// - 组件卸载时自动取消 requestAnimationFrame 并关闭 AudioContext。
#[derive(Clone, Copy)]
pub struct SpectrumVisualizer {
    state: StoredValue<Option<VisualizerState>, VisualizerStorage>,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    ring_color: &'static str,
}

impl SpectrumVisualizer {
    /// `ring_color`：圆环底色（各页面保持原有配色）
    pub fn new(ring_color: &'static str) -> Self {
        let this = Self {
            state: StoredValue::new_with_storage(None),
            ring_color,
        };
        // Only the browser owns audio resources. On the server the owner may be
        // dropped on another tokio thread, and touching LocalStorage there panics.
        #[cfg(target_arch = "wasm32")]
        on_cleanup(move || this.stop());
        this
    }

    /// 停止动画并关闭 AudioContext
    pub fn stop(&self) {
        let old = self.state.try_update_value(|state| state.take()).flatten();
        if let Some(old) = old {
            old.stop();
        }
    }

    /// 浏览器自动播放策略可能让 AudioContext 处于 suspended，播放时恢复。
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn resume(&self) {
        #[cfg(target_arch = "wasm32")]
        let _ = self.state.try_with_value(|state| {
            if let Some(state) = state {
                let _ = state.ctx.resume();
            }
        });
    }

    pub fn start(
        &self,
        #[allow(unused_variables)] audio_ref: NodeRef<leptos::html::Audio>,
        #[allow(unused_variables)] canvas_ref: NodeRef<leptos::html::Canvas>,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            use std::{cell::{Cell, RefCell}, rc::Rc};
            use wasm_bindgen::{JsCast, closure::Closure};
            use web_sys::{AudioContext, CanvasRenderingContext2d};

            // 组件可能已在定时器触发前卸载
            if self.state.try_with_value(|_| ()).is_none() {
                return;
            }
            let (Some(audio), Some(canvas)) = (
                audio_ref.try_get_untracked().flatten(),
                canvas_ref.try_get_untracked().flatten(),
            ) else {
                return;
            };
            let audio: web_sys::HtmlAudioElement = audio.unchecked_into();
            let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();
            let Some(window) = web_sys::window() else { return };

            let width = canvas.parent_element().map(|p| p.client_width().max(0) as u32).unwrap_or(600);
            let height = 300u32; // 固定高度
            canvas.set_width(width);
            canvas.set_height(height);

            audio.set_cross_origin(Some("anonymous"));

            let Ok(ctx) = AudioContext::new() else { return };
            let Ok(analyser) = ctx.create_analyser() else {
                let _ = ctx.close();
                return;
            };
            analyser.set_fft_size(256); // 256 -> 128 个数据点

            // 同一个 audio 元素只能连接一次；失败时保留旧的上下文，避免把正在播放的声音静音
            let Ok(source) = ctx.create_media_element_source(&audio) else {
                let _ = ctx.close();
                return;
            };
            if source.connect_with_audio_node(&analyser).is_err()
                || analyser.connect_with_audio_node(&ctx.destination()).is_err()
            {
                let _ = ctx.close();
                return;
            }
            let Some(ctx_2d) = canvas
                .get_context("2d")
                .ok()
                .flatten()
                .map(|c| c.unchecked_into::<CanvasRenderingContext2d>())
            else {
                let _ = ctx.close();
                return;
            };

            // 新上下文就绪后再释放旧的
            self.stop();

            let buffer_length = analyser.frequency_bin_count();
            let mut data_array = vec![0u8; buffer_length as usize];
            // 语音能量几乎都在 ~8 kHz 以下，而频谱覆盖到采样率的一半（48 kHz 时为 24 kHz），
            // 直接把全部频点铺满一圈会导致后半圈永远没有柱子。
            // 只取有效频段，并左右镜像铺满整圈。
            let nyquist = (ctx.sample_rate() as f64 / 2.0).max(1.0);
            let useful_bins = ((8000.0 / nyquist) * buffer_length as f64).ceil() as u32;
            let useful_bins = useful_bins.clamp(8, buffer_length.max(8)).min(buffer_length);
            let bar_count = useful_bins * 2;
            let canvas_width = width as f64;
            let canvas_height = height as f64;
            let center_x = canvas_width / 2.0;
            let center_y = canvas_height / 2.0;
            let radius = 60.0;
            let ring_color = self.ring_color;

            let alive = Rc::new(Cell::new(true));
            let raf_id = Rc::new(Cell::new(0));
            let frame: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));

            let alive_in = alive.clone();
            let raf_in = raf_id.clone();
            let frame_in = frame.clone();
            let window_in = window.clone();
            *frame.borrow_mut() = Some(Closure::new(move || {
                if !alive_in.get() {
                    return;
                }
                analyser.get_byte_frequency_data(&mut data_array);
                ctx_2d.clear_rect(0.0, 0.0, canvas_width, canvas_height);

                ctx_2d.begin_path();
                let _ = ctx_2d.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
                ctx_2d.set_stroke_style_str(ring_color);
                ctx_2d.set_line_width(2.0);
                ctx_2d.stroke();

                ctx_2d.set_fill_style_str("#FBBF24");
                for i in 0..bar_count {
                    let bin = if i < useful_bins { i } else { bar_count - 1 - i };
                    let value = data_array[bin as usize] as f64;
                    let bar_height = (value / 255.0) * 80.0;
                    let rad = (i as f64 / bar_count as f64) * 2.0 * std::f64::consts::PI;
                    ctx_2d.save();
                    let _ = ctx_2d.translate(center_x, center_y);
                    let _ = ctx_2d.rotate(rad);
                    if bar_height > 0.0 {
                        ctx_2d.fill_rect(-1.5, radius, 3.0, bar_height);
                    }
                    ctx_2d.restore();
                }

                if let Some(cb) = frame_in.borrow().as_ref() {
                    if let Ok(id) = window_in.request_animation_frame(cb.as_ref().unchecked_ref()) {
                        raf_in.set(id);
                    }
                }
            }));

            if let Some(cb) = frame.borrow().as_ref() {
                if let Ok(id) = window.request_animation_frame(cb.as_ref().unchecked_ref()) {
                    raf_id.set(id);
                }
            }

            let mut pending = Some(VisualizerState { ctx, alive, raf_id, frame });
            let _ = self.state.try_update_value(|slot| *slot = pending.take());
            // 存储失败（已卸载）时立即释放
            if let Some(orphan) = pending {
                orphan.stop();
            }
        }
    }
}
