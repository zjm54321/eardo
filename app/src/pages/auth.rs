use std::str::FromStr;

use crate::api::post::VoicePost;
#[cfg(feature = "ssr")]
use crate::api::post::search_voice_post;
use crate::api::user::{
    self, get_user_profile, login, logout, register, update_user_avatar, update_user_profile,
};
use crate::pages::playground::PostMetadata;
use base64::Engine;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use phonenumber::country::Id::CN;
use sha2::{Digest, Sha256};
use wasm_bindgen::JsCast;
use web_sys::{FileReader, HtmlCanvasElement, HtmlImageElement};

#[server]
pub async fn search_posts_info(query: String) -> Result<Vec<VoicePost>, ServerFnError> {
    search_voice_post(query).await
}

#[component]
pub fn LoginPage() -> impl IntoView {
    use email_address::EmailAddress;
    let navigate = use_navigate();
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (show_password, set_show_password) = signal(false);

    let login_action = Action::new(move |_| {
        let u = username.get();
        let p = password.get();
        {
            let value = navigate.clone();
            async move {
                // 每次尝试登录时先清空错误
                set_error_msg.set(None);

                let u = u.trim().to_string();
                if u.is_empty() || p.is_empty() {
                    set_error_msg.set(Some("请输入账号和密码".to_string()));
                    return;
                }

                let mut hasher = Sha256::new();
                hasher.update(p.as_bytes());
                let password_hash = format!("{:x}", hasher.finalize());

                let auth_id = if EmailAddress::is_valid(&u) {
                    match EmailAddress::from_str(&u) {
                        Ok(email) => Some(user::AuthID::Email(email)),
                        Err(_) => None,
                    }
                } else {
                    phonenumber::parse(Some(CN), &u)
                        .or_else(|_| phonenumber::parse(None, &u))
                        .ok()
                        .map(user::AuthID::Phone)
                };

                let Some(auth_id) = auth_id else {
                    set_error_msg.set(Some("请输入有效的邮箱或手机号".to_string()));
                    return;
                };

                let auth_info = user::UserAuth::Password(user::PasswordAuth {
                    auth_id,
                    password_hash,
                });

                match login(auth_info).await {
                    Ok(_) => {
                        value("/profile", Default::default());
                    }
                    Err(e) => set_error_msg.set(Some(e.to_string())),
                }
            }
        }
    });

    // 提取密码输入框组件，减少主组件的重渲染
    let password_input = view! {
        <div class="relative">
            <input
                type=move || if show_password.get() { "text" } else { "password" }
                class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all pr-10"
                placeholder="请输入密码"
                on:input=move |ev| set_password.set(event_target_value(&ev))
                prop:value=password
            />
            <button
                type="button"
                class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600"
                on:click=move |_| set_show_password.update(|s| *s = !*s)
            >
                <Show
                    when=move || show_password.get()
                    fallback=move || {
                        view! {
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke-width="1.5"
                                stroke="currentColor"
                                class="w-5 h-5"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z"
                                />
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                />
                            </svg>
                        }
                    }
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-5 h-5"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88"
                        />
                    </svg>
                </Show>
            </button>
        </div>
    };

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
                            "邮箱/手机号"
                        </label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                            placeholder="请输入邮箱或手机号"
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            prop:value=username
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"密码"</label>
                        {password_input}
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
    use email_address::EmailAddress;
    let navigate = use_navigate();
    // 注册方式：邮箱 / 手机号（二选一）
    let (register_method, set_register_method) = signal(String::from("phone"));
    let (email, set_email) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (show_password, set_show_password) = signal(false);
    let (show_confirm_password, set_show_confirm_password) = signal(false);

    let password_strength = move || {
        let p = password.get();
        if p.is_empty() {
            return 0;
        }
        let has_lower = p.chars().any(|c| c.is_ascii_lowercase());
        let has_upper = p.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = p.chars().any(|c| c.is_ascii_digit());
        let has_special = p.chars().any(|c| c.is_ascii_punctuation() || c.is_ascii());

        let types = has_lower as i32 + has_upper as i32 + has_digit as i32 + has_special as i32;

        if p.len() < 8 || types < 2 {
            1 // Weak
        } else if types == 2 {
            2 // Medium
        } else {
            3 // Strong
        }
    };

    let register_action = Action::new(move |_| {
        let method = register_method.get();
        let e = email.get();
        let ph = phone.get();
        let p = password.get();
        let cp = confirm_password.get();
        {
            let value = navigate.clone();
            async move {
                // 每次尝试注册时先清空错误
                set_error_msg.set(None);

                let p = p.trim().to_string();
                let cp = cp.trim().to_string();

                if p.is_empty() {
                    set_error_msg.set(Some("请输入密码".to_string()));
                    return;
                }
                if p.len() > 32 || !p.is_ascii() {
                    set_error_msg.set(Some("密码只能包含0-32位英文字母、数字或符号".to_string()));
                    return;
                }
                if p != cp {
                    set_error_msg.set(Some("两次输入的密码不一致".to_string()));
                    return;
                }

                let mut hasher = Sha256::new();
                hasher.update(p.as_bytes());
                let password_hash = format!("{:x}", hasher.finalize());

                let auth_id = match method.as_str() {
                    "email" => {
                        let e = e.trim().to_string();
                        if EmailAddress::is_valid(&e) {
                            EmailAddress::from_str(&e).ok().map(user::AuthID::Email)
                        } else {
                            None
                        }
                    }
                    "phone" => {
                        let ph = ph.trim().to_string();
                        if ph.is_empty() {
                            None
                        } else {
                            phonenumber::parse(None, &ph)
                                .or_else(|_| phonenumber::parse(None, &ph))
                                .ok()
                                .map(user::AuthID::Phone)
                        }
                    }
                    _ => None,
                };

                let Some(auth_id) = auth_id else {
                    set_error_msg.set(Some(match method.as_str() {
                        "email" => "请输入有效的邮箱".to_string(),
                        "phone" => "请输入有效的手机号".to_string(),
                        _ => "请选择注册方式".to_string(),
                    }));
                    return;
                };

                let userauth = user::UserAuth::Password(user::PasswordAuth {
                    auth_id,
                    password_hash,
                });

                match register(userauth).await {
                    Ok(_) => {
                        // 注册成功跳转登录
                        value("/login", Default::default());
                    }
                    Err(e) => set_error_msg.set(Some(e.to_string())),
                }
            }
        }
    });

    // 提取密码输入框组件，减少主组件的重渲染
    let password_input = view! {
        <div class="relative">
            <input
                type=move || if show_password.get() { "text" } else { "password" }
                class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all pr-10"
                placeholder="设置密码"
                maxlength="32"
                on:input=move |ev| set_password.set(event_target_value(&ev))
                prop:value=password
            />
            <button
                type="button"
                class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600"
                on:click=move |_| set_show_password.update(|s| *s = !*s)
            >
                <Show
                    when=move || show_password.get()
                    fallback=move || {
                        view! {
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke-width="1.5"
                                stroke="currentColor"
                                class="w-5 h-5"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z"
                                />
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                />
                            </svg>
                        }
                    }
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-5 h-5"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88"
                        />
                    </svg>
                </Show>
            </button>
        </div>
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-base-100 px-4">
            <div class="max-w-md w-full bg-white rounded-xl shadow-soft p-8 space-y-6 border border-gray-100">
                <div class="text-center">
                    <h2 class="text-3xl font-bold text-dark">"创建账号"</h2>
                    <p class="text-gray-500 mt-2">"加入白昼聆夏，开启声音之旅"</p>
                </div>

                <div class="space-y-4">
                    <Show
                        when=move || register_method.get() == "email"
                        fallback=move || {
                            view! {
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "手机号"
                                    </label>
                                    <input
                                        type="tel"
                                        class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                                        placeholder="请输入手机号"
                                        on:input=move |ev| set_phone.set(event_target_value(&ev))
                                        prop:value=phone
                                    />
                                </div>
                            }
                        }
                    >
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "邮箱"
                            </label>
                            <input
                                type="email"
                                class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all"
                                placeholder="请输入邮箱地址"
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                prop:value=email
                            />
                        </div>
                    </Show>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"密码"</label>
                        {password_input}

                        // 密码强度指示器
                        <Show
                            when=move || !password.get().is_empty()
                            fallback=move || {
                                view! {
                                    <div class="text-xs text-gray-400 mt-1.5">
                                        "密码只能包含0-32位英文字母、数字或符号"
                                    </div>
                                }
                            }
                        >
                            <div class="mt-2 flex gap-1 h-1.5">
                                <div class=move || {
                                    format!(
                                        "flex-1 rounded-full transition-colors {}",
                                        if password_strength() >= 1 {
                                            "bg-red-400"
                                        } else {
                                            "bg-gray-200"
                                        },
                                    )
                                }></div>
                                <div class=move || {
                                    format!(
                                        "flex-1 rounded-full transition-colors {}",
                                        if password_strength() >= 2 {
                                            "bg-yellow-400"
                                        } else {
                                            "bg-gray-200"
                                        },
                                    )
                                }></div>
                                <div class=move || {
                                    format!(
                                        "flex-1 rounded-full transition-colors {}",
                                        if password_strength() >= 3 {
                                            "bg-green-400"
                                        } else {
                                            "bg-gray-200"
                                        },
                                    )
                                }></div>
                            </div>
                            <div class="text-xs mt-1.5 flex justify-between">
                                <span class="text-gray-500">
                                    "密码只能包含0-32位英文字母、数字或符号"
                                </span>
                                <span class=move || match password_strength() {
                                    1 => "text-red-500 font-medium",
                                    2 => "text-yellow-500 font-medium",
                                    3 => "text-green-500 font-medium",
                                    _ => "text-gray-500",
                                }>
                                    {move || match password_strength() {
                                        1 => "弱",
                                        2 => "中",
                                        3 => "强",
                                        _ => "",
                                    }}
                                </span>
                            </div>
                        </Show>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "确认密码"
                        </label>
                        <div class="relative">
                            <input
                                type=move || {
                                    if show_confirm_password.get() { "text" } else { "password" }
                                }
                                class="w-full px-4 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary/50 focus:outline-none transition-all pr-10"
                                placeholder="请再次输入密码"
                                maxlength="32"
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                prop:value=confirm_password
                            />
                            <button
                                type="button"
                                class="absolute inset-y-0 right-0 pr-3 flex items-center text-gray-400 hover:text-gray-600"
                                on:click=move |_| set_show_confirm_password.update(|s| *s = !*s)
                            >
                                <Show
                                    when=move || show_confirm_password.get()
                                    fallback=move || {
                                        view! {
                                            <svg
                                                xmlns="http://www.w3.org/2000/svg"
                                                fill="none"
                                                viewBox="0 0 24 24"
                                                stroke-width="1.5"
                                                stroke="currentColor"
                                                class="w-5 h-5"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z"
                                                />
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                                />
                                            </svg>
                                        }
                                    }
                                >
                                    <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke-width="1.5"
                                        stroke="currentColor"
                                        class="w-5 h-5"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88"
                                        />
                                    </svg>
                                </Show>
                            </button>
                        </div>
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

                <div class="flex justify-between items-center text-sm text-gray-500 px-1">
                    <button
                        type="button"
                        class="text-primary hover:underline font-medium"
                        on:click=move |_| {
                            if register_method.get() == "phone" {
                                set_register_method.set("email".to_string());
                            } else {
                                set_register_method.set("phone".to_string());
                            }
                            set_error_msg.set(None);
                        }
                    >
                        {move || {
                            if register_method.get() == "phone" {
                                "使用邮箱注册"
                            } else {
                                "使用手机号注册"
                            }
                        }}
                    </button>
                    <div>
                        "已有账号？ "
                        <A href="/login" attr:class="text-primary hover:underline font-medium">
                            "去登录"
                        </A>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProfileCard(user: crate::api::user::User) -> impl IntoView {
    let user_bio = user.usermeta.bio.clone();
    let user_nickname = user.usermeta.nick_name.clone();
    let navigate = use_navigate();
    let logout_action = Action::new(move |_| {
        let value = navigate.clone();
        async move {
            let _ = crate::api::user::logout().await;
            value("/login", Default::default());
        }
    });

    view! {
        <div class="bg-white rounded-3xl p-8 md:p-10 shadow-soft w-full flex flex-col md:flex-row items-center md:items-start gap-8 md:gap-12 border border-gray-100 relative overflow-hidden group">
            <div class="absolute -top-24 -right-24 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors duration-500"></div>
            <div class="absolute top-1/2 -left-24 w-48 h-48 bg-secondary/5 rounded-full blur-3xl group-hover:bg-secondary/10 transition-colors duration-500"></div>

            // 头像区
            <div class="relative flex-shrink-0">
                <div class="w-32 h-32 md:w-40 md:h-40 rounded-full p-1.5 bg-gradient-to-tr from-primary to-secondary shadow-lg">
                    <div class="w-full h-full rounded-full bg-white overflow-hidden relative border-4 border-white">
                        {if !user.usermeta.avatar_url.is_empty() {
                            view! {
                                <img
                                    src=format!(
                                        "{}?t={}",
                                        user.usermeta.avatar_url,
                                        {
                                            #[cfg(target_arch = "wasm32")] { js_sys::Date::now() }
                                            #[cfg(not(target_arch = "wasm32"))] { 0.0 }
                                        },
                                    )
                                    loading="lazy"
                                    decoding="async"
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
                        }}
                    </div>
                </div>
            </div>

            // 信息内容
            <div class="flex-grow text-center md:text-left space-y-5 z-10 w-full">
                <div class="flex flex-col md:flex-row md:items-start justify-between gap-4">
                    <div class="flex-1">
                        <div class="flex items-start gap-2">
                            <h2 class="text-3xl md:text-4xl font-bold text-dark tracking-tight">
                                {if user_nickname.is_empty() {
                                    "未设置昵称".to_string()
                                } else {
                                    user_nickname
                                }}
                            </h2>
                        </div>
                    </div>

                    <div class="flex gap-2 justify-center md:justify-end">
                        <button
                            class="px-5 py-2 bg-white border border-gray-200 text-gray-500 rounded-full hover:bg-gray-50 hover:text-red-500 hover:border-red-200 transition-all shadow-sm text-sm flex items-center gap-2"
                            on:click=move |_| {
                                logout_action.dispatch(());
                            }
                        >
                            <i class="fa-solid fa-right-from-bracket"></i>
                            "退出登录"
                        </button>
                    </div>
                </div>

                // 简介展示
                <div class="bg-gray-50/80 rounded-xl p-4 border border-gray-100 text-gray-600 text-sm leading-relaxed text-left relative min-h-[80px]">
                    <i class="fa-solid fa-quote-left text-gray-300 absolute -top-2 -left-2 text-xl"></i>
                    <div>
                        <p>
                            {if user_bio.is_empty() {
                                "暂无简介，快来写点什么吧...".to_string()
                            } else {
                                user_bio
                            }}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn WorksList(
    works_resource: Resource<Result<Vec<VoicePost>, leptos::prelude::ServerFnError>>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="flex items-center justify-between px-1">
                <h3 class="text-lg font-bold text-dark flex items-center">
                    <span class="w-1.5 h-6 bg-secondary rounded-full mr-2"></span>
                    "我的作品"
                </h3>
                <A href="/voice" attr:class="text-xs text-gray-400 hover:text-secondary">
                    <i class="fa-solid fa-arrow-right"></i>
                </A>
            </div>

            <div class="bg-white rounded-2xl shadow-soft border border-gray-100 overflow-hidden min-h-[320px]">
                <Suspense fallback=move || {
                    view! {
                        <div class="p-4 text-center text-sm text-gray-400">"加载作品..."</div>
                    }
                }>
                    {move || {
                        match works_resource.get() {
                            Some(Ok(works)) => {
                                if !works.is_empty() {
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
                                } else {
                                    view! {
                                        <div class="p-8 text-center text-gray-400 flex flex-col items-center justify-center h-full min-h-[200px]">
                                            <i class="fa-regular fa-folder-open text-4xl mb-2 opacity-50"></i>
                                            <p class="text-sm">"暂无作品"</p>
                                        </div>
                                    }
                                        .into_any()
                                }
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
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user_resource = Resource::new(|| (), |_| get_user_profile());

    // 使用闭包获取用户信息，避免 Memo 的 PartialEq 限制
    let user_info = move || user_resource.get().and_then(|res| res.ok());

    // 使用用户ID动态加载用户的作品
    let works_resource = Resource::new(
        move || user_info().map(|u| u.id),
        move |user_id_opt| async move {
            match user_id_opt {
                Some(uid) => {
                    let query = format!("uid:{}", uid);
                    search_posts_info(query).await
                }
                _ => Ok(Vec::new()),
            }
        },
    );

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
                                        <ProfileCard user=user />

                                        // 3. 三栏列表区域 (保持不变)
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 w-full items-start">
                                            // 栏目 1: 我的作品
                                            <WorksList works_resource=works_resource.clone() />

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
                                    <div class="text-center py-10 text-red-500">
                                        {format!("加载失败: {}", e)}
                                    </div>
                                }
                                    .into_any()
                            }
                            None => {
                                match user_resource.get() {
                                    Some(Err(e)) => {
                                        view! {
                                            <div class="text-center py-10 text-red-500">
                                                {format!("加载失败: {}", e)}
                                            </div>
                                        }
                                            .into_any()
                                    }
                                    _ => {
                                        view! {
                                            <div class="text-center py-10">"请先登录"</div>
                                        }
                                            .into_any()
                                    }
                                }
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let navigate = use_navigate();
    let user_resource = Resource::new(|| (), |_| get_user_profile());
    let auth_resource = Resource::new(|| (), |_| crate::api::user::get_authinfo());

    // 0: 个人信息, 1: 登录设置
    let (active_tab, set_active_tab) = signal(0);

    let (edit_nickname, set_edit_nickname) = signal(String::new());
    let (edit_bio, set_edit_bio) = signal(String::new());
    let (new_avatar, set_new_avatar) = signal(Option::<String>::None);
    let (new_avatar_webp, set_new_avatar_webp) = signal(Option::<Vec<u8>>::None);

    Effect::new(move |_| {
        if let Some(Ok(user)) = user_resource.get() {
            set_edit_nickname.set(user.usermeta.nick_name.clone());
            set_edit_bio.set(user.usermeta.bio.clone());
        }
    });

    // 绑定/修改状态
    let (show_bind_modal, set_show_bind_modal) = signal(false);
    let (bind_type, set_bind_type) = signal(String::new()); // "phone", "email", "password"
    let (bind_action_type, set_bind_action_type) = signal(String::new()); // "add", "change"
    let (bind_input, set_bind_input) = signal(String::new());
    let (bind_password, set_bind_password) = signal(String::new());
    let (bind_error, set_bind_error) = signal(Option::<String>::None);

    let bind_action = Action::new(move |_| {
        let b_type = bind_type.get();
        let b_action_type = bind_action_type.get();
        let input_val = bind_input.get().trim().to_string();
        let pwd_val = bind_password.get();

        async move {
            set_bind_error.set(None);

            let auth_info = match b_type.as_str() {
                "password" => {
                    if input_val.is_empty() {
                        set_bind_error.set(Some("请输入旧密码".to_string()));
                        return;
                    }
                    if pwd_val.is_empty() {
                        set_bind_error.set(Some("请输入新密码".to_string()));
                        return;
                    }

                    let mut hasher_old = Sha256::new();
                    hasher_old.update(input_val.as_bytes());
                    let old_password_hash = format!("{:x}", hasher_old.finalize());

                    let mut hasher_new = Sha256::new();
                    hasher_new.update(pwd_val.as_bytes());
                    let new_password_hash = format!("{:x}", hasher_new.finalize());

                    user::UpdateAuthInfo::ChangePassword(old_password_hash, new_password_hash)
                }
                _ => {
                    let auth_id = match b_type.as_str() {
                        "phone" => {
                            if input_val.is_empty() {
                                set_bind_error.set(Some("请输入手机号".to_string()));
                                return;
                            }
                            match phonenumber::parse(Some(CN), &input_val)
                                .or_else(|_| phonenumber::parse(None, &input_val))
                            {
                                Ok(p) => user::AuthID::Phone(p),
                                Err(_) => {
                                    set_bind_error.set(Some("手机号格式不正确".to_string()));
                                    return;
                                }
                            }
                        }
                        "email" => {
                            if input_val.is_empty() {
                                set_bind_error.set(Some("请输入邮箱".to_string()));
                                return;
                            }
                            match email_address::EmailAddress::from_str(&input_val) {
                                Ok(e) => user::AuthID::Email(e),
                                Err(_) => {
                                    set_bind_error.set(Some("邮箱格式不正确".to_string()));
                                    return;
                                }
                            }
                        }
                        _ => return,
                    };

                    if b_action_type == "change" {
                        user::UpdateAuthInfo::ChangePassAuth(auth_id)
                    } else {
                        user::UpdateAuthInfo::AddAuth(user::UserAuth::Password(
                            user::PasswordAuth {
                                auth_id,
                                password_hash: String::new(), // 后端会从数据库获取已有的密码哈希
                            },
                        ))
                    }
                }
            };

            match crate::api::user::update_authinfo(auth_info).await {
                Ok(_) => {
                    set_show_bind_modal.set(false);
                    set_bind_input.set(String::new());
                    set_bind_password.set(String::new());
                    auth_resource.refetch();
                    let _ = web_sys::window().unwrap().alert_with_message("操作成功");
                }
                Err(e) => set_bind_error.set(Some(e.to_string())),
            }
        }
    });

    let remove_auth_action = Action::new(move |auth_id: &user::AuthID| {
        let auth_id = auth_id.clone();
        async move {
            match crate::api::user::update_authinfo(user::UpdateAuthInfo::RemoveAuth(auth_id)).await
            {
                Ok(_) => {
                    auth_resource.refetch();
                    let _ = web_sys::window().unwrap().alert_with_message("解绑成功");
                }
                Err(e) => {
                    let _ = web_sys::window()
                        .unwrap()
                        .alert_with_message(&format!("解绑失败: {}", e));
                }
            }
        }
    });

    let update_action = {
        let navigate = navigate.clone();
        Action::new(move |_| {
            let nickname = edit_nickname.get();
            let bio = edit_bio.get();
            let avatar_webp = new_avatar_webp.get();
            let value = navigate.clone();
            async move {
                let Some(Ok(current_user)) = user_resource.get_untracked() else {
                    leptos::logging::error!("更新失败: 未获取到当前用户信息");
                    return;
                };

                let mut updated_user = current_user.clone();
                updated_user.usermeta.nick_name = nickname;
                updated_user.usermeta.bio = bio;

                if let Err(e) = update_user_profile(updated_user).await {
                    leptos::logging::error!("更新用户信息失败: {:?}", e);
                    return;
                }

                if let Some(webp_bytes) = avatar_webp {
                    if let Err(e) = update_user_avatar(webp_bytes).await {
                        leptos::logging::error!("更新头像失败: {:?}", e);
                        return;
                    }
                }

                // 刷新用户信息并返回主页
                user_resource.refetch();
                value("/profile", Default::default());
            }
        })
    };

    let on_file_change = move |ev: web_sys::Event| {
        let target = ev.target().unwrap();
        let input: web_sys::HtmlInputElement = target.unchecked_into();

        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
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
                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();

                        let canvas: HtmlCanvasElement =
                            document.create_element("canvas").unwrap().unchecked_into();
                        canvas.set_width(200);
                        canvas.set_height(200);
                        let ctx = canvas
                            .get_context("2d")
                            .unwrap()
                            .unwrap()
                            .unchecked_into::<web_sys::CanvasRenderingContext2d>();

                        let img = HtmlImageElement::new().unwrap();
                        let img_clone = img.clone();

                        let on_img_load = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            let sw = img_clone.natural_width() as f64;
                            let sh = img_clone.natural_height() as f64;
                            if sw <= 0.0 || sh <= 0.0 {
                                leptos::logging::error!("图片尺寸无效");
                                return;
                            }

                            let tw = 200.0;
                            let th = 200.0;
                            let scale = (tw / sw).max(th / sh);
                            let dw = sw * scale;
                            let dh = sh * scale;
                            let dx = (tw - dw) / 2.0;
                            let dy = (th - dh) / 2.0;

                            ctx.clear_rect(0.0, 0.0, tw, th);
                            if let Err(e) = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                                &img_clone, dx, dy, dw, dh,
                            ) {
                                leptos::logging::error!("绘制头像失败: {:?}", e);
                                return;
                            }

                            let webp_data_url = match canvas.to_data_url_with_type("image/webp") {
                                Ok(v) => v,
                                Err(e) => {
                                    leptos::logging::error!("导出 WebP 失败: {:?}", e);
                                    return;
                                }
                            };

                            set_new_avatar.set(Some(webp_data_url.clone()));

                            let Some((_, b64)) = webp_data_url.split_once(',') else {
                                leptos::logging::error!("WebP data url 格式不正确");
                                return;
                            };

                            let webp_bytes =
                                match base64::engine::general_purpose::STANDARD.decode(b64) {
                                    Ok(bytes) => bytes,
                                    Err(e) => {
                                        leptos::logging::error!("WebP base64 解码失败: {:?}", e);
                                        return;
                                    }
                                };

                            set_new_avatar_webp.set(Some(webp_bytes));
                        })
                            as Box<dyn Fn()>);

                        img.set_onload(Some(on_img_load.as_ref().unchecked_ref()));
                        img.set_src(&base64);
                        on_img_load.forget();
                    }
                })
                    as Box<dyn Fn()>);

                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                reader.read_as_data_url(&file_clone).unwrap();
                onload.forget();
            }
        }
    };

    view! {
        <div class="min-h-screen bg-base-100 pt-20 px-4 pb-20">
            <div class="container mx-auto max-w-4xl">
                <Suspense fallback=move || {
                    view! {
                        <div class="text-center py-10 text-gray-400">"加载用户信息..."</div>
                    }
                }>
                    {move || {
                        match user_resource.get() {
                            Some(Ok(user)) => {
                                view! {
                                    <div class="bg-white rounded-3xl shadow-soft w-full flex flex-col md:flex-row border border-gray-100 overflow-hidden min-h-[600px]">
                                        // 左侧菜单
                                        <div class="w-full md:w-64 bg-gray-50/50 border-r border-gray-100 p-6 flex flex-col gap-2">
                                            <h2 class="text-xl font-bold text-dark mb-4 px-4">
                                                "设置"
                                            </h2>
                                            <button
                                                class=move || {
                                                    format!(
                                                        "text-left px-4 py-3 rounded-xl transition-colors font-medium {}",
                                                        if active_tab.get() == 0 {
                                                            "bg-primary/10 text-primary"
                                                        } else {
                                                            "text-gray-600 hover:bg-gray-100"
                                                        },
                                                    )
                                                }
                                                on:click=move |_| set_active_tab.set(0)
                                            >
                                                <i class="fa-solid fa-user mr-3 w-5 text-center"></i>
                                                "个人信息"
                                            </button>
                                            <button
                                                class=move || {
                                                    format!(
                                                        "text-left px-4 py-3 rounded-xl transition-colors font-medium {}",
                                                        if active_tab.get() == 1 {
                                                            "bg-primary/10 text-primary"
                                                        } else {
                                                            "text-gray-600 hover:bg-gray-100"
                                                        },
                                                    )
                                                }
                                                on:click=move |_| set_active_tab.set(1)
                                            >
                                                <i class="fa-solid fa-shield-halved mr-3 w-5 text-center"></i>
                                                "登录设置"
                                            </button>
                                            <div class="flex-grow"></div>
                                            <button
                                                class="text-left px-4 py-3 rounded-xl transition-colors font-medium text-red-500 hover:bg-red-50 mt-auto"
                                                on:click={
                                                    let navigate = navigate.clone();
                                                    move |_| {
                                                        let navigate = navigate.clone();
                                                        leptos::task::spawn_local(async move {
                                                            let _ = logout().await;
                                                            navigate("/", Default::default());
                                                        });
                                                    }
                                                }
                                            >
                                                <i class="fa-solid fa-right-from-bracket mr-3 w-5 text-center"></i>
                                                "退出登录"
                                            </button>
                                        </div>

                                        // 右侧内容区
                                        <div class="flex-1 p-8 md:p-10">
                                            <Show when=move || active_tab.get() == 0>
                                                <div class="max-w-xl flex flex-col gap-8 animate-fade-in">
                                                    <h3 class="text-2xl font-bold text-dark">"个人信息"</h3>

                                                    <div class="flex flex-col items-start gap-4">
                                                        <div class="relative w-32 h-32 rounded-full p-1.5 bg-gradient-to-tr from-primary to-secondary shadow-lg">
                                                            <div class="w-full h-full rounded-full bg-white overflow-hidden relative border-4 border-white">
                                                                {
                                                                    let avatar_url = user.usermeta.avatar_url.clone();
                                                                    move || {
                                                                        if let Some(preview) = new_avatar.get() {
                                                                            view! {
                                                                                <img
                                                                                    src=preview
                                                                                    class="w-full h-full object-cover"
                                                                                    loading="lazy"
                                                                                    decoding="async"
                                                                                />
                                                                            }
                                                                                .into_any()
                                                                        } else if !avatar_url.is_empty() {
                                                                            view! {
                                                                                <img
                                                                                    src=format!(
                                                                                        "{}?t={}",
                                                                                        avatar_url,
                                                                                        {
                                                                                            #[cfg(target_arch = "wasm32")] { js_sys::Date::now() }
                                                                                            #[cfg(not(target_arch = "wasm32"))] { 0.0 }
                                                                                        },
                                                                                    )
                                                                                    class="w-full h-full object-cover"
                                                                                    loading="lazy"
                                                                                    decoding="async"
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
                                                                    }
                                                                }
                                                            </div>
                                                            <label class="absolute bottom-0 right-0 w-10 h-10 bg-primary text-white rounded-full shadow-md flex items-center justify-center hover:bg-primary-focus transition-all hover:scale-110 border-2 border-white cursor-pointer z-20">
                                                                <i class="fa-solid fa-camera"></i>
                                                                <input
                                                                    type="file"
                                                                    accept="image/*"
                                                                    class="hidden"
                                                                    on:change=on_file_change
                                                                />
                                                            </label>
                                                        </div>
                                                        <span class="text-sm text-gray-500">
                                                            "点击更换头像"
                                                        </span>
                                                    </div>

                                                    <div class="space-y-5">
                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                                                "昵称"
                                                            </label>
                                                            <input
                                                                type="text"
                                                                class="w-full p-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/50 bg-gray-50/50 focus:bg-white transition-colors"
                                                                placeholder="设置昵称"
                                                                prop:value=move || edit_nickname.get()
                                                                on:input=move |ev| {
                                                                    set_edit_nickname.set(event_target_value(&ev))
                                                                }
                                                            />
                                                        </div>

                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                                                "个人简介"
                                                            </label>
                                                            <textarea
                                                                class="w-full p-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/50 bg-gray-50/50 focus:bg-white transition-colors"
                                                                rows="4"
                                                                placeholder="介绍一下你自己..."
                                                                prop:value=move || edit_bio.get()
                                                                on:input=move |ev| set_edit_bio.set(event_target_value(&ev))
                                                            ></textarea>
                                                        </div>
                                                    </div>

                                                    <div class="flex justify-start gap-4 mt-4">
                                                        <button
                                                            class="px-8 py-2.5 bg-primary text-white rounded-full hover:bg-primary-focus transition-colors shadow-md disabled:opacity-50 font-medium"
                                                            on:click=move |_| {
                                                                update_action.dispatch(());
                                                            }
                                                            disabled=move || update_action.pending().get()
                                                        >
                                                            {move || {
                                                                if update_action.pending().get() {
                                                                    "保存中..."
                                                                } else {
                                                                    "保存设置"
                                                                }
                                                            }}
                                                        </button>
                                                    </div>
                                                </div>
                                            </Show>

                                            <Show when=move || active_tab.get() == 1>
                                                <div class="max-w-xl flex flex-col gap-8 animate-fade-in">
                                                    <h3 class="text-2xl font-bold text-dark">"登录设置"</h3>

                                                    <Suspense fallback=move || {
                                                        view! {
                                                            <div class="text-gray-400">"加载登录信息..."</div>
                                                        }
                                                    }>
                                                        {move || {
                                                            match auth_resource.get() {
                                                                Some(Ok(auth_infos)) => {
                                                                    let mut has_phone = false;
                                                                    let mut has_email = false;
                                                                    let mut phone_str = String::new();
                                                                    let mut email_str = String::new();
                                                                    for auth in auth_infos.iter() {
                                                                        if let crate::api::user::UserAuth::Password(p) = auth {
                                                                            match &p.auth_id {
                                                                                crate::api::user::AuthID::Phone(phone) => {
                                                                                    has_phone = true;
                                                                                    phone_str = phone.to_string();
                                                                                }
                                                                                crate::api::user::AuthID::Email(email) => {
                                                                                    has_email = true;
                                                                                    email_str = email.to_string();
                                                                                }
                                                                            }
                                                                        }
                                                                    }

                                                                    view! {
                                                                        <div class="space-y-6">
                                                                            // 绑定手机
                                                                            <div class="p-5 border border-gray-100 rounded-2xl bg-white shadow-sm flex items-center justify-between">
                                                                                <div class="flex items-center gap-4">
                                                                                    <div class="w-10 h-10 rounded-full bg-blue-50 text-blue-500 flex items-center justify-center">
                                                                                        <i class="fa-solid fa-mobile-screen"></i>
                                                                                    </div>
                                                                                    <div>
                                                                                        <h4 class="font-medium text-dark">"绑定手机"</h4>
                                                                                        <p class="text-sm text-gray-500">
                                                                                            {if has_phone {
                                                                                                phone_str.clone()
                                                                                            } else {
                                                                                                "未绑定".to_string()
                                                                                            }}
                                                                                        </p>
                                                                                    </div>
                                                                                </div>
                                                                                <div class="flex gap-2">
                                                                                    <button
                                                                                        class="px-4 py-1.5 text-sm text-primary border border-primary/30 rounded-full hover:bg-primary/5 transition-colors"
                                                                                        on:click=move |_| {
                                                                                            set_bind_type.set("phone".to_string());
                                                                                            set_bind_action_type
                                                                                                .set(
                                                                                                    if has_phone {
                                                                                                        "change".to_string()
                                                                                                    } else {
                                                                                                        "add".to_string()
                                                                                                    },
                                                                                                );
                                                                                            set_show_bind_modal.set(true);
                                                                                        }
                                                                                    >
                                                                                        {if has_phone { "修改" } else { "去绑定" }}
                                                                                    </button>
                                                                                    <Show when=move || {
                                                                                        has_phone
                                                                                    }>
                                                                                        {
                                                                                            let phone_str_clone = phone_str.clone();
                                                                                            view! {
                                                                                                <button
                                                                                                    class="px-4 py-1.5 text-sm text-red-500 border border-red-200 rounded-full hover:bg-red-50 transition-colors"
                                                                                                    on:click=move |_| {
                                                                                                        if let Ok(p) = phonenumber::parse(
                                                                                                                Some(CN),
                                                                                                                &phone_str_clone,
                                                                                                            )
                                                                                                            .or_else(|_| phonenumber::parse(None, &phone_str_clone))
                                                                                                        {
                                                                                                            remove_auth_action.dispatch(user::AuthID::Phone(p));
                                                                                                        }
                                                                                                    }
                                                                                                >
                                                                                                    "解绑"
                                                                                                </button>
                                                                                            }
                                                                                        }
                                                                                    </Show>
                                                                                </div>
                                                                            </div>

                                                                            // 绑定邮箱
                                                                            <div class="p-5 border border-gray-100 rounded-2xl bg-white shadow-sm flex items-center justify-between">
                                                                                <div class="flex items-center gap-4">
                                                                                    <div class="w-10 h-10 rounded-full bg-green-50 text-green-500 flex items-center justify-center">
                                                                                        <i class="fa-regular fa-envelope"></i>
                                                                                    </div>
                                                                                    <div>
                                                                                        <h4 class="font-medium text-dark">"绑定邮箱"</h4>
                                                                                        <p class="text-sm text-gray-500">
                                                                                            {if has_email {
                                                                                                email_str.clone()
                                                                                            } else {
                                                                                                "未绑定".to_string()
                                                                                            }}
                                                                                        </p>
                                                                                    </div>
                                                                                </div>
                                                                                <div class="flex gap-2">
                                                                                    <button
                                                                                        class="px-4 py-1.5 text-sm text-primary border border-primary/30 rounded-full hover:bg-primary/5 transition-colors"
                                                                                        on:click=move |_| {
                                                                                            set_bind_type.set("email".to_string());
                                                                                            set_bind_action_type
                                                                                                .set(
                                                                                                    if has_email {
                                                                                                        "change".to_string()
                                                                                                    } else {
                                                                                                        "add".to_string()
                                                                                                    },
                                                                                                );
                                                                                            set_show_bind_modal.set(true);
                                                                                        }
                                                                                    >
                                                                                        {if has_email { "修改" } else { "去绑定" }}
                                                                                    </button>
                                                                                    <Show when=move || {
                                                                                        has_email
                                                                                    }>
                                                                                        {
                                                                                            let email_str_clone = email_str.clone();
                                                                                            view! {
                                                                                                <button
                                                                                                    class="px-4 py-1.5 text-sm text-red-500 border border-red-200 rounded-full hover:bg-red-50 transition-colors"
                                                                                                    on:click=move |_| {
                                                                                                        if let Ok(e) = email_address::EmailAddress::from_str(
                                                                                                            &email_str_clone,
                                                                                                        ) {
                                                                                                            remove_auth_action.dispatch(user::AuthID::Email(e));
                                                                                                        }
                                                                                                    }
                                                                                                >
                                                                                                    "解绑"
                                                                                                </button>
                                                                                            }
                                                                                        }
                                                                                    </Show>
                                                                                </div>
                                                                            </div>

                                                                            // 修改密码
                                                                            <div class="p-5 border border-gray-100 rounded-2xl bg-white shadow-sm flex items-center justify-between">
                                                                                <div class="flex items-center gap-4">
                                                                                    <div class="w-10 h-10 rounded-full bg-purple-50 text-purple-500 flex items-center justify-center">
                                                                                        <i class="fa-solid fa-lock"></i>
                                                                                    </div>
                                                                                    <div>
                                                                                        <h4 class="font-medium text-dark">"修改密码"</h4>
                                                                                        <p class="text-sm text-gray-500">
                                                                                            "定期修改密码可以保护账号安全"
                                                                                        </p>
                                                                                    </div>
                                                                                </div>
                                                                                <button
                                                                                    class="px-4 py-1.5 text-sm text-gray-600 border border-gray-200 rounded-full hover:bg-gray-50 transition-colors"
                                                                                    on:click=move |_| {
                                                                                        set_bind_type.set("password".to_string());
                                                                                        set_show_bind_modal.set(true);
                                                                                    }
                                                                                >
                                                                                    {"修改密码"}
                                                                                </button>
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                        .into_any()
                                                                }
                                                                Some(Err(e)) => {
                                                                    view! {
                                                                        <div class="text-red-500">
                                                                            {format!("加载失败: {}", e)}
                                                                        </div>
                                                                    }
                                                                        .into_any()
                                                                }
                                                                None => view! { <div></div> }.into_any(),
                                                            }
                                                        }}
                                                    </Suspense>
                                                </div>
                                            </Show>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                            Some(Err(e)) => {
                                view! {
                                    <div class="text-center py-10 text-red-500">
                                        {format!("加载失败: {}", e)}
                                    </div>
                                }
                                    .into_any()
                            }
                            None => {
                                match user_resource.get() {
                                    Some(Err(e)) => {
                                        view! {
                                            <div class="text-center py-10 text-red-500">
                                                {format!("加载失败: {}", e)}
                                            </div>
                                        }
                                            .into_any()
                                    }
                                    _ => {
                                        view! {
                                            <div class="text-center py-10">"请先登录"</div>
                                        }
                                            .into_any()
                                    }
                                }
                            }
                        }
                    }}
                </Suspense>
            </div>

            // 绑定/修改模态框
            <Show when=move || show_bind_modal.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
                    <div class="bg-white rounded-2xl p-8 w-full max-w-md shadow-xl animate-fade-in">
                        <h3 class="text-xl font-bold text-dark mb-6">
                            {move || match bind_type.get().as_str() {
                                "phone" => "绑定/修改手机号",
                                "email" => "绑定/修改邮箱",
                                "password" => "修改密码",
                                _ => "",
                            }}
                        </h3>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    {move || match bind_type.get().as_str() {
                                        "phone" => "新手机号",
                                        "email" => "新邮箱",
                                        "password" => "旧密码",
                                        _ => "",
                                    }}
                                </label>
                                <input
                                    type=move || {
                                        if bind_type.get() == "password" {
                                            "password"
                                        } else {
                                            "text"
                                        }
                                    }
                                    class="w-full p-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/50 bg-gray-50/50 focus:bg-white transition-colors"
                                    placeholder=move || match bind_type.get().as_str() {
                                        "phone" => "请输入手机号",
                                        "email" => "请输入邮箱",
                                        "password" => "请输入旧密码",
                                        _ => "",
                                    }
                                    prop:value=move || bind_input.get()
                                    on:input=move |ev| set_bind_input.set(event_target_value(&ev))
                                />
                            </div>

                            <Show when=move || bind_type.get() == "password">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "新密码"
                                    </label>
                                    <input
                                        type="password"
                                        class="w-full p-3 border border-gray-200 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/50 bg-gray-50/50 focus:bg-white transition-colors"
                                        placeholder="请输入新密码"
                                        prop:value=move || bind_password.get()
                                        on:input=move |ev| {
                                            set_bind_password.set(event_target_value(&ev))
                                        }
                                    />
                                </div>
                            </Show>

                            <Show when=move || bind_error.get().is_some()>
                                <div class="text-red-500 text-sm">
                                    {move || bind_error.get().unwrap_or_default()}
                                </div>
                            </Show>
                        </div>

                        <div class="flex justify-end gap-4 mt-8">
                            <button
                                class="px-6 py-2 text-gray-600 hover:bg-gray-100 rounded-full transition-colors"
                                on:click=move |_| {
                                    set_show_bind_modal.set(false);
                                    set_bind_error.set(None);
                                    set_bind_input.set(String::new());
                                    set_bind_password.set(String::new());
                                }
                            >
                                "取消"
                            </button>
                            <button
                                class="px-6 py-2 bg-primary text-white rounded-full hover:bg-primary-focus transition-colors shadow-md disabled:opacity-50"
                                on:click=move |_| {
                                    bind_action.dispatch(());
                                }
                                disabled=move || bind_action.pending().get()
                            >
                                {move || {
                                    if bind_action.pending().get() {
                                        "提交中..."
                                    } else {
                                        "确认"
                                    }
                                }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
