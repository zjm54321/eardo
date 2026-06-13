use crate::api;
use crate::api::voice::Parametic;
use crate::pages::component::{InstructionParams, TraditionalParams, ai_rewrite_text};
use crate::pages::share::{ShareVoiceConfigModal, ShareVoicePostModal};
use leptos::logging::debug_error;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn HomePage() -> impl IntoView {
    // 状态
    // 1. 获取 URL 查询参数
    let query = use_query_map();

    // 获取可选的字符串参数
    let get_str_param_opt = |key: &str| {
        query.with_untracked(|q| {
            q.get(key)
                .map(|arg0: std::string::String| ToString::to_string(&arg0))
        })
    };

    let text_signal = RwSignal::new(get_str_param_opt("text").unwrap_or_default());

    // 初始化声线 ID 和参数（默认值）
    let voice_signal = RwSignal::new(String::new());

    let parse_param = |key: &str, default: f32| {
        get_str_param_opt(key)
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(default)
    };

    let param_signal = RwSignal::new(Parametic {
        pitch: parse_param("pitch", 1.0),
        speed: parse_param("speed", 1.0),
        volume: parse_param("volume", 1.0),
    });

    // 指令参数状态
    let is_instruction_mode = RwSignal::new(true);
    let instruction_text = RwSignal::new(get_str_param_opt("instruction").unwrap_or_default());

    // 分享弹窗状态（声音配置）
    let (show_share, set_show_share) = signal(false);

    // 分享弹窗状态（生成的声音）
    let (show_voice_share, set_show_voice_share) = signal(false);

    // Resource 用于异步获取数据
    let voices_resource = Resource::new(|| (), |_| api::voice::list_voice_models());

    // 检查是否有 meta_id 参数，如果有则加载对应的 meta 数据
    let url_meta_id = get_str_param_opt("meta_id");
    let meta_resource = Resource::new_blocking(
        move || url_meta_id.clone(),
        move |meta_id_opt| async move {
            if let Some(meta_id_str) = meta_id_opt {
                if let Ok(meta_id) = uuid::Uuid::parse_str(&meta_id_str) {
                    api::voice::get_meta(meta_id).await.ok()
                } else {
                    None
                }
            } else {
                None
            }
        },
    );

    // 使用 Effect 来应用从 meta_id 加载的数据
    Effect::new(move |_| {
        if let Some(Some(meta)) = meta_resource.get() {
            // 设置 voice_model_id
            voice_signal.set(meta.voice_model_id.to_string());

            let has_instruction = meta
                .instruction
                .as_ref()
                .map(|instruction| !instruction.trim().is_empty())
                .unwrap_or(false);

            // meta 可能同时包含参数和指令，两个都要应用；不能用 else-if 丢掉指令。
            if let Some(parametric) = meta.parametric {
                param_signal.set(parametric);
            }

            if let Some(instruction) = meta.instruction {
                instruction_text.set(instruction);
            } else {
                instruction_text.set(String::new());
            }

            is_instruction_mode.set(has_instruction);
        }
    });

    Effect::new(move |_| {
        if voice_signal.get().is_empty() {
            if let Some(Ok(voices)) = voices_resource.get() {
                if let Some(first_voice) = voices.first() {
                    let id = first_voice.id.to_string();
                    voice_signal.set(id);
                }
            }
        }
    });

    // 创建 Action 处理生成请求
    // Action 自动管理 pending (加载中) 和 value (返回值) 状态
    let generate_action = Action::new(move |_| {
        let text = text_signal.get();
        let voice_id = voice_signal.get();
        let pitch = param_signal.get().pitch;
        let speed = param_signal.get().speed;
        let volume = param_signal.get().volume;
        let instruction = instruction_text.get();

        async move {
            let voice_model_id = match uuid::Uuid::parse_str(&voice_id) {
                Ok(id) => id,
                Err(e) => {
                    debug_error!("无效的语音模型 ID: {}", e);
                    return Err(ServerFnError::new("无效的语音模型 ID"));
                }
            };

            // 创建 VoiceMeta 对象（两种参数同时生效）
            let voice_meta = api::voice::VoiceMeta {
                voice_model_id,
                parametric: Some(api::voice::Parametic {
                    pitch,
                    speed,
                    volume,
                }),
                instruction: if !instruction.trim().is_empty() {
                    Some(instruction)
                } else {
                    None
                },
            };

            let voice_meta_id = match api::voice::generate_meta(voice_meta).await {
                Ok(id) => id,
                Err(e) => {
                    debug_error!("生成语音元数据失败: {}", e);
                    return Err(ServerFnError::new("生成语音元数据失败"));
                }
            };

            let result = api::voice::generate_voice(voice_meta_id, text.clone()).await;

            result
        }
    });

    // 从 generate_action 结果派生 library_id
    let library_id_signal = Signal::derive(move || {
        generate_action
            .value()
            .get()
            .and_then(|r| r.ok())
            .map(|id| id.to_string())
            .unwrap_or_default()
    });

    view! {
        <div class="min-h-screen bg-base-100 pb-12">
            <div class="container mx-auto px-4 py-8 md:py-12 max-w-6xl">

                // 右上角切换到 Setup 模式按钮
                <div class="flex justify-end mb-4">
                    <a
                        href="/setup"
                        class="inline-flex items-center px-4 py-2 rounded-full bg-primary/10 hover:bg-primary/20 text-primary text-sm font-medium transition-all group"
                    >
                        <i class="fa-solid fa-wand-magic-sparkles mr-2 group-hover:rotate-12 transition-transform"></i>
                        "渐进式配置"
                        <i class="fa-solid fa-arrow-right ml-2 text-xs opacity-60 group-hover:translate-x-0.5 transition-transform"></i>
                    </a>
                </div>

                <section class="text-center mb-12">
                    <h2 class="text-[clamp(1.8rem,4vw,2.5rem)] font-bold mb-4 text-shadow text-dark">
                        "声音，也能如此多彩"
                    </h2>
                    <p class="text-gray-600 max-w-2xl mx-auto">
                        "输入文本，选择喜欢的声线，调整参数，体验声音的奇妙变化"
                    </p>
                </section>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 items-stretch">

                    // --- 左侧栏 (声线) ---
                    <div class="lg:col-span-1 space-y-8 h-full">
                        <VoiceSelectorCard
                            selected_voice=voice_signal
                            voices_resource=voices_resource
                        />
                    </div>

                    // --- 右侧栏 (输入 + 参数 + 结果) ---
                    <div class="lg:col-span-2 space-y-8">
                        // 1. 文字输入
                        <TextInputCard text=text_signal />
                        // 2. 参数调节
                        <ParameterControlCard
                            selected_param=param_signal
                            selected_voice=voice_signal
                            is_instruction_mode=is_instruction_mode
                            instruction_text=instruction_text
                            voices_resource=voices_resource
                            set_show_share=set_show_share
                        />
                        // 3. 输出结果 (核心功能)
                        <AudioResultCard
                            generate_action=generate_action
                            set_show_voice_share=set_show_voice_share
                            text_signal=text_signal
                            voice_signal=voice_signal
                        />
                    </div>
                </div>
            </div>

            // 分享声音配置弹窗
            <ShareVoiceConfigModal
                show=show_share
                set_show=set_show_share
                voice_model_id=voice_signal
                parametric=param_signal
                is_instruction_mode=is_instruction_mode
                instruction_text=instruction_text
                voices_resource=voices_resource
            />

            // 分享声音作品弹窗
            <ShareVoicePostModal
                show=show_voice_share
                set_show=set_show_voice_share
                library_id=library_id_signal
                voice_model_id=voice_signal
                text_signal=text_signal
                voices_resource=voices_resource
            />
        </div>
    }
}

#[component]
pub fn TextInputCard(
    /// 用于存储输入文本的信号，由父组件传入
    text: RwSignal<String>,
) -> impl IntoView {
    // 内部状态：控制是否全屏
    let is_fullscreen = RwSignal::new(false);
    let show_ai_modal = RwSignal::new(false);
    let scene = RwSignal::new("汇报".to_string());
    let audience = RwSignal::new("老师".to_string());
    let duration = RwSignal::new("3".to_string());
    let output_text = RwSignal::new(String::new());
    let original_text = RwSignal::new(String::new());
    let active_tab = RwSignal::new("assistant".to_string());
    let status_text = RwSignal::new(String::new());

    let ai_action = Action::new(move |action: &String| {
        let action = action.clone();
        let input = text.get();
        let scene = scene.get();
        let audience = audience.get();
        let duration = duration.get();
        async move {
            let prompt = format!(
                "你是表达助手。请严格遵守：\n\
                1) 不生成违法、暴力、仇恨、色情、诈骗、隐私泄露等有害内容；\n\
                2) 若用户意图不当，给出拒绝并建议合法替代方案；\n\
                3) 输出应清晰、可编辑、面向口播；\n\
                4) 只输出结果内容，不要附加解释。\n\n\
                任务：{action}\n\
                场景：{scene}\n\
                受众：{audience}\n\
                时长：{duration}分钟\n\n\
                原始文本：\n{input}",
            );

            ai_rewrite_text(prompt).await
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(result)) = ai_action.value().get() {
            output_text.set(result);
            active_tab.set("assistant".to_string());
            status_text.set("生成成功".to_string());
        } else if let Some(Err(err)) = ai_action.value().get() {
            status_text.set(format!("生成失败：{}", err));
            debug_error!("AI 处理失败: {}", err);
        }
    });

    view! {
        // 卡片容器
        <section
            id="text-input-section"
            class="bg-white shadow-soft transition-all duration-300 ease-in-out rounded-xl scroll-mt-28"
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
                    <i class="fa-solid fa-comment text-primary mr-2"></i>
                    "文本输入"
                </div>

                // 全屏模式下的右上角关闭按钮 (作为备用退出方式)
                <Show when=move || is_fullscreen.get()>
                    <button
                        class="text-gray-400 hover:text-dark transition-colors p-2 hover:bg-gray-100 rounded-full"
                        on:click=move |_| is_fullscreen.set(false)
                        title="退出全屏"
                    >
                        <i class="fa-solid fa-xmark text-xl"></i>
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

                // AI 按钮 (悬浮在 Textarea 右下角内部，高斯模糊背景)
                <button
                    class="absolute bottom-3 right-3 w-10 h-10 rounded-full bg-white/20 hover:bg-white/30 backdrop-blur-md text-gray-700 hover:text-gray-800 shadow-lg hover:shadow-xl transition-all duration-300 group z-10 flex items-center justify-center"
                    on:click=move |_| {
                        original_text.set(text.get());
                        output_text.set(String::new());
                        status_text.set(String::new());
                        active_tab.set("assistant".to_string());
                        show_ai_modal.set(true);
                    }
                    title="表达助手"
                >
                    <i class="fa-solid fa-wand-magic-sparkles text-lg group-hover:scale-110 transition-transform duration-300"></i>
                </button>
            </div>

            // 底部提示 (仅在非全屏时显示，全屏时专注于写作)
            <Show when=move || !is_fullscreen.get()>
                <p class="text-xs text-gray-500 mt-2 shrink-0">
                    "输入文本将通过后端 TTS 转换为音频"
                </p>
            </Show>
        </section>

        // 表达助手弹窗（可选）
        <Show when=move || show_ai_modal.get()>
            <div
                class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4"
                on:click=move |_| show_ai_modal.set(false)
            >
                <div
                    class="bg-white rounded-2xl shadow-2xl w-full max-w-3xl max-h-[90vh] flex flex-col"
                    on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                >
                    <div class="flex justify-between items-center p-6 border-b border-gray-200">
                        <h2 class="text-xl font-bold text-gray-800">"表达助手"</h2>
                        <button
                            class="text-gray-400 hover:text-gray-600"
                            on:click=move |_| show_ai_modal.set(false)
                        >
                            <i class="fa-solid fa-xmark text-xl"></i>
                        </button>
                    </div>

                    <div class="flex-1 overflow-y-auto p-6 space-y-5">
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-2">
                                    "场景"
                                </label>
                                <select
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50"
                                    on:change=move |ev| scene.set(event_target_value(&ev))
                                >
                                    <option selected=move || {
                                        scene.get() == "汇报"
                                    }>"汇报"</option>
                                    <option selected=move || {
                                        scene.get() == "科普"
                                    }>"科普"</option>
                                    <option selected=move || {
                                        scene.get() == "答辩"
                                    }>"答辩"</option>
                                    <option selected=move || {
                                        scene.get() == "其他"
                                    }>"其他"</option>
                                </select>
                            </div>

                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-2">
                                    "受众"
                                </label>
                                <select
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50"
                                    on:change=move |ev| audience.set(event_target_value(&ev))
                                >
                                    <option selected=move || {
                                        audience.get() == "老师"
                                    }>"老师"</option>
                                    <option selected=move || {
                                        audience.get() == "同学"
                                    }>"同学"</option>
                                    <option selected=move || {
                                        audience.get() == "公众"
                                    }>"公众"</option>
                                </select>
                            </div>

                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-2">
                                    "时长"
                                </label>
                                <div class="inline-flex w-full rounded-lg border border-gray-300 overflow-hidden">
                                    <button
                                        class="flex-1 py-2 text-sm"
                                        class:bg-primary=move || duration.get() == "1"
                                        class:text-white=move || duration.get() == "1"
                                        class:text-gray-700=move || duration.get() != "1"
                                        on:click=move |_| duration.set("1".to_string())
                                    >
                                        "1分钟"
                                    </button>
                                    <button
                                        class="flex-1 py-2 text-sm border-l border-gray-300"
                                        class:bg-primary=move || duration.get() == "3"
                                        class:text-white=move || duration.get() == "3"
                                        class:text-gray-700=move || duration.get() != "3"
                                        on:click=move |_| duration.set("3".to_string())
                                    >
                                        "3分钟"
                                    </button>
                                    <button
                                        class="flex-1 py-2 text-sm border-l border-gray-300"
                                        class:bg-primary=move || duration.get() == "5"
                                        class:text-white=move || duration.get() == "5"
                                        class:text-gray-700=move || duration.get() != "5"
                                        on:click=move |_| duration.set("5".to_string())
                                    >
                                        "5分钟"
                                    </button>
                                </div>
                            </div>
                        </div>

                        <div class="flex flex-wrap gap-3">
                            <button
                                class="px-4 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors text-sm font-medium"
                                on:click=move |_| {
                                    status_text.set("生成中...".to_string());
                                    let _ = ai_action.dispatch("生成提纲".to_string());
                                }
                                disabled=move || ai_action.pending().get()
                            >
                                "生成提纲"
                            </button>
                            <button
                                class="px-4 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors text-sm font-medium"
                                on:click=move |_| {
                                    status_text.set("生成中...".to_string());
                                    let _ = ai_action.dispatch("口播改写".to_string());
                                }
                                disabled=move || ai_action.pending().get()
                            >
                                "口播改写"
                            </button>
                            <button
                                class="px-4 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors text-sm font-medium"
                                on:click=move |_| {
                                    status_text.set("生成中...".to_string());
                                    let _ = ai_action.dispatch("术语轻解释".to_string());
                                }
                                disabled=move || ai_action.pending().get()
                            >
                                "术语轻解释"
                            </button>
                        </div>

                        <div class="border rounded-lg">
                            <div class="flex border-b">
                                <button
                                    class="px-4 py-2 text-sm font-medium"
                                    class:text-primary=move || active_tab.get() == "assistant"
                                    class:bg-primary-50=move || active_tab.get() == "assistant"
                                    on:click=move |_| active_tab.set("assistant".to_string())
                                >
                                    "助手文本"
                                </button>
                                <button
                                    class="px-4 py-2 text-sm font-medium border-l"
                                    class:text-primary=move || active_tab.get() == "original"
                                    class:bg-primary-50=move || active_tab.get() == "original"
                                    on:click=move |_| active_tab.set("original".to_string())
                                >
                                    "原始文本"
                                </button>
                            </div>
                            <div class="p-3">
                                <textarea
                                    class="w-full min-h-[180px] resize-y border border-gray-200 rounded-lg p-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                                    prop:value=move || {
                                        if active_tab.get() == "assistant" {
                                            output_text.get()
                                        } else {
                                            original_text.get()
                                        }
                                    }
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        if active_tab.get() == "assistant" {
                                            output_text.set(v);
                                        } else {
                                            original_text.set(v);
                                        }
                                    }
                                ></textarea>
                            </div>
                        </div>

                        <div class="flex flex-wrap gap-3">
                            <button
                                class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-lg text-sm font-medium"
                                on:click=move |_| {
                                    text.set(output_text.get());
                                    show_ai_modal.set(false);
                                }
                            >
                                "一键填入"
                            </button>
                            <button
                                class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-lg text-sm font-medium"
                                on:click=move |_| {
                                    text.set(output_text.get());
                                }
                            >
                                "覆盖"
                            </button>
                            <button
                                class="px-4 py-2 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-lg text-sm font-medium"
                                on:click=move |_| {
                                    let mut v = text.get();
                                    let add = output_text.get();
                                    if !v.trim().is_empty() && !add.trim().is_empty() {
                                        v.push_str("\n\n");
                                    }
                                    v.push_str(&add);
                                    text.set(v);
                                }
                            >
                                "追加"
                            </button>
                        </div>

                        <div class="text-sm">
                            <Show
                                when=move || !status_text.get().is_empty()
                                fallback=move || view! { <span class="text-gray-400">""</span> }
                            >
                                <span
                                    class="text-gray-600"
                                    class:text-primary=move || status_text.get() == "生成成功"
                                    class:text-red-500=move || {
                                        status_text.get().starts_with("生成失败")
                                    }
                                >
                                    {move || status_text.get()}
                                </span>
                            </Show>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn VoiceSelectorCard(
    /// 当前选中的声线 ID (双向绑定)
    selected_voice: RwSignal<String>,
    /// 声线列表资源
    voices_resource: Resource<Result<Vec<api::voice::VoiceModel>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <section
            id="voice-selector"
            class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover h-full flex flex-col lg:max-h-[1100px] overflow-hidden scroll-mt-28"
        >
            <h3 class="text-lg font-semibold mb-4 flex items-center">
                <i class="fa-solid fa-circle-user text-primary mr-2"></i>
                "声线选择"
            </h3>

            <div class="flex-1 min-h-0">
                <Suspense fallback=move || {
                    view! {
                        <div class="flex justify-center items-center py-8 text-gray-400 animate-pulse">
                            <i class="fa-solid fa-spinner fa-spin mr-2"></i>
                            "加载声线库..."
                        </div>
                    }
                }>
                    {move || match voices_resource.get() {
                        None => {
                            // 1. 加载中 (虽然 Suspense 会处理，但 Resource 初始可能为 None)
                            view! {
                                <div class="flex justify-center items-center py-8 text-gray-400 animate-pulse">
                                    <i class="fa-solid fa-spinner fa-spin mr-2"></i>
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
                                    <i class="fa-solid fa-circle-exclamation mr-2"></i>
                                    "加载失败，请刷新重试"
                                </div>
                            }
                                .into_any()
                        }
                        Some(Ok(voices)) => {

                            // 3. 加载成功
                            view! {
                                // 使用 h-full 让容器占满父容器，overflow-y-auto 实现滚动条
                                // pr-2 是为了防止滚动条遮挡内容
                                <div class="flex flex-col gap-3 h-full overflow-y-auto pr-2">
                                    <For
                                        each=move || voices.clone()
                                        key=|voice| voice.id.to_string()
                                        children=move |voice| {
                                            let voice_id = voice.id.to_string();
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
                                                            {voice.info.name.clone()}
                                                        </div>
                                                        <div class="text-sm text-gray-500">
                                                            {move || {
                                                                let desc = voice.info.description.clone();
                                                                if desc.is_empty() {
                                                                    "暂无描述".to_string()
                                                                } else {
                                                                    desc
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
                                                        <i class="fa-solid fa-circle-check text-xl"></i>
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
    selected_param: RwSignal<Parametic>,
    /// 当前选中的声线
    selected_voice: RwSignal<String>,
    /// 是否为指令模式
    is_instruction_mode: RwSignal<bool>,
    /// 指令文本
    instruction_text: RwSignal<String>,
    /// 声线列表资源
    voices_resource: Resource<Result<Vec<api::voice::VoiceModel>, ServerFnError>>,
    /// 打开分享声音配置弹窗
    set_show_share: WriteSignal<bool>,
) -> impl IntoView {
    let selected_ability = move || {
        let voice_id = selected_voice.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == voice_id) {
                return Some(voice.ability.clone());
            }
        }
        None
    };

    Effect::new(move |_| {
        if let Some(ability) = selected_ability() {
            if ability.instruction_control && !ability.parametric_control {
                is_instruction_mode.set(true);
            } else if ability.parametric_control && !ability.instruction_control {
                is_instruction_mode.set(false);
            }
        }
    });

    view! {
        <section
            id="parameter-control"
            class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover scroll-mt-28"
        >
            // 标题 + 切换块 + 分享按钮
            <div class="flex items-center gap-3 mb-6 flex-wrap">
                <h3 class="text-lg font-semibold flex items-center shrink-0">
                    <i class="fa-solid fa-sliders text-primary mr-2"></i>
                    "参数调节"
                </h3>
                // 切换块（紧邻标题右侧）
                <Suspense fallback=move || {
                    view! { <div class="h-8 w-44 bg-gray-100 rounded-lg animate-pulse"></div> }
                }>
                    <Show when=move || {
                        if let Some(ability) = selected_ability() {
                            ability.instruction_control && ability.parametric_control
                        } else {
                            false
                        }
                    }>
                        <div class="flex items-center gap-2">
                            <div class="flex items-center gap-1 bg-gray-100 rounded-lg p-1">
                                <button
                                    class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all"
                                    class:bg-white=move || !is_instruction_mode.get()
                                    class:shadow-sm=move || !is_instruction_mode.get()
                                    class:text-primary=move || !is_instruction_mode.get()
                                    class:text-gray-500=move || is_instruction_mode.get()
                                    on:click=move |_| is_instruction_mode.set(false)
                                >
                                    <i class="fa-solid fa-sliders text-[10px]"></i>
                                    "传统参数"
                                </button>
                                <button
                                    class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all"
                                    class:bg-white=move || is_instruction_mode.get()
                                    class:shadow-sm=move || is_instruction_mode.get()
                                    class:text-primary=move || is_instruction_mode.get()
                                    class:text-gray-500=move || !is_instruction_mode.get()
                                    on:click=move |_| is_instruction_mode.set(true)
                                >
                                    <i class="fa-solid fa-comment-dots text-[10px]"></i>
                                    "语言控制"
                                </button>
                            </div>
                            <span class="text-[10px] text-gray-400 whitespace-nowrap">"两种参数可同时配置"</span>
                        </div>
                    </Show>
                </Suspense>

                // 分享按钮（推到最右侧）
                <button
                    class="ml-auto shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-green-100 hover:bg-green-200 text-green-700 text-sm font-medium transition-all"
                    title="分享当前声音配置"
                    on:click=move |_| set_show_share.set(true)
                >
                    <i class="fa-solid fa-share-nodes text-xs"></i>
                    "分享"
                </button>
            </div>

            // 内容区域
            <Suspense fallback=move || {
                view! { <div class="h-32 bg-gray-100 rounded-lg animate-pulse"></div> }
            }>
                <div class="space-y-8">
                    {move || {
                        if let Some(ability) = selected_ability() {
                            if ability.instruction_control || ability.parametric_control {
                                let show_instruction = if is_instruction_mode.get() {
                                    ability.instruction_control
                                } else {
                                    !ability.parametric_control
                                };
                                if show_instruction {

                                    view! {
                                        <InstructionParams instruction_text=instruction_text />
                                    }
                                        .into_any()
                                } else {
                                    view! { <TraditionalParams param_signal=selected_param /> }
                                        .into_any()
                                }
                            } else {
                                view! {
                                    <div class="text-center py-8 text-gray-500 bg-gray-50 rounded-xl border border-gray-200">
                                        <i class="fa-solid fa-circle-info text-4xl mb-3 opacity-50"></i>
                                        <p>"此模型不支持参数调节"</p>
                                    </div>
                                }
                                    .into_any()
                            }
                        } else {
                            view! {
                                <div class="text-center py-8 text-gray-500 bg-gray-50 rounded-xl border border-gray-200">
                                    <i class="fa-solid fa-circle-info text-4xl mb-3 opacity-50"></i>
                                    <p>"加载中..."</p>
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </div>
            </Suspense>
        </section>
    }
}

#[component]
pub fn AudioResultCard(
    /// 生成动作 (Action) - 返回音频 URL
    generate_action: Action<(), Result<uuid::Uuid, ServerFnError>>,
    /// 打开分享声音作品弹窗
    set_show_voice_share: WriteSignal<bool>,
    text_signal: RwSignal<String>,
    voice_signal: RwSignal<String>,
) -> impl IntoView {
    // 获取 Action 的状态信号
    let is_pending = generate_action.pending();
    let value = generate_action.value();

    // 绑定 audio 元素和 canvas 元素
    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    // 视觉效果状态
    let is_playing = RwSignal::new(false);
    #[cfg_attr(feature = "ssr", allow(unused_variables))]
    let (current_time, set_current_time) = signal(0.0_f64);
    #[cfg_attr(feature = "ssr", allow(unused_variables))]
    let (duration, set_duration) = signal(0.0_f64);
    let (is_seeking, set_is_seeking) = signal(false);

    // 播放/暂停
    let toggle_play = move |_| {
        if let Some(audio) = audio_ref.get() {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                let audio_el: web_sys::HtmlAudioElement = audio.unchecked_into();
                if is_playing.get() {
                    let _ = audio_el.pause();
                } else {
                    let _ = audio_el.play();
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            let _ = audio;
        }
    };

    // 时间更新
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

    // 加载元数据
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

    // 进度条拖动
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

    // 格式化时间
    let format_time = move |seconds: f64| -> String {
        if seconds.is_nan() || seconds.is_infinite() {
            return "0:00".to_string();
        }
        let mins = (seconds / 60.0).floor() as i32;
        let secs = (seconds % 60.0).floor() as i32;
        format!("{}:{:02}", mins, secs)
    };

    // 可视化逻辑
    let setup_visualizer = move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::closure::Closure;
            use web_sys::{AudioContext, CanvasRenderingContext2d};

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
        <section
            id="audio-result"
            class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover text-dark border border-gray-100 scroll-mt-28"
        >
            <h3 class="text-lg font-semibold mb-4 flex items-center text-dark">
                <i class="fa-solid fa-volume-high text-primary mr-2"></i>
                "输出结果"
            </h3>

            // --- 生成 / 分享按钮 ---
            <div class="flex flex-wrap gap-3 mb-6">
                {move || {
                    let has_result = value.get().is_some_and(|r| r.is_ok());
                    if has_result && !is_pending.get() {
                        // 生成成功后：绿色分享按钮
                        view! {
                            <button
                                class="bg-green-500 hover:bg-green-600 text-white py-3 px-6 rounded-lg font-medium transition-all duration-300 flex items-center justify-center w-full shadow-md hover:shadow-lg active:scale-[0.98]"
                                on:click=move |_| set_show_voice_share.set(true)
                            >
                                <i class="fa-solid fa-share-nodes mr-2"></i>
                                "分享这段声音"
                            </button>
                        }.into_any()
                    } else {
                        // 默认：生成按钮
                        view! {
                            <button
                                id="generate-btn"
                                class="bg-primary hover:bg-primary-focus text-white py-3 px-6 rounded-lg font-medium transition-all duration-300 flex items-center justify-center w-full shadow-md hover:shadow-lg active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
                                on:click=move |_| {
                                    if !voice_signal.get().is_empty() && !text_signal.get().trim().is_empty() {
                                        generate_action.dispatch(());
                                    }
                                }
                                disabled=move || is_pending.get() || voice_signal.get().is_empty() || text_signal.get().trim().is_empty()
                            >
                                {move || {
                                    if is_pending.get() {
                                        view! {
                                            <>
                                                <i class="fa-solid fa-circle-notch fa-spin mr-2"></i>
                                                "正在生成..."
                                            </>
                                        }
                                            .into_any()
                                    } else if voice_signal.get().is_empty() {
                                        view! {
                                            <>
                                                <i class="fa-solid fa-circle-notch fa-spin mr-2"></i>
                                                "正在加载声线..."
                                            </>
                                        }
                                            .into_any()
                                    } else if text_signal.get().trim().is_empty() {
                                        view! {
                                            <>
                                                <i class="fa-solid fa-pen mr-2"></i>
                                                "请先输入文本"
                                            </>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <>
                                                <i class="fa-solid fa-wand-magic-sparkles mr-2"></i>
                                                "生成音频"
                                            </>
                                        }
                                            .into_any()
                                    }
                                }}
                            </button>
                        }.into_any()
                    }
                }}
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
                    (false, Some(Ok(library_id))) => {
                        let audio_url = format!("/api/audio/{}", library_id);
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
                                <div class="w-full p-4 bg-gradient-to-r from-amber-50 to-yellow-50 border-t border-amber-100 rounded-b-xl">
                                    <div class="flex items-center justify-between text-xs text-gray-500 mb-3">
                                        <span class="flex items-center">
                                            <i class="fa-solid fa-check-circle text-green-500 mr-1.5"></i>
                                            "生成完成"
                                        </span>
                                        <button
                                            class="text-primary hover:text-secondary hover:underline flex items-center"
                                            on:click={
                                                #[allow(unused_variables)]
                                                let value = audio_url.clone();
                                                move |_| {
                                                    #[cfg(target_arch = "wasm32")]
                                                    {
                                                        use wasm_bindgen::JsCast;
                                                        let a = web_sys::window()
                                                            .and_then(|w| w.document())
                                                            .and_then(|d| d.create_element("a").ok())
                                                            .and_then(|a| {
                                                                a.dyn_into::<web_sys::HtmlAnchorElement>().ok()
                                                            });
                                                        if let Some(a) = a {
                                                            a.set_href(&value);
                                                            a.set_download("tts_audio.mp3");
                                                            a.click();
                                                        }
                                                    }
                                                }
                                            }
                                        >
                                            <i class="fa-solid fa-download mr-1"></i>
                                            "下载"
                                        </button>
                                    </div>

                                    <div class="flex items-center gap-3">
                                        // 播放/暂停按钮
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
                                            // 进度条
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
                                            // 时间显示
                                            <div class="flex justify-between text-xs text-gray-500">
                                                <span>{move || format_time(current_time.get())}</span>
                                                <span>{move || format_time(duration.get())}</span>
                                            </div>
                                        </div>
                                    </div>

                                    // 隐藏的音频元素
                                    <audio
                                        node_ref=audio_ref
                                        autoplay
                                        class="hidden"
                                        src=audio_url.clone()
                                        on:play=move |_| is_playing.set(true)
                                        on:pause=move |_| is_playing.set(false)
                                        on:timeupdate=on_time_update
                                        on:loadedmetadata=on_loaded_metadata
                                        on:durationchange=on_loaded_metadata
                                        on:ended=move |_| is_playing.set(false)
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
                                <i class="fa-solid fa-triangle-exclamation text-4xl mb-3 opacity-50"></i>
                                <p>"生成失败"</p>
                                <p class="text-sm opacity-70">{e.to_string()}</p>
                            </div>
                        }
                            .into_any()
                    }
                    _ => {
                        view! {
                            <div class="w-full h-full min-h-[300px] flex flex-col items-center justify-center text-center text-gray-400 bg-gray-50 rounded-xl border border-dashed border-gray-200 p-8">
                                <i class="fa-solid fa-headphones text-6xl mb-4 opacity-30"></i>
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
#[allow(dead_code)]
fn request_animation_frame(f: &wasm_bindgen::closure::Closure<dyn FnMut()>) {
    use wasm_bindgen::JsCast;
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn request_animation_frame(_f: &impl std::any::Any) {} // SSR 空实现
