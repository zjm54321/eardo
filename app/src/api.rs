use crate::data;
use crate::pages::homepage::GenerateParams;
#[cfg(not(target_arch = "wasm32"))]
use base64::{Engine as _, engine::general_purpose};
use chrono::DateTime;
#[cfg(not(target_arch = "wasm32"))]
use chrono::{Duration, Utc};
#[cfg(not(target_arch = "wasm32"))]
use futures_util::{SinkExt, StreamExt};
#[cfg(not(target_arch = "wasm32"))]
use http::{HeaderMap, HeaderValue, header::SET_COOKIE};
#[cfg(not(target_arch = "wasm32"))]
use leptos::logging::{debug_log, error};
use leptos::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use sqlx::SqlitePool;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};
#[cfg(not(target_arch = "wasm32"))]
use uuid::Uuid;

// --- 数据库相关 API (保持不变) ---

#[server]
pub async fn get_voices() -> Result<Vec<data::VoiceOption>, ServerFnError> {
    let pool =
        use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Database pool missing"))?;

    let voices =
        sqlx::query_as::<_, data::VoiceOption>("SELECT id, name, description as desc FROM voices")
            .fetch_all(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("DB query failed: {}", e)))?;

    Ok(voices)
}

#[server]
pub async fn get_voice_filters() -> Result<Vec<data::VoiceFilter>, ServerFnError> {
    let pool =
        use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Database pool missing"))?;

    let filters_db = sqlx::query_as::<_, data::VoiceFilterDb>("SELECT * FROM voice_filters")
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB query failed: {}", e)))?;

    let filters = filters_db.into_iter().map(|f| f.to_domain()).collect();

    Ok(filters)
}

#[server]
pub async fn get_featured_works() -> Result<Vec<data::VoiceWork>, ServerFnError> {
    let pool =
        use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Database pool missing"))?;

    let works_db = sqlx::query_as::<_, data::VoiceWorkDb>(
        "SELECT * FROM voice_works WHERE is_featured = 1 ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Featured works DB query failed: {}", e)))?;

    let works = works_db.into_iter().map(|w| w.to_domain()).collect();
    Ok(works)
}

#[server]
pub async fn get_latest_works(
    page: usize,
    page_size: usize,
) -> Result<Vec<data::VoiceWork>, ServerFnError> {
    let pool =
        use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Database pool missing"))?;

    let limit = page_size as i64;
    let offset = ((page - 1) * page_size) as i64;

    let works_db = sqlx::query_as::<_, data::VoiceWorkDb>(
        "SELECT * FROM voice_works WHERE is_featured = 0 ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Latest works DB query failed: {}", e)))?;

    let works = works_db.into_iter().map(|w| w.to_domain()).collect();
    Ok(works)
}

// --- CosyVoice WebSocket 协议结构定义 ---

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
struct CosyHeader {
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<String>,
    task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    streaming: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
struct CosyParams {
    text_type: String,
    voice: String,
    format: String,
    sample_rate: u32,
    volume: u32,
    rate: f32,
    pitch: f32,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
struct CosyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
struct CosyPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    task_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<CosyParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<CosyInput>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize, Debug)]
struct CosyMessage {
    header: CosyHeader,
    payload: CosyPayload,
}

#[server]
pub async fn generate_audio(params: GenerateParams) -> Result<String, ServerFnError> {
    let api_key = std::env::var("ALIYUN_API_KEY").unwrap_or("".into());
    if api_key.is_empty() {
        return Err(ServerFnError::new("API Key missing"));
    }
    debug_log!(
        "开始 CosyVoice 生成任务: voice={}, text={}",
        params.voice_id,
        params.text
    );

    let ws_url = "wss://dashscope.aliyuncs.com/api-ws/v1/inference";
    let task_id = Uuid::new_v4().to_string().replace("-", "");

    // 1. 构建 WebSocket 请求 (带 Header)
    // 修复：先生成标准的 WebSocket 请求，再追加自定义 Header
    let mut request = ws_url
        .into_client_request()
        .map_err(|e| ServerFnError::new(format!("WS Req build error: {}", e)))?;

    let headers = request.headers_mut();
    headers.insert(
        "Authorization",
        format!("bearer {}", api_key).parse().unwrap(),
    );
    headers.insert("X-DashScope-DataInspection", "enable".parse().unwrap());

    // 2. 建立连接
    let (mut ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| ServerFnError::new(format!("WS Connect failed: {}", e)))?;

    // 3. 发送 run-task 指令
    let run_task_msg = CosyMessage {
        header: CosyHeader {
            action: Some("run-task".to_string()),
            task_id: task_id.clone(),
            streaming: Some("duplex".to_string()),
            event: None,
            error_code: None,
            error_message: None,
        },
        payload: CosyPayload {
            task_group: Some("audio".to_string()),
            task: Some("tts".to_string()),
            function: Some("SpeechSynthesizer".to_string()),
            model: Some("cosyvoice-v3-flash".to_string()), // 使用 v2 模型
            parameters: Some(CosyParams {
                text_type: "PlainText".to_string(),
                voice: params.voice_id.clone(), // 使用前端传来的 voice_id
                format: "mp3".to_string(),
                sample_rate: 22050,
                volume: 50,
                rate: params.voice_param.speed,  // 使用前端参数
                pitch: params.voice_param.pitch, // 使用前端参数
            }),
            input: Some(CosyInput { text: None }), // run-task 时不发文本
        },
    };

    let json_str = serde_json::to_string(&run_task_msg)
        .map_err(|e| ServerFnError::new(format!("Serialize run-task failed: {}", e)))?;
    ws_stream
        .send(Message::Text(json_str))
        .await
        .map_err(|e| ServerFnError::new(format!("Send run-task failed: {}", e)))?;

    let mut audio_buffer: Vec<u8> = Vec::new();
    let mut task_started = false;

    // 4. 循环处理消息
    while let Some(msg) = ws_stream.next().await {
        let msg = msg.map_err(|e| ServerFnError::new(format!("WS Read error: {}", e)))?;

        match msg {
            Message::Text(text) => {
                // 解析 JSON 事件
                // 注意：有些 Text 消息可能不是 JSON (虽然 CosyVoice 协议说是 JSON)，或者解析失败
                // 我们可以先尝试解析，失败打印日志但不 panic
                let event: CosyMessage = match serde_json::from_str(&text) {
                    Ok(e) => e,
                    Err(e) => {
                        debug_log!("Parse event error: {}, text: {}", e, text);
                        continue;
                    }
                };

                if let Some(event_type) = event.header.event {
                    match event_type.as_str() {
                        "task-started" => {
                            debug_log!("收到 task-started, 发送文本...");
                            task_started = true;

                            // 发送 continue-task (文本)
                            let continue_msg = CosyMessage {
                                header: CosyHeader {
                                    action: Some("continue-task".to_string()),
                                    task_id: task_id.clone(),
                                    streaming: Some("duplex".to_string()),
                                    event: None,
                                    error_code: None,
                                    error_message: None,
                                },
                                payload: CosyPayload {
                                    input: Some(CosyInput {
                                        text: Some(params.text.clone()),
                                    }),
                                    task_group: None,
                                    task: None,
                                    function: None,
                                    model: None,
                                    parameters: None,
                                },
                            };
                            ws_stream
                                .send(Message::Text(serde_json::to_string(&continue_msg).unwrap()))
                                .await
                                .map_err(|e| {
                                    ServerFnError::new(format!("Send continue-task failed: {}", e))
                                })?;

                            // 发送 finish-task
                            let finish_msg = CosyMessage {
                                header: CosyHeader {
                                    action: Some("finish-task".to_string()),
                                    task_id: task_id.clone(),
                                    streaming: Some("duplex".to_string()),
                                    event: None,
                                    error_code: None,
                                    error_message: None,
                                },
                                payload: CosyPayload {
                                    input: Some(CosyInput { text: None }), // 空 input
                                    task_group: None,
                                    task: None,
                                    function: None,
                                    model: None,
                                    parameters: None,
                                },
                            };
                            ws_stream
                                .send(Message::Text(serde_json::to_string(&finish_msg).unwrap()))
                                .await
                                .map_err(|e| {
                                    ServerFnError::new(format!("Send finish-task failed: {}", e))
                                })?;
                        }
                        "task-finished" => {
                            debug_log!("任务完成，接收到音频字节数: {}", audio_buffer.len());
                            break; // 退出循环
                        }
                        "task-failed" => {
                            let err_msg = event
                                .header
                                .error_message
                                .unwrap_or("Unknown error".to_string());
                            error!("CosyVoice 任务失败: {}", err_msg);
                            return Err(ServerFnError::new(format!("Task failed: {}", err_msg)));
                        }
                        "result-generated" => {
                            // 忽略，继续接收音频
                        }
                        _ => {
                            debug_log!("未知事件: {}", event_type);
                        }
                    }
                }
            }
            Message::Binary(bin) => {
                // 收到音频流
                audio_buffer.extend_from_slice(&bin);
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    if !task_started {
        return Err(ServerFnError::new(
            "Connection closed before task started. Check API Key or Network.",
        ));
    }

    if audio_buffer.is_empty() {
        return Err(ServerFnError::new("No audio data received"));
    }

    /*     // 5. 编码为 Base64 Data URI 返回
    let content_type = "audio/mp3";
    let base64_data = general_purpose::STANDARD.encode(&audio_buffer);
    let data_uri = format!("data:{};base64,{}", content_type, base64_data);

    Ok(data_uri) */

    // --- 修改点：保存到数据库 ---
    let audio_id = Uuid::new_v4().to_string();

    let pool =
        use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Database pool missing"))?;
    // 执行插入
    // 假设您在 api.rs 顶部已经正确引入了 sqlx
    sqlx::query("INSERT INTO audio_files (id, data) VALUES (?, ?)")
        .bind(&audio_id)
        .bind(&audio_buffer)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to save audio to DB: {}", e)))?;

    debug_log!("音频已保存到数据库，ID: {}", audio_id);

    // 返回本地访问 URL
    let audio_url = format!("/api/audio/{}", audio_id);
    Ok(audio_url)
}

// --- Auth APIs ---

#[server]
pub async fn register(username: String, password: String) -> Result<(), ServerFnError> {
    let pool = use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Pool missing"))?;

    // 1. 检查用户名是否存在
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)")
        .bind(&username)
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB Check failed: {}", e)))?;

    if exists {
        return Err(ServerFnError::new("Username already taken"));
    }

    // 2. 哈希密码
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| ServerFnError::new(format!("Hashing failed: {}", e)))?;

    // 3. 插入用户
    let user_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id, username, password_hash) VALUES (?, ?, ?)")
        .bind(&user_id)
        .bind(&username)
        .bind(&password_hash)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Insert user failed: {}", e)))?;

    Ok(())
}

#[server]
pub async fn login(username: String, password: String) -> Result<data::User, ServerFnError> {
    let pool = use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Pool missing"))?;

    // 1. 查询用户
    let user_db = sqlx::query_as::<_, data::UserDb>("SELECT * FROM users WHERE username = ?")
        .bind(&username)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("DB query failed: {}", e)))?;

    let user_db = match user_db {
        Some(u) => u,
        None => return Err(ServerFnError::new("User not found")),
    };

    // 2. 验证密码
    let valid = bcrypt::verify(password, &user_db.password_hash)
        .map_err(|_| ServerFnError::new("Password verify error"))?;

    if !valid {
        return Err(ServerFnError::new("Invalid password"));
    }

    // 3. 创建 Session
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7); // 7天过期

    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(&user_db.id)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Session creation failed: {}", e)))?;

    // 4. 设置 Cookie
    // 注意：在 Server Fn 中操作 header 需要使用 expect_context 或 ResponseOptions
    // 这里我们简单地构建 Set-Cookie 字符串
    let cookie_val = format!(
        "auth_token={}; Path=/; HttpOnly; Max-Age={}",
        session_id,
        60 * 60 * 24 * 7
    );

    // 使用 leptos_axum 提供的帮助函数设置 header (如果有) 或者直接操作
    // Leptos 0.6+ 推荐使用 ResponseOptions
    if let Some(res_options) = use_context::<leptos_axum::ResponseOptions>() {
        res_options.insert_header(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());
    }

    Ok(data::User {
        id: user_db.id,
        username: user_db.username,
        avatar: user_db.avatar,
    })
}

#[server]
pub async fn get_current_user() -> Result<Option<data::User>, ServerFnError> {
    let pool = use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Pool missing"))?;

    // 1. 获取 Cookie
    // 在 Axum 中，我们可以从 HeaderMap 获取
    let headers =
        use_context::<HeaderMap>().ok_or_else(|| ServerFnError::new("Headers missing"))?;

    let cookie_header = match headers.get(http::header::COOKIE) {
        Some(h) => h.to_str().unwrap_or(""),
        None => return Ok(None),
    };

    // 简单解析 auth_token
    let session_id = cookie_header.split(';').find_map(|s| {
        let s = s.trim();
        if s.starts_with("auth_token=") {
            Some(s.trim_start_matches("auth_token="))
        } else {
            None
        }
    });

    let session_id = match session_id {
        Some(sid) => sid,
        None => return Ok(None),
    };

    // 2. 查询 Session 和 User
    let result = sqlx::query_as::<_, (String, String, Option<String>, DateTime<Utc>)>(
        r#"
        SELECT u.id, u.username, u.avatar, s.expires_at
        FROM sessions s
        JOIN users u ON s.user_id = u.id
        WHERE s.id = ? AND s.expires_at > ?
        "#,
    )
    .bind(session_id)
    .bind(Utc::now())
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Session query failed: {}", e)))?;

    match result {
        Some((id, username, avatar, _)) => Ok(Some(data::User {
            id,
            username,
            avatar,
        })),
        None => Ok(None),
    }
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let pool = use_context::<SqlitePool>().ok_or_else(|| ServerFnError::new("Pool missing"))?;
    let headers =
        use_context::<HeaderMap>().ok_or_else(|| ServerFnError::new("Headers missing"))?;

    // 1. 获取 Session ID (为了从 DB 删除)
    if let Some(cookie_header) = headers.get(http::header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            if let Some(session_id) = cookie_str.split(';').find_map(|s| {
                let s = s.trim();
                if s.starts_with("auth_token=") {
                    Some(s.trim_start_matches("auth_token="))
                } else {
                    None
                }
            }) {
                // 删除 session
                let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
                    .bind(session_id)
                    .execute(&pool)
                    .await;
            }
        }
    }

    // 2. 清除 Cookie
    let cookie_val = "auth_token=; Path=/; HttpOnly; Max-Age=0";
    if let Some(res_options) = use_context::<leptos_axum::ResponseOptions>() {
        res_options.insert_header(SET_COOKIE, HeaderValue::from_str(cookie_val).unwrap());
    }

    Ok(())
}
