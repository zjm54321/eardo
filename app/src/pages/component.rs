use crate::api::voice::Parametic;
use leptos::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Server function: AI 文本改写（InstructionParams 依赖）
// ═══════════════════════════════════════════════════════════════

#[server]
pub async fn ai_rewrite_text(input: String) -> Result<String, ServerFnError> {
    let api_url = std::env::var("OPENROUTER_API_BASE")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string());
    let api_token = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
        ServerFnError::new("服务器未配置 OpenRouter API Key，请联系管理员。".to_string())
    })?;

    let model = std::env::var("OPENROUTER_MODEL")
        .unwrap_or_else(|_| "openai/gpt-oss-120b:free".to_string());

    let payload = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": input }]
    });

    let site_url =
        std::env::var("OPENROUTER_SITE_URL").unwrap_or_else(|_| "https://eardoai.com".to_string());
    let app_name = std::env::var("OPENROUTER_APP_NAME").unwrap_or_else(|_| "EarDo".to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(api_url)
        .bearer_auth(api_token)
        .header("HTTP-Referer", site_url)
        .header("X-OpenRouter-Title", app_name.clone())
        .header("X-Title", app_name)
        .json(&payload)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("AI 请求失败: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ServerFnError::new(format!(
            "AI 返回错误: {} - {}",
            status, body
        )));
    }

    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("AI 响应解析失败: {}", e)))?;

    let content = value
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    if content.trim().is_empty() {
        return Err(ServerFnError::new("AI 响应为空，请稍后重试。".to_string()));
    }

    Ok(content)
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
