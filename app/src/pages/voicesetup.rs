use crate::api;
use crate::api::voice::Parametic;
use crate::pages::component::{InstructionParams, TraditionalParams, ai_rewrite_text};
use crate::pages::share::{ShareVoiceConfigModal, ShareVoicePostModal};
use leptos::logging::debug_error;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

// ─── 主组件 ────────────────────────────────────────────────────

#[component]
pub fn VoiceSetupPage() -> impl IntoView {
    // URL 参数
    let query = use_query_map();
    let get_str_param_opt = |key: &str| query.with_untracked(|q| q.get(key).map(|s| s.to_string()));

    // ── 步骤状态 ──
    let current_step = RwSignal::new(1u8);

    // ── 声线 ──
    let voice_signal = RwSignal::new(String::new());
    let initial_voice_id = RwSignal::new(String::new());
    let voices_resource = Resource::new(|| (), |_| api::voice::list_voice_models());

    // ── 参数 ──
    let param_signal = RwSignal::new(Parametic {
        pitch: 1.0,
        speed: 1.0,
        volume: 1.0,
    });
    let is_instruction_mode = RwSignal::new(false);
    let instruction_text = RwSignal::new(String::new());

    // ── 分享弹窗 ──
    let (show_share, set_show_share) = signal(false);

    // ── 分享声音作品弹窗 ──
    let (show_voice_share, set_show_voice_share) = signal(false);

    // ── 文本 ──
    let text_signal = RwSignal::new(String::new());

    // ── meta_id 预加载 ──
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

    Effect::new(move |_| {
        if let Some(Some(meta)) = meta_resource.get() {
            voice_signal.set(meta.voice_model_id.to_string());
            initial_voice_id.set(meta.voice_model_id.to_string());

            let has_instruction = meta
                .instruction
                .as_ref()
                .map(|instruction| !instruction.trim().is_empty())
                .unwrap_or(false);

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

    // 默认选中第一个声线
    Effect::new(move |_| {
        if voice_signal.get().is_empty() {
            if let Some(Ok(voices)) = voices_resource.get() {
                if let Some(first_voice) = voices.first() {
                    let id = first_voice.id.to_string();
                    initial_voice_id.set(id.clone());
                    voice_signal.set(id);
                }
            }
        }
    });

    // ── 生成 Action ──
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

    // 生成成功自动跳到第四步
    Effect::new(move |_| {
        if let Some(Ok(_)) = generate_action.value().get() {
            current_step.set(4);
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

    // ── 步骤标签 ──
    let steps: Vec<(&str, &str)> = vec![
        ("fa-microphone", "选择声线"),
        ("fa-sliders", "配置参数"),
        ("fa-pen-fancy", "输入文本"),
        ("fa-play", "生成播放"),
    ];

    view! {
        <div class="min-h-screen pb-12">
            <div class="container mx-auto px-4 py-8 max-w-5xl">

                // 右上角切换到专业模式按钮
                <div class="flex justify-end mb-4">
                    <a
                        href="/home"
                        class="inline-flex items-center px-4 py-2 rounded-full bg-gray-100 hover:bg-gray-200 text-gray-600 text-sm font-medium transition-all group"
                    >
                        <i class="fa-solid fa-table-columns mr-2 group-hover:scale-110 transition-transform"></i>
                        "专业模式"
                        <i class="fa-solid fa-arrow-right ml-2 text-xs opacity-60 group-hover:translate-x-0.5 transition-transform"></i>
                    </a>
                </div>

                // ── 进度指示器 ──
                <div class="flex items-center justify-center mb-12">
                    {steps
                        .into_iter()
                        .enumerate()
                        .map(|(i, (icon, label))| {
                            let step_num = (i + 1) as u8;
                            let is_last = i == 3;
                            view! {
                                <div class="flex items-center">
                                    <button
                                        class="flex flex-col items-center group cursor-pointer"
                                        on:click=move |_| {
                                            if step_num <= current_step.get() {
                                                current_step.set(step_num);
                                            }
                                        }
                                    >
                                        <div
                                            class=move || {
                                                let cur = current_step.get();
                                                if cur == step_num {
                                                    "w-12 h-12 rounded-full flex items-center justify-center transition-all duration-300 text-lg bg-primary text-white shadow-lg scale-110"
                                                } else if cur > step_num {
                                                    "w-12 h-12 rounded-full flex items-center justify-center transition-all duration-300 text-lg bg-green-500 text-white"
                                                } else {
                                                    "w-12 h-12 rounded-full flex items-center justify-center transition-all duration-300 text-lg bg-gray-200 text-gray-400"
                                                }
                                            }
                                        >
                                            <i class=format!("fa-solid {}", icon)></i>
                                        </div>
                                        <span class=move || {
                                            let cur = current_step.get();
                                            if cur == step_num {
                                                "mt-2 text-xs font-medium text-primary"
                                            } else if cur > step_num {
                                                "mt-2 text-xs font-medium text-green-600"
                                            } else {
                                                "mt-2 text-xs font-medium text-gray-400"
                                            }
                                        }>
                                            {label}
                                        </span>
                                    </button>
                                    {if !is_last {
                                        view! {
                                            <div class=move || {
                                                if current_step.get() > step_num {
                                                    "w-16 md:w-24 h-0.5 mx-2 mt-[-1rem] bg-green-400 transition-colors duration-300"
                                                } else {
                                                    "w-16 md:w-24 h-0.5 mx-2 mt-[-1rem] bg-gray-200 transition-colors duration-300"
                                                }
                                            }></div>
                                        }
                                            .into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </div>
                            }
                        })
                        .collect_view()}
                </div>

                // ── 步骤内容 ──
                <div class="relative min-h-[500px]">
                    // Step 1: 选择声线
                    <Show when=move || current_step.get() == 1>
                        <StepVoiceSelect
                            voice_signal=voice_signal
                            voices_resource=voices_resource
                            on_next=move || current_step.set(2)
                        />
                    </Show>

                    // Step 2: 配置参数
                    <Show when=move || current_step.get() == 2>
                        <StepParamConfig
                            param_signal=param_signal
                            voice_signal=voice_signal
                            is_instruction_mode=is_instruction_mode
                            instruction_text=instruction_text
                            voices_resource=voices_resource
                            set_show_share=set_show_share
                            on_back=move || current_step.set(1)
                            on_next=move || current_step.set(3)
                        />
                    </Show>

                    // Step 3: 输入文本
                    <Show when=move || current_step.get() == 3>
                        <StepTextInput
                            text_signal=text_signal
                            generate_action=generate_action
                            on_back=move || current_step.set(2)
                        />
                    </Show>

                    // Step 4: 生成 & 可视化
                    <Show when=move || current_step.get() == 4>
                        <StepGenerate
                            generate_action=generate_action
                            param_signal=param_signal
                            voice_signal=voice_signal
                            voices_resource=voices_resource
                            is_instruction_mode=is_instruction_mode
                            instruction_text=instruction_text
                            set_show_voice_share=set_show_voice_share
                            set_show_share=set_show_share
                        />
                    </Show>
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

// ═══════════════════════════════════════════════════════════════
// Step 1 · 选择声线（卡片式）
// ═══════════════════════════════════════════════════════════════

#[component]
fn StepVoiceSelect(
    voice_signal: RwSignal<String>,
    voices_resource: Resource<Result<Vec<api::voice::VoiceModel>, ServerFnError>>,
    on_next: impl Fn() + 'static + Clone + Send,
) -> impl IntoView {
    view! {
        <div class="animate-fade-in">
            <div class="text-center mb-8">
                <h2 class="text-2xl font-bold text-dark mb-2">"选择一个声线"</h2>
                <p class="text-gray-500">"每种声线都有独特的音色风格，选择最适合你的那一个"</p>
            </div>

            <Suspense fallback=move || {
                view! {
                    <div class="flex justify-center py-16">
                        <div class="w-10 h-10 border-4 border-primary/30 border-t-primary rounded-full animate-spin"></div>
                    </div>
                }
            }>
                {move || {
                    match voices_resource.get() {
                        Some(Ok(voices)) => {
                            let on_next = on_next.clone();
                            view! {
                                <div class="max-h-[60vh] overflow-y-auto pr-1 scrollbar-thin">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
                                    <For
                                        each=move || voices.clone()
                                        key=|v| v.id.to_string()
                                        children=move |voice| {
                                            let vid = voice.id.to_string();
                                            let stored_id = StoredValue::new(vid);
                                            let avatar_url = voice.avatar_url.clone();
                                            let name = voice.info.name.clone();
                                            let desc = voice.info.description.clone();
                                            view! {
                                                <div
                                                    class=move || {
                                                        let base = "flex items-stretch bg-white rounded-xl border-2 cursor-pointer transition-all duration-200 overflow-hidden group hover:shadow-lg";
                                                        if voice_signal.get() == stored_id.get_value() {
                                                            format!("{} border-primary shadow-md ring-2 ring-primary/30", base)
                                                        } else {
                                                            format!("{} border-gray-200", base)
                                                        }
                                                    }
                                                    on:click=move |_| {
                                                        voice_signal.set(stored_id.get_value())
                                                    }
                                                >
                                                    // 左侧图片
                                                    <div class="w-24 h-24 md:w-28 md:h-28 flex-shrink-0 bg-gray-100 flex items-center justify-center overflow-hidden">
                                                        <img
                                                            src=avatar_url.clone()
                                                            alt=name.clone()
                                                            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                                                        />
                                                    </div>
                                                    // 右侧内容
                                                    <div class="flex-1 p-4 flex flex-col justify-between min-w-0">
                                                        <div>
                                                            <h4 class="font-bold text-gray-800 text-lg mb-1 truncate group-hover:text-primary transition-colors">
                                                                {name.clone()}
                                                            </h4>
                                                            <p class="text-sm text-gray-500 line-clamp-2">
                                                                {if desc.is_empty() {
                                                                    "暂无描述".to_string()
                                                                } else {
                                                                    desc.clone()
                                                                }}
                                                            </p>
                                                        </div>
                                                        // 选择按钮
                                                        <div class="flex justify-end mt-2">
                                                            <div
                                                                class="text-xs px-3 py-1 rounded-full transition-all"
                                                                class:bg-primary=move || {
                                                                    voice_signal.get() == stored_id.get_value()
                                                                }
                                                                class:text-white=move || {
                                                                    voice_signal.get() == stored_id.get_value()
                                                                }
                                                                class:bg-gray-100=move || {
                                                                    voice_signal.get() != stored_id.get_value()
                                                                }
                                                                class:text-gray-500=move || {
                                                                    voice_signal.get() != stored_id.get_value()
                                                                }
                                                            >
                                                                {move || {
                                                                    if voice_signal.get() == stored_id.get_value() {
                                                                        "已选择"
                                                                    } else {
                                                                        "选择"
                                                                    }
                                                                }}
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                                </div>

                                // 下一步按钮
                                <div class="flex justify-center mt-6">
                                    <button
                                        class="px-10 py-3 bg-primary hover:bg-primary/90 text-white rounded-xl font-bold text-lg shadow-lg hover:shadow-xl transition-all active:scale-[0.98] disabled:opacity-50"
                                        on:click=move |_| on_next()
                                        disabled=move || voice_signal.get().is_empty()
                                    >
                                        "下一步"
                                        <i class="fa-solid fa-arrow-right ml-2"></i>
                                    </button>
                                </div>
                            }
                                .into_any()
                        }
                        Some(Err(e)) => {
                            debug_error!("加载声线库失败: {:?}", e);
                            view! {
                                <div class="text-red-500 text-center py-8 bg-red-50 rounded-xl border border-red-200">
                                    <i class="fa-solid fa-circle-exclamation mr-2"></i>
                                    "加载失败，请刷新重试"
                                </div>
                            }
                                .into_any()
                        }
                        None => {
                            view! {
                                <div class="flex justify-center py-16">
                                    <div class="w-10 h-10 border-4 border-primary/30 border-t-primary rounded-full animate-spin"></div>
                                </div>
                            }
                                .into_any()
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════
// Step 2 · 配置参数
// ═══════════════════════════════════════════════════════════════

#[component]
fn StepParamConfig(
    param_signal: RwSignal<Parametic>,
    voice_signal: RwSignal<String>,
    is_instruction_mode: RwSignal<bool>,
    instruction_text: RwSignal<String>,
    voices_resource: Resource<Result<Vec<api::voice::VoiceModel>, ServerFnError>>,
    set_show_share: WriteSignal<bool>,
    on_back: impl Fn() + 'static + Clone + Send,
    on_next: impl Fn() + 'static + Clone + Send,
) -> impl IntoView {
    let selected_ability = move || {
        let voice_id = voice_signal.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == voice_id) {
                return Some(voice.ability.clone());
            }
        }
        None
    };

    let selected_voice_name = move || {
        let voice_id = voice_signal.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == voice_id) {
                return voice.info.name.clone();
            }
        }
        "未知声线".to_string()
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
        <div class="animate-fade-in">
            <div class="text-center mb-8">
                <h2 class="text-2xl font-bold text-dark mb-2">"配置声音参数"</h2>
                <p class="text-gray-500">
                    "当前声线："
                    <span class="text-primary font-semibold">{selected_voice_name}</span>
                </p>
            </div>

            <div class="max-w-2xl mx-auto">
                <div class="bg-white rounded-2xl p-8 shadow-soft border border-gray-100">
                    // 模式切换标签
                    <Suspense fallback=move || {
                        view! {
                            <div class="h-10 bg-gray-100 rounded-lg animate-pulse"></div>
                        }
                    }>
                        <Show when=move || {
                            selected_ability()
                                .map(|a| a.instruction_control && a.parametric_control)
                                .unwrap_or(false)
                        }>
                            <div class="mb-8">
                                <div class="flex bg-gray-100 rounded-lg p-1 gap-1">
                                    <button
                                        class="flex-1 py-2.5 px-4 rounded-md text-sm font-medium transition-all flex items-center justify-center gap-2"
                                        class:bg-white=move || !is_instruction_mode.get()
                                        class:shadow-sm=move || !is_instruction_mode.get()
                                        class:text-primary=move || !is_instruction_mode.get()
                                        class:text-gray-500=move || is_instruction_mode.get()
                                        on:click=move |_| is_instruction_mode.set(false)
                                    >
                                        <i class="fa-solid fa-sliders"></i>
                                        "传统参数"
                                    </button>
                                    <button
                                        class="flex-1 py-2.5 px-4 rounded-md text-sm font-medium transition-all flex items-center justify-center gap-2"
                                        class:bg-white=move || is_instruction_mode.get()
                                        class:shadow-sm=move || is_instruction_mode.get()
                                        class:text-primary=move || is_instruction_mode.get()
                                        class:text-gray-500=move || !is_instruction_mode.get()
                                        on:click=move |_| is_instruction_mode.set(true)
                                    >
                                        <i class="fa-solid fa-comment-dots"></i>
                                        "语言控制"
                                    </button>
                                </div>
                                <p class="text-center text-xs text-gray-400 mt-2">"两种参数可同时配置，均会生效"</p>
                            </div>
                        </Show>
                    </Suspense>

                    // 参数内容
                    <Suspense fallback=move || {
                        view! {
                            <div class="h-32 bg-gray-100 rounded-lg animate-pulse"></div>
                        }
                    }>
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
                                        view! {
                                            <TraditionalParams param_signal=param_signal />
                                        }
                                            .into_any()
                                    }
                                } else {
                                    view! {
                                        <div class="text-center py-12 text-gray-400">
                                            <i class="fa-solid fa-circle-info text-4xl mb-3"></i>
                                            <p>"此模型不支持参数调节，将使用默认配置"</p>
                                        </div>
                                    }
                                        .into_any()
                                }
                            } else {
                                view! {
                                    <div class="text-center py-12 text-gray-400">
                                        <div class="w-8 h-8 border-4 border-primary/30 border-t-primary rounded-full animate-spin mx-auto mb-3"></div>
                                        <p>"加载中..."</p>
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                    </Suspense>
                </div>

                // 分享按钮
                <button
                    class="w-full mt-6 py-3.5 bg-green-500 hover:bg-green-600 text-white rounded-xl font-bold text-base shadow-md hover:shadow-lg transition-all active:scale-[0.98] flex items-center justify-center gap-2"
                    on:click=move |_| set_show_share.set(true)
                >
                    <i class="fa-solid fa-share-nodes text-lg"></i>
                    "分享此声音配置"
                </button>

                // 导航按钮
                <div class="flex justify-between mt-8">
                    <button
                        class="px-8 py-3 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-xl font-medium transition-all"
                        on:click=move |_| on_back()
                    >
                        <i class="fa-solid fa-arrow-left mr-2"></i>
                        "上一步"
                    </button>
                    <button
                        class="px-10 py-3 bg-primary hover:bg-primary/90 text-white rounded-xl font-bold text-lg shadow-lg hover:shadow-xl transition-all active:scale-[0.98]"
                        on:click=move |_| on_next()
                    >
                        "下一步"
                        <i class="fa-solid fa-arrow-right ml-2"></i>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════
// Step 3 · 输入文本 + 表达助手浮窗
// ═══════════════════════════════════════════════════════════════

#[component]
fn StepTextInput(
    text_signal: RwSignal<String>,
    generate_action: Action<(), Result<uuid::Uuid, ServerFnError>>,
    on_back: impl Fn() + 'static + Clone + Send,
) -> impl IntoView {
    let show_assistant = RwSignal::new(false);
    let scene = RwSignal::new("汇报".to_string());
    let audience = RwSignal::new("老师".to_string());
    let duration = RwSignal::new("3".to_string());
    let output_text = RwSignal::new(String::new());
    let active_tab = RwSignal::new("assistant".to_string());
    let status_text = RwSignal::new(String::new());

    let ai_action = Action::new(move |action: &String| {
        let action = action.clone();
        let input = text_signal.get();
        let scene = scene.get();
        let audience = audience.get();
        let duration = duration.get();
        async move {
            let prompt = format!(
                "你是表达助手。请严格遵守：\n\
                1) 不生成有害内容；\n\
                2) 输出应清晰、可编辑、面向口播；\n\
                3) 只输出结果内容，不要附加解释。\n\n\
                任务：{action}\n\
                场景：{scene}\n受众：{audience}\n时长：{duration}分钟\n\n\
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
        }
    });

    let is_pending = generate_action.pending();

    view! {
        <div class="animate-fade-in relative">
            <div class="text-center mb-6">
                <h2 class="text-2xl font-bold text-dark mb-2">"输入你的文本"</h2>
                <p class="text-gray-500">"输入想要转换的文字内容"</p>
            </div>

            <div class="relative max-w-3xl mx-auto">
                // 全屏文本输入框
                <div class="relative">
                    <textarea
                        class="w-full min-h-[400px] p-6 text-lg leading-relaxed border-2 border-gray-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all resize-none bg-white shadow-sm"
                        placeholder="请输入你想转换的文字...\n\n例如：\n你好，欢迎来到耳朵，这是一个参数化声音创作平台。\n在这里你可以选择不同的声线，调整参数，生成属于你的声音。"
                        prop:value=move || text_signal.get()
                        on:input=move |ev| text_signal.set(event_target_value(&ev))
                    ></textarea>

                    // 字数统计
                    <div class="absolute bottom-4 left-6 text-sm text-gray-400">
                        {move || format!("{} 字", text_signal.get().chars().count())}
                    </div>

                    // 表达助手浮窗按钮
                    <button
                        class="absolute top-4 right-4 w-10 h-10 rounded-full bg-white/80 hover:bg-primary/10 backdrop-blur-sm shadow-md hover:shadow-lg text-primary transition-all flex items-center justify-center group"
                        on:click=move |_| show_assistant.set(!show_assistant.get())
                        title="表达助手"
                    >
                        <i class="fa-solid fa-wand-magic-sparkles text-lg group-hover:scale-110 transition-transform"></i>
                    </button>
                </div>

                // 表达助手浮窗
                <Show when=move || show_assistant.get()>
                    <div class="absolute top-16 right-0 w-80 bg-white rounded-xl shadow-2xl border border-gray-200 z-20 overflow-hidden">
                        <div class="flex items-center justify-between p-4 bg-primary/5 border-b border-gray-100">
                            <h4 class="font-bold text-gray-800 flex items-center">
                                <i class="fa-solid fa-wand-magic-sparkles text-primary mr-2"></i>
                                "表达助手"
                            </h4>
                            <button
                                class="text-gray-400 hover:text-gray-600"
                                on:click=move |_| show_assistant.set(false)
                            >
                                <i class="fa-solid fa-xmark"></i>
                            </button>
                        </div>

                        <div class="p-4 space-y-3">
                            <div class="grid grid-cols-3 gap-2">
                                <div>
                                    <label class="block text-xs text-gray-500 mb-1">"场景"</label>
                                    <select
                                        class="w-full px-2 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50"
                                        on:change=move |ev| scene.set(event_target_value(&ev))
                                    >
                                        <option>"汇报"</option>
                                        <option>"科普"</option>
                                        <option>"答辩"</option>
                                        <option>"其他"</option>
                                    </select>
                                </div>
                                <div>
                                    <label class="block text-xs text-gray-500 mb-1">"受众"</label>
                                    <select
                                        class="w-full px-2 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50"
                                        on:change=move |ev| audience.set(event_target_value(&ev))
                                    >
                                        <option>"老师"</option>
                                        <option>"同学"</option>
                                        <option>"公众"</option>
                                    </select>
                                </div>
                                <div>
                                    <label class="block text-xs text-gray-500 mb-1">"时长"</label>
                                    <select
                                        class="w-full px-2 py-1.5 border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50"
                                        on:change=move |ev| duration.set(event_target_value(&ev))
                                    >
                                        <option value="1">"1分钟"</option>
                                        <option value="3" selected>"3分钟"</option>
                                        <option value="5">"5分钟"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="flex flex-wrap gap-2">
                                <button
                                    class="px-3 py-1.5 bg-primary hover:bg-primary/90 text-white rounded-lg text-xs font-medium disabled:opacity-50"
                                    on:click=move |_| {
                                        status_text.set("生成中...".to_string());
                                        ai_action.dispatch("生成提纲".to_string());
                                    }
                                    disabled=move || ai_action.pending().get()
                                >
                                    "生成提纲"
                                </button>
                                <button
                                    class="px-3 py-1.5 bg-primary hover:bg-primary/90 text-white rounded-lg text-xs font-medium disabled:opacity-50"
                                    on:click=move |_| {
                                        status_text.set("生成中...".to_string());
                                        ai_action.dispatch("口播改写".to_string());
                                    }
                                    disabled=move || ai_action.pending().get()
                                >
                                    "口播改写"
                                </button>
                                <button
                                    class="px-3 py-1.5 bg-primary hover:bg-primary/90 text-white rounded-lg text-xs font-medium disabled:opacity-50"
                                    on:click=move |_| {
                                        status_text.set("生成中...".to_string());
                                        ai_action.dispatch("术语轻解释".to_string());
                                    }
                                    disabled=move || ai_action.pending().get()
                                >
                                    "术语轻解释"
                                </button>
                            </div>

                            <div class="border rounded-lg">
                                <div class="flex border-b text-xs">
                                    <button
                                        class=move || {
                                            if active_tab.get() == "assistant" {
                                                "flex-1 py-2 font-medium text-primary bg-primary/5"
                                            } else {
                                                "flex-1 py-2 font-medium"
                                            }
                                        }
                                        on:click=move |_| {
                                            active_tab.set("assistant".to_string())
                                        }
                                    >
                                        "助手文本"
                                    </button>
                                    <button
                                        class=move || {
                                            if active_tab.get() == "original" {
                                                "flex-1 py-2 font-medium border-l text-primary bg-primary/5"
                                            } else {
                                                "flex-1 py-2 font-medium border-l"
                                            }
                                        }
                                        on:click=move |_| {
                                            active_tab.set("original".to_string())
                                        }
                                    >
                                        "原始文本"
                                    </button>
                                </div>
                                <textarea
                                    class="w-full min-h-[100px] resize-y border-0 rounded-b-lg p-3 text-sm focus:outline-none"
                                    prop:value=move || {
                                        if active_tab.get() == "assistant" {
                                            output_text.get()
                                        } else {
                                            text_signal.get()
                                        }
                                    }
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        if active_tab.get() == "assistant" {
                                            output_text.set(v);
                                        } else {
                                            text_signal.set(v);
                                        }
                                    }
                                ></textarea>
                            </div>

                            <div class="flex gap-2">
                                <button
                                    class="flex-1 px-3 py-1.5 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-lg text-xs font-medium"
                                    on:click=move |_| {
                                        text_signal.set(output_text.get());
                                        show_assistant.set(false);
                                    }
                                >
                                    "一键填入"
                                </button>
                                <button
                                    class="flex-1 px-3 py-1.5 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-lg text-xs font-medium"
                                    on:click=move |_| {
                                        let mut v = text_signal.get();
                                        if !v.trim().is_empty()
                                            && !output_text.get().trim().is_empty()
                                        {
                                            v.push_str("\n\n");
                                        }
                                        v.push_str(&output_text.get());
                                        text_signal.set(v);
                                    }
                                >
                                    "追加"
                                </button>
                            </div>

                            <Show when=move || !status_text.get().is_empty()>
                                <p
                                    class="text-xs"
                                    class:text-primary=move || status_text.get() == "生成成功"
                                    class:text-red-500=move || {
                                        status_text.get().starts_with("生成失败")
                                    }
                                    class:text-gray-500=move || status_text.get() == "生成中..."
                                >
                                    {move || status_text.get()}
                                </p>
                            </Show>
                        </div>
                    </div>
                </Show>

                // 导航：上一步 + 生成按钮
                <div class="flex justify-between mt-8">
                    <button
                        class="px-8 py-3 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-xl font-medium transition-all"
                        on:click=move |_| on_back()
                    >
                        <i class="fa-solid fa-arrow-left mr-2"></i>
                        "上一步"
                    </button>
                    <button
                        class="px-12 py-4 bg-gradient-to-r from-primary to-amber-400 hover:from-primary/90 hover:to-amber-400/90 text-white rounded-2xl font-black text-xl shadow-xl hover:shadow-2xl transition-all active:scale-[0.97] disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                        on:click=move |_| { generate_action.dispatch(()); }
                        disabled=move || {
                            text_signal.get().trim().is_empty() || is_pending.get()
                        }
                    >
                        {move || {
                            if is_pending.get() {
                                view! {
                                    <>
                                        <i class="fa-solid fa-circle-notch fa-spin mr-3"></i>
                                        "生成中..."
                                    </>
                                }
                                    .into_view()
                            } else {
                                view! {
                                    <>
                                        <i class="fa-solid fa-bolt mr-3"></i>
                                        "生成声音"
                                    </>
                                }
                                    .into_view()
                            }
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════
// Step 4 · 生成 & 可视化
// ═══════════════════════════════════════════════════════════════

#[component]
fn StepGenerate(
    generate_action: Action<(), Result<uuid::Uuid, ServerFnError>>,
    param_signal: RwSignal<Parametic>,
    voice_signal: RwSignal<String>,
    voices_resource: Resource<Result<Vec<api::voice::VoiceModel>, ServerFnError>>,
    is_instruction_mode: RwSignal<bool>,
    instruction_text: RwSignal<String>,
    set_show_voice_share: WriteSignal<bool>,
    set_show_share: WriteSignal<bool>,
) -> impl IntoView {
    let _ = is_instruction_mode;
    let value = generate_action.value();
    let is_pending = generate_action.pending();
    let is_playing = RwSignal::new(false);
    #[cfg_attr(feature = "ssr", allow(unused_variables))]
    let (current_time, set_current_time) = signal(0.0_f64);
    #[cfg_attr(feature = "ssr", allow(unused_variables))]
    let (duration, set_duration) = signal(0.0_f64);
    let (is_seeking, set_is_seeking) = signal(false);

    let audio_ref = NodeRef::<leptos::html::Audio>::new();
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

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

    let selected_voice_name = move || {
        let voice_id = voice_signal.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == voice_id) {
                return voice.info.name.clone();
            }
        }
        "未知".to_string()
    };

    let selected_ability = move || {
        let voice_id = voice_signal.get();
        if let Some(Ok(voices)) = voices_resource.get() {
            if let Some(voice) = voices.iter().find(|v| v.id.to_string() == voice_id) {
                return Some(voice.ability.clone());
            }
        }
        None
    };

    let setup_visualizer = move || {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::closure::Closure;
            use web_sys::{AudioContext, CanvasRenderingContext2d};

            let audio_el = audio_ref.get();
            let canvas_el = canvas_ref.get();

            if let (Some(audio), Some(canvas)) = (audio_el, canvas_el) {
                use wasm_bindgen::JsCast;
                let audio: web_sys::HtmlAudioElement = audio.unchecked_into();
                let canvas: web_sys::HtmlCanvasElement = canvas.unchecked_into();

                let parent = canvas.parent_element().unwrap();
                let width = parent.client_width() as u32;
                let height = 300;
                canvas.set_width(width);
                canvas.set_height(height);

                audio.set_cross_origin(Some("anonymous"));

                let ctx =
                    AudioContext::new().unwrap_or_else(|_| panic!("Failed to create AudioContext"));
                let analyser = ctx.create_analyser().unwrap();
                analyser.set_fft_size(256);

                let source = match ctx.create_media_element_source(&audio) {
                    Ok(src) => src,
                    Err(_) => return,
                };

                source.connect_with_audio_node(&analyser).unwrap();
                analyser
                    .connect_with_audio_node(&ctx.destination())
                    .unwrap();

                let buffer_length = analyser.frequency_bin_count();
                let mut data_array = vec![0u8; buffer_length as usize];

                let ctx_2d: CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                let f = std::rc::Rc::new(std::cell::RefCell::new(None));
                let g = f.clone();

                let canvas_width = width as f64;
                let canvas_height = height as f64;
                let center_x = canvas_width / 2.0;
                let center_y = canvas_height / 2.0;
                let radius = 60.0;

                *g.borrow_mut() = Some(Closure::new(move || {
                    analyser.get_byte_frequency_data(&mut data_array);
                    ctx_2d.clear_rect(0.0, 0.0, canvas_width, canvas_height);

                    ctx_2d.begin_path();
                    ctx_2d
                        .arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI)
                        .unwrap();
                    ctx_2d.set_stroke_style_str("rgba(75, 85, 99, 0.15)");
                    ctx_2d.set_line_width(2.0);
                    ctx_2d.stroke();

                    let bars = buffer_length;
                    ctx_2d.set_fill_style_str("#FBBF24");

                    for i in 0..bars {
                        let value = data_array[i as usize] as f64;
                        let bar_height = (value / 255.0) * 80.0;
                        let rad = (i as f64 / bars as f64) * 2.0 * std::f64::consts::PI;

                        ctx_2d.save();
                        ctx_2d.translate(center_x, center_y).unwrap();
                        ctx_2d.rotate(rad).unwrap();

                        if bar_height > 0.0 {
                            ctx_2d.fill_rect(-1.5, radius, 3.0, bar_height);
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
                move || setup_visualizer(),
                std::time::Duration::from_millis(100),
            );
        }
    });

    view! {
        <div class="animate-fade-in">
            <div class="text-center mb-8">
                <h2 class="text-2xl font-bold text-dark mb-2">"生成结果"</h2>
                <p class="text-gray-500">"试听你的声音作品"</p>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // ── 左栏：配置面板 + 分享配置按钮 ──
                <div class="lg:col-span-1 space-y-4">
                    <div class="bg-white rounded-2xl p-6 shadow-soft border border-gray-100 space-y-4">
                        <h4 class="font-bold text-gray-800 flex items-center">
                            <i class="fa-solid fa-info-circle text-primary mr-2"></i>
                            "当前配置"
                        </h4>

                        // 模型名称
                        <div class="flex items-center gap-2 text-sm text-gray-700">
                            <i class="fa-solid fa-microphone text-primary text-xs"></i>
                            <span class="font-medium">"模型："</span>
                            <span>{selected_voice_name}</span>
                        </div>

                        // 指令（模型支持且非空时显示）
                        {move || {
                            let ability = selected_ability();
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
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}

                        // 参数药丸（模型支持参数时显示）
                        {move || {
                            let ability = selected_ability();
                            let supports_parametric = ability.as_ref().map_or(false, |a| a.parametric_control);
                            if supports_parametric {
                                let p = param_signal.get();
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

                    // 分享声音配置按钮
                    <button
                        class="w-full py-3 bg-primary/10 hover:bg-primary/20 text-primary rounded-xl font-medium text-sm transition-all active:scale-[0.98] flex items-center justify-center gap-2"
                        on:click=move |_| set_show_share.set(true)
                    >
                        <i class="fa-solid fa-share-nodes"></i>
                        "分享声音配置"
                    </button>
                </div>

                // ── 右栏：可视化 + 播放 + 分享按钮 ──
                <div class="lg:col-span-2 space-y-4">
                    <div class="bg-white rounded-2xl shadow-soft border border-gray-100 overflow-hidden">
                        {move || {
                            match (is_pending.get(), value.get()) {
                                (true, _) => {
                                    view! {
                                        <div class="flex flex-col items-center justify-center py-20">
                                            <div class="w-16 h-16 border-4 border-primary/30 border-t-primary rounded-full animate-spin mb-6"></div>
                                            <p class="text-gray-500 text-lg">
                                                "AI 正在合成您的声音..."
                                            </p>
                                        </div>
                                    }
                                        .into_any()
                                }
                                (false, Some(Ok(library_id))) => {
                                    let audio_url = format!("/api/audio/{}", library_id);
                                    view! {
                                        <div class="flex flex-col">
                                            // 可视化区域
                                            <div class="w-full h-[300px] bg-gradient-to-b from-gray-50 to-white flex items-center justify-center overflow-hidden">
                                                <canvas
                                                    node_ref=canvas_ref
                                                    class="z-10"
                                                    width="600"
                                                    height="300"
                                                ></canvas>
                                            </div>

                                            // 播放器控制栏
                                            <div class="p-6 bg-gradient-to-r from-amber-50 to-yellow-50 border-t border-amber-100">
                                                <div class="flex items-center justify-between text-xs text-gray-500 mb-3">
                                                    <span class="flex items-center">
                                                        <i class="fa-solid fa-check-circle text-green-500 mr-1.5"></i>
                                                        "生成完成"
                                                    </span>
                                                    <button
                                                        class="text-primary hover:text-primary/80 flex items-center hover:underline"
                                                        on:click={
                                                            #[allow(unused_variables)]
                                                            let url = audio_url.clone();
                                                            move |_| {
                                                                #[cfg(target_arch = "wasm32")]
                                                                {
                                                                    use wasm_bindgen::JsCast;
                                                                    let a = web_sys::window()
                                                                        .and_then(|w| w.document())
                                                                        .and_then(|d| d.create_element("a").ok())
                                                                        .and_then(|a| {
                                                                            a
                                                                                .dyn_into::<web_sys::HtmlAnchorElement>()
                                                                                .ok()
                                                                        });
                                                                    if let Some(a) = a {
                                                                        a.set_href(&url);
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
                                    debug_error!("生成失败: {:?}", e);
                                    view! {
                                        <div class="text-center py-16 text-red-500">
                                            <i class="fa-solid fa-triangle-exclamation text-5xl mb-4 opacity-50"></i>
                                            <p class="text-lg font-medium">"生成失败"</p>
                                            <p class="text-sm opacity-70 mt-2">{e.to_string()}</p>
                                        </div>
                                    }
                                        .into_any()
                                }
                                _ => {
                                    view! {
                                        <div class="flex flex-col items-center justify-center py-20 text-gray-400">
                                            <i class="fa-solid fa-headphones text-6xl mb-4 opacity-30"></i>
                                            <p class="text-lg">"等待生成..."</p>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }
                        }}
                    </div>

                    // 分享声音作品按钮（与可视化同宽）
                    <Show when=move || generate_action.value().get().is_some_and(|r| r.is_ok())>
                        <button
                            class="w-full py-3.5 bg-green-500 hover:bg-green-600 text-white rounded-xl font-bold text-base shadow-md hover:shadow-lg transition-all active:scale-[0.98] flex items-center justify-center gap-2"
                            on:click=move |_| set_show_voice_share.set(true)
                        >
                            <i class="fa-solid fa-share-nodes text-lg"></i>
                            "分享这段声音"
                        </button>
                    </Show>
                </div>
            </div>

        </div>
    }
}

// 辅助函数
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
fn request_animation_frame(_f: &impl std::any::Any) {}
