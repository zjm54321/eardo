use crate::api;
use crate::data::{Emotion, VoiceParams};
use leptos::logging::{debug_error, debug_log};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateParams {
    pub text: String,
    pub voice_id: String,
    pub voice_param: VoiceParams,
}

// 用于 create_post_action 的有效负载
#[derive(Clone)]
pub struct CreatePostPayload {
    title: String,
    content: String,
    voice_id: String,
    audio_data: Vec<u8>,
}

#[component]
pub fn HomePage() -> impl IntoView {
    // 状态
    // 1. 获取 URL 查询参数
    let query = use_query_map();

    // 2. 初始化信号 (优先从 URL 参数读取，否则用默认值)
    // 使用 with_untracked 避免初始化时的响应式追踪警告
    let get_f32_param = |key: &str, default: f32| {
        query.with_untracked(|q| {
            q.get(key)
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(default)
        })
    };
    let get_str_param = |key: &str, default: &str| {
        query.with_untracked(|q| {
            q.get(key)
                .map(|arg0: std::string::String| ToString::to_string(&arg0))
                .unwrap_or(default.to_string())
        })
    };

    let text_signal = RwSignal::new(String::new());

    // 初始化声线 ID
    let initial_voice_id = get_str_param("voice_id", "longxiaoxia");
    let voice_signal = RwSignal::new(initial_voice_id.clone());

    // 初始化参数
    let initial_pitch = get_f32_param("pitch", 1.0);
    let initial_speed = get_f32_param("speed", 1.0);
    let emotion_str = get_str_param("emotion", "normal");

    let initial_emotion = match emotion_str.as_str() {
        "生气" => Emotion::Angry,
        "冷静" => Emotion::Calm,
        "激动" => Emotion::Excited,
        "开心" => Emotion::Happy,
        "平静" => Emotion::Peaceful,
        "悲伤" => Emotion::Sad,
        "惊讶" => Emotion::Suprised,
        _ => Emotion::Normal,
    };

    let param_signal = RwSignal::new(VoiceParams {
        pitch: initial_pitch,
        speed: initial_speed,
        emotion: initial_emotion.clone(),
    });

    // 保存生成成功的数据用于分享
    let (_generated_text, set_generated_text) = signal(String::new());
    let (_generated_voice_name, set_generated_voice_name) = signal(String::new());
    let (generated_voice_id, set_generated_voice_id) = signal(String::new());
    let (generated_audio_data, set_generated_audio_data) = signal(Vec::<u8>::new());
    let (show_share_popup, set_show_share_popup) = signal(false);
    let (show_share_modal, set_show_share_modal) = signal(false);
    let (share_title, set_share_title) = signal(String::new());
    let (share_content, set_share_content) = signal(String::new());

    // 声音滤镜分享弹窗状态与字段
    let (show_filter_share, set_show_filter_share) = signal(false);
    let (filter_share_title, set_filter_share_title) = signal(String::new());
    let (filter_share_intro, set_filter_share_intro) = signal(String::new());

    // 创建 Action 处理生成请求
    // Action 自动管理 pending (加载中) 和 value (返回值) 状态
    let generate_action = Action::new(move |_| {
        let text = text_signal.get();
        let voice_id = voice_signal.get();
        let pitch = param_signal.get().pitch;
        let speed = param_signal.get().speed;
        let emotion = param_signal.get().emotion.to_string();

        async move {
            // 创建 VoiceMetaInfo 对象
            let voice_meta = api::VoiceMetaInfo {
                id: voice_id.clone(),
                name: voice_id.clone(),
                metadata: serde_json::json!({
                    "base_model_id": voice_id.clone(),
                    "pitch": pitch,
                    "speed": speed,
                    "emotion": emotion,
                })
                .to_string(),
            };

            debug_log!(
                "生成音频: voice_id={}, text={}, pitch={}, speed={}, emotion={}",
                voice_id,
                text,
                pitch,
                speed,
                emotion
            );

            let result = api::generate_audio(voice_meta.clone(), text.clone()).await;

            // 生成成功后保存数据供分享使用
            if let Ok(audio_data) = &result {
                set_generated_text.set(text.clone());
                set_generated_voice_name.set(voice_id.clone());
                set_generated_voice_id.set(voice_id.clone());
                set_generated_audio_data.set(audio_data.clone());
                set_show_share_popup.set(true);
                // 设置默认分享内容
                set_share_title.set(format!("我的{}", voice_id));
                set_share_content.set(format!("我使用{}创建了:{}", voice_id, text));
            }

            result
        }
    });

    // 创建帖子的 Action
    let create_post_action = Action::new(move |payload: &CreatePostPayload| {
        let payload = payload.clone();
        async move {
            let post = api::PostInfo {
                id: String::new(), // 服务器端生成 UUID
                title: payload.title,
                metadata: serde_json::json!({
                    "description": payload.content,
                    "audio_data": base64_encode(&payload.audio_data),
                    "voice_meta_id": payload.voice_id,
                })
                .to_string(),
            };
            api::create_post(post).await
        }
    });

    // 监听 create_post_action 的完成
    Effect::new(move |_| {
        match create_post_action.value().get() {
            Some(Ok(())) => {
                debug_log!("帖子创建成功");
                set_show_share_modal.set(false);
                set_generated_text.set(String::new());
                set_generated_voice_name.set(String::new());
                set_generated_voice_id.set(String::new());
                set_generated_audio_data.set(Vec::new());
                set_share_title.set(String::new());
                set_share_content.set(String::new());
            }
            Some(Err(e)) => {
                debug_error!("帖子创建失败: {}", e);
            }
            None => {
                // 还没有结果
            }
        }
    });

    view! {
        <div class="min-h-screen bg-base-100 pb-12">
            <div class="container mx-auto px-4 py-8 md:py-12 max-w-6xl">

                <section class="text-center mb-12">
                    <h2 class="text-[clamp(1.8rem,4vw,2.5rem)] font-bold mb-4 text-shadow text-dark">
                        "声音，也能如此多彩"
                    </h2>
                    <p class="text-gray-600 max-w-2xl mx-auto">
                        "输入文本，选择喜欢的声线，调整参数，体验声音的奇妙变化"
                    </p>
                </section>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">

                    // --- 左侧栏 (输入 + 声线) ---
                    <div class="lg:col-span-1 space-y-8">
                        <TextInputCard text=text_signal />
                        <VoiceSelectorCard selected_voice=voice_signal />
                    </div>

                    // --- 右侧栏 (参数 + 结果) ---
                    <div class="lg:col-span-2 space-y-8">
                        // 1. 参数调节 + 分享按钮
                        <ParameterControlCard
                            selected_param=param_signal
                            selected_voice=voice_signal
                            initial_voice_id=initial_voice_id.clone()
                            initial_param=VoiceParams {
                                pitch: initial_pitch,
                                speed: initial_speed,
                                emotion: initial_emotion,
                            }
                            open_filter_share=set_show_filter_share
                        />
                        // 2. 输出结果 (核心功能)
                        <AudioResultCard generate_action=generate_action />
                    </div>
                </div>
            </div>

            // 分享弹窗
            <Show when=move || show_share_popup.get()>
                <div
                    class="fixed bottom-6 right-6 bg-white rounded-lg shadow-lg border border-gray-200 p-4 z-40 animate-in slide-in-from-bottom-4 duration-300"
                    style="width: 280px;"
                >
                    <div class="flex items-start justify-between mb-3">
                        <h4 class="font-semibold text-gray-800">"是否分享此作品"</h4>
                        <button
                            class="text-gray-400 hover:text-gray-600 transition-colors"
                            on:click=move |_| set_show_share_popup.set(false)
                        >
                            <i class="fa fa-times"></i>
                        </button>
                    </div>
                    <p class="text-sm text-gray-500 mb-4">
                        "分享到社区，与其他用户一起欣赏你的创作"
                    </p>
                    <div class="flex gap-3">
                        <button
                            class="flex-1 px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-lg transition-colors text-sm font-medium"
                            on:click=move |_| set_show_share_popup.set(false)
                        >
                            稍后再说
                        </button>
                        <button
                            class="flex-1 px-3 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors text-sm font-medium"
                            on:click=move |_| {
                                set_show_share_popup.set(false);
                                set_show_share_modal.set(true);
                            }
                        >
                            是的
                        </button>
                    </div>
                </div>
            </Show>

            // 分享模态窗口
            <Show when=move || show_share_modal.get()>
                <div
                    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
                    on:click=move |_| set_show_share_modal.set(false)
                >
                    <div
                        class="bg-white rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] flex flex-col"
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                    >
                        // 头部
                        <div class="flex justify-between items-center p-6 border-b border-gray-200">
                            <h2 class="text-2xl font-bold text-gray-800">"创建分享作品"</h2>
                            <button
                                class="text-gray-400 hover:text-gray-600 transition-colors"
                                on:click=move |_| set_show_share_modal.set(false)
                            >
                                <i class="fa fa-times text-xl"></i>
                            </button>
                        </div>

                        // 内容区域 (可滚动)
                        <div class="flex-1 overflow-y-auto p-6 space-y-6">
                            // 标题输入
                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-2">
                                    "作品标题"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                                    placeholder="请输入作品标题..."
                                    prop:value=move || share_title.get()
                                    on:input=move |ev| set_share_title.set(event_target_value(&ev))
                                />
                            </div>

                            // 内容输入
                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-2">
                                    "作品描述"
                                </label>
                                <textarea
                                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all resize-none"
                                    rows="8"
                                    placeholder="请输入作品描述..."
                                    prop:value=move || share_content.get()
                                    on:input=move |ev| {
                                        set_share_content.set(event_target_value(&ev))
                                    }
                                ></textarea>
                            </div>

                            // 提示信息
                            <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
                                <p class="text-xs text-blue-700">
                                    <i class="fa fa-info-circle mr-2"></i>
                                    "分享后将在社区中显示，所有用户都可以欣赏和互动"
                                </p>
                            </div>
                        </div>

                        // 底部操作栏
                        <div class="flex justify-end gap-3 p-6 border-t border-gray-200 bg-gray-50">
                            <button
                                class="px-6 py-2 bg-gray-200 hover:bg-gray-300 text-gray-800 rounded-lg transition-colors font-medium"
                                on:click=move |_| set_show_share_modal.set(false)
                            >
                                "取消"
                            </button>
                            <button
                                class="px-6 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
                                on:click=move |_| {
                                    let title = share_title.get();
                                    let content = share_content.get();
                                    let voice_id = generated_voice_id.get();
                                    let audio_data = generated_audio_data.get();
                                    if title.trim().is_empty() || content.trim().is_empty() {
                                        return;
                                    }
                                    create_post_action
                                        .dispatch(CreatePostPayload {
                                            title,
                                            content,
                                            voice_id,
                                            audio_data,
                                        });
                                }
                                disabled=move || {
                                    share_title.get().trim().is_empty()
                                        || share_content.get().trim().is_empty()
                                }
                            >
                                "创建"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // 声音滤镜分享全屏弹窗
            <Show when=move || show_filter_share.get()>
                <div
                    class="fixed inset-0 bg-black/60 z-50 flex items-center justify-center p-4"
                    on:click=move |_| set_show_filter_share.set(false)
                >
                    <div
                        class="bg-white rounded-2xl shadow-2xl w-full max-w-3xl max-h-[90vh] flex flex-col"
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                    >
                        <div class="flex justify-between items-center p-6 border-b border-gray-200">
                            <h2 class="text-2xl font-bold text-gray-800">"分享声音滤镜"</h2>
                            <button
                                class="text-gray-400 hover:text-gray-600"
                                on:click=move |_| set_show_filter_share.set(false)
                            >
                                <i class="fa fa-times text-xl"></i>
                            </button>
                        </div>

                        <div class="flex-1 overflow-y-auto p-6 space-y-6">
                            // 标题与介绍（限制 60 字）
                            <div class="grid grid-cols-1 gap-4">
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                        "标题（最多60字）"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50"
                                        placeholder="为滤镜取一个标题"
                                        prop:value=move || filter_share_title.get()
                                        on:input=move |ev| {
                                            let mut v = event_target_value(&ev);
                                            if v.chars().count() > 60 {
                                                v = v.chars().take(60).collect();
                                            }
                                            set_filter_share_title.set(v);
                                        }
                                    />
                                    <div class="text-xs text-gray-400 text-right mt-1">
                                        {move || {
                                            format!("{}/60", filter_share_title.get().chars().count())
                                        }}
                                    </div>
                                </div>
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                        "介绍（最多60字）"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50"
                                        placeholder="简要介绍滤镜用途"
                                        prop:value=move || filter_share_intro.get()
                                        on:input=move |ev| {
                                            let mut v = event_target_value(&ev);
                                            if v.chars().count() > 60 {
                                                v = v.chars().take(60).collect();
                                            }
                                            set_filter_share_intro.set(v);
                                        }
                                    />
                                    <div class="text-xs text-gray-400 text-right mt-1">
                                        {move || {
                                            format!("{}/60", filter_share_intro.get().chars().count())
                                        }}
                                    </div>
                                </div>
                            </div>

                            // 参数预览
                            <div class="bg-gray-50 border border-gray-200 rounded-lg p-4">
                                <h4 class="text-sm font-semibold text-gray-700 mb-3">
                                    "具体参数"
                                </h4>
                                <div class="grid grid-cols-2 gap-3 text-sm text-gray-600">
                                    <div>
                                        <span class="text-gray-400">"声线："</span>
                                        {move || voice_signal.get()}
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"音高："</span>
                                        {move || format!("{:.2}", param_signal.get().pitch)}
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"语速："</span>
                                        {move || format!("{:.2}", param_signal.get().speed)}
                                    </div>
                                    <div>
                                        <span class="text-gray-400">"情绪："</span>
                                        {move || param_signal.get().emotion.to_string()}
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="p-6 border-t border-gray-200 bg-gray-50 flex justify-end">
                            <button
                                class="px-6 py-2 bg-secondary hover:bg-secondary/90 text-white rounded-lg font-medium disabled:opacity-50 disabled:cursor-not-allowed"
                                on:click=move |_| {
                                    let title = filter_share_title.get();
                                    let intro = filter_share_intro.get();
                                    let p = param_signal.get();
                                    let v = voice_signal.get();
                                    spawn_local(async move {
                                        match share_voice_filter_to_db(
                                                title,
                                                intro,
                                                v,
                                                p.pitch,
                                                p.speed,
                                                p.emotion.to_string(),
                                            )
                                            .await
                                        {
                                            Ok(_id) => {
                                                leptos::logging::debug_log!("滤镜分享已入库");
                                            }
                                            Err(e) => {
                                                leptos::logging::error!("滤镜分享失败: {}", e);
                                            }
                                        }
                                    });
                                    set_show_filter_share.set(false);
                                }
                                disabled=move || {
                                    filter_share_title.get().trim().is_empty()
                                        || filter_share_intro.get().trim().is_empty()
                                }
                            >
                                <i class="fa fa-share mr-2"></i>
                                "分享"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
fn copy_to_clipboard(text: &str) {
    use js_sys::{Function, Reflect};
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;

    if let Some(window) = web_sys::window() {
        // 尝试 navigator.clipboard.writeText
        if let Ok(nav) = Reflect::get(&window, &JsValue::from_str("navigator")) {
            if let Ok(clipboard) = Reflect::get(&nav, &JsValue::from_str("clipboard")) {
                if let Ok(write_text) = Reflect::get(&clipboard, &JsValue::from_str("writeText")) {
                    if let Some(f) = write_text.dyn_ref::<Function>() {
                        let _ = f.call1(&clipboard, &JsValue::from_str(text));
                        return;
                    }
                }
            }
        }

        // 回退方案：创建隐藏的 textarea，使用 execCommand('copy')
        if let Some(document) = window.document() {
            if let Ok(el) = document.create_element("textarea") {
                if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                    textarea.set_value(text);
                    let _ =
                        textarea.set_attribute("style", "position:fixed;top:-1000px;left:-1000px;");
                    if let Some(body) = document.body() {
                        let _ = body.append_child(&textarea);
                        textarea.select();
                        if let Ok(exec_cmd) = js_sys::Reflect::get(
                            &document,
                            &wasm_bindgen::JsValue::from_str("execCommand"),
                        ) {
                            if let Some(f) = exec_cmd.dyn_ref::<js_sys::Function>() {
                                let _ =
                                    f.call1(&document, &wasm_bindgen::JsValue::from_str("copy"));
                            }
                        }
                        let _ = body.remove_child(&textarea);
                    }
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn copy_to_clipboard(_text: &str) {}

// 将当前声线与参数作为“声音滤镜”分享入库
#[server]
pub async fn share_voice_filter_to_db(
    title: String,
    intro: String,
    base_model_id: String,
    pitch: f32,
    speed: f32,
    emotion: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use anyhow::Context;
        use axum_extra::extract::cookie::Cookie;
        use http::HeaderMap;
        use leptos::prelude::use_context;
        use sqlx::prelude::*;
        use uuid::Uuid;

        let pool = use_context::<sqlx::SqlitePool>()
            .ok_or_else(|| ServerFnError::new("未找到数据库连接池"))?;

        // 从 Cookie 读取 session_token
        let headers =
            use_context::<HeaderMap>().ok_or_else(|| ServerFnError::new("未找到请求头信息"))?;
        let cookie_header = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ServerFnError::new("未找到 Cookie"))?;
        let mut session_token: Option<String> = None;
        for cookie_str in cookie_header.split(';') {
            if let Ok(c) = Cookie::parse(cookie_str.trim()) {
                if c.name() == "session_token" {
                    session_token = Some(c.value().to_string());
                    break;
                }
            }
        }
        let token = session_token.ok_or_else(|| ServerFnError::new("未登录或会话失效"))?;

        // 查 user_id
        let (user_id,): (String,) = sqlx::query_as(
            "SELECT user_id FROM user_sessions WHERE token = ? AND expires_at > datetime('now')",
        )
        .bind(&token)
        .fetch_one(&pool)
        .await
        .map_err(|_| ServerFnError::new("会话已过期或无效"))?;

        // 生成 ID 并插入 voice_meta_infos
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO voice_meta_infos
               (id, user_id, name, description, base_model_id,
                pitch, speed, volume, emotion, usage_count,
                is_public, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, 'normal', CURRENT_TIMESTAMP)"#,
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&title)
        .bind(&intro)
        .bind(&base_model_id)
        .bind(pitch as f64)
        .bind(speed as f64)
        .bind(1.0_f64) // 默认音量
        .bind(&emotion)
        .execute(&pool)
        .await
        .context("创建声音滤镜失败")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(id)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(String::new())
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(data)
}

#[component]
pub fn TextInputCard(
    /// 用于存储输入文本的信号，由父组件传入
    text: RwSignal<String>,
) -> impl IntoView {
    // 内部状态：控制是否全屏
    let is_fullscreen = RwSignal::new(false);

    view! {
        // 卡片容器
        <section
            class="bg-white shadow-soft transition-all duration-300 ease-in-out rounded-xl"
            // 普通模式样式
            class:p-6=move || !is_fullscreen.get()
            class:hover:shadow-hover=move || !is_fullscreen.get()
            class:relative=move || !is_fullscreen.get()

            // 全屏模式样式 (固定定位，覆盖全屏，高层级)
            class:fixed=move || is_fullscreen.get()
            class:inset-20=move || is_fullscreen.get()
            class:z-50=move || is_fullscreen.get()
            // 全屏时稍微增加 padding，并使用 flex 布局让 textarea 居中或占满
            class:p-12=move || is_fullscreen.get()
            class:flex=move || is_fullscreen.get()
            class:flex-col=move || is_fullscreen.get()
        >
            // 标题区域
            <h3 class="text-lg font-semibold mb-4 flex items-center shrink-0 justify-between">
                <div class="flex items-center">
                    <i class="fa fa-comment text-primary mr-2"></i>
                    "文本输入"
                </div>

                // 全屏模式下的右上角关闭按钮 (作为备用退出方式)
                <Show when=move || is_fullscreen.get()>
                    <button
                        class="text-gray-400 hover:text-dark transition-colors p-2 hover:bg-gray-100 rounded-full"
                        on:click=move |_| is_fullscreen.set(false)
                        title="退出全屏"
                    >
                        <i class="fa fa-times text-xl"></i>
                    </button>
                </Show>
            </h3>

            // 输入区域容器 (相对定位用于放置右下角按钮)
            <div
                class="relative w-full transition-all duration-300 bg-white rounded-lg shadow-sm"
                // 全屏时占满剩余空间，但可以留一点边距
                class:flex-grow=move || is_fullscreen.get()
                class:h-auto=move || is_fullscreen.get()
                // 全屏时给容器加一个最大宽度，防止在大屏上太宽难以阅读
                class:mx-auto=move || is_fullscreen.get()
            >
                <textarea
                    id="text-input"
                    class="w-full p-4 border border-gray-200 rounded-lg \
                     focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary \
                     transition-all duration-300 resize-none font-sans text-gray-700 placeholder-gray-400"
                    // 动态高度
                    class:h-32=move || !is_fullscreen.get()
                    // 全屏时占满父容器高度
                    class:h-full=move || is_fullscreen.get()

                    // 全屏时字体和行高优化
                    class:text-lg=move || is_fullscreen.get()
                    class:leading-loose=move || is_fullscreen.get()
                    // 全屏时增加内边距
                    class:p-5=move || is_fullscreen.get()

                    placeholder="请输入你想转换的文字...\n例如：你好，欢迎使用白昼聆夏"

                    prop:value=move || text.get()
                    on:input=move |ev| text.set(event_target_value(&ev))
                ></textarea>

                // 全屏切换按钮 (悬浮在 Textarea 右下角内部)
                // 修改：调小尺寸，调整位置，增加透明度避免太抢眼
                <button
                    class="absolute bottom-3 right-3 p-2 bg-white/80 hover:bg-white backdrop-blur-sm rounded-md text-gray-400 hover:text-primary hover:border-primary transition-all shadow-sm group z-10"
                    on:click=move |_| is_fullscreen.update(|v| *v = !*v)
                    title=move || if is_fullscreen.get() { "退出全屏" } else { "全屏编辑" }
                >
                    <i
                        class="fa transition-transform duration-300 group-hover:scale-110 text-sm"
                        class:fa-expand=move || !is_fullscreen.get()
                        class:fa-compress=move || is_fullscreen.get()
                    ></i>
                </button>
            </div>

            // 底部提示 (仅在非全屏时显示，全屏时专注于写作)
            <Show when=move || !is_fullscreen.get()>
                <p class="text-xs text-gray-500 mt-2 shrink-0">
                    "输入文本将通过后端 TTS 转换为音频"
                </p>
            </Show>
        </section>
    }
}

#[component]
pub fn VoiceSelectorCard(
    /// 当前选中的声线 ID (双向绑定)
    selected_voice: RwSignal<String>,
) -> impl IntoView {
    // Resource 用于异步获取数据
    let voices_resource = Resource::new(|| (), |_| api::list_voice_models());

    view! {
        <section class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover">
            <h3 class="text-lg font-semibold mb-4 flex items-center">
                <i class="fa fa-user-circle text-primary mr-2"></i>
                "声线选择"
            </h3>

            <div id="voice-selector" class="grid grid-cols-1 gap-3">
                <Suspense fallback=move || {
                    view! {
                        <div class="flex justify-center items-center py-8 text-gray-400 animate-pulse">
                            <i class="fa fa-spinner fa-spin mr-2"></i>
                            "加载声线库..."
                        </div>
                    }
                }>
                    {move || match voices_resource.get() {
                        None => {
                            // 1. 加载中 (虽然 Suspense 会处理，但 Resource 初始可能为 None)
                            view! {
                                <div class="flex justify-center items-center py-8 text-gray-400 animate-pulse">
                                    <i class="fa fa-spinner fa-spin mr-2"></i>
                                    "加载声线库..."
                                </div>
                            }
                                .into_any()
                        }
                        Some(Err(e)) => {
                            debug_error!("加载声线库失败: {:?}", e);

                            // 2. 加载失败
                            view! {
                                <div class="text-red-500 text-center py-4 border border-red-200 rounded bg-red-50">
                                    <i class="fa fa-exclamation-circle mr-2"></i>
                                    "加载失败，请刷新重试"
                                </div>
                            }
                                .into_any()
                        }
                        Some(Ok(voices)) => {

                            // 3. 加载成功
                            view! {
                                // 添加 max-h-[300px] 和 overflow-y-auto 来实现滚动条
                                // pr-2 是为了防止滚动条遮挡内容
                                <div class="grid grid-cols-1 gap-3 max-h-[300px] overflow-y-auto pr-2 custom-scrollbar">
                                    <For
                                        each=move || voices.clone()
                                        key=|voice| voice.id.clone()
                                        children=move |voice| {
                                            let voice_id = voice.id.clone();
                                            let stored_voice_id = StoredValue::new(voice_id);
                                            // 移除 let is_active = ... 变量定义，直接在属性中使用
                                            // 或者使用 StoredValue 来存储 voice_id 以避免多次克隆的开销（对于字符串 ID 来说微乎其微）

                                            // 为了清晰和解决移动问题，我们在每个闭包中直接捕获 voice_id 的克隆
                                            // 由于 String 是 Clone 的，我们可以为每个属性闭包克隆一份 voice_id
                                            // 但更高效的方法是使用 StoredValue 存储 voice_id

                                            view! {
                                                <div
                                                    class="voice-option p-4 border rounded-lg cursor-pointer transition-all duration-200 flex justify-between items-center group"
                                                    // 动态样式
                                                    class:border-primary=move || {
                                                        selected_voice.get() == stored_voice_id.get_value()
                                                    }
                                                    class:bg-primary-50=move || {
                                                        selected_voice.get() == stored_voice_id.get_value()
                                                    }
                                                    class:border-gray-200=move || {
                                                        selected_voice.get() != stored_voice_id.get_value()
                                                    }
                                                    class:hover:border-primary=true
                                                    // 点击事件
                                                    on:click=move |_| {
                                                        selected_voice.set(stored_voice_id.get_value())
                                                    }
                                                >
                                                    <div>
                                                        <div class="font-medium group-hover:text-primary transition-colors">
                                                            {voice.name.clone()}
                                                        </div>
                                                        // 从 metadata JSON 中提取 description
                                                        <div class="text-sm text-gray-500">
                                                            {move || {
                                                                if let Ok(meta) = serde_json::from_str::<
                                                                    serde_json::Value,
                                                                >(&voice.metadata) {
                                                                    meta.get("description")
                                                                        .and_then(|v| v.as_str())
                                                                        .unwrap_or("暂无描述")
                                                                        .to_string()
                                                                } else {
                                                                    "暂无描述".to_string()
                                                                }
                                                            }}
                                                        </div>
                                                    </div>

                                                    // 选中图标
                                                    <div
                                                        class="text-primary transition-opacity duration-200"
                                                        class:hidden=move || {
                                                            selected_voice.get() != stored_voice_id.get_value()
                                                        }
                                                    >
                                                        <i class="fa fa-check-circle text-xl"></i>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
pub fn ParameterControlCard(
    /// 选中的参数 (双向绑定)
    selected_param: RwSignal<VoiceParams>,
    /// 当前选中的声线
    selected_voice: RwSignal<String>,
    /// 初始基线：声线ID
    initial_voice_id: String,
    /// 初始基线：参数
    initial_param: VoiceParams,
    /// 触发分享弹窗
    open_filter_share: WriteSignal<bool>,
) -> impl IntoView {
    // 判定是否有改动
    let is_modified = Memo::new(move |_| {
        let v = selected_voice.get();
        let p = selected_param.get();
        let dv = v != initial_voice_id;
        let dp = (p.pitch - initial_param.pitch).abs() > 1e-4
            || (p.speed - initial_param.speed).abs() > 1e-4
            || p.emotion != initial_param.emotion;
        dv || dp
    });

    view! {
        <section class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover">
            // 标题
            <div class="flex items-center justify-between mb-6">
                <h3 class="text-lg font-semibold flex items-center">
                    <i class="fa fa-sliders text-primary mr-2"></i>
                    "参数调节"
                </h3>
                <Show when=move || is_modified.get()>
                    <button
                        class="text-sm px-3 py-1.5 rounded-lg bg-secondary/10 text-secondary hover:bg-secondary/20 transition-colors flex items-center"
                        on:click=move |_| open_filter_share.set(true)
                        title="分享当前声线与参数"
                    >
                        <i class="fa fa-share mr-2"></i>
                        "分享"
                    </button>
                </Show>
            </div>

            // 模拟的内容（模糊处理）
            <div class="space-y-8">
                <div>
                    <div class="flex justify-between mb-2">
                        <label class="font-medium text-gray-700">"音高 (Pitch)"</label>
                        <span class="text-sm text-primary font-bold">
                            {move || format!("{}", selected_param.get().pitch)}
                        </span>
                    </div>
                    <div class="relative flex items-center">
                        <span class="text-xs text-gray-400 absolute left-0 -bottom-5">"0.5"</span>
                        <input
                            type="range"
                            min="0.5"
                            max="2.0"
                            step="0.01"
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-primary hover:accent-primary-focus transition-all"
                            // 双向绑定逻辑
                            prop:value=move || selected_param.get().pitch
                            on:input=move |ev| {
                                let val = event_target_value(&ev).parse::<f32>().unwrap_or(0.0);
                                selected_param.update(|p| p.pitch = val);
                            }
                        />
                        <span class="text-xs text-gray-400 absolute right-0 -bottom-5">"2.0"</span>
                    </div>
                </div>

                // --- 2. 语速 (Speed) ---
                <div>
                    <div class="flex justify-between mb-2">
                        <label class="font-medium text-gray-700">"语速 (Speed)"</label>
                        <span class="text-sm text-primary font-bold">
                            {move || format!("{:.2}x", selected_param.get().speed)}
                        </span>
                    </div>
                    <div class="relative flex items-center">
                        <span class="text-xs text-gray-400 absolute left-0 -bottom-5">"0.5x"</span>
                        <input
                            type="range"
                            min="0.5"
                            max="2.0"
                            step="0.01"
                            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-primary hover:accent-primary-focus transition-all"
                            prop:value=move || selected_param.get().speed
                            on:input=move |ev| {
                                let val = event_target_value(&ev).parse::<f32>().unwrap_or(1.0);
                                selected_param.update(|p| p.speed = val);
                            }
                        />
                        <span class="text-xs text-gray-400 absolute right-0 -bottom-5">"2.0x"</span>
                    </div>
                </div>

                // --- 3. 情绪 (Emotion) ---
                <div class="pt-4">
                    <div class="flex justify-between mb-3">
                        <label class="font-medium text-gray-700">"情感 (Emotion)"</label>

                    </div>

                    // 滚动容器
                    <div class="relative group/scroll">
                        // 左右渐变遮罩 (提示可滚动)
                        <div class="absolute left-0 top-0 bottom-0 w-4 bg-gradient-to-r from-white to-transparent z-10 pointer-events-none"></div>
                        <div class="absolute right-0 top-0 bottom-0 w-4 bg-gradient-to-l from-white to-transparent z-10 pointer-events-none"></div>

                        <div class="flex overflow-x-auto pb-4 pt-1 px-1 gap-3 scrollbar-hide snap-x">
                            {Emotion::all()
                                .into_iter()
                                .map(|emo| {
                                    let emo_store = StoredValue::new(emo);

                                    view! {
                                        <button
                                            class="flex-shrink-0 px-4 py-2 rounded-full border transition-all duration-200 snap-start text-sm font-medium"
                                            // 直接在每个属性中使用独立的闭包
                                            class:bg-primary=move || {
                                                selected_param.get().emotion == emo_store.get_value()
                                            }
                                            class:text-white=move || {
                                                selected_param.get().emotion == emo_store.get_value()
                                            }
                                            class:border-primary=move || {
                                                selected_param.get().emotion == emo_store.get_value()
                                            }
                                            class:shadow-md=move || {
                                                selected_param.get().emotion == emo_store.get_value()
                                            }

                                            class:bg-white=move || {
                                                selected_param.get().emotion != emo_store.get_value()
                                            }
                                            class:text-gray-600=move || {
                                                selected_param.get().emotion != emo_store.get_value()
                                            }
                                            class:border-gray-200=move || {
                                                selected_param.get().emotion != emo_store.get_value()
                                            }
                                            class:hover:border-primary=move || {
                                                selected_param.get().emotion != emo_store.get_value()
                                            }
                                            class:hover:text-primary=move || {
                                                selected_param.get().emotion != emo_store.get_value()
                                            }

                                            on:click=move |_| {
                                                selected_param
                                                    .update(|p| p.emotion = emo_store.get_value());
                                            }
                                        >
                                            // 这里可以根据情绪添加不同的 emoji，暂时只显示文字
                                            {emo_store.get_value().to_string()}
                                        </button>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn AudioResultCard(
    /// 生成动作 (Action) - 返回二进制音频数据
    generate_action: Action<(), Result<Vec<u8>, ServerFnError>>,
) -> impl IntoView {
    // 获取 Action 的状态信号
    let is_pending = generate_action.pending();
    let value = generate_action.value();

    // 绑定 audio 元素和 canvas 元素
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    // 视觉效果状态
    let is_playing = RwSignal::new(false);

    // 可视化逻辑
    let setup_visualizer = move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::closure::Closure;
            use web_sys::{AnalyserNode, AudioContext, CanvasRenderingContext2d};

            let audio_el = audio_ref.get();
            let canvas_el = canvas_ref.get();

            if let (Some(audio), Some(canvas)) = (audio_el, canvas_el) {
                // Leptos 的 NodeRef deref 得到的是 HtmlElement<Audio>
                // 我们需要将其转换为 web_sys::HtmlAudioElement
                // 由于 Leptos 的元素类型通常可以直接转换，我们尝试直接使用或者通过 JsCast
                use wasm_bindgen::JsCast;
                let audio: web_sys::HtmlAudioElement = audio.unchecked_into();
                let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();

                // 1. 设置 Canvas 尺寸
                let parent = canvas.parent_element().unwrap();
                let width = parent.client_width() as u32;
                let height = 300; // 固定高度
                canvas.set_width(width);
                canvas.set_height(height);

                // 2. 初始化 Audio Context
                audio.set_cross_origin(Some("anonymous"));

                let ctx =
                    AudioContext::new().unwrap_or_else(|_| panic!("Failed to create AudioContext"));
                let analyser = ctx.create_analyser().unwrap();
                analyser.set_fft_size(256); // 256 -> 128 个数据点

                // 创建源并连接
                let source = match ctx.create_media_element_source(&audio) {
                    Ok(src) => src,
                    Err(_) => return, // 可能已经连接过
                };

                source.connect_with_audio_node(&analyser).unwrap();
                analyser
                    .connect_with_audio_node(&ctx.destination())
                    .unwrap();

                let buffer_length = analyser.frequency_bin_count();
                let mut data_array = vec![0u8; buffer_length as usize];

                let ctx_2d: CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                // 渲染一帧
                let f = std::rc::Rc::new(std::cell::RefCell::new(None));
                let g = f.clone();

                let canvas_width = width as f64;
                let canvas_height = height as f64;
                let center_x = canvas_width / 2.0;
                let center_y = canvas_height / 2.0;
                // 稍微减小圆环半径，给波形留出更多空间
                let radius = 60.0;

                *g.borrow_mut() = Some(Closure::new(move || {
                    analyser.get_byte_frequency_data(&mut data_array);

                    ctx_2d.clear_rect(0.0, 0.0, canvas_width, canvas_height);

                    // 绘制圆形背景边框 (灰色 -> 浅暖色)
                    ctx_2d.begin_path();
                    ctx_2d
                        .arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI)
                        .unwrap();
                    ctx_2d.set_stroke_style_str("rgba(75, 85, 99, 0.2)"); // 浅灰色，适配白色背景
                    ctx_2d.set_line_width(2.0);
                    ctx_2d.stroke();

                    // 绘制可视化
                    let bars = buffer_length;
                    // 使用暖黄色 (#FBBF24) 作为柱状图颜色
                    let bar_color = "#FBBF24";
                    ctx_2d.set_fill_style_str(bar_color);

                    for i in 0..bars {
                        let value = data_array[i as usize] as f64;
                        let bar_height = (value / 255.0) * 80.0; // 调整波形最大高度

                        let rad = (i as f64 / bars as f64) * 2.0 * std::f64::consts::PI;

                        ctx_2d.save();
                        ctx_2d.translate(center_x, center_y).unwrap();
                        ctx_2d.rotate(rad).unwrap();

                        let bar_width = 3.0;
                        if bar_height > 0.0 {
                            ctx_2d.fill_rect(-bar_width / 2.0, radius, bar_width, bar_height);
                        }

                        ctx_2d.restore();
                    }

                    request_animation_frame(f.borrow().as_ref().unwrap());
                }));

                request_animation_frame(g.borrow().as_ref().unwrap());
            }
        }
    };

    Effect::new(move |_| {
        if let Some(Ok(_)) = value.get() {
            set_timeout(
                move || {
                    setup_visualizer();
                },
                std::time::Duration::from_millis(100),
            );
        }
    });

    view! {
        <section class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover text-dark border border-gray-100">
            <h3 class="text-lg font-semibold mb-4 flex items-center text-dark">
                <i class="fa fa-volume-up text-primary mr-2"></i>
                "输出结果"
            </h3>

            // --- 生成按钮 ---
            <div class="flex flex-wrap gap-3 mb-6">
                <button
                    id="generate-btn"
                    class="bg-primary hover:bg-primary-focus text-white py-3 px-6 rounded-lg font-medium transition-all duration-300 flex items-center justify-center w-full shadow-md hover:shadow-lg active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
                    on:click=move |_| {
                        generate_action.dispatch(());
                    }
                    disabled=move || is_pending.get()
                >
                    {move || {
                        if is_pending.get() {
                            view! {
                                <>
                                    <i class="fa fa-circle-o-notch fa-spin mr-2"></i>
                                    "正在生成..."
                                </>
                            }
                                .into_view()
                        } else {
                            view! {
                                <>
                                    <i class="fa fa-magic mr-2"></i>
                                    "生成音频"
                                </>
                            }
                                .into_view()
                        }
                    }}
                </button>
            </div>

            // --- 状态展示区域 ---
            <div class="min-h-[300px] flex items-center justify-center relative bg-light/50 rounded-xl">
                {move || match (is_pending.get(), value.get()) {
                    (true, _) => {
                        view! {
                            <div class="flex flex-col items-center justify-center py-8 animate-fade-in text-gray-500">
                                <div class="w-12 h-12 border-4 border-primary/30 border-t-primary rounded-full animate-spin mb-4"></div>
                                <p class="text-gray-500">"AI 正在合成您的声音..."</p>
                            </div>
                        }
                            .into_any()
                    }
                    (false, Some(Ok(audio_bytes))) => {
                        #[cfg(target_arch = "wasm32")]
                        let audio_url = {
                            use wasm_bindgen::JsCast;
                            let blob = web_sys::Blob::new_with_u8_array_sequence(
                                &wasm_bindgen::JsValue::from(
                                    &web_sys::js_sys::Array::of1(
                                        &wasm_bindgen::JsValue::from(
                                            js_sys::Uint8Array::from(audio_bytes.as_slice()),
                                        ),
                                    ),
                                ),
                            );
                            if let Ok(blob) = blob {
                                web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default()
                            } else {
                                String::new()
                            }
                        };
                        #[cfg(not(target_arch = "wasm32"))]
                        let audio_url = String::new();
                        // 将二进制音频数据转换为 Blob URL

                        view! {
                            // 使用 Flex 布局垂直排列 Canvas 和 Controls
                            <div class="w-full animate-slide-up flex flex-col">

                                // Canvas 区域
                                <div class="w-full h-[300px] bg-light/30 rounded-t-xl flex items-center justify-center overflow-hidden relative border-b border-gray-200">
                                    <canvas
                                        node_ref=canvas_ref
                                        class="z-10"
                                        width="600"
                                        height="300"
                                    ></canvas>
                                </div>

                                // 播放器控制栏
                                <div class="w-full p-4 bg-white/80 rounded-b-xl flex flex-col gap-3">
                                    <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
                                        <span>"生成完成"</span>
                                        <button
                                            class="text-primary hover:text-secondary hover:underline flex items-center"
                                            on:click=move |_| {
                                                #[cfg(target_arch = "wasm32")]
                                                {
                                                    let bytes = audio_bytes.clone();
                                                    use wasm_bindgen::JsCast;
                                                    if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(
                                                        &wasm_bindgen::JsValue::from(
                                                            &web_sys::js_sys::Array::of1(
                                                                &wasm_bindgen::JsValue::from(
                                                                    js_sys::Uint8Array::from(bytes.as_slice()),
                                                                ),
                                                            ),
                                                        ),
                                                    ) {
                                                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(
                                                            &blob,
                                                        ) {
                                                            let a = web_sys::window()
                                                                .and_then(|w| w.document())
                                                                .and_then(|d| d.create_element("a").ok())
                                                                .and_then(|a| {
                                                                    a.dyn_into::<web_sys::HtmlAnchorElement>().ok()
                                                                });
                                                            if let Some(a) = a {
                                                                a.set_href(&url);
                                                                a.set_download("tts_audio.mp3");
                                                                a.click();
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        >
                                            <i class="fa fa-download mr-1"></i>
                                            "下载"
                                        </button>
                                    </div>
                                    <audio
                                        node_ref=audio_ref
                                        controls
                                        autoplay
                                        class="w-full h-8 custom-audio-player"
                                        src=audio_url
                                        on:play=move |_| is_playing.set(true)
                                        on:pause=move |_| is_playing.set(false)
                                        crossorigin="anonymous"
                                    ></audio>
                                </div>
                            </div>
                        }
                            .into_any()
                    }
                    (false, Some(Err(e))) => {
                        debug_error!("生成音频失败: {:?}", e);
                        view! {
                            <div class="text-center py-8 text-red-500 bg-red-50 rounded-xl border border-red-200">
                                <i class="fa fa-exclamation-triangle text-4xl mb-3 opacity-50"></i>
                                <p>"生成失败"</p>
                                <p class="text-sm opacity-70">{e.to_string()}</p>
                            </div>
                        }
                            .into_any()
                    }
                    _ => {
                        view! {
                            <div class="w-full h-full min-h-[300px] flex flex-col items-center justify-center text-center text-gray-400 bg-gray-50 rounded-xl border border-dashed border-gray-200 p-8">
                                <i class="fa fa-headphones text-6xl mb-4 opacity-30"></i>
                                <p class="text-base font-medium">"等待生成"</p>
                                <p class="text-sm mt-2 opacity-70">
                                    "在上方输入文本并点击生成按钮"
                                </p>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </section>
    }
}

// 辅助函数：requestAnimationFrame
#[cfg(target_arch = "wasm32")]
fn request_animation_frame(f: &wasm_bindgen::closure::Closure<dyn FnMut()>) {
    use wasm_bindgen::JsCast;
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[cfg(not(target_arch = "wasm32"))]
fn request_animation_frame(_f: &impl std::any::Any) {} // SSR 空实现
