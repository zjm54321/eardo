#![recursion_limit = "256"]

use app::*;
use axum::{
    Router,
    body::Body,
    extract::{FromRef, Path, Request, State},
    http::{HeaderMap, Method, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use dotenv::dotenv;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, io, str::FromStr, sync::Arc, time::Duration};
use tower_http::limit::RequestBodyLimitLayer;

#[derive(Clone, FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    pg_pool: PgPool,
    auth_provider: api::user::AuthProvider,
    user_provider: api::user::UserProvider,
    voice_meta_post_provider: api::post::VoiceMetaPostProvider,
    voice_post_provider: api::post::VoicePostProvider,
    voice_provider: api::voice::VoiceProvider,
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    let check = tokio::time::timeout(
        Duration::from_secs(1),
        sqlx::query_as::<_, (i32, bool)>(r#"SELECT 1, to_regclass('public."user"') IS NOT NULL"#)
            .fetch_one(&state.pg_pool),
    )
    .await;
    let (status, body) = if matches!(check, Ok(Ok((1, true)))) {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "unavailable")
    };
    (status, [(header::CACHE_CONTROL, "no-store")], body)
}

// Compare complete origins, never string prefixes (which would allow lookalike hosts).
fn origin_from_uri(value: &str) -> Option<String> {
    let uri: Uri = value.parse().ok()?;
    let scheme = uri.scheme_str()?;
    let authority = uri.authority()?.as_str();
    if !matches!(scheme, "http" | "https") || authority.contains('@') {
        return None;
    }
    Some(format!("{scheme}://{authority}"))
}

async fn check_api_origin(
    State(allowed_origins): State<Arc<Vec<String>>>,
    req: Request,
    next: Next,
) -> Response {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return next.run(req).await;
    }

    // An explicit Origin takes precedence; Referer is only a fallback when absent.
    let origin = match req.headers().get(header::ORIGIN) {
        Some(value) => value.to_str().ok().map(str::to_owned),
        None => req
            .headers()
            .get(header::REFERER)
            .and_then(|value| value.to_str().ok())
            .and_then(origin_from_uri),
    };
    if !origin.is_some_and(|origin| allowed_origins.contains(&origin)) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }
    next.run(req).await
}

fn positive_env<T>(name: &str, default: T) -> io::Result<T>
where
    T: FromStr + Default + PartialOrd,
{
    match env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .ok()
            .filter(|value| *value > T::default())
            .ok_or_else(|| io::Error::other(format!("{name} must be a positive integer"))),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(_) => Err(io::Error::other(format!("{name} must be valid UTF-8"))),
    }
}

fn allowed_origins() -> io::Result<Option<Arc<Vec<String>>>> {
    let value = match env::var("ALLOWED_ORIGINS") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(_) => return Err(io::Error::other("ALLOWED_ORIGINS must be valid UTF-8")),
    };
    let mut origins = Vec::new();
    for value in value.split(',').map(str::trim) {
        let origin = origin_from_uri(value)
            .filter(|origin| value == origin || value.strip_suffix('/') == Some(origin.as_str()));
        origins.push(origin.ok_or_else(|| {
            io::Error::other("ALLOWED_ORIGINS must contain comma-separated HTTP(S) origins")
        })?);
    }
    Ok(Some(Arc::new(origins)))
}

fn shutdown_signal(timeout: Duration) -> io::Result<impl std::future::Future<Output = ()>> {
    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|err| {
        io::Error::other(format!("failed to register SIGTERM handler: {err}"))
    })?;

    Ok(async move {
        let ctrl_c = async {
            if let Err(err) = tokio::signal::ctrl_c().await {
                eprintln!("failed to listen for Ctrl-C: {err}; shutting down");
            }
        };
        #[cfg(unix)]
        tokio::select! {
            () = ctrl_c => {},
            _ = terminate.recv() => {},
        }
        #[cfg(not(unix))]
        ctrl_c.await;

        log!("shutdown requested; draining requests");
        // This remains active through both HTTP draining and pool closure.
        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;
            eprintln!("shutdown drain deadline exceeded; exiting");
            std::process::exit(1);
        });
    })
}

// 1. API 路由处理函数 (用于 CSR / Server Functions)
// 这里已经正确注入了 Headers
async fn server_fn_handler(
    State(app_state): State<AppState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    req: Request<Body>,
) -> impl IntoResponse {
    let headers = req.headers().clone();

    handle_server_fns_with_context(
        move || {
            // 基础依赖
            provide_context(app_state.pg_pool.clone());
            provide_context(app_state.leptos_options.clone());
            provide_context(headers.clone());
            provide_context(addr);

            // !!! 关键修复：必须手动注入 AppState 中的 Provider !!!
            provide_context(app_state.auth_provider.clone());
            provide_context(app_state.user_provider.clone());
            provide_context(app_state.voice_meta_post_provider.clone());
            provide_context(app_state.voice_post_provider.clone());
            provide_context(app_state.voice_provider.clone());
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
    let parsed_uuid = uuid::Uuid::parse_str(&id);

    if let Ok(uuid) = parsed_uuid {
        let result: Result<(Vec<u8>,), sqlx::Error> =
            sqlx::query_as("SELECT audio_data FROM voice_library WHERE id = $1")
                .bind(uuid)
                .fetch_one(&state.pg_pool)
                .await;

        if let Ok((data,)) = result {
            return (
                [
                    (header::CONTENT_TYPE, "audio/mpeg"),
                    (header::CACHE_CONTROL, "public, max-age=3600"),
                ],
                data,
            )
                .into_response();
        }
    }

    (StatusCode::NOT_FOUND, "Audio not found").into_response()
}

// 2.1 头像获取处理函数
async fn get_avatar_handler(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let parsed_uuid = uuid::Uuid::parse_str(&user_id);

    if let Ok(uuid) = parsed_uuid {
        let result: Result<(Vec<u8>,), sqlx::Error> =
            sqlx::query_as(r#"SELECT avatar FROM "user" WHERE id = $1 AND avatar IS NOT NULL"#)
                .bind(uuid)
                .fetch_one(&state.pg_pool)
                .await;
        if let Ok((data,)) = result {
            return (
                [
                    (header::CONTENT_TYPE, "image/webp"),
                    (header::CACHE_CONTROL, "public, max-age=86400"),
                ],
                data,
            )
                .into_response();
        }
    }
    // 返回默认头像 - 重定向到 DiceBear API
    (
        StatusCode::FOUND,
        [(
            header::LOCATION,
            format!(
                "https://api.dicebear.com/7.x/avataaars/svg?seed={}",
                user_id
            ),
        )],
    )
        .into_response()
}

// 2.2 帖子音频获取处理函数
async fn get_post_audio_handler(
    Path(post_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let parsed_uuid = uuid::Uuid::parse_str(&post_id);

    if let Ok(uuid) = parsed_uuid {
        let result: Result<(Vec<u8>,), sqlx::Error> =
            sqlx::query_as("SELECT audio_data FROM voice_library vl JOIN voice_post vp ON vl.id = vp.library_id WHERE vp.id = $1 AND vp.status = 'Normal'")
                .bind(uuid)
                .fetch_one(&state.pg_pool)
                .await;

        match result {
            Ok((data,)) => {
                return (
                    [
                        (header::CONTENT_TYPE, "audio/mpeg"),
                        (header::CACHE_CONTROL, "public, max-age=86400"),
                    ],
                    data,
                )
                    .into_response();
            }
            Err(_) => {}
        }
    }
    (StatusCode::NOT_FOUND, "Post audio not found").into_response()
}

// 2.3 语音模型头像获取处理函数
async fn get_voice_avatar_handler(
    Path(voice_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let parsed_uuid = uuid::Uuid::parse_str(&voice_id);

    if let Ok(uuid) = parsed_uuid {
        let result: Result<(Vec<u8>,), sqlx::Error> = sqlx::query_as(
            r#"SELECT avatar FROM voice_model WHERE id = $1 AND avatar IS NOT NULL"#,
        )
        .bind(uuid)
        .fetch_one(&state.pg_pool)
        .await;
        if let Ok((data,)) = result {
            return (
                [
                    (header::CONTENT_TYPE, "image/webp"),
                    (header::CACHE_CONTROL, "public, max-age=86400"),
                ],
                data,
            )
                .into_response();
        }
    }
    // 返回默认头像 - 重定向到 DiceBear API
    (
        StatusCode::FOUND,
        [(
            header::LOCATION,
            format!("https://api.dicebear.com/7.x/bottts/svg?seed={}", voice_id),
        )],
    )
        .into_response()
}

// 3. 新增：SSR 页面渲染处理函数
// 专门用于处理页面请求，确保在服务端渲染时也能获取到 Headers (Cookie)
async fn ssr_handler(
    State(app_state): State<AppState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Response {
    let options = app_state.leptos_options.clone();
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            // 基础依赖
            provide_context(app_state.pg_pool.clone());
            provide_context(app_state.leptos_options.clone());
            provide_context(headers.clone());
            provide_context(addr);

            // !!! 关键修复：同样需要在 SSR 渲染时注入这些 Provider !!!
            provide_context(app_state.auth_provider.clone());
            provide_context(app_state.user_provider.clone());
            provide_context(app_state.voice_meta_post_provider.clone());
            provide_context(app_state.voice_post_provider.clone());
            provide_context(app_state.voice_provider.clone());
        },
        move || shell(options.clone()),
    );
    handler(req).await.into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let conf = get_configuration(None)
        .map_err(|err| io::Error::other(format!("failed to load Leptos configuration: {err}")))?;
    let addr = conf.leptos_options.site_addr;
    let mut leptos_options = conf.leptos_options;
    // cargo-leptos (hash-files = true) writes hash.txt next to the server binary and names
    // /pkg assets eardo.<hash>.js/.wasm/.css, so browsers and Cloudflare never mix an old JS
    // glue file with a new wasm after a deploy. Enable it automatically when the file exists
    // (leptos panics if hash_files is on but hash.txt is missing).
    if env::var_os("LEPTOS_HASH_FILES").is_none()
        && env::current_exe()
            .map(|exe| exe.with_file_name(leptos_options.hash_file.as_ref()).is_file())
            .unwrap_or(false)
    {
        leptos_options.hash_files = true;
    }
    let routes = generate_route_list(App);

    let production = env::var("LEPTOS_ENV").is_ok_and(|value| value.eq_ignore_ascii_case("prod"));
    let pg_url = match env::var("PG_DATABASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        Err(env::VarError::NotPresent) if !production => {
            "postgres://postgres:postgres@localhost:5432/eardo".to_string()
        }
        _ => return Err(io::Error::other("PG_DATABASE_URL is required and must be valid UTF-8 (no database fallback in production)").into()),
    };
    if env::var("ALIYUN_API_KEY").map_or(true, |value| value.trim().is_empty()) {
        eprintln!("warning: ALIYUN_API_KEY is missing; voice synthesis will be unavailable");
    }
    let max_connections = positive_env("PG_MAX_CONNECTIONS", 5_u32)?;
    let max_body_bytes = positive_env("MAX_BODY_BYTES", 4 * 1024 * 1024_usize)?;
    let shutdown_timeout = Duration::from_secs(positive_env("SHUTDOWN_TIMEOUT_SECS", 25_u64)?);
    let allowed_origins = allowed_origins()?;
    let shutdown = shutdown_signal(shutdown_timeout)?;
    log!("正在连接PG数据库");

    let pg_pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&pg_url)
        .await
        // Connection errors can contain user-supplied connection options. Never log the URL.
        .map_err(|_| {
            io::Error::other(
                "failed to connect to PostgreSQL; check PG_DATABASE_URL and database availability",
            )
        })?;

    // 1. 实例化 ServiceProvider (指定泛型为 PgPool)
    let new_service_impl = api::ServiceProvider {
        pool: pg_pool.clone(),
    };

    // 2. 包装成 Arc (Provider)
    // 这里的 service_impl 实现了所有 Trait，所以可以被转型
    // ServiceProvider<PgPool> -> Arc<dyn AuthService>
    let auth_provider = Arc::new(new_service_impl.clone());
    let user_provider = Arc::new(new_service_impl.clone());
    let voice_meta_post_provider = Arc::new(new_service_impl.clone());
    let voice_post_provider = Arc::new(new_service_impl.clone());
    let voice_provider = Arc::new(new_service_impl.clone());

    // 3. 组装 AppState
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pg_pool: pg_pool.clone(),
        auth_provider,
        user_provider,
        voice_meta_post_provider,
        voice_post_provider,
        voice_provider,
    };

    // 构建 Router
    let mut app = Router::<AppState>::new()
        .route(
            "/api/{*fn_name}",
            post(server_fn_handler).layer(RequestBodyLimitLayer::new(max_body_bytes)),
        )
        .route("/api/audio/{id}", get(get_audio_handler))
        .route("/api/avatar/{user_id}", get(get_avatar_handler))
        .route(
            "/api/voice_avatar/{voice_id}",
            get(get_voice_avatar_handler),
        )
        .route("/api/post/audio/{post_id}", get(get_post_audio_handler));

    if let Some(allowed_origins) = allowed_origins {
        app = app.route_layer(middleware::from_fn_with_state(
            allowed_origins,
            check_api_origin,
        ));
    }
    app = app.route("/healthz", get(healthz));

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
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|err| {
        io::Error::other(format!("failed to bind HTTP listener on {addr}: {err}"))
    })?;
    let result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await;
    pg_pool.close().await;
    result.map_err(|err| io::Error::other(format!("HTTP server failed: {err}")))?;
    Ok(())
}
