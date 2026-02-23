use crate::api::{ServiceProvider, user::*};
use sqlx::Postgres;

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
async fn get_user_id_from_session(pool: &sqlx::Pool<Postgres>) -> anyhow::Result<UserID> {
    let token = get_session_token().ok_or_else(|| anyhow!("未找到 session token"))?;
    let token_uuid = Uuid::parse_str(&token).context("无效的 session token")?;

    let (user_id,): (Uuid,) = sqlx::query_as(
        "SELECT user_id FROM user_session 
         WHERE id = $1 AND expires_at > NOW()",
    )
    .bind(token_uuid)
    .fetch_one(pool)
    .await
    .context("Session 已过期或无效")?;

    Ok(user_id)
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl AuthService for ServiceProvider<Postgres> {
    async fn register(&self, userauth: UserAuth) -> anyhow::Result<UserID> {
        let (auth_type, provider_id, credential) = match userauth {
            UserAuth::Password(auth) => {
                let (auth_type, provider_id) = match auth.auth_id {
                    AuthID::Email(email) => ("password_email", email.to_string()),
                    AuthID::Phone(phone) => ("password_phone", phone.to_string()),
                };
                // 对前端传来的哈希值再进行一次 bcrypt 加盐哈希
                let password_hash =
                    hash(&auth.password_hash, DEFAULT_COST).context("密码加密失败")?;
                (auth_type, provider_id, password_hash)
            }
            UserAuth::OAuth(_) => unimplemented!("OAuth 注册尚未实现"),
            UserAuth::Passkey() => unimplemented!("Passkey 注册尚未实现"),
        };

        // 检查是否已存在

        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM user_auth WHERE auth_type = $1::auth_type_enum AND provider_id = $2",
        )
        .bind(auth_type)
        .bind(&provider_id)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(anyhow!("该账号已被注册"));
        }

        // 开始注册流程
        let default_nickname = format!(
            "user_{}",
            Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
        );

        let mut tx = self.pool.begin().await?;

        // 插入用户基本信息
        let (user_id,): (Uuid,) =
            sqlx::query_as(r#"INSERT INTO "user" (nick_name) VALUES ($1) RETURNING id"#)
                .bind(&default_nickname) // 默认昵称为user+随机数
                .fetch_one(&mut *tx)
                .await
                .context("创建用户失败")?;

        // 插入用户认证信息
        sqlx::query(
            "INSERT INTO user_auth (user_id, auth_type, provider_id, credential) 
             VALUES ($1, $2::auth_type_enum, $3, $4)",
        )
        .bind(user_id)
        .bind(auth_type)
        .bind(&provider_id)
        .bind(&credential)
        .execute(&mut *tx)
        .await
        .context("创建用户认证信息失败")?;

        tx.commit().await?;

        Ok(user_id)
    }

    async fn login(&self, userauth: UserAuth) -> anyhow::Result<UserID> {
        let (auth_type, provider_id, credential) = match userauth {
            UserAuth::Password(auth) => {
                let (auth_type, provider_id) = match auth.auth_id {
                    AuthID::Email(email) => ("password_email", email.to_string()),
                    AuthID::Phone(phone) => ("password_phone", phone.to_string()),
                };
                (auth_type, provider_id, auth.password_hash)
            }
            UserAuth::OAuth(_) => unimplemented!("OAuth 登录尚未实现"),
            UserAuth::Passkey() => unimplemented!("Passkey 登录尚未实现"),
        };

        let user: Option<(Uuid, String)> = sqlx::query_as(
            "SELECT user_id, credential FROM user_auth 
             WHERE auth_type = $1::auth_type_enum AND provider_id = $2",
        )
        .bind(auth_type)
        .bind(&provider_id)
        .fetch_optional(&self.pool)
        .await?;

        let (user_id, password_hash) = user.ok_or_else(|| anyhow!("账号或密码错误"))?;

        if !verify(&credential, &password_hash).context("密码验证失败")? {
            return Err(anyhow!("账号或密码错误"));
        }

        let expires_at = Utc::now() + Duration::days(7);

        let headers = use_context::<HeaderMap>();
        let user_agent = headers
            .as_ref()
            .and_then(|h| h.get("user-agent"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        let ip_address = headers
            .as_ref()
            .and_then(|h| {
                h.get("x-forwarded-for")
                    .or_else(|| h.get("x-real-ip"))
                    .or_else(|| h.get("cf-connecting-ip"))
            })
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next()) // x-forwarded-for 可能是逗号分隔的列表，取第一个
            .map(|s| s.trim());

        let (session_id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO user_session (user_id, auth_type, user_agent, ip_address, expires_at) 
             VALUES ($1, $2::auth_type_enum, $3, $4::inet, $5) RETURNING id",
        )
        .bind(user_id)
        .bind(auth_type)
        .bind(user_agent)
        .bind(ip_address)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .context("创建 session 失败")?;

        let token = session_id.to_string();

        if let Some(options) = use_context::<leptos_axum::ResponseOptions>() {
            let cookie = Cookie::build(("session_token", token))
                .path("/")
                .max_age(time::Duration::days(7))
                .same_site(SameSite::Lax)
                .http_only(true)
                .build();

            if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
                options.insert_header(SET_COOKIE, header_value);
            }
        }

        Ok(user_id)
    }

    async fn logout(&self) -> anyhow::Result<()> {
        if let Some(token) = get_session_token() {
            if let Ok(token_uuid) = Uuid::parse_str(&token) {
                sqlx::query("DELETE FROM user_session WHERE id = $1")
                    .bind(token_uuid)
                    .execute(&self.pool)
                    .await
                    .context("删除 session 失败")?;
            }
        }

        if let Some(options) = use_context::<leptos_axum::ResponseOptions>() {
            let cookie = Cookie::build(("session_token", ""))
                .path("/")
                .max_age(time::Duration::days(0))
                .same_site(SameSite::Lax)
                .http_only(true)
                .build();

            if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
                options.insert_header(SET_COOKIE, header_value);
            }
        }

        Ok(())
    }

    async fn update_authinfo(&self, userauth: UserAuth) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let (auth_type, provider_id, credential) = match userauth {
            UserAuth::Password(auth) => {
                let (auth_type, provider_id) = match auth.auth_id {
                    AuthID::Email(email) => ("password_email", email.to_string()),
                    AuthID::Phone(phone) => ("password_phone", phone.to_string()),
                };
                // 对前端传来的哈希值再进行一次 bcrypt 加盐哈希
                let password_hash =
                    hash(&auth.password_hash, DEFAULT_COST).context("密码加密失败")?;
                (auth_type, provider_id, password_hash)
            }
            UserAuth::OAuth(_) => unimplemented!("OAuth 更新尚未实现"),
            UserAuth::Passkey() => unimplemented!("Passkey 更新尚未实现"),
        };

        // 检查是否已经绑定了该类型的认证
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM user_auth WHERE user_id = $1 AND auth_type = $2::auth_type_enum",
        )
        .bind(user_id)
        .bind(auth_type)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            // 更新
            sqlx::query(
                "UPDATE user_auth SET provider_id = $1, credential = $2 
                 WHERE user_id = $3 AND auth_type = $4::auth_type_enum",
            )
            .bind(&provider_id)
            .bind(&credential)
            .bind(user_id)
            .bind(auth_type)
            .execute(&self.pool)
            .await
            .context("更新认证信息失败")?;
        } else {
            // 插入
            sqlx::query(
                "INSERT INTO user_auth (user_id, auth_type, provider_id, credential) 
                 VALUES ($1, $2::auth_type_enum, $3, $4)",
            )
            .bind(user_id)
            .bind(auth_type)
            .bind(&provider_id)
            .bind(&credential)
            .execute(&self.pool)
            .await
            .context("添加认证信息失败")?;
        }

        Ok(())
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl UserService for ServiceProvider<Postgres> {
    async fn get_user_profile(&self) -> anyhow::Result<User> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let mut usermeta: UserMeta =
            sqlx::query_as(r#"SELECT nick_name, bio, role, level FROM "user" WHERE id = $1"#)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .context("获取用户信息失败")?;

        usermeta.avatar_url = format!("/api/avatar/{}", user_id);

        Ok(User {
            id: user_id,
            usermeta,
        })
    }

    async fn update_user_profile(&self, user: &User) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        sqlx::query(r#"UPDATE "user" SET nick_name = $1, bio = $2 WHERE id = $3"#)
            .bind(&user.usermeta.nick_name)
            .bind(&user.usermeta.bio)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .context("更新用户信息失败")?;

        Ok(())
    }

    async fn update_user_avatar(&self, avatar_data: Vec<u8>) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        sqlx::query(r#"UPDATE "user" SET avatar = $1 WHERE id = $2"#)
            .bind(avatar_data)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .context("更新用户头像失败")?;

        Ok(())
    }
}
