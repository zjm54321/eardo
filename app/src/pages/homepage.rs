use leptos::prelude::*;
use leptos_router::components::A;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VoiceOption {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub category: String, // 例如 "male", "female", "summer"
}


#[component]
pub fn HomePage() -> impl IntoView {
    // 状态提升：所有数据状态都在 HomePage 管理
    let text_signal = RwSignal::new("".to_string());
    // 默认选中 "summer"
    let voice_signal = RwSignal::new("summer".to_string());

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
                    
                    // 调试信息 (可选)
                    <div class="text-xs text-gray-400 mt-2 flex justify-center gap-4">
                         <span>"字数: " {move || text_signal.get().chars().count()}</span>
                         <span>"当前声线: " {move || voice_signal.get()}</span>
                    </div>
                </section>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    
                    // --- 左侧栏 ---
                    <div class="lg:col-span-1 space-y-8">
                        // 文本输入
                        <TextInputCard text=text_signal />
                        
                        // 声线选择 (这里替换了之前的 TODO)
                        <VoiceSelectorCard selected_voice=voice_signal />
                    </div>

                    // --- 右侧栏 ---
                    <div class="lg:col-span-2 space-y-8">
                        <div class="bg-white/50 rounded-xl p-12 border border-dashed border-gray-300 text-center text-gray-400 h-full flex items-center justify-center flex-col">
                            <i class="fa fa-sliders text-4xl mb-4 opacity-50"></i>
                            <p>"右侧区域：参数调节与生成结果"</p>
                            <p class="text-sm mt-2">"(下一步实现)"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}


#[component]
pub fn Header() -> impl IntoView {
    view! {
        // 顶部导航栏容器
        // sticky top-0: 吸顶
        // bg-white/70 backdrop-blur-sm: 磨砂玻璃效果
        // shadow-soft: 柔和阴影
        <header class="py-4 px-6 md:px-12 bg-white/70 backdrop-blur-sm shadow-soft sticky top-0 z-50 transition-all duration-300">
            <div class="container mx-auto flex justify-between items-center">
                
                // --- 左侧：Logo ---
                // 使用 <A> 组件包裹以便点击 Logo 返回首页
                <A href="/" attr:class="flex items-center space-x-2 group text-decoration-none cursor-pointer">
                    // 旋转动画效果
                    <i class="fa fa-headphones text-primary text-2xl group-hover:rotate-12 transition-transform duration-300"></i>
                    <h1 class="text-xl md:text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                        "耳朵"
                    </h1>
                </A>

                // --- 中间：页面跳转 (声音广场) ---
                <nav class="absolute left-1/2 transform -translate-x-1/2">
                    <A 
                        href="/voice" 
                        attr:class="flex items-center space-x-2 px-4 py-2 rounded-full hover:bg-primary/10 transition-colors duration-300 group"
                    >
                        <i class="fa fa-music text-gray-400 group-hover:text-primary transition-colors"></i>
                        <span class="text-gray-600 font-medium group-hover:text-primary transition-colors">"声音广场"</span>
                    </A>
                </nav>

                // --- 右侧：头像框 (Todo) ---
                <div class="relative group">
                    // 头像容器：带渐变边框
                    <button class="w-10 h-10 rounded-full p-[2px] bg-gradient-to-tr from-primary to-accent shadow-sm hover:shadow-md transition-all duration-300">
                        <div class="w-full h-full rounded-full bg-white flex items-center justify-center overflow-hidden">
                            // 默认用户图标
                            <i class="fa fa-user text-gray-400 text-lg"></i>
                        </div>
                    </button>
                    
                    // Todo Tooltip (Hover 时显示)
                    <div class="absolute right-0 top-full mt-2 w-32 bg-dark text-xs rounded-lg py-2 px-3 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 transform translate-y-2 group-hover:translate-y-0 text-center shadow-lg z-50">
                        "登录功能开发中..."
                        // 小三角
                        <div class="absolute -top-1 right-3 w-2 h-2 bg-dark transform rotate-45"></div>
                    </div>
                </div>
            </div>
        </header>
    }
}


#[component]
pub fn TextInputCard(
    /// 用于存储输入文本的信号，由父组件传入
    text: RwSignal<String>
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
    selected_voice: RwSignal<String>
) -> impl IntoView {
    // Resource 用于异步获取数据
    let voices_resource = Resource::new(|| (), |_| get_voices());

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
                        // 这里我们忽略 Error，直接解包 Ok，如果失败则不渲染
                        // map 在 Option 上操作，Ok(voices) 才会进入内部
                        voices_resource.get().and_then(|res| res.ok()).map(|voices| view! {
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
                        })
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[server]
pub async fn get_voices() -> Result<Vec<VoiceOption>, ServerFnError> {
    // 这里是服务器端代码
    // 模拟数据库查询，返回硬编码数据
    let voices = vec![
        VoiceOption {
            id: "summer".to_string(),
            name: "夏以昼".to_string(),
            desc: "清澈明亮的少年音".to_string(),
            category: "summer".to_string(),
        },
        VoiceOption {
            id: "female".to_string(),
            name: "温柔女声".to_string(),
            desc: "温婉柔和的女性声线".to_string(),
            category: "female".to_string(),
        },
        VoiceOption {
            id: "male".to_string(),
            name: "沉稳男声".to_string(),
            desc: "成熟稳重的男性声线".to_string(),
            category: "male".to_string(),
        },
    ];

    Ok(voices)
}