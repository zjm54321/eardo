use crate::api;
use crate::data::{Emotion, VoiceParams};
use leptos::logging::{debug_error, debug_log};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateParams {
    pub text: String,
    pub voice_id: String,
    pub voice_param: VoiceParams,
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
    let voice_signal = RwSignal::new(initial_voice_id);

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
        emotion: initial_emotion,
    });

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

            api::generate_audio(voice_meta, text).await
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
                        // 1. 参数调节 (占位符)
                        <ParameterControlCard selected_param=param_signal />
                        // 2. 输出结果 (核心功能)
                        <AudioResultCard generate_action=generate_action />
                    </div>
                </div>
            </div>
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
            <div class="relative w-full transition-all duration-300 bg-white rounded-lg shadow-sm"
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
                    class:h-full=move || is_fullscreen.get() // 全屏时占满父容器高度

                    // 全屏时字体和行高优化
                    class:text-lg=move || is_fullscreen.get()
                    class:leading-loose=move || is_fullscreen.get()
                    class:p-5=move || is_fullscreen.get() // 全屏时增加内边距

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
                    <i class="fa transition-transform duration-300 group-hover:scale-110 text-sm"
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
    let voices_resource = Resource::new(|| (), |_| api::list_voice_metadata());

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
                                                                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&voice.metadata) {
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
) -> impl IntoView {
    view! {
        <section class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover">
            // 标题
            <h3 class="text-lg font-semibold mb-6 flex items-center">
                <i class="fa fa-sliders text-primary mr-2"></i>
                "参数调节"
            </h3>

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
                        // 将二进制音频数据转换为 Blob URL
                        #[cfg(target_arch = "wasm32")]
                        let audio_url = {
                            use wasm_bindgen::JsCast;
                            let blob = web_sys::Blob::new_with_u8_array_sequence(&wasm_bindgen::JsValue::from(&web_sys::js_sys::Array::of1(
                                &wasm_bindgen::JsValue::from(
                                    js_sys::Uint8Array::from(audio_bytes.as_slice())
                                )
                            )));
                            if let Ok(blob) = blob {
                                web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default()
                            } else {
                                String::new()
                            }
                        };

                        #[cfg(not(target_arch = "wasm32"))]
                        let audio_url = String::new();

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
                                                    if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&wasm_bindgen::JsValue::from(&web_sys::js_sys::Array::of1(
                                                        &wasm_bindgen::JsValue::from(
                                                            js_sys::Uint8Array::from(bytes.as_slice())
                                                        )
                                                    ))) {
                                                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                                                            let a = web_sys::window()
                                                                .and_then(|w| w.document())
                                                                .and_then(|d| d.create_element("a").ok())
                                                                .and_then(|a| a.dyn_into::<web_sys::HtmlAnchorElement>().ok());
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
                                            <i class="fa fa-download mr-1"></i> "下载"
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
                                <p class="text-sm mt-2 opacity-70">"在上方输入文本并点击生成按钮"</p>
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
