use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn WelcomePage() -> impl IntoView {
    view! {
        <div class="relative w-full h-screen overflow-hidden bg-gradient-summer">

            <div class="absolute inset-0 flex flex-col items-center justify-center">

                // 1. Logo and Title
                <div class="text-center z-10 flex flex-col items-center -translate-y-32 scale-90">
                    <div class="mb-6 relative inline-block">
                        <img
                            src="/logo.png"
                            alt="Eardo Logo"
                            class="w-32 h-32 md:w-48 md:h-48 lg:w-64 lg:h-64 object-contain drop-shadow-lg"
                        />
                        <div class="absolute inset-0 bg-primary/20 blur-3xl rounded-full -z-10 animate-pulse"></div>
                    </div>

                    <h1 class="text-5xl md:text-7xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-primary to-secondary mb-2 drop-shadow-sm">
                        "白昼聆夏"
                    </h1>
                    <p class="text-gray-400 text-lg md:text-xl font-medium tracking-[0.5em] uppercase">
                        "EARDO"
                    </p>
                </div>

                // 2. Slogan
                <div class="absolute top-[60%] text-center max-w-2xl px-6 -translate-y-8">
                    <h2 class="text-3xl md:text-5xl font-bold text-dark leading-tight text-shadow">
                        "声音也能如此多彩"
                    </h2>
                    <div class="w-24 h-1.5 bg-gradient-to-r from-primary to-secondary rounded-full mx-auto mt-6"></div>
                </div>

                // 3. Button
                <div class="absolute top-[80%]">
                    <A href="/setup" attr:class="group relative inline-flex items-center justify-center px-10 py-5 text-xl font-bold text-white transition-all duration-300 bg-gradient-to-r from-primary to-secondary rounded-full shadow-lg hover:shadow-2xl hover:-translate-y-1 focus:outline-none focus:ring-4 focus:ring-primary/30 overflow-hidden">
                        <span class="relative z-10 flex items-center">
                            "开启我的旅程"
                            <i class="fa-solid fa-arrow-right ml-3 group-hover:translate-x-1 transition-transform"></i>
                        </span>
                        <div class="absolute inset-0 -translate-x-full group-hover:translate-x-0 bg-white/20 transition-transform duration-500 skew-x-12"></div>
                    </A>
                </div>

            </div>

        </div>
    }
}
