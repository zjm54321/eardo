#![cfg(feature = "ssr")]
use {
    crate::api::{ServiceProvider, post::*, user::UserID},
    anyhow::{Context, anyhow},
    axum_extra::extract::cookie::Cookie,
    http::HeaderMap,
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

#[async_trait::async_trait]
impl VoiceMetaPostService for ServiceProvider<Postgres> {
    async fn create(&self, post: VoiceMetaPost) -> anyhow::Result<VoiceMetaPostID> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let (id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO voice_meta_post (title, content, meta_id, author, status) 
             VALUES ($1, $2, $3, $4, $5::post_status) RETURNING id",
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.meta_id)
        .bind(user_id)
        .bind(match post.status {
            PostStatus::Normal => "Normal",
            PostStatus::Deleted => "Deleted",
            PostStatus::Banned => "Banned",
            PostStatus::Recommended => "Recommended",
        })
        .fetch_one(&self.pool)
        .await
        .context("创建帖子失败")?;

        Ok(id)
    }

    async fn get(&self, post_id: VoiceMetaPostID) -> anyhow::Result<VoiceMetaPost> {
        let post = sqlx::query_as::<_, VoiceMetaPost>(
            "SELECT id, title, content, meta_id, author, status, comments_count, likes_count FROM voice_meta_post WHERE id = $1",
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await
        .context("获取帖子失败")?;

        Ok(post)
    }

    async fn update(&self, post: VoiceMetaPost) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_meta_post SET title = $1, content = $2, meta_id = $3, status = $4::post_status 
             WHERE id = $5 AND author = $6"
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.meta_id)
        .bind(match post.status {
            PostStatus::Normal => "Normal",
            PostStatus::Deleted => "Deleted",
            PostStatus::Banned => "Banned",
            PostStatus::Recommended => "Recommended",
        })
        .bind(post.id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("更新帖子失败")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("帖子不存在或无权修改"));
        }

        Ok(())
    }

    async fn delete(&self, post_id: VoiceMetaPostID) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_meta_post SET status = 'Deleted'::post_status WHERE id = $1 AND author = $2"
        )
        .bind(post_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("删除帖子失败")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("帖子不存在或无权删除"));
        }

        Ok(())
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<VoiceMetaPost>> {
        let search_pattern = format!("%{}%", query);

        let posts = sqlx::query_as::<_, VoiceMetaPost>(
            "SELECT id, title, content, meta_id, author, status, comments_count, likes_count FROM voice_meta_post 
             WHERE status != 'Deleted'::post_status AND status != 'Banned'::post_status 
             AND (title ILIKE $1 OR content ILIKE $2)
             ORDER BY created_at DESC",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await
        .context("搜索帖子失败")?;

        Ok(posts)
    }

    async fn list(
        &self,
        status: Option<PostStatus>,
        number: u8,
    ) -> anyhow::Result<Vec<VoiceMetaPost>> {
        let status_str = match status {
            Some(PostStatus::Normal) => Some("Normal"),
            Some(PostStatus::Deleted) => Some("Deleted"),
            Some(PostStatus::Banned) => Some("Banned"),
            Some(PostStatus::Recommended) => Some("Recommended"),
            None => None,
        };

        let posts = if let Some(s) = status_str {
            sqlx::query_as::<_, VoiceMetaPost>(
                "SELECT id, title, content, meta_id, author, status, comments_count, likes_count FROM voice_meta_post 
                 WHERE status = $1::post_status
                 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(s)
            .bind(number as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, VoiceMetaPost>(
                "SELECT id, title, content, meta_id, author, status, comments_count, likes_count FROM voice_meta_post 
                 WHERE status != 'Deleted'::post_status AND status != 'Banned'::post_status
                 ORDER BY created_at DESC LIMIT $1",
            )
            .bind(number as i64)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(posts)
    }

    async fn action(&self, post_id: VoiceMetaPostID, action: Action) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        match action {
            Action::LikeDislike => {
                // 检查是否已经点赞
                let existing: Option<(Uuid,)> = sqlx::query_as(
                    "SELECT post_id FROM voice_meta_post_like WHERE post_id = $1 AND user_id = $2",
                )
                .bind(post_id)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

                if existing.is_some() {
                    // 取消点赞
                    sqlx::query(
                        "DELETE FROM voice_meta_post_like WHERE post_id = $1 AND user_id = $2",
                    )
                    .bind(post_id)
                    .bind(user_id)
                    .execute(&self.pool)
                    .await?;
                } else {
                    // 点赞
                    sqlx::query(
                        "INSERT INTO voice_meta_post_like (post_id, user_id) VALUES ($1, $2)",
                    )
                    .bind(post_id)
                    .bind(user_id)
                    .execute(&self.pool)
                    .await?;
                }
            }
            Action::Comment(content) => {
                sqlx::query("INSERT INTO voice_meta_post_comment (post_id, user_id, content) VALUES ($1, $2, $3)")
                    .bind(post_id)
                    .bind(user_id)
                    .bind(content)
                    .execute(&self.pool)
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_comments(
        &self,
        post_id: VoiceMetaPostID,
    ) -> anyhow::Result<Vec<(UserID, String)>> {
        let comments = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT user_id, content FROM voice_meta_post_comment WHERE post_id = $1 ORDER BY created_at ASC"
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .context("获取评论失败")?;

        Ok(comments)
    }
}

#[async_trait::async_trait]
impl VoicePostService for ServiceProvider<Postgres> {
    async fn create(&self, post: VoicePost) -> anyhow::Result<VoicePostID> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let (id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO voice_post (title, content, library_id, author, status) 
             VALUES ($1, $2, $3, $4, $5::post_status) RETURNING id",
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.library_id)
        .bind(user_id)
        .bind(match post.status {
            PostStatus::Normal => "Normal",
            PostStatus::Deleted => "Deleted",
            PostStatus::Banned => "Banned",
            PostStatus::Recommended => "Recommended",
        })
        .fetch_one(&self.pool)
        .await
        .context("创建帖子失败")?;

        Ok(id)
    }

    async fn get(&self, post_id: VoicePostID) -> anyhow::Result<VoicePost> {
        let post = sqlx::query_as::<_, VoicePost>(
            "SELECT id, title, content, library_id, author, status, comments_count, likes_count FROM voice_post WHERE id = $1",
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await
        .context("获取帖子失败")?;

        Ok(post)
    }

    async fn update(&self, post: VoicePost) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_post SET title = $1, content = $2, library_id = $3, status = $4::post_status 
             WHERE id = $5 AND author = $6"
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.library_id)
        .bind(match post.status {
            PostStatus::Normal => "Normal",
            PostStatus::Deleted => "Deleted",
            PostStatus::Banned => "Banned",
            PostStatus::Recommended => "Recommended",
        })
        .bind(post.id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("更新帖子失败")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("帖子不存在或无权修改"));
        }

        Ok(())
    }

    async fn delete(&self, post_id: VoicePostID) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_post SET status = 'Deleted'::post_status WHERE id = $1 AND author = $2",
        )
        .bind(post_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .context("删除帖子失败")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("帖子不存在或无权删除"));
        }

        Ok(())
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<VoicePost>> {
        let search_pattern = format!("%{}%", query);

        let posts = sqlx::query_as::<_, VoicePost>(
            "SELECT id, title, content, library_id, author, status, comments_count, likes_count FROM voice_post 
             WHERE status != 'Deleted'::post_status AND status != 'Banned'::post_status 
             AND (title ILIKE $1 OR content ILIKE $2)
             ORDER BY created_at DESC",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await
        .context("搜索帖子失败")?;

        Ok(posts)
    }

    async fn list(&self, status: Option<PostStatus>, number: u8) -> anyhow::Result<Vec<VoicePost>> {
        let status_str = match status {
            Some(PostStatus::Normal) => Some("Normal"),
            Some(PostStatus::Deleted) => Some("Deleted"),
            Some(PostStatus::Banned) => Some("Banned"),
            Some(PostStatus::Recommended) => Some("Recommended"),
            None => None,
        };

        let posts = if let Some(s) = status_str {
            sqlx::query_as::<_, VoicePost>(
                "SELECT id, title, content, library_id, author, status, comments_count, likes_count FROM voice_post 
                 WHERE status = $1::post_status
                 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(s)
            .bind(number as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, VoicePost>(
                "SELECT id, title, content, library_id, author, status, comments_count, likes_count FROM voice_post 
                 WHERE status != 'Deleted'::post_status AND status != 'Banned'::post_status
                 ORDER BY created_at DESC LIMIT $1",
            )
            .bind(number as i64)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(posts)
    }

    async fn action(&self, post_id: VoicePostID, action: Action) -> anyhow::Result<()> {
        let user_id = get_user_id_from_session(&self.pool).await?;

        match action {
            Action::LikeDislike => {
                // 检查是否已经点赞
                let existing: Option<(Uuid,)> = sqlx::query_as(
                    "SELECT post_id FROM voice_post_like WHERE post_id = $1 AND user_id = $2",
                )
                .bind(post_id)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

                if existing.is_some() {
                    // 取消点赞
                    sqlx::query("DELETE FROM voice_post_like WHERE post_id = $1 AND user_id = $2")
                        .bind(post_id)
                        .bind(user_id)
                        .execute(&self.pool)
                        .await?;
                } else {
                    // 点赞
                    sqlx::query("INSERT INTO voice_post_like (post_id, user_id) VALUES ($1, $2)")
                        .bind(post_id)
                        .bind(user_id)
                        .execute(&self.pool)
                        .await?;
                }
            }
            Action::Comment(content) => {
                sqlx::query("INSERT INTO voice_post_comment (post_id, user_id, content) VALUES ($1, $2, $3)")
                    .bind(post_id)
                    .bind(user_id)
                    .bind(content)
                    .execute(&self.pool)
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_comments(&self, post_id: VoicePostID) -> anyhow::Result<Vec<(UserID, String)>> {
        let comments = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT user_id, content FROM voice_post_comment WHERE post_id = $1 ORDER BY created_at ASC"
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .context("获取评论失败")?;

        Ok(comments)
    }
}
