use crate::api;
use leptos::logging::{debug_log, debug_warn};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct VoiceParams {
    pitch: f32,
    speed: f32,
    emotion: String,
}

impl Default for VoiceParams {
    fn default() -> Self {
        VoiceParams {
            pitch: 0.0,
            speed: 1.0,
            emotion: "happy".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateParams {
    pub text: String,
    pub voice_id: String,
    pub pitch: f32,
    pub speed: f32,
    pub emotion: String,
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
            pitch: param_signal.get().pitch,
            speed: param_signal.get().speed,
            emotion: param_signal.get().emotion.clone(),
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
                <Suspense fallback=move || view! {
                    <div class="flex justify-center items-center py-8 text-gray-400 animate-pulse">
                        <i class="fa fa-spinner fa-spin mr-2"></i>
                        "加载声线库..."
                    </div>
                }>
                    {move || {
                        match voices_resource.get() {
                            None => view! {
                                <div class="flex justify-center items-center py-8 text-gray-400 animate-pulse">
                                    <i class="fa fa-spinner fa-spin mr-2"></i>
                                    "加载声线库..."
                                </div>
                            }.into_view(),
                            Some(Err(e)) =>
                                view! {
                                <div class="flex justify-center items-center py-8 text-gray-400 animate-pulse">
                                    <i class="fa fa-spinner fa-spin mr-2"></i>
                                    {move || debug_log!("加载声线库失败: {}", e)}
                                    "加载声线库失败"
                                </div>
                            }.into_view(),
                            Some(Ok(voices)) => view! {
                            <div class="grid grid-cols-1 gap-3">
                                <For
                                    each=move || voices.clone()
                                    key=|voice| voice.id.clone()
                                    children=move |voice| {
                                        let voice_id = voice.id.clone();
                                        // is_active 是一个闭包：Fn() -> bool
                                        let is_active = move || selected_voice.get() == voice_id;

                                        view! {
                                            <div
                                                class="voice-option p-4 border rounded-lg cursor-pointer transition-all duration-200 flex justify-between items-center group"
                                                // 1. 选中状态: 边框变黄
                                                //class:border-primary=is_active
                                                // 2. 选中状态: 背景变淡黄 (使用 opacity 语法，因为 primary 是单色)
                                                //class:bg-primary\/10=is_active

                                                // --- 修复点在这里 ---
                                                // 错误写法: !is_active
                                                // 正确写法: move || !is_active()
                                                //class:border-gray-200=move || !is_active()

                                                class:hover:border-primary=true
                                                on:click=move |_| selected_voice.set(voice.id.clone())
                                            >
                                                <div>
                                                    <div class="font-medium group-hover:text-primary transition-colors">
                                                        {voice.name}
                                                    </div>
                                                    <div class="text-sm text-gray-500">
                                                        {voice.desc}
                                                    </div>
                                                </div>

                                                // 选中时的图标
                                                <div class="text-primary transition-opacity duration-200"
                                                     // 这里也是同样的逻辑：需要调用闭包并取反
                                                     class:hidden=move || !is_active()
                                                >
                                                    <i class="fa fa-check-circle text-xl"></i>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }.into_view(),
                    } // 正常情况继续往下

                        }
                    }
                </Suspense>
            </div>
        </section>
    }
}

#[component]
fn ParameterControlCard(selected_param: RwSignal<VoiceParams>) -> impl IntoView {
    let _ = selected_param;
    view! {
        <section class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover relative overflow-hidden group">
            // 标题
            <h3 class="text-lg font-semibold mb-6 flex items-center text-gray-400">
                <i class="fa fa-sliders mr-2"></i>
                "参数调节"
            </h3>

            // 模拟的内容（模糊处理）
            <div class="space-y-6 opacity-30 pointer-events-none select-none filter blur-[1px]">
                <div>
                    <div class="flex justify-between mb-2">
                        <label class="font-medium">"音高 (Pitch)"</label>
                        <span class="text-sm text-primary">"0"</span>
                    </div>
                    <input type="range" class="w-full h-2 bg-gray-200 rounded-lg" />
                </div>
                <div>
                    <div class="flex justify-between mb-2">
                        <label class="font-medium">"语速 (Speed)"</label>
                        <span class="text-sm text-primary">"1.0x"</span>
                    </div>
                    <input type="range" class="w-full h-2 bg-gray-200 rounded-lg" />
                </div>
            </div>

            // 待开发提示遮罩
            <div class="absolute inset-0 flex items-center justify-center bg-white/10 backdrop-blur-[1px]">
                <div class="bg-white/80 px-4 py-2 rounded-full border border-dashed border-gray-300 text-gray-500 text-sm shadow-sm flex items-center">
                    <i class="fa fa-wrench mr-2"></i>
                    "参数调节功能开发中..."
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
                    {move || if is_pending.get() {
                        view! { <> <i class="fa fa-circle-o-notch fa-spin mr-2"></i> "正在生成..." </> }.into_view()
                    } else {
                        view! { <> <i class="fa fa-magic mr-2"></i> "生成音频" </> }.into_view()
                    }}
                </button>
            </div>

            // --- 状态展示区域 (使用 match 替代 if-else) ---
            <div>
                {move || match (is_pending.get(), value.get()) {
                    // 1. 正在加载
                    (true, _) => view! {
                        <div class="flex flex-col items-center justify-center py-8 animate-fade-in">
                            <div class="w-12 h-12 border-4 border-primary/30 border-t-primary rounded-full animate-spin mb-4"></div>
                            <p class="text-gray-500">"AI 正在合成您的声音..."</p>
                        </div>
                    }.into_view(),

                    // 2. 加载完成，成功获取 URL
                    (false, Some(Ok(url))) => view! {
                        <div class="border border-green-200 bg-green-50 rounded-xl p-6 animate-slide-up">
                            <div class="flex items-center mb-4">
                                <div class="bg-green-100 p-2 rounded-full mr-3">
                                    <i class="fa fa-check text-green-600"></i>
                                </div>
                                <h4 class="font-semibold text-green-800">"生成完成！"</h4>
                            </div>
                            <div class="mb-4">
                                <p class="text-sm text-gray-600 mb-2">"处理后的音频："</p>
                                <div class="flex flex-col gap-3">
                                    <audio controls autoplay class="w-full" src=url.clone()></audio>
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
                    }.into_view(),

                    // 3. 失败 (可选处理)
                    (false, Some(Err(e))) => view! {
                        <div class="text-center py-8 text-red-500 bg-red-50 rounded-xl border border-red-200">
                            <i class="fa fa-exclamation-triangle text-4xl mb-3 opacity-50"></i>
                            <p>"生成失败: "</p>
                            {move || debug_warn!("生成音频失败: {:?}", e)}
                        </div>
                    }.into_view(),

                    // 4. 初始状态 / 空闲 (None)
                    _ => view! {
                        <div class="text-center py-12 text-gray-400 bg-gray-50 rounded-xl border border-dashed border-gray-200">
                            <i class="fa fa-headphones text-4xl mb-3 opacity-30"></i>
                            <p class="text-sm">"输入文本后点击生成按钮"</p>
                        </div>
                    }.into_view()
                }}
            </div>
        </section>
    }
}
