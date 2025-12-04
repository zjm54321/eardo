#![recursion_limit = "256"]

use app::*;
use axum::{
    Router,
    body::Body,
    extract::{FromRef, Path, Request, State}, // 引入 Path 用于提取 URL 参数
    http::{StatusCode, header},               // 引入 HTTP 响应相关类型
    response::IntoResponse,                   // 用于返回响应
    routing::{get, post},                     // 引入 get 路由
};
use dotenv::dotenv;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list, handle_server_fns_with_context};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::env;

// 1. 定义 AppState 并派生 FromRef
// 这会自动生成 impl FromRef<AppState> for LeptosOptions 和 for SqlitePool
#[derive(Clone, FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    pool: SqlitePool,
}

// 2. 独立的 API 处理函数
async fn server_fn_handler(
    State(app_state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(app_state.pool.clone());
            provide_context(app_state.leptos_options.clone());
        },
        req,
    )
    .await
}

// --- 新增：音频文件下载/播放 处理函数 ---
async fn get_audio_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // 从数据库查询 BLOB 数据
    let result: Result<(Vec<u8>,), sqlx::Error> =
        sqlx::query_as("SELECT data FROM audio_files WHERE id = ?")
            .bind(id)
            .fetch_one(&state.pool)
            .await;

    match result {
        Ok((data,)) => {
            // 返回音频流，设置正确的 Content-Type
            (
                [
                    (header::CONTENT_TYPE, "audio/mp3"),
                    (header::CACHE_CONTROL, "public, max-age=3600"), // 允许缓存 1 小时
                ],
                data,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Audio not found").into_response(),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let db_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:eardo.db?mode=rwc".to_string());
    log!("连接数据库: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.");

    // 3. 初始化 AppState
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
    };

    // 4. 构建 Router
    // 关键点：显式指定 Router 的状态类型为 <AppState>，帮助编译器推断
    let app = Router::<AppState>::new()
        .route("/api/{*fn_name}", post(server_fn_handler))
        .route("/api/audio/{id}", get(get_audio_handler))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || {
                    provide_context(app_state.pool.clone());
                    provide_context(app_state.leptos_options.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        // 关键点：显式指定泛型 <AppState, _>，消除 FromRef 的歧义
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
