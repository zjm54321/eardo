use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

mod api;
mod data;
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
                    href="https://cdn.jsdelivr.net/npm/font-awesome@4.7.0/css/font-awesome.min.css"
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
                <pages::Header />
                <Routes fallback=|| pages::NotFound()>
                    <Route path=StaticSegment("") view=pages::homepage::HomePage />
                // <Route path=StaticSegment("playground") view=Playground/>
                    <Route path=StaticSegment("filters") view=pages::voicefilter::VoiceFilterPage/>
                </Routes>
            </main>
            <footer class="mt-auto py-6 px-4 bg-white/70 backdrop-blur-sm border-t border-gray-100">
                <div class="container mx-auto text-center text-gray-500 text-sm">
                    // 更新版权信息
                    <p>"© 2025 白昼聆夏 - 耳朵项目 | 声音转换技术展示平台"</p>
                </div>
            </footer>
        </Router>
    }
}
