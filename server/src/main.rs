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
use std::{env, sync::Arc};

#[derive(Clone, FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    pool: SqlitePool,
    auth_provider: api::AuthProvider,
    user_provider: api::UserServiceProvider,
    voice_model_provider: api::VoiceModelProvider,
    voice_metadata_provider: api::VoiceMetadataProvider,
    post_provider: api::PostProvider,
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
            // 基础依赖
            provide_context(app_state.pool.clone());
            provide_context(app_state.leptos_options.clone());
            provide_context(headers.clone());

            // !!! 关键修复：必须手动注入 AppState 中的 Provider !!!
            provide_context(app_state.auth_provider.clone());
            provide_context(app_state.user_provider.clone());
            provide_context(app_state.voice_model_provider.clone());
            provide_context(app_state.voice_metadata_provider.clone());
            provide_context(app_state.post_provider.clone());
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

// 2.1 头像获取处理函数
async fn get_avatar_handler(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result: Result<(Vec<u8>, String), sqlx::Error> = sqlx::query_as(
        "SELECT avatar_data, avatar_mime FROM users WHERE id = ? AND avatar_data IS NOT NULL",
    )
    .bind(&user_id)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok((data, mime)) => (
            [
                (header::CONTENT_TYPE, mime.as_str()),
                (header::CACHE_CONTROL, "public, max-age=86400"),
            ],
            data,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Avatar not found").into_response(),
    }
}

// 2.2 帖子音频获取处理函数
async fn get_post_audio_handler(
    Path(post_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result: Result<(Vec<u8>,), sqlx::Error> =
        sqlx::query_as("SELECT generated_audio_data FROM posts WHERE id = ? AND status = 'normal'")
            .bind(&post_id)
            .fetch_one(&state.pool)
            .await;

    match result {
        Ok((data,)) => (
            [
                (header::CONTENT_TYPE, "audio/mpeg"),
                (header::CACHE_CONTROL, "public, max-age=86400"),
            ],
            data,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Post audio not found").into_response(),
    }
}

// 3. 新增：SSR 页面渲染处理函数
// 专门用于处理页面请求，确保在服务端渲染时也能获取到 Headers (Cookie)
async fn ssr_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Response {
    let options = app_state.leptos_options.clone();
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            // 基础依赖
            provide_context(app_state.pool.clone());
            provide_context(app_state.leptos_options.clone());
            provide_context(headers.clone());

            // !!! 关键修复：同样需要在 SSR 渲染时注入这些 Provider !!!
            provide_context(app_state.auth_provider.clone());
            provide_context(app_state.user_provider.clone());
            provide_context(app_state.voice_model_provider.clone());
            provide_context(app_state.voice_metadata_provider.clone());
            provide_context(app_state.post_provider.clone());
        },
        move || shell(options.clone()),
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
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:eardo.sqlite?mode=rwc".to_string());
    log!("连接数据库: {}", db_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.");

    // 1. 实例化 ServiceProvider (指定泛型为 SqlitePool)
    let service_impl = api::ServiceProvider { pool: pool.clone() };

    // 2. 包装成 Arc (Provider)
    // 这里的 service_impl 实现了所有 Trait，所以可以被转型
    // ServiceProvider<SqlitePool> -> Arc<dyn AuthService>
    let auth_provider = Arc::new(service_impl.clone());
    let user_provider = Arc::new(service_impl.clone());
    let voice_model_provider = Arc::new(service_impl.clone());
    let voice_metadata_provider = Arc::new(service_impl.clone());
    let post_provider = Arc::new(service_impl.clone());

    // 3. 组装 AppState
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool: pool.clone(),
        auth_provider,
        user_provider,
        voice_model_provider,
        voice_metadata_provider,
        post_provider,
    };

    // 构建 Router
    let mut app = Router::<AppState>::new()
        .route("/api/{*fn_name}", post(server_fn_handler))
        .route("/api/audio/{id}", get(get_audio_handler))
        .route("/api/avatar/{user_id}", get(get_avatar_handler))
        .route("/api/post/audio/{post_id}", get(get_post_audio_handler));

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
