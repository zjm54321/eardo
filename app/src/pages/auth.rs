use crate::api::{get_current_user, login, logout, register};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let login_action = Action::new(move |_| {
        let u = username.get();
        let p = password.get();
        {
            let value = navigate.clone();
            async move {
                match login(u, p).await {
                    Ok(_) => {
                        value("/profile", Default::default());
                    }
                    Err(e) => set_error_msg.set(Some(e.to_string())),
                }
            }
        }
    });

    view! {
        <div class="min-h-screen flex items-center justify-center bg-base-100 px-4">
            <div class="max-w-md w-full bg-white rounded-xl shadow-soft p-8 space-y-6 border border-gray-100">
                <div class="text-center">
                    <h2 class="text-3xl font-bold text-dark">"欢迎回来"</h2>
                    <p class="text-gray-500 mt-2">"请登录您的耳朵账号"</p>
                </div>

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"用户名"</label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="请输入用户名"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            prop:value=username
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"密码"</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="请输入密码"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=password
                        />
                    </div>
                </div>

                <Show when=move || error_msg.get().is_some()>
                    <div class="text-red-500 text-sm text-center bg-red-50 p-2 rounded">
                        {move || error_msg.get()}
                    </div>
                </Show>

                <button
                    class="w-full bg-primary hover:bg-primary-focus text-white font-bold py-3 rounded-lg transition-all shadow-md hover:shadow-lg active:scale-[0.98]"
                    on:click=move |_| { login_action.dispatch(()); }
                    disabled=move || login_action.pending().get()
                >
                    {move || if login_action.pending().get() { "登录中..." } else { "立即登录" }}
                </button>

                <div class="text-center text-sm text-gray-500">
                    "还没有账号？ "
                    <A href="/register" attr:class="text-primary hover:underline font-medium">"去注册"</A>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let navigate = use_navigate();
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let register_action = Action::new(move |_| {
        let u = username.get();
        let p = password.get();
        let cp = confirm_password.get();
        {
            let value = navigate.clone();
            async move {
                if p != cp {
                    set_error_msg.set(Some("两次输入的密码不一致".to_string()));
                    return;
                }
                match register(u, p).await {
                    Ok(_) => {
                        // 注册成功跳转登录
                        value("/login", Default::default());
                    }
                    Err(e) => set_error_msg.set(Some(e.to_string())),
                }
            }
        }
    });

    view! {
        <div class="min-h-screen flex items-center justify-center bg-base-100 px-4">
            <div class="max-w-md w-full bg-white rounded-xl shadow-soft p-8 space-y-6 border border-gray-100">
                <div class="text-center">
                    <h2 class="text-3xl font-bold text-dark">"创建账号"</h2>
                    <p class="text-gray-500 mt-2">"加入白昼聆夏，开启声音之旅"</p>
                </div>

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"用户名"</label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="设置用户名"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            prop:value=username
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"密码"</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="设置密码"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=password
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"确认密码"</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="再次输入密码"
                            on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            prop:value=confirm_password
                        />
                    </div>
                </div>

                <Show when=move || error_msg.get().is_some()>
                    <div class="text-red-500 text-sm text-center bg-red-50 p-2 rounded">
                        {move || error_msg.get()}
                    </div>
                </Show>

                <button
                    class="w-full bg-secondary hover:bg-secondary/90 text-white font-bold py-3 rounded-lg transition-all shadow-md hover:shadow-lg active:scale-[0.98]"
                    on:click=move |_| { register_action.dispatch(()); }
                    disabled=move || register_action.pending().get()
                >
                    {move || if register_action.pending().get() { "创建中..." } else { "注册账号" }}
                </button>

                <div class="text-center text-sm text-gray-500">
                    "已有账号？ "
                    <A href="/login" attr:class="text-primary hover:underline font-medium">"去登录"</A>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let navigate = use_navigate();
    let user_resource = Resource::new(|| (), |_| get_current_user());

    let logout_action = Action::new(move |_| {
        let value = navigate.clone();
        async move {
            let _ = logout().await;
            value("/login", Default::default());
        }
    });

    view! {
        <div class="min-h-screen bg-base-100 pt-20 px-4 pb-20">
            <div class="container mx-auto max-w-6xl">
                <Suspense fallback=move || view! { <div class="text-center py-10 text-gray-400">"加载用户信息..."</div> }>
                    {move || {
                        match user_resource.get() {
                            Some(Ok(Some(user))) => view! {
                                <div class="flex flex-col items-center animate-fade-in space-y-12">

                                    // 1. Slogan 区域
                                    <div class="text-center space-y-3 mt-4">
                                        <h1 class="text-4xl md:text-6xl font-black text-transparent bg-clip-text bg-gradient-to-r from-primary via-secondary to-accent tracking-tighter uppercase italic drop-shadow-sm">
                                            "Forge Your Unique Voice"
                                        </h1>
                                        <p class="text-gray-400 tracking-[0.5em] text-xs md:text-sm uppercase font-medium">"Create · Share · Inspire"</p>
                                    </div>

                                    // 2. 个人信息卡片
                                    <div class="bg-white rounded-3xl p-8 md:p-10 shadow-soft w-full flex flex-col md:flex-row items-center md:items-start gap-8 md:gap-12 border border-gray-100 relative overflow-hidden group">
                                        // 装饰背景
                                        <div class="absolute -top-24 -right-24 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors duration-500"></div>
                                        <div class="absolute top-1/2 -left-24 w-48 h-48 bg-secondary/5 rounded-full blur-3xl group-hover:bg-secondary/10 transition-colors duration-500"></div>

                                        // 头像
                                        <div class="relative flex-shrink-0">
                                            <div class="w-32 h-32 md:w-40 md:h-40 rounded-full p-1.5 bg-gradient-to-tr from-primary to-secondary shadow-lg">
                                                <div class="w-full h-full rounded-full bg-white overflow-hidden relative border-4 border-white">
                                                    {if let Some(avatar) = user.avatar.clone() {
                                                        view! { <img src=avatar class="w-full h-full object-cover" /> }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="w-full h-full flex items-center justify-center bg-gray-50 text-gray-200">
                                                                <i class="fa fa-user text-6xl"></i>
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            </div>
                                            <button class="absolute bottom-2 right-2 w-9 h-9 bg-white text-gray-500 rounded-full shadow-md flex items-center justify-center hover:text-primary transition-all hover:scale-110 border border-gray-100 cursor-pointer">
                                                <i class="fa fa-camera text-sm"></i>
                                            </button>
                                        </div>

                                        // 信息内容
                                        <div class="flex-grow text-center md:text-left space-y-5 z-10 w-full">
                                            <div class="flex flex-col md:flex-row md:items-end justify-between gap-4">
                                                <div>
                                                    <h2 class="text-3xl md:text-4xl font-bold text-dark tracking-tight">{user.username.clone()}</h2>
                                                    <div class="flex items-center justify-center md:justify-start gap-2 mt-1">
                                                        <span class="px-2 py-0.5 bg-gray-100 text-gray-500 text-xs rounded font-mono">"UID: " {user.id.clone()}</span>
                                                        <span class="px-2 py-0.5 bg-primary/10 text-primary text-xs rounded font-medium">"Pro Member"</span>
                                                    </div>
                                                </div>

                                                <button
                                                    class="px-5 py-2 bg-white border border-gray-200 text-gray-500 rounded-full hover:bg-gray-50 hover:text-red-500 hover:border-red-200 transition-all shadow-sm text-sm flex items-center gap-2 mx-auto md:mx-0"
                                                    on:click=move |_| { logout_action.dispatch(()); }
                                                >
                                                    <i class="fa fa-sign-out"></i>
                                                    "退出登录"
                                                </button>
                                            </div>

                                            <div class="bg-gray-50/80 rounded-xl p-4 border border-gray-100 text-gray-600 text-sm leading-relaxed text-left relative">
                                                <i class="fa fa-quote-left text-gray-300 absolute -top-2 -left-2 text-xl"></i>
                                                "热爱声音，热爱创作。这是我的声音工坊，存放着我所有的灵感与作品。探索无限可能，用声音连接世界。"
                                                <button class="ml-2 text-primary text-xs hover:underline font-medium">"编辑简介"</button>
                                            </div>

                                            <div class="flex flex-wrap gap-4 justify-center md:justify-start pt-1">
                                                <div class="flex flex-col items-center md:items-start px-2">
                                                    <span class="text-xl font-bold text-dark">"12"</span>
                                                    <span class="text-xs text-gray-400 uppercase tracking-wide">"Works"</span>
                                                </div>
                                                <div class="w-px h-8 bg-gray-200 my-auto"></div>
                                                <div class="flex flex-col items-center md:items-start px-2">
                                                    <span class="text-xl font-bold text-dark">"48"</span>
                                                    <span class="text-xs text-gray-400 uppercase tracking-wide">"Audios"</span>
                                                </div>
                                                <div class="w-px h-8 bg-gray-200 my-auto"></div>
                                                <div class="flex flex-col items-center md:items-start px-2">
                                                    <span class="text-xl font-bold text-dark">"1.2k"</span>
                                                    <span class="text-xs text-gray-400 uppercase tracking-wide">"Likes"</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // 3. 三栏列表区域
                                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 w-full items-start">

                                        // 栏目 1: 我的作品
                                        <div class="space-y-4">
                                            <div class="flex items-center justify-between px-1">
                                                <h3 class="text-lg font-bold text-dark flex items-center">
                                                    <span class="w-1.5 h-6 bg-secondary rounded-full mr-2"></span>
                                                    "我的作品"
                                                </h3>
                                                <button class="text-xs text-gray-400 hover:text-secondary"><i class="fa fa-arrow-right"></i></button>
                                            </div>

                                            <div class="bg-white rounded-2xl shadow-soft border border-gray-100 overflow-hidden min-h-[320px]">
                                                <ul class="divide-y divide-gray-50">
                                                    {(1..=4).map(|i| view! {
                                                        <li class="p-4 hover:bg-gray-50/80 transition-colors cursor-pointer group">
                                                            <div class="flex items-center justify-between mb-1">
                                                                <span class="font-medium text-dark text-sm group-hover:text-secondary transition-colors">"配音作品 #" {i}</span>
                                                                <span class="text-[10px] text-gray-400 bg-gray-100 px-1.5 py-0.5 rounded">"公开"</span>
                                                            </div>
                                                            <p class="text-xs text-gray-500 line-clamp-2 mt-1">"这是一个很棒的配音作品，使用了多种音效..."</p>
                                                            <div class="flex items-center gap-3 mt-2 text-xs text-gray-400">
                                                                <span><i class="fa fa-play-circle mr-1"></i>"128"</span>
                                                                <span><i class="fa fa-heart mr-1"></i>"32"</span>
                                                                <span class="ml-auto">"2天前"</span>
                                                            </div>
                                                        </li>
                                                    }).collect::<Vec<_>>()}
                                                </ul>
                                                <div class="p-3 text-center border-t border-gray-50">
                                                    <button class="text-xs font-medium text-secondary hover:underline">"查看全部作品"</button>
                                                </div>
                                            </div>
                                        </div>

                                        // 栏目 2: 声音文件
                                        <div class="space-y-4">
                                            <div class="flex items-center justify-between px-1">
                                                <h3 class="text-lg font-bold text-dark flex items-center">
                                                    <span class="w-1.5 h-6 bg-primary rounded-full mr-2"></span>
                                                    "声音文件"
                                                </h3>
                                                <button class="text-xs text-gray-400 hover:text-primary"><i class="fa fa-plus"></i></button>
                                            </div>

                                            <div class="bg-white rounded-2xl shadow-soft border border-gray-100 overflow-hidden min-h-[320px]">
                                                <ul class="divide-y divide-gray-50">
                                                    {(1..=5).map(|i| view! {
                                                        <li class="flex items-center p-3 hover:bg-gray-50/80 transition-colors cursor-pointer group">
                                                            <div class="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center text-primary mr-3 shrink-0 group-hover:bg-primary group-hover:text-white transition-colors">
                                                                <i class="fa fa-music text-sm"></i>
                                                            </div>
                                                            <div class="flex-grow min-w-0">
                                                                <div class="font-medium text-dark text-sm truncate">"录音_2025050" {i} ".mp3"</div>
                                                                <div class="text-xs text-gray-400 mt-0.5">"00:45 • 1.2MB"</div>
                                                            </div>
                                                            <button class="text-gray-300 hover:text-dark w-8 h-8 flex items-center justify-center rounded-full hover:bg-gray-200 transition-colors">
                                                                <i class="fa fa-ellipsis-v text-xs"></i>
                                                            </button>
                                                        </li>
                                                    }).collect::<Vec<_>>()}
                                                </ul>
                                            </div>
                                        </div>

                                        // 栏目 3: 声音分析 (模糊处理)
                                        <div class="space-y-4">
                                            <div class="flex items-center justify-between px-1">
                                                <h3 class="text-lg font-bold text-dark flex items-center">
                                                    <span class="w-1.5 h-6 bg-accent rounded-full mr-2"></span>
                                                    "声音分析"
                                                </h3>
                                                <span class="text-[10px] bg-accent/10 text-accent px-2 py-0.5 rounded-full font-bold">"PRO"</span>
                                            </div>

                                            <div class="bg-white rounded-2xl p-0 shadow-soft min-h-[320px] border border-gray-100 relative overflow-hidden group">
                                                // 背景内容 (被模糊)
                                                <div class="p-5 space-y-4 filter blur-[6px] opacity-60 select-none pointer-events-none transform scale-105 group-hover:scale-110 transition-transform duration-700">
                                                    <div class="flex items-center justify-between">
                                                        <div class="h-4 bg-gray-200 rounded w-1/3"></div>
                                                        <div class="h-4 bg-gray-200 rounded w-1/4"></div>
                                                    </div>
                                                    <div class="h-32 bg-gradient-to-b from-blue-50 to-purple-50 rounded-xl w-full border border-gray-100"></div>
                                                    <div class="space-y-2">
                                                        <div class="h-3 bg-gray-100 rounded w-full"></div>
                                                        <div class="h-3 bg-gray-100 rounded w-5/6"></div>
                                                        <div class="h-3 bg-gray-100 rounded w-4/6"></div>
                                                    </div>
                                                    <div class="grid grid-cols-2 gap-3 mt-4">
                                                        <div class="h-20 bg-gray-50 rounded-lg"></div>
                                                        <div class="h-20 bg-gray-50 rounded-lg"></div>
                                                    </div>
                                                </div>

                                                // 覆盖层
                                                <div class="absolute inset-0 flex flex-col items-center justify-center bg-white/40 backdrop-blur-[2px] p-6 text-center z-10 transition-all">
                                                    <div class="w-16 h-16 bg-white rounded-2xl shadow-xl flex items-center justify-center mb-5 text-accent transform -rotate-6 group-hover:rotate-0 transition-transform duration-300">
                                                        <i class="fa fa-lock text-2xl"></i>
                                                    </div>
                                                    <h4 class="font-bold text-dark text-lg mb-2">"解锁声音特质分析"</h4>
                                                    <p class="text-gray-500 text-sm mb-6 max-w-[200px] leading-relaxed">"获取关于您音色、音域及情感特征的深度 AI 报告"</p>
                                                    <button class="px-6 py-2.5 bg-gradient-to-r from-accent to-pink-600 text-white rounded-full text-sm font-bold shadow-lg hover:shadow-xl hover:-translate-y-0.5 transition-all">
                                                        "升级会员"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>

                                    </div>

                                    // 4. 返回主页
                                    <div class="pt-4 pb-8">
                                        <A href="/" attr:class="inline-flex items-center px-6 py-3 bg-white border border-gray-200 rounded-full text-gray-500 hover:text-primary hover:border-primary/30 hover:shadow-md transition-all group font-medium">
                                            <i class="fa fa-arrow-left mr-2 group-hover:-translate-x-1 transition-transform"></i>
                                            "返回主页"
                                        </A>
                                    </div>

                                </div>
                            }.into_any(),
                            Some(Ok(None)) => {
                                // 未登录，重定向
                                view! {
                                    <div class="min-h-[60vh] flex flex-col items-center justify-center">
                                        <div class="w-24 h-24 bg-gray-100 rounded-full flex items-center justify-center mb-6 text-gray-300">
                                            <i class="fa fa-user-lock text-4xl"></i>
                                        </div>
                                        <h2 class="text-2xl font-bold mb-2 text-dark">"未登录"</h2>
                                        <p class="text-gray-500 mb-8">"请登录以查看您的个人主页"</p>
                                        <A href="/login" attr:class="bg-primary text-white px-8 py-3 rounded-full hover:bg-primary-focus transition-all shadow-md hover:shadow-lg font-bold">
                                            "去登录"
                                        </A>
                                    </div>
                                }.into_any()
                            },
                            Some(Err(e)) => view! {
                                <div class="text-center py-20 bg-white rounded-xl shadow-soft max-w-lg mx-auto">
                                    <div class="text-red-500 text-6xl mb-4"><i class="fa fa-exclamation-circle"></i></div>
                                    <h2 class="text-2xl font-bold mb-2 text-dark">"加载用户信息失败"</h2>
                                    <p class="text-gray-600 mb-6 bg-gray-100 inline-block px-4 py-2 rounded font-mono text-sm mx-4 break-all">{e.to_string()}</p>
                                    <button
                                        class="text-primary hover:underline font-medium"
                                        on:click=move |_| window().location().reload().unwrap()
                                    >
                                        "刷新页面重试"
                                    </button>
                                </div>
                            }.into_any(),
                            None => view! { <div class="text-center py-20 text-gray-400">"初始化中..."</div> }.into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
