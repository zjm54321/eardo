use crate::api::{
    PostInfo, PostProvider, UserAuthInfo, UserInfo, get_user_profile, login, logout, register,
    update_user_profile,
};
use crate::pages::playground::PostMetadata;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;
use web_sys::FileReader;

#[server]
pub async fn search_posts_info(query: String) -> Result<Vec<PostInfo>, ServerFnError> {
    let post_provider = use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?;

    let post_ids = post_provider
        .search_post(&query)
        .await
        .map_err(|e| ServerFnError::new(format!("搜索帖子失败: {}", e)))?;

    let mut posts = Vec::new();
    for id in post_ids {
        match post_provider.get_post(&id).await {
            Ok(post) => posts.push(post),
            Err(e) => leptos::logging::error!("获取帖子 {} 失败: {}", id, e),
        }
    }

    Ok(posts)
}

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
                let auth_info = UserAuthInfo {
                    username: Some(u),
                    email: None,
                    phone: None,
                };
                match login(auth_info, p).await {
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
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "用户名"
                        </label>
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
                    on:click=move |_| {
                        login_action.dispatch(());
                    }
                    disabled=move || login_action.pending().get()
                >
                    {move || {
                        if login_action.pending().get() { "登录中..." } else { "立即登录" }
                    }}
                </button>

                <div class="text-center text-sm text-gray-500">
                    "还没有账号？ "
                    <A href="/register" attr:class="text-primary hover:underline font-medium">
                        "去注册"
                    </A>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let navigate = use_navigate();
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let register_action = Action::new(move |_| {
        let u = username.get();
        let e = email.get();
        let ph = phone.get();
        let p = password.get();
        let cp = confirm_password.get();
        {
            let value = navigate.clone();
            async move {
                if p != cp {
                    set_error_msg.set(Some("两次输入的密码不一致".to_string()));
                    return;
                }
                let auth_info = UserAuthInfo {
                    username: Some(u),
                    email: if e.is_empty() { None } else { Some(e) },
                    phone: if ph.is_empty() { None } else { Some(ph) },
                };
                match register(auth_info, p).await {
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
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "用户名"
                        </label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="设置用户名"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            prop:value=username
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "邮箱 (可选)"
                        </label>
                        <input
                            type="email"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="输入邮箱地址"
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            prop:value=email
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "手机号 (可选)"
                        </label>
                        <input
                            type="tel"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="输入手机号"
                            on:input=move |ev| set_phone.set(event_target_value(&ev))
                            prop:value=phone
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
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "确认密码"
                        </label>
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
                    on:click=move |_| {
                        register_action.dispatch(());
                    }
                    disabled=move || register_action.pending().get()
                >
                    {move || {
                        if register_action.pending().get() {
                            "创建中..."
                        } else {
                            "注册账号"
                        }
                    }}
                </button>

                <div class="text-center text-sm text-gray-500">
                    "已有账号？ "
                    <A href="/login" attr:class="text-primary hover:underline font-medium">
                        "去登录"
                    </A>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let navigate = use_navigate();
    let user_resource = Resource::new(|| (), |_| get_user_profile());

    // 使用用户ID动态加载用户的作品
    let works_resource = Resource::new(
        move || {
            user_resource
                .get()
                .map(|user_opt| user_opt.ok().map(|u| u.id))
        },
        move |user_id_opt| async move {
            match user_id_opt {
                Some(Some(uid)) => {
                    let query = format!("uid:{}", uid);
                    search_posts_info(query).await
                }
                _ => Ok(Vec::new()),
            }
        },
    );

    // 编辑状态
    let (is_editing, set_is_editing) = signal(false);
    let (edit_nickname, set_edit_nickname) = signal(String::new());
    let (edit_bio, set_edit_bio) = signal(String::new());
    let (new_avatar, set_new_avatar) = signal(Option::<String>::None);

    let logout_action = Action::new(move |_| {
        let value = navigate.clone();
        async move {
            let _ = logout().await;
            value("/login", Default::default());
        }
    });

    let update_action = Action::new(move |_| {
        let nickname = edit_nickname.get();
        let bio = edit_bio.get();
        let avatar_url = new_avatar.get().unwrap_or_default();
        async move {
            // 构造 UserInfo 对象
            let user_info = UserInfo {
                id: "current".to_string(), // 服务端会自动填充当前用户 ID
                username: "".to_string(),  // 保持原用户名
                avatar_url,
                status: crate::api::UserStatus::Normal,
                nickname,
                bio,
                level: 0,
                role: "user".to_string(),
            };
            match update_user_profile(user_info).await {
                Ok(_) => {
                    set_is_editing.set(false);
                    // 刷新用户信息
                    user_resource.refetch();
                }
                Err(e) => leptos::logging::error!("更新失败: {:?}", e),
            }
        }
    });

    // 处理文件选择
    let on_file_change = move |ev: web_sys::Event| {
        // 在 web_sys 中，target() 返回 EventTarget，需要转换
        let target = ev.target().unwrap();
        let input: web_sys::HtmlInputElement = target.unchecked_into();

        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                // 修复乘法类型错误：统一使用浮点数
                if file.size() > 2.0 * 1024.0 * 1024.0 {
                    leptos::logging::warn!("文件太大");
                    return;
                }

                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let file_clone = file.clone();

                let onload = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                    let result = reader_clone.result().unwrap();
                    if let Some(base64) = result.as_string() {
                        set_new_avatar.set(Some(base64));
                    }
                })
                    as Box<dyn Fn()>);

                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                reader.read_as_data_url(&file_clone).unwrap();
                onload.forget(); // 防止被回收
            }
        }
    };

    view! {
        <div class="min-h-screen bg-base-100 pt-20 px-4 pb-20">
            <div class="container mx-auto max-w-6xl">
                <Suspense fallback=move || {
                    view! {
                        <div class="text-center py-10 text-gray-400">"加载用户信息..."</div>
                    }
                }>
                    {move || {
                        match user_resource.get() {
                            Some(Ok(user)) => {
                                let user_bio = user.bio.clone();
                                let user_bio_clone = user_bio.clone();
                                let user_nickname = user.nickname.clone();
                                let user_nickname_clone = user_nickname.clone();
                                let user_username_clone1 = user.username.clone();
                                let user_username_clone2 = user.username.clone();
                                Effect::new(move |_| {
                                    if !is_editing.get_untracked() {
                                        set_edit_bio.set(user_bio_clone.clone());
                                        set_edit_nickname.set(user_nickname_clone.clone());
                                    }
                                });

                                view! {
                                    <div class="flex flex-col items-center animate-fade-in space-y-12">

                                        // 1. Slogan 区域
                                        <div class="text-center space-y-3 mt-4">
                                            <h1 class="text-4xl md:text-6xl font-black text-transparent bg-clip-text bg-gradient-to-r from-primary via-secondary to-accent tracking-tighter uppercase italic drop-shadow-sm">
                                                "Forge Your Unique Voice"
                                            </h1>
                                            <p class="text-gray-400 tracking-[0.5em] text-xs md:text-sm uppercase font-medium">
                                                "Create · Share · Inspire"
                                            </p>
                                        </div>

                                        // 2. 个人信息卡片
                                        <div class="bg-white rounded-3xl p-8 md:p-10 shadow-soft w-full flex flex-col md:flex-row items-center md:items-start gap-8 md:gap-12 border border-gray-100 relative overflow-hidden group">
                                            <div class="absolute -top-24 -right-24 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors duration-500"></div>
                                            <div class="absolute top-1/2 -left-24 w-48 h-48 bg-secondary/5 rounded-full blur-3xl group-hover:bg-secondary/10 transition-colors duration-500"></div>

                                            // 头像区
                                            <div class="relative flex-shrink-0">
                                                <div class="w-32 h-32 md:w-40 md:h-40 rounded-full p-1.5 bg-gradient-to-tr from-primary to-secondary shadow-lg">
                                                    <div class="w-full h-full rounded-full bg-white overflow-hidden relative border-4 border-white">
                                                        // 优先显示新上传的头像预览
                                                        {move || {
                                                            if let Some(preview) = new_avatar.get() {
                                                                view! {
                                                                    <img src=preview class="w-full h-full object-cover" />
                                                                }
                                                                    .into_any()
                                                            } else if !user.avatar_url.is_empty() {
                                                                view! {
                                                                    <img
                                                                        src=user.avatar_url.clone()
                                                                        class="w-full h-full object-cover"
                                                                    />
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! {
                                                                    <div class="w-full h-full flex items-center justify-center bg-gray-50 text-gray-200">
                                                                        <i class="fa-solid fa-user text-6xl"></i>
                                                                    </div>
                                                                }
                                                                    .into_any()
                                                            }
                                                        }}
                                                    </div>
                                                </div>

                                                // 编辑模式下的上传按钮
                                                <Show when=move || is_editing.get()>
                                                    <label class="absolute bottom-2 right-2 w-9 h-9 bg-primary text-white rounded-full shadow-md flex items-center justify-center hover:bg-primary-focus transition-all hover:scale-110 border-2 border-white cursor-pointer z-20">
                                                        <i class="fa-solid fa-camera text-sm"></i>
                                                        <input
                                                            type="file"
                                                            accept="image/*"
                                                            class="hidden"
                                                            on:change=on_file_change
                                                        />
                                                    </label>
                                                </Show>
                                            </div>

                                            // 信息内容
                                            <div class="flex-grow text-center md:text-left space-y-5 z-10 w-full">
                                                <div class="flex flex-col md:flex-row md:items-start justify-between gap-4">
                                                    <div class="flex-1">
                                                        <Show
                                                            when=move || is_editing.get()
                                                            fallback=move || {
                                                                let nickname_display = user_nickname.clone();
                                                                let username_display = user_username_clone1.clone();
                                                                view! {
                                                                    <>
                                                                        <div class="flex items-start gap-2">
                                                                            <h2 class="text-3xl md:text-4xl font-bold text-dark tracking-tight">
                                                                                {if nickname_display.is_empty() {
                                                                                    "未设置昵称".to_string()
                                                                                } else {
                                                                                    nickname_display
                                                                                }}
                                                                            </h2>
                                                                            <button
                                                                                class="text-primary text-sm hover:underline font-medium mt-1"
                                                                                on:click=move |_| set_is_editing.set(true)
                                                                            >
                                                                                <i class="fa-solid fa-pen"></i>
                                                                                "编辑"
                                                                            </button>
                                                                        </div>
                                                                        <div class="flex items-center justify-center md:justify-start gap-2 mt-1">
                                                                            <span class="px-2 py-0.5 bg-gray-100 text-gray-500 text-xs rounded font-mono">
                                                                                "username: " {username_display}
                                                                            </span>
                                                                        </div>
                                                                    </>
                                                                }
                                                            }
                                                        >
                                                            <input
                                                                type="text"
                                                                class="text-3xl md:text-4xl font-bold border border-gray-200 rounded-lg px-3 py-1 focus:outline-none focus:ring-2 focus:ring-primary/50 w-full md:w-auto"
                                                                placeholder="设置昵称"
                                                                prop:value=move || edit_nickname.get()
                                                                on:input=move |ev| {
                                                                    set_edit_nickname.set(event_target_value(&ev))
                                                                }
                                                            />
                                                            <div class="flex items-center justify-center md:justify-start gap-2 mt-1">
                                                                <span class="px-2 py-0.5 bg-gray-100 text-gray-500 text-xs rounded font-mono">
                                                                    "username: " {user_username_clone2.clone()}
                                                                </span>
                                                            </div>
                                                        </Show>
                                                    </div>

                                                    <div class="flex gap-2 justify-center md:justify-end">
                                                        <Show when=move || !is_editing.get()>
                                                            <button
                                                                class="px-5 py-2 bg-white border border-gray-200 text-gray-500 rounded-full hover:bg-gray-50 hover:text-red-500 hover:border-red-200 transition-all shadow-sm text-sm flex items-center gap-2"
                                                                on:click=move |_| {
                                                                    logout_action.dispatch(());
                                                                }
                                                            >
                                                                <i class="fa-solid fa-right-from-bracket"></i>
                                                                "退出登录"
                                                            </button>
                                                        </Show>
                                                    </div>
                                                </div>

                                                // 简介展示与编辑
                                                <div class="bg-gray-50/80 rounded-xl p-4 border border-gray-100 text-gray-600 text-sm leading-relaxed text-left relative min-h-[80px]">
                                                    <i class="fa-solid fa-quote-left text-gray-300 absolute -top-2 -left-2 text-xl"></i>

                                                    <Show
                                                        when=move || is_editing.get()
                                                        fallback=move || {
                                                            let bio_display = user_bio.clone();
                                                            view! {
                                                                <div>
                                                                    <p>
                                                                        {if bio_display.is_empty() {
                                                                            "暂无简介，快来写点什么吧...".to_string()
                                                                        } else {
                                                                            bio_display
                                                                        }}
                                                                    </p>
                                                                </div>
                                                            }
                                                        }
                                                    >
                                                        <div class="space-y-3">
                                                            <textarea
                                                                class="w-full p-2 border border-gray-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 text-sm bg-white"
                                                                rows="3"
                                                                placeholder="介绍一下你自己..."
                                                                prop:value=move || edit_bio.get()
                                                                on:input=move |ev| set_edit_bio.set(event_target_value(&ev))
                                                            ></textarea>
                                                            <div class="flex justify-end gap-2">
                                                                <button
                                                                    class="px-3 py-1 text-xs text-gray-500 hover:bg-gray-200 rounded"
                                                                    on:click=move |_| {
                                                                        set_is_editing.set(false);
                                                                        set_new_avatar.set(None);
                                                                    }
                                                                >
                                                                    "取消"
                                                                </button>
                                                                <button
                                                                    class="px-4 py-1 text-xs bg-primary text-white rounded hover:bg-primary-focus disabled:opacity-50"
                                                                    on:click=move |_| {
                                                                        update_action.dispatch(());
                                                                    }
                                                                    disabled=move || update_action.pending().get()
                                                                >
                                                                    {move || {
                                                                        if update_action.pending().get() {
                                                                            "保存中..."
                                                                        } else {
                                                                            "保存"
                                                                        }
                                                                    }}
                                                                </button>
                                                            </div>
                                                        </div>
                                                    </Show>
                                                </div>
                                            </div>
                                        </div>

                                        // 3. 三栏列表区域 (保持不变)
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 w-full items-start">
                                            // 栏目 1: 我的作品
                                            <div class="space-y-4">
                                                <div class="flex items-center justify-between px-1">
                                                    <h3 class="text-lg font-bold text-dark flex items-center">
                                                        <span class="w-1.5 h-6 bg-secondary rounded-full mr-2"></span>
                                                        "我的作品"
                                                    </h3>
                                                    <A
                                                        href="/voice"
                                                        attr:class="text-xs text-gray-400 hover:text-secondary"
                                                    >
                                                        <i class="fa-solid fa-arrow-right"></i>
                                                    </A>
                                                </div>

                                                <div class="bg-white rounded-2xl shadow-soft border border-gray-100 overflow-hidden min-h-[320px]">
                                                    <Suspense fallback=move || {
                                                        view! {
                                                            <div class="p-4 text-center text-sm text-gray-400">
                                                                "加载作品..."
                                                            </div>
                                                        }
                                                    }>
                                                        {move || {
                                                            match works_resource.get() {
                                                                Some(Ok(works)) if !works.is_empty() => {
                                                                    view! {
                                                                        <ul class="divide-y divide-gray-50">
                                                                            <For
                                                                                each=move || works.clone()
                                                                                key=|w| w.id.clone()
                                                                                children=move |work| {
                                                                                    let meta = PostMetadata::from_post(&work);
                                                                                    view! {
                                                                                        <li class="p-4 hover:bg-gray-50/80 transition-colors cursor-pointer group">
                                                                                            <div class="flex items-center justify-between mb-1">
                                                                                                <span class="font-medium text-dark text-sm group-hover:text-secondary transition-colors truncate pr-2">
                                                                                                    {work.title}
                                                                                                </span>
                                                                                                <span class="text-[10px] text-gray-400 bg-gray-100 px-1.5 py-0.5 rounded shrink-0">
                                                                                                    "公开"
                                                                                                </span>
                                                                                            </div>
                                                                                            <p class="text-xs text-gray-500 line-clamp-2 mt-1 h-8">
                                                                                                {meta.description}
                                                                                            </p>
                                                                                            <div class="flex items-center gap-3 mt-2 text-xs text-gray-400">
                                                                                                <span>
                                                                                                    <i class="fa-solid fa-comments mr-1"></i>
                                                                                                    {meta.comments}
                                                                                                </span>
                                                                                                <span>
                                                                                                    <i class="fa-solid fa-heart mr-1"></i>
                                                                                                    {meta.likes}
                                                                                                </span>
                                                                                                <span class="ml-auto">{meta.time}</span>
                                                                                            </div>
                                                                                        </li>
                                                                                    }
                                                                                }
                                                                            />
                                                                        </ul>
                                                                    }
                                                                        .into_any()
                                                                }
                                                                _ => {
                                                                    view! {
                                                                        <div class="p-8 text-center text-gray-400 flex flex-col items-center justify-center h-full min-h-[200px]">
                                                                            <i class="fa-regular fa-folder-open text-4xl mb-2 opacity-50"></i>
                                                                            <p class="text-sm">"暂无作品"</p>
                                                                        </div>
                                                                    }
                                                                        .into_any()
                                                                }
                                                            }
                                                        }}
                                                    </Suspense>
                                                    <div class="p-3 text-center border-t border-gray-50">
                                                        <button class="text-xs font-medium text-secondary hover:underline">
                                                            "查看全部作品"
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>

                                            // 栏目 2: 我的声音 (模糊处理)
                                            <div class="space-y-4">
                                                <div class="flex items-center justify-between px-1">
                                                    <h3 class="text-lg font-bold text-dark flex items-center">
                                                        <span class="w-1.5 h-6 bg-primary rounded-full mr-2"></span>
                                                        "我的声音"
                                                    </h3>
                                                    <button class="text-xs text-gray-400 hover:text-primary">
                                                        <i class="fa-solid fa-plus"></i>
                                                    </button>
                                                </div>

                                                <div class="bg-white rounded-2xl shadow-soft border border-gray-100 overflow-hidden min-h-[320px] relative group">
                                                    // 模糊内容
                                                    <div class="filter blur-[4px] opacity-60 select-none pointer-events-none p-2">
                                                        <ul class="divide-y divide-gray-50">
                                                            {(1..=5)
                                                                .map(|i| {
                                                                    view! {
                                                                        <li class="flex items-center p-3">
                                                                            <div class="w-10 h-10 rounded-xl bg-gray-100 flex items-center justify-center text-gray-400 mr-3 shrink-0">
                                                                                <i class="fa-solid fa-music text-sm"></i>
                                                                            </div>
                                                                            <div class="flex-grow min-w-0">
                                                                                <div class="font-medium text-dark text-sm truncate">
                                                                                    "录音_2025050" {i} ".mp3"
                                                                                </div>
                                                                                <div class="text-xs text-gray-400 mt-0.5">
                                                                                    "00:45 • 1.2MB"
                                                                                </div>
                                                                            </div>
                                                                        </li>
                                                                    }
                                                                })
                                                                .collect::<Vec<_>>()}
                                                        </ul>
                                                    </div>

                                                    // 覆盖层
                                                    <div class="absolute inset-0 flex flex-col items-center justify-center bg-white/40 backdrop-blur-[1px] p-6 text-center z-10">
                                                        <div class="w-12 h-12 bg-white rounded-full shadow-md flex items-center justify-center mb-3 text-primary">
                                                            <i class="fa-solid fa-folder text-xl"></i>
                                                        </div>
                                                        <h4 class="font-bold text-dark text-sm mb-1">
                                                            "私有声音库"
                                                        </h4>
                                                        <p class="text-gray-500 text-xs">"功能开发中..."</p>
                                                    </div>
                                                </div>
                                            </div>

                                            // 栏目 3: 声音分析 (模糊处理 - 未完成)
                                            <div class="space-y-4">
                                                <div class="flex items-center justify-between px-1">
                                                    <h3 class="text-lg font-bold text-dark flex items-center">
                                                        <span class="w-1.5 h-6 bg-accent rounded-full mr-2"></span>
                                                        "声音分析"
                                                    </h3>
                                                </div>

                                                <div class="bg-white rounded-2xl p-0 shadow-soft min-h-[320px] border border-gray-100 relative overflow-hidden group">
                                                    // 背景内容 (被模糊)
                                                    <div class="p-5 space-y-4 filter blur-[6px] opacity-60 select-none pointer-events-none transform scale-105">
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

                                                    // 覆盖层 - 未完成提示
                                                    <div class="absolute inset-0 flex flex-col items-center justify-center bg-white/40 backdrop-blur-[2px] p-6 text-center z-10 transition-all">
                                                        <div class="w-16 h-16 bg-white rounded-2xl shadow-xl flex items-center justify-center mb-5 text-gray-400">
                                                            <i class="fa-solid fa-wrench text-2xl"></i>
                                                        </div>
                                                        <h4 class="font-bold text-dark text-lg mb-2">
                                                            "功能开发中"
                                                        </h4>
                                                        <p class="text-gray-500 text-sm mb-6 max-w-[200px] leading-relaxed">
                                                            "正在为您构建专业的 AI 声音分析模型，敬请期待。"
                                                        </p>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // 4. 返回主页
                                        <div class="pt-4 pb-8">
                                            <A
                                                href="/"
                                                attr:class="inline-flex items-center px-6 py-3 bg-white border border-gray-200 rounded-full text-gray-500 hover:text-primary hover:border-primary/30 hover:shadow-md transition-all group font-medium"
                                            >
                                                <i class="fa-solid fa-arrow-left mr-2 group-hover:-translate-x-1 transition-transform"></i>
                                                "返回主页"
                                            </A>
                                        </div>

                                    </div>
                                }
                                    .into_any()
                            }
                            Some(Err(e)) => {
                                view! {
                                    <div class="min-h-[60vh] flex flex-col items-center justify-center">
                                        <div class="w-24 h-24 bg-gray-100 rounded-full flex items-center justify-center mb-6 text-gray-300">
                                            <i class="fa-solid fa-lock text-4xl"></i>
                                        </div>
                                        <h2 class="text-2xl font-bold mb-2 text-dark">
                                            "加载失败"
                                        </h2>
                                        <p class="text-gray-500 mb-8">{format!("错误: {}", e)}</p>
                                        <A
                                            href="/login"
                                            attr:class="bg-primary text-white px-8 py-3 rounded-full hover:bg-primary-focus transition-all shadow-md hover:shadow-lg font-bold"
                                        >
                                            "去登录"
                                        </A>
                                    </div>
                                }
                                    .into_any()
                            }
                            _ => {
                                view! {
                                    <div class="min-h-[60vh] flex flex-col items-center justify-center">
                                        <div class="w-24 h-24 bg-gray-100 rounded-full flex items-center justify-center mb-6 text-gray-300">
                                            <i class="fa-solid fa-lock text-4xl"></i>
                                        </div>
                                        <h2 class="text-2xl font-bold mb-2 text-dark">
                                            "未登录"
                                        </h2>
                                        <p class="text-gray-500 mb-8">
                                            "请登录以查看您的个人主页"
                                        </p>
                                        <A
                                            href="/login"
                                            attr:class="bg-primary text-white px-8 py-3 rounded-full hover:bg-primary-focus transition-all shadow-md hover:shadow-lg font-bold"
                                        >
                                            "去登录"
                                        </A>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
