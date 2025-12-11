#![recursion_limit = "256"]

use app::*;
use axum::{
    Router,
    body::Body,
    extract::{FromRef, Path, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use dotenv::dotenv;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::env;

#[derive(Clone, FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    pool: SqlitePool,
}

// 1. API 路由处理函数 (用于 CSR / Server Functions)
// 这里已经正确注入了 Headers
async fn server_fn_handler(
    State(app_state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    let headers = req.headers().clone();

    handle_server_fns_with_context(
        move || {
            provide_context(app_state.pool.clone());
            provide_context(app_state.leptos_options.clone());
            provide_context(headers.clone());
        },
        req,
    )
    .await
}

// 2. 音频流处理函数
async fn get_audio_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result: Result<(Vec<u8>,), sqlx::Error> =
        sqlx::query_as("SELECT data FROM audio_files WHERE id = ?")
            .bind(id)
            .fetch_one(&state.pool)
            .await;

    match result {
        Ok((data,)) => (
            [
                (header::CONTENT_TYPE, "audio/mp3"),
                (header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            data,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Audio not found").into_response(),
    }
}

// 3. 新增：SSR 页面渲染处理函数
// 专门用于处理页面请求，确保在服务端渲染时也能获取到 Headers (Cookie)
async fn ssr_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap, // 提取请求头
    req: Request<Body>,
) -> Response {
    let options = app_state.leptos_options.clone();
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            provide_context(app_state.pool.clone());
            provide_context(app_state.leptos_options.clone());
            // 关键：在 SSR 上下文中注入 Headers，解决刷新页面报错问题
            provide_context(headers.clone());
        },
        move || shell(options.clone()), // 修改：将 shell 包装在无参闭包中
    );
    handler(req).await.into_response()
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let db_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:eardo.db?mode=rwc".to_string());
    log!("连接数据库: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.");

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
    };

    // 构建 Router
    let mut app = Router::<AppState>::new()
        .route("/api/{*fn_name}", post(server_fn_handler))
        .route("/api/audio/{id}", get(get_audio_handler));

    // 为每个 Leptos 路由注册我们自定义的 ssr_handler
    // 替代之前的 .leptos_routes_with_context()
    for listing in routes {
        let path = listing.path();
        app = app.route(path, get(ssr_handler));
    }

    // 处理 404 和静态文件 fallback
    // 注意：这里使用 let app shadowing 之前的 app，因为 with_state 改变了 Router 的类型 (从 Router<AppState> 变为 Router<()>)
    let app = app
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
