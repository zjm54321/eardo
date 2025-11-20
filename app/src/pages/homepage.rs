use crate::api;
use leptos::logging::{debug_error, debug_log};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Emotion {
    Normal,
    Angry,
    Calm,
    Excited,
    Happy,
    Peaceful,
    Sad,
    Suprised,
}

impl fmt::Display for Emotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Emotion::Normal => "正常",
            Emotion::Angry => "生气",
            Emotion::Calm => "冷静",
            Emotion::Excited => "激动",
            Emotion::Happy => "开心",
            Emotion::Peaceful => "平静",
            Emotion::Sad => "悲伤",
            Emotion::Suprised => "惊讶",
        };
        write!(f, "{}", s)
    }
}

impl Emotion {
    pub fn all() -> Vec<Emotion> {
        vec![
            Emotion::Normal,
            Emotion::Angry,
            Emotion::Calm,
            Emotion::Excited,
            Emotion::Happy,
            Emotion::Peaceful,
            Emotion::Sad,
            Emotion::Suprised,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceParams {
    pub pitch: f32,
    pub speed: f32,
    pub emotion: Emotion,
}

impl Default for VoiceParams {
    fn default() -> Self {
        VoiceParams {
            pitch: 0.0,
            speed: 1.0,
            emotion: Emotion::Normal,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateParams {
    pub text: String,
    pub voice_id: String,
    pub voice_param: VoiceParams,
}

#[component]
pub fn HomePage() -> impl IntoView {
    // 状态
    let text_signal = RwSignal::new(String::new());
    let voice_signal = RwSignal::new(String::new());
    let param_signal = RwSignal::new(VoiceParams::default());

    // 创建 Action 处理生成请求
    // Action 自动管理 pending (加载中) 和 value (返回值) 状态
    let generate_action = Action::new(move |_| {
        let voice_params = GenerateParams {
            text: text_signal.get(),
            voice_id: voice_signal.get(),
            voice_param: param_signal.get(),
        };
        debug_log!("使用参数生成音频: {:?}", voice_params);
        return api::generate_audio(voice_params);
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
    view! {
        // 卡片容器：白色背景、圆角、阴影
        <section class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover">
            // 标题区域
            <h3 class="text-lg font-semibold mb-4 flex items-center">
                <i class="fa fa-comment text-primary mr-2"></i>
                "文本输入"
            </h3>
            // 输入区域
            <textarea
                id="text-input"
                // Tailwind 样式复刻原版设计
                class="w-full p-4 border border-gray-200 rounded-lg \
                focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary \
                transition-all duration-300 resize-none h-32 font-sans text-gray-700 placeholder-gray-400"
                placeholder="请输入你想转换的文字...\n例如：你好，欢迎使用白昼聆夏"
                // --- 核心逻辑：绑定信号 ---
                // 1. 当信号改变时，更新 textarea 的值
                prop:value=move || text.get()
                // 2. 当用户输入时，更新信号的值
                on:input=move |ev| text.set(event_target_value(&ev))
            ></textarea>
            // 底部提示
            <p class="text-xs text-gray-500 mt-2">
                "输入文本将通过后端 TTS 转换为音频"
            </p>
        </section>
    }
}

#[component]
pub fn VoiceSelectorCard(
    /// 当前选中的声线 ID (双向绑定)
    selected_voice: RwSignal<String>,
) -> impl IntoView {
    // Resource 用于异步获取数据
    let voices_resource = Resource::new(|| (), |_| api::get_voices());

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
                                                            {voice.name}
                                                        </div>
                                                        <div class="text-sm text-gray-500">{voice.desc}</div>
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
                        <span class="text-xs text-gray-400 absolute left-0 -bottom-5">"-2.0"</span>
                        <input
                            type="range"
                            min="-2.0"
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
    /// 生成动作 (Action)
    generate_action: Action<(), Result<String, ServerFnError>>,
) -> impl IntoView {
    // 获取 Action 的状态信号
    let is_pending = generate_action.pending();
    let value = generate_action.value();

    view! {
        <section class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover">
            <h3 class="text-lg font-semibold mb-4 flex items-center">
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

            // --- 状态展示区域 (使用 match 替代 if-else) ---
            <div>
                {move || match (is_pending.get(), value.get()) {
                    (true, _) => {
                        // 1. 正在加载
                        view! {
                            <div class="flex flex-col items-center justify-center py-8 animate-fade-in">
                                <div class="w-12 h-12 border-4 border-primary/30 border-t-primary rounded-full animate-spin mb-4"></div>
                                <p class="text-gray-500">"AI 正在合成您的声音..."</p>
                            </div>
                        }
                            .into_any()
                    }
                    (false, Some(Ok(url))) => {

                        // 2. 加载完成，成功获取 URL
                        view! {
                            <div class="border border-green-200 bg-green-50 rounded-xl p-6 animate-slide-up">
                                <div class="flex items-center mb-4">
                                    <div class="bg-green-100 p-2 rounded-full mr-3">
                                        <i class="fa fa-check text-green-600"></i>
                                    </div>
                                    <h4 class="font-semibold text-green-800">"生成完成！"</h4>
                                </div>
                                <div class="mb-4">
                                    <p class="text-sm text-gray-600 mb-2">
                                        "处理后的音频："
                                    </p>
                                    <div class="flex flex-col gap-3">
                                        <audio
                                            controls
                                            autoplay
                                            class="w-full"
                                            src=url.clone()
                                        ></audio>
                                        <a
                                            href=url
                                            download="tts_audio.mp3"
                                            target="_blank"
                                            class="bg-white border border-green-200 text-green-700 hover:bg-green-100 px-4 py-2 rounded-lg text-sm flex items-center justify-center transition-colors"
                                        >
                                            <i class="fa fa-download mr-2"></i>
                                            "下载音频"
                                        </a>
                                    </div>
                                </div>
                            </div>
                        }
                            .into_any()
                    }
                    (false, Some(Err(e))) => {
                        debug_error!("生成音频失败: {:?}", e);
                        // 3. 失败 (可选处理)
                        view! {
                            <div class="text-center py-8 text-red-500 bg-red-50 rounded-xl border border-red-200">
                                <i class="fa fa-exclamation-triangle text-4xl mb-3 opacity-50"></i>
                                <p>"生成失败"</p>
                            </div>
                        }
                            .into_any()
                    }
                    _ => {

                        // 4. 初始状态 / 空闲 (None)
                        view! {
                            <div class="text-center py-12 text-gray-400 bg-gray-50 rounded-xl border border-dashed border-gray-200">
                                <i class="fa fa-headphones text-4xl mb-3 opacity-30"></i>
                                <p class="text-sm">"输入文本后点击生成按钮"</p>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </section>
    }
}
