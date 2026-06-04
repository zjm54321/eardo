use leptos::prelude::*;
use leptos_router::components::A;

pub mod auth;
pub mod component;
pub mod help; // 帮助中心
pub mod homepage;
pub mod playground;
pub mod share;
pub mod voicefilter;
pub mod voicesetup;
pub mod guide;
pub mod welcome; // 新增

#[component]
pub fn Header() -> impl IntoView {
    // 检查用户登录状态的资源
    // 这是一个简单的做法，实际上可能需要一个全局 Context 来存储 User
    let user_resource = Resource::new(|| (), |_| crate::api::user::get_user_profile());
    let (show_menu, set_show_menu) = signal(false);
    let (show_guide, set_show_guide) = signal(false);

    view! {
        // 顶部导航栏容器
        // sticky top-0: 吸顶
        // bg-white/70 backdrop-blur-sm: 磨砂玻璃效果
        // shadow-soft: 柔和阴影
        <header class="py-4 px-6 md:px-12 bg-white/70 backdrop-blur-sm shadow-soft sticky top-0 z-50 transition-all duration-300">
            <div class="container mx-auto flex justify-between items-center">

                // --- 左侧：Logo ---
                // 点击 Logo 返回首页
                <A
                    href="/"
                    attr:class="flex items-center space-x-2 group text-decoration-none cursor-pointer"
                >
                    // 旋转动画效果
                    <i class="fa-solid fa-headphones text-primary text-2xl group-hover:rotate-12 transition-transform duration-300"></i>
                    <h1 class="text-xl md:text-2xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                        "耳朵"
                    </h1>
                </A>

                // --- 中间：导航菜单 ---
                // 使用 flex 和 gap 来排列多个链接
                // hidden md:flex: 在移动端隐藏，桌面端显示 (移动端通常需要汉堡菜单，这里暂略)
                <nav class="absolute left-1/2 transform -translate-x-1/2 hidden md:flex items-center space-x-1">
                    // 1. 首页
                    <A
                        href="/setup"
                        attr:class="flex items-center space-x-2 px-4 py-2 rounded-full hover:bg-primary/10 transition-colors duration-300 group"
                    >
                        <i class="fa-solid fa-house text-gray-400 group-hover:text-primary transition-colors"></i>
                        <span class="text-gray-600 font-medium group-hover:text-primary transition-colors">
                            "首页"
                        </span>
                    </A>

                    // 2. 声音广场
                    <A
                        href="/voice"
                        attr:class="flex items-center space-x-2 px-4 py-2 rounded-full hover:bg-primary/10 transition-colors duration-300 group"
                    >
                        <i class="fa-solid fa-music text-gray-400 group-hover:text-primary transition-colors"></i>
                        <span class="text-gray-600 font-medium group-hover:text-primary transition-colors">
                            "声音广场"
                        </span>
                    </A>

                    // 3. 声音滤镜
                    <A
                        href="/filters"
                        attr:class="flex items-center space-x-2 px-4 py-2 rounded-full hover:bg-primary/10 transition-colors duration-300 group"
                    >
                        <i class="fa-solid fa-sliders text-gray-400 group-hover:text-primary transition-colors"></i>
                        <span class="text-gray-600 font-medium group-hover:text-primary transition-colors">
                            "声音滤镜"
                        </span>
                    </A>

                    // 4. 帮助中心
                    <A
                        href="/help"
                        attr:class="flex items-center space-x-2 px-4 py-2 rounded-full hover:bg-primary/10 transition-colors duration-300 group"
                    >
                        <i class="fa-solid fa-circle-question text-gray-400 group-hover:text-primary transition-colors"></i>
                        <span class="text-gray-600 font-medium group-hover:text-primary transition-colors">
                            "帮助"
                        </span>
                    </A>

                    // 5. 新手引导
                    <div class="relative">
                        <button
                            class="flex items-center space-x-2 px-4 py-2 rounded-full hover:bg-accent/10 transition-colors duration-300 group"
                            on:click=move |_| set_show_guide.update(|v| *v = !*v)
                        >
                            <i class="fa-solid fa-lightbulb text-gray-400 group-hover:text-accent transition-colors"></i>
                            <span class="text-gray-600 font-medium group-hover:text-accent transition-colors">
                                "新手引导"
                            </span>
                        </button>

                        <Show when=move || show_guide.get()>
                            // 透明遮罩，点击关闭引导
                            <div
                                class="fixed inset-0 z-40"
                                on:click=move |_| set_show_guide.set(false)
                            ></div>
                            // 引导列表
                            <div class="absolute right-0 mt-2 w-72 bg-white rounded-lg shadow-xl py-3 z-50 border border-gray-100 animate-fade-in">
                                <div class="px-4 py-2 border-b border-gray-100">
                                    <h3 class="text-sm font-bold text-gray-800 flex items-center">
                                        <i class="fa-solid fa-graduation-cap text-accent mr-2"></i>
                                        "快速上手指南"
                                    </h3>
                                </div>

                                // 引导项 1: 声线选择
                                <A
                                    href="/setup"
                                    attr:class="block px-4 py-3 hover:bg-accent/5 transition-colors border-b border-gray-50 group"
                                    on:click=move |_| set_show_guide.set(false)
                                >
                                    <div class="flex items-start space-x-3">
                                        <div class="w-8 h-8 rounded-full bg-accent/10 flex items-center justify-center flex-shrink-0 group-hover:bg-accent/20 transition-colors">
                                            <i class="fa-solid fa-microphone text-accent text-sm"></i>
                                        </div>
                                        <div class="flex-1">
                                            <div class="font-medium text-gray-800 text-sm group-hover:text-accent transition-colors">
                                                "1. 选择声线"
                                            </div>
                                            <div class="text-xs text-gray-500 mt-0.5">
                                                "从声音库中挑选合适的声线"
                                            </div>
                                        </div>
                                        <i class="fa-solid fa-chevron-right text-gray-300 text-xs mt-1 group-hover:text-accent transition-colors"></i>
                                    </div>
                                </A>

                                // 引导项 2: 文本输入
                                <A
                                    href="/setup"
                                    attr:class="block px-4 py-3 hover:bg-accent/5 transition-colors border-b border-gray-50 group"
                                    on:click=move |_| set_show_guide.set(false)
                                >
                                    <div class="flex items-start space-x-3">
                                        <div class="w-8 h-8 rounded-full bg-accent/10 flex items-center justify-center flex-shrink-0 group-hover:bg-accent/20 transition-colors">
                                            <i class="fa-solid fa-keyboard text-accent text-sm"></i>
                                        </div>
                                        <div class="flex-1">
                                            <div class="font-medium text-gray-800 text-sm group-hover:text-accent transition-colors">
                                                "2. 输入文本"
                                            </div>
                                            <div class="text-xs text-gray-500 mt-0.5">
                                                "输入想要转换的文字内容"
                                            </div>
                                        </div>
                                        <i class="fa-solid fa-chevron-right text-gray-300 text-xs mt-1 group-hover:text-accent transition-colors"></i>
                                    </div>
                                </A>

                                // 引导项 3: 参数调节
                                <A
                                    href="/setup"
                                    attr:class="block px-4 py-3 hover:bg-accent/5 transition-colors border-b border-gray-50 group"
                                    on:click=move |_| set_show_guide.set(false)
                                >
                                    <div class="flex items-start space-x-3">
                                        <div class="w-8 h-8 rounded-full bg-accent/10 flex items-center justify-center flex-shrink-0 group-hover:bg-accent/20 transition-colors">
                                            <i class="fa-solid fa-sliders text-accent text-sm"></i>
                                        </div>
                                        <div class="flex-1">
                                            <div class="font-medium text-gray-800 text-sm group-hover:text-accent transition-colors">
                                                "3. 调节参数"
                                            </div>
                                            <div class="text-xs text-gray-500 mt-0.5">
                                                "调整语速、音调等参数"
                                            </div>
                                        </div>
                                        <i class="fa-solid fa-chevron-right text-gray-300 text-xs mt-1 group-hover:text-accent transition-colors"></i>
                                    </div>
                                </A>

                                // 引导项 4: 生成与输出
                                <A
                                    href="/setup"
                                    attr:class="block px-4 py-3 hover:bg-accent/5 transition-colors group"
                                    on:click=move |_| set_show_guide.set(false)
                                >
                                    <div class="flex items-start space-x-3">
                                        <div class="w-8 h-8 rounded-full bg-accent/10 flex items-center justify-center flex-shrink-0 group-hover:bg-accent/20 transition-colors">
                                            <i class="fa-solid fa-wand-magic-sparkles text-accent text-sm"></i>
                                        </div>
                                        <div class="flex-1">
                                            <div class="font-medium text-gray-800 text-sm group-hover:text-accent transition-colors">
                                                "4. 生成结果"
                                            </div>
                                            <div class="text-xs text-gray-500 mt-0.5">
                                                "点击生成并播放/下载音频"
                                            </div>
                                        </div>
                                        <i class="fa-solid fa-chevron-right text-gray-300 text-xs mt-1 group-hover:text-accent transition-colors"></i>
                                    </div>
                                </A>

                                // 底部提示
                                <div class="px-4 py-2 mt-2 border-t border-gray-100">
                                    <p class="text-xs text-gray-400 text-center">
                                        "点击任意步骤开始体验"
                                    </p>
                                </div>
                            </div>
                        </Show>
                    </div>
                </nav>

                // --- 右侧：头像框 ---
                <div class="relative group">
                    <Suspense fallback=move || {
                        view! {
                            <div class="w-10 h-10 rounded-full bg-gray-200 animate-pulse"></div>
                        }
                    }>
                        {move || {
                            match user_resource.get() {
                                Some(Ok(user)) => {
                                    view! {
                                        <div class="relative">
                                            <button
                                                class="w-10 h-10 rounded-full p-[2px] bg-gradient-to-tr from-primary to-accent shadow-sm hover:shadow-md transition-all duration-300 overflow-hidden"
                                                on:click=move |_| set_show_menu.update(|v| *v = !*v)
                                            >
                                                <div class="w-full h-full rounded-full bg-white flex items-center justify-center overflow-hidden font-bold text-primary">
                                                    // 显示头像
                                                    {if user.usermeta.avatar_url.starts_with("/api/")
                                                        || user.usermeta.avatar_url.starts_with("http")
                                                    {
                                                        view! {
                                                            <img
                                                                src=user.usermeta.avatar_url.clone()
                                                                alt="avatar"
                                                                class="w-full h-full object-cover"
                                                            />
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! { <i class="fa-solid fa-user text-lg"></i> }
                                                            .into_any()
                                                    }}
                                                </div>
                                            </button>

                                            <Show when=move || show_menu.get()>
                                                // 透明遮罩，点击关闭菜单
                                                <div
                                                    class="fixed inset-0 z-40"
                                                    on:click=move |_| set_show_menu.set(false)
                                                ></div>
                                                // 下拉菜单
                                                <div class="absolute right-0 mt-2 w-32 bg-white rounded-lg shadow-lg py-2 z-50 border border-gray-100 animate-fade-in">
                                                    <A
                                                        href="/profile"
                                                        attr:class="block px-4 py-2 text-sm text-gray-700 hover:bg-primary/10 hover:text-primary transition-colors"
                                                        on:click=move |_| set_show_menu.set(false)
                                                    >
                                                        <i class="fa-solid fa-user mr-2"></i>
                                                        "我的主页"
                                                    </A>
                                                    <A
                                                        href="/settings"
                                                        attr:class="block px-4 py-2 text-sm text-gray-700 hover:bg-primary/10 hover:text-primary transition-colors"
                                                        on:click=move |_| set_show_menu.set(false)
                                                    >
                                                        <i class="fa-solid fa-gear mr-2"></i>
                                                        "设置"
                                                    </A>
                                                    <div class="border-t border-gray-100 my-1"></div>
                                                    <button
                                                        class="w-full text-left block px-4 py-2 text-sm text-red-500 hover:bg-red-50 transition-colors"
                                                        on:click=move |_| {
                                                            set_show_menu.set(false);
                                                            leptos::task::spawn_local(async move {
                                                                let _ = crate::api::user::logout().await;
                                                                #[cfg(target_arch = "wasm32")]
                                                                if let Some(window) = web_sys::window() {
                                                                    let _ = window.location().set_href("/");
                                                                }
                                                            });
                                                        }
                                                    >
                                                        <i class="fa-solid fa-right-from-bracket mr-2"></i>
                                                        "退出登录"
                                                    </button>
                                                </div>
                                            </Show>
                                        </div>
                                    }
                                        .into_any()
                                }
                                _ => {
                                    view! {
                                        <A href="/login">
                                            <button class="px-4 py-2 rounded-full bg-dark text-white text-sm hover:bg-gray-700 transition-colors shadow-sm">
                                                "登录"
                                            </button>
                                        </A>
                                    }
                                        .into_any()
                                }
                            }
                        }}
                    </Suspense>
                </div>
            </div>
        </header>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        // 容器：flex 居中，使用稍微深一点的背景色突出卡片
        <div class="min-h-[80vh] flex flex-col items-center justify-center text-center px-4">

            // 错误信息卡片 (前景)
            // shadow-2xl: 更深的阴影
            // max-w-3xl: 稍微加宽一点
            <div class="relative z-10 bg-white p-12 md:p-20 rounded-3xl shadow-2xl border border-gray-100 max-w-3xl w-full mx-auto overflow-visible">

                // 图标区域
                <div class="mb-8  relative inline-block">
                    // 背景光晕 (调小模糊半径 blur-2xl)
                    <div class="absolute inset-0 bg-primary/20 rounded-full blur-2xl transform scale-110"></div>

                    // 图标本身：SVG
                    // text-primary: 保持主题色
                    // w-32 h-32 md:w-40 md:h-40: 调小尺寸 (128px - 160px)，更精致
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        // 稍微加粗线条，因为图标变小了
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="w-32 h-32 md:w-40 md:h-40 text-primary relative z-10 animate-bounce-slow"
                    >
                        <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" />
                        <path d="M12 9v4" />
                        <path d="M12 17h.01" />
                    </svg>
                </div>

                <h2 class="text-4xl md:text-5xl font-black text-dark mb-6 tracking-tight relative z-20">
                    "哎呀！页面走丢了"
                </h2>

                <div class="text-gray-500 text-lg md:text-xl mb-10 leading-relaxed space-y-2 relative z-20">
                    <p>"你寻找的声音似乎不在这里..."</p>
                    <p>"但不要灰心，也许它藏在别的地方。"</p>
                </div>

                // 返回首页按钮
                <a
                    href="/"
                    class="relative z-20 inline-flex items-center px-8 py-4 bg-primary hover:bg-primary/90 text-white text-lg font-bold rounded-2xl transition-all shadow-lg hover:shadow-xl hover:-translate-y-1 active:translate-y-0 active:shadow-md group"
                >
                    // Home icon SVG
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        class="mr-2 w-5 h-5 group-hover:scale-110 transition-transform"
                    >
                        <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
                        <polyline points="9 22 9 12 15 12 15 22" />
                    </svg>
                    "返回首页"
                </a>
            </div>
        </div>
    }
}
