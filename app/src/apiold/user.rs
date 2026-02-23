#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
use {
    anyhow::{Context, anyhow},
    axum_extra::extract::cookie::{Cookie, SameSite},
    bcrypt::{DEFAULT_COST, hash, verify},
    chrono::{Duration, Utc},
    http::{HeaderMap, HeaderValue, header::SET_COOKIE},
    leptos::prelude::use_context,
    uuid::Uuid,
};

/// 从 Cookie 中获取 session token
#[cfg(feature = "ssr")]
fn get_session_token() -> Option<String> {
    let headers = use_context::<HeaderMap>()?;
    let cookie_header = headers.get("cookie")?.to_str().ok()?;

    // 解析 cookies
    for cookie_str in cookie_header.split(';') {
        let cookie_str = cookie_str.trim();
        if let Ok(cookie) = Cookie::parse(cookie_str) {
            if cookie.name() == "session_token" {
                return Some(cookie.value().to_string());
            }
        }
    }
    None
}

/// 从 session token 获取用户 ID
#[cfg(feature = "ssr")]
async fn get_user_id_from_session(pool: &sqlx::Pool<Sqlite>) -> anyhow::Result<String> {
    let token = get_session_token().ok_or_else(|| anyhow!("未找到 session token"))?;

    let (user_id,): (String,) = sqlx::query_as(
        "SELECT user_id FROM user_sessions 
         WHERE token = ? AND expires_at > datetime('now')",
    )
    .bind(&token)
    .fetch_one(pool)
    .await
    .context("Session 已过期或无效")?;

    Ok(user_id)
}

/// 创建新的 session
#[cfg(feature = "ssr")]
async fn create_session(pool: &sqlx::Pool<Sqlite>, user_id: &str) -> anyhow::Result<String> {
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(30);

    // 从 headers 获取一些信息（可选）
    let headers = use_context::<HeaderMap>();
    let user_agent = headers
        .as_ref()
        .and_then(|h| h.get("user-agent"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    sqlx::query(
        "INSERT INTO user_sessions (token, user_id, user_agent, expires_at) 
         VALUES (?, ?, ?, ?)",
    )
    .bind(&token)
    .bind(user_id)
    .bind(user_agent)
    .bind(expires_at.format("%Y-%m-%d %H:%M:%S").to_string())
    .execute(pool)
    .await
    .context("创建 session 失败")?;

    Ok(token)
}

/// 删除 session
#[cfg(feature = "ssr")]
async fn delete_session(pool: &sqlx::Pool<Sqlite>) -> anyhow::Result<()> {
    if let Some(token) = get_session_token() {
        sqlx::query("DELETE FROM user_sessions WHERE token = ?")
            .bind(token)
            .execute(pool)
            .await
            .context("删除 session 失败")?;
    }
    Ok(())
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl AuthService for ServiceProvider<Sqlite> {
    async fn register(&self, userauth: UserAuthInfo, password: &str) -> anyhow::Result<()> {
        // 1. 验证输入
        let username = userauth.username.ok_or_else(|| anyhow!("用户名不能为空"))?;

        // 2. 检查用户名是否已存在
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT user_id FROM user_auth WHERE username = ?")
                .bind(&username)
                .fetch_optional(&self.pool)
                .await?;

        if existing.is_some() {
            return Err(anyhow!("用户名已存在"));
        }

        // 3. 检查邮箱是否已存在（如果提供）
        if let Some(ref email) = userauth.email {
            let existing: Option<(String,)> =
                sqlx::query_as("SELECT user_id FROM user_auth WHERE email = ?")
                    .bind(email)
                    .fetch_optional(&self.pool)
                    .await?;

            if existing.is_some() {
                return Err(anyhow!("邮箱已被注册"));
            }
        }

        // 4. 加密密码
        let password_hash = hash(password, DEFAULT_COST).context("密码加密失败")?;

        // 5. 创建用户
        let user_id = Uuid::new_v4().to_string();

        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 插入用户基本信息
        sqlx::query(
            "INSERT INTO users (id, nickname, level, status, role) 
             VALUES (?, ?, 1, 'normal', 'user')",
        )
        .bind(&user_id)
        .bind(&username)
        .execute(&mut *tx)
        .await
        .context("创建用户失败")?;

        // 插入用户认证信息
        sqlx::query(
            "INSERT INTO user_auth (user_id, username, password_hash, email, phone) 
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(&username)
        .bind(&password_hash)
        .bind(&userauth.email)
        .bind(&userauth.phone)
        .execute(&mut *tx)
        .await
        .context("创建用户认证信息失败")?;

        tx.commit().await?;

        leptos::logging::debug_log!("用户注册成功: {}", username);
        Ok(())
    }

    async fn login(&self, userauth: UserAuthInfo, password: &str) -> anyhow::Result<()> {
        // 1. 查找用户
        let username = userauth
            .username
            .or(userauth.email.clone())
            .or(userauth.phone.clone())
            .ok_or_else(|| anyhow!("请提供用户名、邮箱或手机号"))?;

        let user: (String, String, String) = sqlx::query_as(
            "SELECT ua.user_id, ua.password_hash, u.status 
             FROM user_auth ua
             JOIN users u ON ua.user_id = u.id
             WHERE ua.username = ? OR ua.email = ? OR ua.phone = ?",
        )
        .bind(&username)
        .bind(&username)
        .bind(&username)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("用户名或密码错误"))?;

        let (user_id, password_hash, status) = user;

        // 2. 检查用户状态
        match status.as_str() {
            "deleted" => return Err(anyhow!("该账号已注销")),
            "banned" => return Err(anyhow!("该账号已被封禁")),
            _ => {}
        }

        // 3. 验证密码
        if !verify(password, &password_hash).context("密码验证失败")? {
            return Err(anyhow!("用户名或密码错误"));
        }

        // 4. 创建 session
        let token = create_session(&self.pool, &user_id).await?;

        // 5. 设置 Cookie (通过 Leptos response options)
        if let Some(options) = use_context::<leptos_axum::ResponseOptions>() {
            let cookie = Cookie::build(("session_token", token))
                .path("/")
                .max_age(time::Duration::days(30))
                .same_site(SameSite::Lax)
                .http_only(true)
                .build();

            if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
                options.insert_header(SET_COOKIE, header_value);
            }
        }

        leptos::logging::debug_log!("用户登录成功: {}", user_id);
        Ok(())
    }

    async fn logout(&self) -> anyhow::Result<()> {
        // 1. 删除 session
        delete_session(&self.pool).await?;

        // 2. 清除 Cookie
        if let Some(options) = use_context::<leptos_axum::ResponseOptions>() {
            let cookie = Cookie::build(("session_token", ""))
                .path("/")
                .max_age(time::Duration::seconds(0))
                .build();

            if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
                options.insert_header(SET_COOKIE, header_value);
            }
        }

        leptos::logging::debug_log!("用户登出成功");
        Ok(())
    }

    async fn get_current_user(&self) -> anyhow::Result<()> {
        // 验证当前用户是否登录
        let user_id = get_user_id_from_session(&self.pool).await?;

        leptos::logging::debug_log!("当前用户: {}", user_id);
        Ok(())
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl UserService for ServiceProvider<Sqlite> {
    async fn get_user_profile(&self) -> anyhow::Result<UserInfo> {
        // 1. 获取当前用户 ID
        let user_id = get_user_id_from_session(&self.pool).await?;

        // 2. 查询用户信息
        let user: (
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            String,
            String,
        ) = sqlx::query_as(
            "SELECT u.id, ua.username, u.nickname, u.bio, u.level, u.status, u.role
             FROM users u
             JOIN user_auth ua ON u.id = ua.user_id
             WHERE u.id = ?",
        )
        .bind(&user_id)
        .fetch_one(&self.pool)
        .await
        .context("获取用户信息失败")?;

        let (id, username, nickname, bio, level, status, role) = user;

        // 3. 检查是否有头像
        let has_avatar: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM users WHERE id = ? AND avatar_data IS NOT NULL")
                .bind(&id)
                .fetch_optional(&self.pool)
                .await?;

        // 4. 构建头像 URL - 如果有自定义头像则使用 API 路径，否则使用默认头像
        let avatar_url = if has_avatar.is_some() {
            format!("/api/avatar/{}", id)
        } else {
            format!("https://api.dicebear.com/7.x/avataaars/svg?seed={}", id)
        };

        Ok(UserInfo {
            id,
            username,
            avatar_url,
            status: match status.as_str() {
                "deleted" => UserStatus::Deleted,
                "banned" => UserStatus::Banned,
                _ => UserStatus::Normal,
            },
            nickname: nickname.unwrap_or_default(),
            bio: bio.unwrap_or_default(),
            level,
            role,
        })
    }

    async fn update_user_profile(&self, user: &UserInfo) -> anyhow::Result<()> {
        // 1. 获取当前用户 ID
        let user_id = get_user_id_from_session(&self.pool).await?;

        // 2. 更新用户信息（直接使用 session 中的 user_id，无需验证前端传来的 id）
        sqlx::query(
            "UPDATE users 
             SET nickname = ?, bio = ?, updated_at = datetime('now')
             WHERE id = ?",
        )
        .bind(&user.nickname)
        .bind(&user.bio)
        .bind(&user_id)
        .execute(&self.pool)
        .await
        .context("更新用户信息失败")?;

        // 4. 如果头像是 base64，则更新头像
        if user.avatar_url.starts_with("data:") {
            if let Some((mime, data_str)) = user.avatar_url.split_once(";base64,") {
                let mime = mime.strip_prefix("data:").unwrap_or("image/png");
                if let Ok(data) =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data_str)
                {
                    sqlx::query("UPDATE users SET avatar_data = ?, avatar_mime = ? WHERE id = ?")
                        .bind(&data)
                        .bind(mime)
                        .bind(&user_id)
                        .execute(&self.pool)
                        .await
                        .context("更新头像失败")?;
                }
            }
        }

        leptos::logging::debug_log!("用户资料更新成功: {}", user.username);
        Ok(())
    }
}
