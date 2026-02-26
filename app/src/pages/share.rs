use crate::api;
use leptos::prelude::*;
use uuid::Uuid;

/// 分享声音配置的弹窗组件
///
/// 用户填写标题和描述后，组件会：
/// 1. 调用 `generate_meta` 保存当前声音配置（VoiceMeta）
/// 2. 调用 `create_voice_meta_post` 创建分享帖子（VoiceMetaPost）
/// 3. 生成可复制的分享链接
#[component]
pub fn ShareVoiceConfigModal(
    /// 控制弹窗显示/隐藏
    show: ReadSignal<bool>,
    /// 控制弹窗关闭的写信号
    set_show: WriteSignal<bool>,
    /// 当前选中的声线模型 ID（字符串形式的 UUID）
    voice_model_id: RwSignal<String>,
    /// 当前参数化设置
    parametric: RwSignal<api::voice::Parametic>,
    /// 是否为指令模式
    is_instruction_mode: RwSignal<bool>,
    /// 指令文本
    instruction_text: RwSignal<String>,
    /// 声线列表资源（用于查找模型名称）
    voices_resource: Resource<Result<Vec<api::voice::VoiceModel>, ServerFnError>>,
) -> impl IntoView {
    // ── 表单状态 ──
    let (title, set_title) = signal(String::new());
    let (content, set_content) = signal(String::new());

    // ── 结果状态：None=未提交, Ok(meta_id)=成功, Err(msg)=失败 ──
    let (share_result, set_share_result) = signal::<Option<Result<String, String>>>(None);
    let (copied, set_copied) = signal(false);

    // 弹窗打开时重置表单
    Effect::new(move |_| {
        if show.get() {
            set_title.set(String::new());
            set_content.set(String::new());
            set_share_result.set(None);
            set_copied.set(false);
        }
    });

    // 根据 voice_model_id 查找模型名称
    let voice_model_name = move || -> String {
        let id = voice_model_id.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == id) {
                return voice.info.name.clone();
            }
        }
        "未知模型".to_string()
    };

    // 根据 voice_model_id 查找模型能力
    let voice_model_ability = move || -> Option<api::voice::VoiceModelAbility> {
        let id = voice_model_id.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == id) {
                return Some(voice.ability.clone());
            }
        }
        None
    };

    // ── 分享 Action：generate_meta → create_voice_meta_post ──
    let share_action = Action::new(move |(title_val, content_val): &(String, String)| {
        let title_val = title_val.clone();
        let content_val = content_val.clone();
        let voice_id_str = voice_model_id.get();
        let is_instruction = is_instruction_mode.get();
        let params = parametric.get();
        let instruction = instruction_text.get();

        async move {
            // 解析声线 ID
            let voice_model_uuid = Uuid::parse_str(&voice_id_str)
                .map_err(|e| format!("无效的声线 ID: {}", e))?;

            // 构建 VoiceMeta
            let voice_meta = api::voice::VoiceMeta {
                voice_model_id: voice_model_uuid,
                parametric: if !is_instruction {
                    Some(params)
                } else {
                    None
                },
                instruction: if is_instruction {
                    Some(instruction)
                } else {
                    None
                },
            };

            // 第一步：保存声音配置
            let meta_id = api::voice::generate_meta(voice_meta)
                .await
                .map_err(|e| format!("保存声音配置失败: {}", e))?;

            // 第二步：创建分享帖子
            let post = api::post::VoiceMetaPost {
                id: Uuid::nil(),     // 由服务端自动分配
                title: title_val,
                content: content_val,
                meta_id,
                author: Uuid::nil(), // 由服务端从 session 获取
                status: api::post::PostStatus::Normal,
                comments_count: 0,
                likes_count: 0,
                is_liked_by_current_user: false,
            };

            api::post::create_voice_meta_post(post)
                .await
                .map_err(|e| format!("创建分享失败: {}", e))?;

            // 返回 meta_id，用于构造分享链接
            Ok::<String, String>(meta_id.to_string())
        }
    });

    // 监听 Action 结果
    Effect::new(move |_| {
        if let Some(action_result) = share_action.value().get() {
            match action_result {
                Ok(meta_id) => set_share_result.set(Some(Ok(meta_id))),
                Err(e) => set_share_result.set(Some(Err(e.to_string()))),
            }
        }
    });

    // 提交表单
    let handle_submit = move |_: web_sys::MouseEvent| {
        let t = title.get();
        let c = content.get();
        if t.trim().is_empty() || c.trim().is_empty() {
            return;
        }
        share_action.dispatch((t, c));
    };

    // 复制链接到剪贴板
    let handle_copy = move |link: String| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let nav = window.navigator();
                let clipboard = nav.clipboard();
                let _ = clipboard.write_text(&link);
                set_copied.set(true);
                set_timeout(
                    move || set_copied.set(false),
                    std::time::Duration::from_secs(2),
                );
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = link;
        }
    };

    // 构造分享链接
    let build_share_link = move |meta_id: &str| -> String {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(origin) = window.location().origin() {
                    return format!("{}/home?meta_id={}", origin, meta_id);
                }
            }
        }
        format!("/home?meta_id={}", meta_id)
    };

    let is_pending = share_action.pending();

    // 关闭弹窗的辅助闭包
    let close = move || set_show.set(false);

    view! {
        {move || {
            if !show.get() {
                return view! { <div class="hidden"></div> }.into_any();
            }

            view! {
                // ── 遮罩层 ──
                <div
                    class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4 backdrop-blur-sm"
                    on:click=move |_| close()
                >
                    // ── 弹窗主体 ──
                    <div
                        class="bg-white rounded-2xl shadow-2xl w-full max-w-lg flex flex-col overflow-hidden"
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                    >
                        {move || {
                            match share_result.get() {
                                // ── 成功状态 ──
                                Some(Ok(meta_id)) => {
                                    let link = build_share_link(&meta_id);
                                    let link_for_copy = link.clone();

                                    view! {
                                        <div class="p-8 text-center space-y-6">
                                            // 成功图标
                                            <div class="w-16 h-16 mx-auto bg-green-100 rounded-full flex items-center justify-center">
                                                <i class="fa-solid fa-check text-green-500 text-2xl"></i>
                                            </div>

                                            <div>
                                                <h3 class="text-xl font-bold text-gray-800 mb-2">
                                                    "分享成功！"
                                                </h3>
                                                <p class="text-sm text-gray-500">
                                                    "你的声音配置已分享到社区，其他用户可以通过链接直接加载你的配置"
                                                </p>
                                            </div>

                                            // 分享链接
                                            <div class="bg-gray-50 rounded-xl p-4 space-y-3">
                                                <label class="block text-xs font-semibold text-gray-500 uppercase tracking-wide text-left">
                                                    "分享链接"
                                                </label>
                                                <div class="flex items-center gap-2">
                                                    <input
                                                        type="text"
                                                        readonly
                                                        class="flex-1 px-3 py-2 bg-white border border-gray-200 rounded-lg text-sm text-gray-700 font-mono select-all"
                                                        prop:value=link
                                                    />
                                                    <button
                                                        class=move || {
                                                            if copied.get() {
                                                                "px-4 py-2 rounded-lg text-sm font-medium transition-all whitespace-nowrap bg-green-500 text-white"
                                                            } else {
                                                                "px-4 py-2 rounded-lg text-sm font-medium transition-all whitespace-nowrap bg-primary text-white hover:bg-primary/90"
                                                            }
                                                        }
                                                        on:click=move |_| handle_copy(link_for_copy.clone())
                                                    >
                                                        {move || {
                                                            if copied.get() {
                                                                view! {
                                                                    <>
                                                                        <i class="fa-solid fa-check mr-1"></i>
                                                                        "已复制"
                                                                    </>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! {
                                                                    <>
                                                                        <i class="fa-regular fa-copy mr-1"></i>
                                                                        "复制"
                                                                    </>
                                                                }
                                                                    .into_any()
                                                            }
                                                        }}
                                                    </button>
                                                </div>
                                            </div>

                                            // 完成按钮
                                            <button
                                                class="w-full px-6 py-3 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-xl transition-colors font-medium"
                                                on:click=move |_| close()
                                            >
                                                "完成"
                                            </button>
                                        </div>
                                    }
                                        .into_any()
                                }

                                // ── 错误状态 ──
                                Some(Err(err_msg)) => {
                                    view! {
                                        <div class="p-8 text-center space-y-6">
                                            <div class="w-16 h-16 mx-auto bg-red-100 rounded-full flex items-center justify-center">
                                                <i class="fa-solid fa-xmark text-red-500 text-2xl"></i>
                                            </div>
                                            <div>
                                                <h3 class="text-xl font-bold text-gray-800 mb-2">
                                                    "分享失败"
                                                </h3>
                                                <p class="text-sm text-red-500">{err_msg}</p>
                                            </div>
                                            <div class="flex gap-3">
                                                <button
                                                    class="flex-1 px-6 py-3 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-xl transition-colors font-medium"
                                                    on:click=move |_| close()
                                                >
                                                    "关闭"
                                                </button>
                                                <button
                                                    class="flex-1 px-6 py-3 bg-primary hover:bg-primary/90 text-white rounded-xl transition-colors font-medium"
                                                    on:click=move |_| set_share_result.set(None)
                                                >
                                                    "重试"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                }

                                // ── 表单状态（默认） ──
                                None => {
                                    view! {
                                        <>
                                            // 头部
                                            <div class="flex justify-between items-center px-6 py-5 border-b border-gray-100">
                                                <div class="flex items-center gap-3">
                                                    <div class="w-10 h-10 bg-primary/10 rounded-xl flex items-center justify-center">
                                                        <i class="fa-solid fa-share-nodes text-primary"></i>
                                                    </div>
                                                    <div>
                                                        <h2 class="text-lg font-bold text-gray-800">
                                                            "分享声音配置"
                                                        </h2>
                                                        <p class="text-xs text-gray-400">
                                                            "让更多人体验你的声音调配"
                                                        </p>
                                                    </div>
                                                </div>
                                                <button
                                                    class="w-8 h-8 flex items-center justify-center rounded-lg text-gray-400 hover:bg-gray-100 hover:text-gray-600 transition-all"
                                                    on:click=move |_| close()
                                                >
                                                    <i class="fa-solid fa-xmark"></i>
                                                </button>
                                            </div>

                                            // 表单区域
                                            <div class="px-6 py-5 space-y-5">
                                                // 标题
                                                <div>
                                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                                        "标题"
                                                        <span class="text-red-400 ml-0.5">"*"</span>
                                                    </label>
                                                    <input
                                                        type="text"
                                                        class="w-full px-4 py-2.5 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary transition-all text-sm"
                                                        placeholder="给你的声音配置起个名字..."
                                                        maxlength="100"
                                                        prop:value=move || title.get()
                                                        on:input=move |ev| set_title.set(event_target_value(&ev))
                                                    />
                                                </div>

                                                // 描述（最多50字）
                                                <div>
                                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                                        "描述"
                                                        <span class="text-red-400 ml-0.5">"*"</span>
                                                    </label>
                                                    <textarea
                                                        class="w-full px-4 py-2.5 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary transition-all resize-none text-sm leading-relaxed"
                                                        rows="2"
                                                        placeholder="简要描述这组声音配置的特点..."
                                                        maxlength="50"
                                                        prop:value=move || content.get()
                                                        on:input=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            // 按字符数截断到50
                                                            let truncated: String = val.chars().take(50).collect();
                                                            set_content.set(truncated);
                                                        }
                                                    ></textarea>
                                                    <div class="flex justify-end mt-1">
                                                        <span class="text-xs text-gray-400">
                                                            {move || content.get().chars().count()}
                                                            "/50"
                                                        </span>
                                                    </div>
                                                </div>

                                                // 当前配置预览
                                                <div class="bg-gray-50 rounded-xl p-4 space-y-3">
                                                    <p class="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                                                        "当前配置预览"
                                                    </p>

                                                    // 模型名称
                                                    <div class="flex items-center gap-2 text-sm text-gray-700">
                                                        <i class="fa-solid fa-microphone text-primary text-xs"></i>
                                                        <span class="font-medium">"模型："</span>
                                                        <span>{move || voice_model_name()}</span>
                                                    </div>

                                                    // 指令（模型支持指令 且 内容非空时显示）
                                                    {move || {
                                                        let ability = voice_model_ability();
                                                        let supports_instruction = ability.as_ref().map_or(false, |a| a.instruction_control);
                                                        if supports_instruction {
                                                            let text = instruction_text.get();
                                                            if !text.trim().is_empty() {
                                                                let display: String = if text.chars().count() > 30 {
                                                                    let s: String = text.chars().take(30).collect();
                                                                    format!("{}...", s)
                                                                } else {
                                                                    text
                                                                };
                                                                view! {
                                                                    <div class="flex items-start gap-2 text-sm text-gray-700">
                                                                        <i class="fa-solid fa-wand-magic-sparkles text-purple-500 text-xs mt-0.5"></i>
                                                                        <span class="font-medium">"指令："</span>
                                                                        <span class="text-gray-500">{display}</span>
                                                                    </div>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! { <div></div> }.into_any()
                                                            }
                                                        } else {
                                                            view! { <div></div> }.into_any()
                                                        }
                                                    }}

                                                    // 参数药丸（模型支持参数时始终显示）
                                                    {move || {
                                                        let ability = voice_model_ability();
                                                        let supports_parametric = ability.as_ref().map_or(false, |a| a.parametric_control);
                                                        if supports_parametric {
                                                            let p = parametric.get();
                                                            view! {
                                                                <div class="flex flex-wrap gap-2">
                                                                    <span class="inline-flex items-center px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-xs font-medium">
                                                                        <i class="fa-solid fa-gauge-high mr-1.5"></i>
                                                                        "语速 "
                                                                        {format!("{:.2}x", p.speed)}
                                                                    </span>
                                                                    <span class="inline-flex items-center px-3 py-1 bg-amber-100 text-amber-700 rounded-full text-xs font-medium">
                                                                        <i class="fa-solid fa-arrow-up-right-dots mr-1.5"></i>
                                                                        "音调 "
                                                                        {format!("{:.2}", p.pitch)}
                                                                    </span>
                                                                    <span class="inline-flex items-center px-3 py-1 bg-green-100 text-green-700 rounded-full text-xs font-medium">
                                                                        <i class="fa-solid fa-volume-high mr-1.5"></i>
                                                                        "音量 "
                                                                        {format!("{:.2}%", p.volume * 100.0)}
                                                                    </span>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! { <div></div> }.into_any()
                                                        }
                                                    }}
                                                </div>

                                                // 提示
                                                <div class="flex items-start gap-2 p-3 bg-blue-50 border border-blue-100 rounded-xl">
                                                    <i class="fa-solid fa-circle-info text-blue-400 mt-0.5 text-sm"></i>
                                                    <p class="text-xs text-blue-600 leading-relaxed">
                                                        "分享后，其他用户可以通过链接一键加载你的声音配置参数，在此基础上进行创作。"
                                                    </p>
                                                </div>
                                            </div>

                                            // 底部操作栏
                                            <div class="flex justify-end gap-3 px-6 py-4 border-t border-gray-100 bg-gray-50/50">
                                                <button
                                                    class="px-5 py-2.5 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-xl transition-colors text-sm font-medium"
                                                    on:click=move |_| close()
                                                    disabled=move || is_pending.get()
                                                >
                                                    "取消"
                                                </button>
                                                <button
                                                    class="px-5 py-2.5 bg-primary hover:bg-primary/90 text-white rounded-xl transition-colors text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                                                    on:click=handle_submit
                                                    disabled=move || {
                                                        is_pending.get()
                                                            || title.get().trim().is_empty()
                                                            || content.get().trim().is_empty()
                                                    }
                                                >
                                                    {move || {
                                                        if is_pending.get() {
                                                            view! {
                                                                <>
                                                                    <i class="fa-solid fa-circle-notch fa-spin"></i>
                                                                    "分享中..."
                                                                </>
                                                            }
                                                                .into_any()
                                                        } else {
                                                            view! {
                                                                <>
                                                                    <i class="fa-solid fa-share-nodes"></i>
                                                                    "分享"
                                                                </>
                                                            }
                                                                .into_any()
                                                        }
                                                    }}
                                                </button>
                                            </div>
                                        </>
                                    }
                                        .into_any()
                                }
                            }
                        }}
                    </div>
                </div>
            }
                .into_any()
        }}
    }
}

// ═══════════════════════════════════════════════════════════════
// 右下角弹出提示：生成成功后询问是否分享
// ═══════════════════════════════════════════════════════════════

#[component]
pub fn ShareVoicePopup(
    /// 是否显示
    show: ReadSignal<bool>,
    /// 关闭
    set_show: WriteSignal<bool>,
    /// 点击"分享"后打开完整弹窗
    set_show_modal: WriteSignal<bool>,
) -> impl IntoView {
    let close = move || set_show.set(false);
    let open_modal = move || {
        set_show.set(false);
        set_show_modal.set(true);
    };

    view! {
        {move || {
            if !show.get() {
                return view! { <div class="hidden"></div> }.into_any();
            }

            view! {
                <div class="fixed bottom-6 right-6 z-40 w-72 bg-white rounded-xl shadow-lg border border-gray-200 p-5 animate-in slide-in-from-bottom-4 duration-300">
                    <div class="flex items-start justify-between mb-3">
                        <div class="flex items-center gap-2">
                            <div class="w-8 h-8 bg-green-100 rounded-lg flex items-center justify-center">
                                <i class="fa-solid fa-check text-green-500 text-sm"></i>
                            </div>
                            <h4 class="font-semibold text-gray-800 text-sm">"生成成功！"</h4>
                        </div>
                        <button
                            class="text-gray-400 hover:text-gray-600 transition-colors -mt-1 -mr-1"
                            on:click=move |_| close()
                        >
                            <i class="fa-solid fa-xmark text-xs"></i>
                        </button>
                    </div>
                    <p class="text-xs text-gray-500 mb-4">
                        "把这段声音分享给更多人听听？"
                    </p>
                    <div class="flex gap-2">
                        <button
                            class="flex-1 px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-lg transition-colors text-xs font-medium"
                            on:click=move |_| close()
                        >
                            "暂不"
                        </button>
                        <button
                            class="flex-1 px-3 py-2 bg-green-500 hover:bg-green-600 text-white rounded-lg transition-colors text-xs font-medium"
                            on:click=move |_| open_modal()
                        >
                            <i class="fa-solid fa-share-nodes mr-1"></i>
                            "去分享"
                        </button>
                    </div>
                </div>
            }
                .into_any()
        }}
    }
}

// ═══════════════════════════════════════════════════════════════
// 分享生成的声音（VoicePost）
// ═══════════════════════════════════════════════════════════════

#[component]
pub fn ShareVoicePostModal(
    /// 控制弹窗显示/隐藏
    show: ReadSignal<bool>,
    /// 关闭弹窗
    set_show: WriteSignal<bool>,
    /// 生成的 VoiceLibraryID（字符串）
    library_id: Signal<String>,
    /// 默认标题来源：声线名称
    voice_model_id: RwSignal<String>,
    /// 默认内容来源：生成时的文本
    text_signal: RwSignal<String>,
    /// 声线列表（查模型名称）
    voices_resource: Resource<Result<Vec<api::voice::VoiceModel>, ServerFnError>>,
) -> impl IntoView {
    // ── 表单状态 ──
    let (title, set_title) = signal(String::new());
    let (content, set_content) = signal(String::new());
    let (share_result, set_share_result) = signal::<Option<Result<(), String>>>(None);

    // 查找模型名称
    let voice_model_name = move || -> String {
        let id = voice_model_id.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == id) {
                return voice.info.name.clone();
            }
        }
        "AI 语音作品".to_string()
    };

    // 弹窗打开时用默认值填充
    Effect::new(move |_| {
        if show.get() {
            set_title.set(voice_model_name());
            set_content.set(text_signal.get());
            set_share_result.set(None);
        }
    });

    // ── 分享 Action ──
    let share_action = Action::new(
        move |(title_val, content_val, lib_id_str): &(String, String, String)| {
            let title_val = title_val.clone();
            let content_val = content_val.clone();
            let lib_id_str = lib_id_str.clone();

            async move {
                let lib_id = Uuid::parse_str(&lib_id_str)
                    .map_err(|e| format!("无效的音频 ID: {}", e))?;

                let post = api::post::VoicePost {
                    id: Uuid::nil(),
                    title: title_val,
                    content: content_val,
                    library_id: lib_id,
                    author: Uuid::nil(),
                    status: api::post::PostStatus::Normal,
                    comments_count: 0,
                    likes_count: 0,
                };

                api::post::create_voice_post(post)
                    .await
                    .map_err(|e| format!("创建分享失败: {}", e))?;

                Ok::<(), String>(())
            }
        },
    );

    // 监听结果
    Effect::new(move |_| {
        if let Some(result) = share_action.value().get() {
            match result {
                Ok(()) => set_share_result.set(Some(Ok(()))),
                Err(e) => set_share_result.set(Some(Err(e.to_string()))),
            }
        }
    });

    let handle_submit = move |_: web_sys::MouseEvent| {
        let t = title.get();
        let c = content.get();
        let lib = library_id.get();
        if t.trim().is_empty() || c.trim().is_empty() || lib.is_empty() {
            return;
        }
        share_action.dispatch((t, c, lib));
    };

    let is_pending = share_action.pending();
    let close = move || set_show.set(false);

    view! {
        {move || {
            if !show.get() {
                return view! { <div class="hidden"></div> }.into_any();
            }

            view! {
                // 遮罩
                <div
                    class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4 backdrop-blur-sm"
                    on:click=move |_| close()
                >
                    // 弹窗
                    <div
                        class="bg-white rounded-2xl shadow-2xl w-full max-w-lg flex flex-col overflow-hidden"
                        on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                    >
                        {move || {
                            let audio_url_inner = format!("/api/audio/{}", library_id.get());

                            match share_result.get() {
                                // ── 成功 ──
                                Some(Ok(())) => {
                                    view! {
                                        <div class="p-8 text-center space-y-6">
                                            <div class="w-16 h-16 mx-auto bg-green-100 rounded-full flex items-center justify-center">
                                                <i class="fa-solid fa-check text-green-500 text-2xl"></i>
                                            </div>
                                            <div>
                                                <h3 class="text-xl font-bold text-gray-800 mb-2">
                                                    "分享成功！"
                                                </h3>
                                                <p class="text-sm text-gray-500">
                                                    "你的声音作品已发布到社区"
                                                </p>
                                            </div>
                                            <button
                                                class="w-full px-6 py-3 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-xl transition-colors font-medium"
                                                on:click=move |_| close()
                                            >
                                                "完成"
                                            </button>
                                        </div>
                                    }
                                        .into_any()
                                }

                                // ── 失败 ──
                                Some(Err(err_msg)) => {
                                    view! {
                                        <div class="p-8 text-center space-y-6">
                                            <div class="w-16 h-16 mx-auto bg-red-100 rounded-full flex items-center justify-center">
                                                <i class="fa-solid fa-xmark text-red-500 text-2xl"></i>
                                            </div>
                                            <div>
                                                <h3 class="text-xl font-bold text-gray-800 mb-2">
                                                    "分享失败"
                                                </h3>
                                                <p class="text-sm text-red-500">{err_msg}</p>
                                            </div>
                                            <div class="flex gap-3">
                                                <button
                                                    class="flex-1 px-6 py-3 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-xl transition-colors font-medium"
                                                    on:click=move |_| close()
                                                >
                                                    "关闭"
                                                </button>
                                                <button
                                                    class="flex-1 px-6 py-3 bg-primary hover:bg-primary/90 text-white rounded-xl transition-colors font-medium"
                                                    on:click=move |_| set_share_result.set(None)
                                                >
                                                    "重试"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                }

                                // ── 表单 ──
                                None => {
                                    view! {
                                        <>
                                            // 头部
                                            <div class="flex justify-between items-center px-6 py-5 border-b border-gray-100">
                                                <div class="flex items-center gap-3">
                                                    <div class="w-10 h-10 bg-green-100 rounded-xl flex items-center justify-center">
                                                        <i class="fa-solid fa-music text-green-600"></i>
                                                    </div>
                                                    <div>
                                                        <h2 class="text-lg font-bold text-gray-800">
                                                            "分享声音作品"
                                                        </h2>
                                                        <p class="text-xs text-gray-400">
                                                            "让更多人听到你的创作"
                                                        </p>
                                                    </div>
                                                </div>
                                                <button
                                                    class="w-8 h-8 flex items-center justify-center rounded-lg text-gray-400 hover:bg-gray-100 hover:text-gray-600 transition-all"
                                                    on:click=move |_| close()
                                                >
                                                    <i class="fa-solid fa-xmark"></i>
                                                </button>
                                            </div>

                                            // 表单
                                            <div class="px-6 py-5 space-y-5">
                                                // 标题
                                                <div>
                                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                                        "标题"
                                                        <span class="text-red-400 ml-0.5">"*"</span>
                                                    </label>
                                                    <input
                                                        type="text"
                                                        class="w-full px-4 py-2.5 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-green-300 focus:border-green-400 transition-all text-sm"
                                                        placeholder="给这段声音起个名字..."
                                                        maxlength="100"
                                                        prop:value=move || title.get()
                                                        on:input=move |ev| set_title.set(event_target_value(&ev))
                                                    />
                                                </div>

                                                // 介绍
                                                <div>
                                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                                        "介绍"
                                                        <span class="text-red-400 ml-0.5">"*"</span>
                                                    </label>
                                                    <textarea
                                                        class="w-full px-4 py-2.5 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-green-300 focus:border-green-400 transition-all resize-none text-sm leading-relaxed"
                                                        rows="3"
                                                        placeholder="描述一下这段声音..."
                                                        prop:value=move || content.get()
                                                        on:input=move |ev| set_content.set(event_target_value(&ev))
                                                    ></textarea>
                                                </div>

                                                // 音频预览
                                                <div class="bg-gray-50 rounded-xl p-4 space-y-3">
                                                    <p class="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                                                        "音频预览"
                                                    </p>
                                                    <audio
                                                        controls
                                                        class="w-full h-10"
                                                        src=audio_url_inner
                                                        crossorigin="anonymous"
                                                    ></audio>
                                                </div>
                                            </div>

                                            // 底部
                                            <div class="flex justify-end gap-3 px-6 py-4 border-t border-gray-100 bg-gray-50/50">
                                                <button
                                                    class="px-5 py-2.5 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-xl transition-colors text-sm font-medium"
                                                    on:click=move |_| close()
                                                    disabled=move || is_pending.get()
                                                >
                                                    "取消"
                                                </button>
                                                <button
                                                    class="px-5 py-2.5 bg-green-500 hover:bg-green-600 text-white rounded-xl transition-colors text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                                                    on:click=handle_submit
                                                    disabled=move || {
                                                        is_pending.get()
                                                            || title.get().trim().is_empty()
                                                            || content.get().trim().is_empty()
                                                            || library_id.get().is_empty()
                                                    }
                                                >
                                                    {move || {
                                                        if is_pending.get() {
                                                            view! {
                                                                <>
                                                                    <i class="fa-solid fa-circle-notch fa-spin"></i>
                                                                    "分享中..."
                                                                </>
                                                            }
                                                                .into_any()
                                                        } else {
                                                            view! {
                                                                <>
                                                                    <i class="fa-solid fa-share-nodes"></i>
                                                                    "分享"
                                                                </>
                                                            }
                                                                .into_any()
                                                        }
                                                    }}
                                                </button>
                                            </div>
                                        </>
                                    }
                                        .into_any()
                                }
                            }
                        }}
                    </div>
                </div>
            }
                .into_any()
        }}
    }
}
