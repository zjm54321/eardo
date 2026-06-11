use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

pub mod api;
mod pages;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="zh-CN">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />

                <Stylesheet id="leptos" href="/pkg/eardo.css" />
                <link
                    href="https://cdn.jsdelivr.net/npm/@fortawesome/fontawesome-free@6/css/all.min.css"
                    rel="stylesheet"
                />

                <MetaTags />
            </head>

            <body class="bg-gradient-summer min-h-screen font-sans text-dark">
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // sets the document title
        <Title text="耳朵 - 白昼聆夏" />

        // content for this welcome page
        <Router>
            <main>
                // 在 Welcome 页面，我们可能不需要 Header (全屏体验)，
                // 但为了保持一致性和导航便利，我们保留 Header，或者你可以选择在 Welcome 页面隐藏它。
                // 这里的实现是所有页面都有 Header。
                <pages::Header />
                <Routes fallback=|| pages::NotFound()>
                    // 根路径改为 Welcome Page
                    <Route path=StaticSegment("") view=pages::welcome::WelcomePage />

                    // 工作台首页
                    <Route path=StaticSegment("home") view=pages::workbench::WorkbenchHomePage />
                    <Route path=StaticSegment("scene-create") view=pages::scene_creation::SceneCreationPage />
                    <Route path=StaticSegment("voice-style") view=pages::scene_creation::VoiceStylePage />
                    // 保留原有生成型首页到新路由
                    <Route path=StaticSegment("generate") view=pages::homepage::HomePage />
                    <Route path=StaticSegment("setup") view=pages::voicesetup::VoiceSetupPage />

                    <Route path=StaticSegment("voice") view=pages::playground::Playground />
                    <Route path=StaticSegment("filters") view=pages::voicefilter::VoiceFilterPage />
                    <Route path=StaticSegment("help") view=pages::help::HelpPage />

                    <Route path=StaticSegment("login") view=pages::auth::LoginPage />
                    <Route path=StaticSegment("register") view=pages::auth::RegisterPage />
                    <Route path=StaticSegment("profile") view=pages::auth::ProfilePage />
                    <Route path=StaticSegment("settings") view=pages::auth::SettingsPage />
                </Routes>
            </main>

            // 页脚可以保留
            <footer class="mt-auto py-6 px-4 bg-white/70 backdrop-blur-sm border-t border-gray-100 relative z-50">
                <div class="container mx-auto text-center text-gray-500 text-sm">
                    <p>"© 2026 白昼聆夏 - 耳朵项目 | 声音转换技术展示平台"</p>
                </div>
            </footer>
        </Router>
    }
}
