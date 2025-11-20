use leptos::prelude::*;
use leptos_router::components::A;

pub mod homepage;

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
