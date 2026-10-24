#![cfg(feature = "ssr")]
use {
    crate::api::{ServiceProvider, user::*},
    anyhow::{Context, anyhow},
    axum_extra::extract::cookie::{Cookie, SameSite},
    bcrypt::{DEFAULT_COST, hash, verify},
    chrono::{Duration, Utc},
    http::{HeaderMap, HeaderValue, header::SET_COOKIE},
    leptos::prelude::use_context,
    sqlx::Postgres,
    uuid::Uuid,
};

/// 从 Cookie 中获取 session token
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
pub(super) async fn get_authenticated_user(
    pool: &sqlx::Pool<Postgres>,
) -> anyhow::Result<(UserID, UserRole)> {
    let token = get_session_token().ok_or_else(|| anyhow!("未找到 session token"))?;
    let token_uuid = Uuid::parse_str(&token).context("无效的 session token")?;

    let user: (Uuid, UserRole) = sqlx::query_as(
        r#"SELECT u.id, u.role FROM user_session s
           JOIN "user" u ON u.id = s.user_id
           WHERE s.id = $1 AND s.expires_at > NOW()
             AND u.role NOT IN ('BannedUser', 'DeletedUser')"#,
    )
    .bind(token_uuid)
    .fetch_one(pool)
    .await
    .context("Session 已过期或无效")?;

    Ok(user)
}

pub(super) async fn get_user_id_from_session(
    pool: &sqlx::Pool<Postgres>,
) -> anyhow::Result<UserID> {
    Ok(get_authenticated_user(pool).await?.0)
}

fn cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE").is_ok_and(|value| value.eq_ignore_ascii_case("true"))
        || std::env::var("LEPTOS_ENV").is_ok_and(|value| value.eq_ignore_ascii_case("PROD"))
}

async fn hash_password(password: String) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || hash(password, DEFAULT_COST))
        .await
        .context("密码加密任务失败")?
        .context("密码加密失败")
}

async fn verify_password(password: String, credential: String) -> anyhow::Result<bool> {
    tokio::task::spawn_blocking(move || verify(password, &credential))
        .await
        .context("密码验证任务失败")?
        .context("密码验证失败")
}

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
                let password_hash = hash_password(auth.password_hash).await?;
                (auth_type, provider_id, password_hash)
            }
            UserAuth::OAuth(_) => return Err(anyhow!("OAuth 注册尚未实现")),
            UserAuth::Passkey() => return Err(anyhow!("Passkey 注册尚未实现")),
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
            UserAuth::OAuth(_) => return Err(anyhow!("OAuth 登录尚未实现")),
            UserAuth::Passkey() => return Err(anyhow!("Passkey 登录尚未实现")),
        };

        let user: Option<(Uuid, String)> = sqlx::query_as(
            r#"SELECT a.user_id, a.credential FROM user_auth a
               JOIN "user" u ON u.id = a.user_id
               WHERE a.auth_type = $1::auth_type_enum AND a.provider_id = $2
                 AND u.role NOT IN ('BannedUser', 'DeletedUser')"#,
        )
        .bind(auth_type)
        .bind(&provider_id)
        .fetch_optional(&self.pool)
        .await?;

        let (user_id, password_hash) = user.ok_or_else(|| anyhow!("账号或密码错误"))?;

        if !verify_password(credential, password_hash).await? {
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
                h.get("cf-connecting-ip")
                    .or_else(|| h.get("x-forwarded-for"))
            })
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next()) // x-forwarded-for 可能是逗号分隔的列表，取第一个
            .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
            .map(|ip| ip.to_string());

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
                .secure(cookie_secure())
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
                .secure(cookie_secure())
                .build();

            if let Ok(header_value) = HeaderValue::from_str(&cookie.to_string()) {
                options.insert_header(SET_COOKIE, header_value);
            }
        }

        Ok(())
    }

    async fn get_authinfo(&self) -> anyhow::Result<Vec<UserAuth>> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let auths: Vec<(String, String)> =
            sqlx::query_as("SELECT auth_type::text, provider_id FROM user_auth WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(&self.pool)
                .await
                .context("获取认证信息失败")?;

        let mut result = Vec::new();
        for (auth_type, provider_id) in auths {
            match auth_type.as_str() {
                "password_email" => {
                    if let Ok(email) = provider_id.parse() {
                        result.push(UserAuth::Password(PasswordAuth {
                            auth_id: AuthID::Email(email),
                            password_hash: String::new(), // 不返回密码哈希
                        }));
                    }
                }
                "password_phone" => {
                    if let Ok(phone) = provider_id.parse() {
                        result.push(UserAuth::Password(PasswordAuth {
                            auth_id: AuthID::Phone(phone),
                            password_hash: String::new(), // 不返回密码哈希
                        }));
                    }
                }
                "oauth_github" => result.push(UserAuth::OAuth(OAuthProvider::Github())),
                "oauth_wechat" => result.push(UserAuth::OAuth(OAuthProvider::WeChat())),
                "passkey" => result.push(UserAuth::Passkey()),
                _ => {}
            }
        }

        Ok(result)
    }

    async fn update_authinfo(&self, update_info: UpdateAuthInfo) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        match update_info {
            UpdateAuthInfo::ChangePassword(old_password, new_password) => {
                let session_id = get_session_token()
                    .and_then(|token| Uuid::parse_str(&token).ok())
                    .ok_or_else(|| anyhow!("无效的 session token"))?;
                let mut tx = self.pool.begin().await?;
                sqlx::query(r#"SELECT id FROM "user" WHERE id = $1 FOR UPDATE"#)
                    .bind(user_id)
                    .fetch_one(&mut *tx)
                    .await?;
                // 1. Verify old password
                let auths: Vec<(String, String)> = sqlx::query_as(
                    "SELECT auth_type::text, credential FROM user_auth WHERE user_id = $1 AND auth_type IN ('password_email', 'password_phone')"
                )
                .bind(user_id)
                .fetch_all(&mut *tx)
                .await
                .context("获取认证信息失败")?;

                if auths.is_empty() {
                    return Err(anyhow!("用户未设置密码"));
                }

                let mut verified = false;
                for (_, credential) in &auths {
                    if verify_password(old_password.clone(), credential.clone()).await? {
                        verified = true;
                        break;
                    }
                }

                if !verified {
                    return Err(anyhow!("旧密码错误"));
                }

                // 2. Hash new password
                let new_password_hash = hash_password(new_password).await?;

                // 3. Update all password_email and password_phone for this user
                sqlx::query(
                    "UPDATE user_auth SET credential = $1 WHERE user_id = $2 AND auth_type IN ('password_email', 'password_phone')"
                )
                .bind(&new_password_hash)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .context("修改密码失败")?;
                sqlx::query("DELETE FROM user_session WHERE user_id = $1 AND id <> $2")
                    .bind(user_id)
                    .bind(session_id)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
            }
            UpdateAuthInfo::AddAuth(userauth) => {
                let (auth_type, provider_id) = match userauth {
                    UserAuth::Password(auth) => {
                        let (auth_type, provider_id) = match auth.auth_id {
                            AuthID::Email(email) => ("password_email", email.to_string()),
                            AuthID::Phone(phone) => ("password_phone", phone.to_string()),
                        };
                        (auth_type, provider_id)
                    }
                    UserAuth::OAuth(_) => return Err(anyhow!("OAuth 添加尚未实现")),
                    UserAuth::Passkey() => return Err(anyhow!("Passkey 添加尚未实现")),
                };

                let existing: Option<(Uuid,)> = sqlx::query_as(
                    "SELECT id FROM user_auth WHERE auth_type = $1::auth_type_enum AND provider_id = $2"
                )
                .bind(auth_type)
                .bind(&provider_id)
                .fetch_optional(&self.pool)
                .await?;

                if existing.is_some() {
                    return Err(anyhow!("该账号已被绑定"));
                }

                // 获取当前用户已有的密码哈希
                let existing_credential: Option<(String,)> = sqlx::query_as(
                    "SELECT credential FROM user_auth WHERE user_id = $1 AND auth_type IN ('password_email', 'password_phone') LIMIT 1"
                )
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

                let credential = match existing_credential {
                    Some((cred,)) => cred,
                    None => return Err(anyhow!("用户未设置密码，无法绑定新账号")),
                };

                sqlx::query(
                    "INSERT INTO user_auth (user_id, auth_type, provider_id, credential) VALUES ($1, $2::auth_type_enum, $3, $4)"
                )
                .bind(user_id)
                .bind(auth_type)
                .bind(&provider_id)
                .bind(&credential)
                .execute(&self.pool)
                .await
                .context("添加认证信息失败")?;
            }
            UpdateAuthInfo::RemoveAuth(auth_id) => {
                let (auth_type, provider_id) = match auth_id {
                    AuthID::Email(email) => ("password_email", email.to_string()),
                    AuthID::Phone(phone) => ("password_phone", phone.to_string()),
                };

                let mut tx = self.pool.begin().await?;
                sqlx::query(r#"SELECT id FROM "user" WHERE id = $1 FOR UPDATE"#)
                    .bind(user_id)
                    .fetch_one(&mut *tx)
                    .await?;
                let count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM user_auth WHERE user_id = $1")
                        .bind(user_id)
                        .fetch_one(&mut *tx)
                        .await?;

                if count.0 <= 1 {
                    return Err(anyhow!("无法解绑最后一个认证方式"));
                }

                sqlx::query(
                    "DELETE FROM user_auth WHERE user_id = $1 AND auth_type = $2::auth_type_enum AND provider_id = $3"
                )
                .bind(user_id)
                .bind(auth_type)
                .bind(&provider_id)
                .execute(&mut *tx)
                .await
                .context("解绑认证信息失败")?;
                tx.commit().await?;
            }
            UpdateAuthInfo::ChangePassAuth(auth_id) => {
                let (auth_type, provider_id) = match auth_id {
                    AuthID::Email(email) => ("password_email", email.to_string()),
                    AuthID::Phone(phone) => ("password_phone", phone.to_string()),
                };

                let existing: Option<(Uuid,)> = sqlx::query_as(
                    "SELECT id FROM user_auth WHERE auth_type = $1::auth_type_enum AND provider_id = $2"
                )
                .bind(auth_type)
                .bind(&provider_id)
                .fetch_optional(&self.pool)
                .await?;

                if existing.is_some() {
                    return Err(anyhow!("该账号已被绑定"));
                }

                sqlx::query(
                    "UPDATE user_auth SET provider_id = $1 WHERE user_id = $2 AND auth_type = $3::auth_type_enum"
                )
                .bind(&provider_id)
                .bind(user_id)
                .bind(auth_type)
                .execute(&self.pool)
                .await
                .context("修改认证信息失败")?;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl UserService for ServiceProvider<Postgres> {
    async fn get_user_profile(&self) -> anyhow::Result<User> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        #[derive(sqlx::FromRow)]
        struct UserProfileRow {
            #[sqlx(flatten)]
            usermeta: UserMeta,
            updated_at: chrono::DateTime<Utc>,
        }

        let mut row: UserProfileRow = sqlx::query_as(
            r#"SELECT nick_name, bio, role, level, updated_at FROM "user" WHERE id = $1"#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .context("获取用户信息失败")?;

        row.usermeta.avatar_url =
            format!("/api/avatar/{}?t={}", user_id, row.updated_at.timestamp());

        Ok(User {
            id: user_id,
            usermeta: row.usermeta,
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
