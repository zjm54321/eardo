#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
use {
    anyhow::{Context, anyhow},
    axum_extra::extract::cookie::Cookie,
    http::HeaderMap,
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
    .context("会话已过期或无效")?;

    Ok(user_id)
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl PostService for ServiceProvider<Sqlite> {
    async fn list_posts(&self) -> anyhow::Result<Vec<PostInfo>> {
        leptos::logging::debug_log!("列出所有帖子");

        let pool = &self.pool;

        // 获取当前用户ID（如果已登录）
        let current_user_id = get_user_id_from_session(pool).await.ok();

        let posts = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
                i64,
                i64,
                String,
                String,
            ),
        >(
            "SELECT 
                p.id, p.title, p.content, p.user_id, 
                ua.username, u.nickname, u.avatar_mime,
                p.likes_count, p.comments_count, p.created_at,
                vm.name as voice_name
            FROM posts p
            JOIN users u ON p.user_id = u.id
            JOIN user_auth ua ON u.id = ua.user_id
            JOIN voice_meta_infos vm ON p.voice_meta_id = vm.id
            WHERE p.status = 'normal'
            ORDER BY p.created_at DESC",
        )
        .fetch_all(pool)
        .await
        .context("获取帖子列表失败")?;

        let mut result = Vec::new();
        for (
            id,
            title,
            content,
            user_id,
            username,
            nickname,
            _avatar_mime,
            likes_count,
            comments_count,
            created_at,
            voice_name,
        ) in posts
        {
            // 检查当前用户是否点赞了该帖子
            let is_liked = if let Some(ref uid) = current_user_id {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM post_likes WHERE user_id = ? AND post_id = ?",
                )
                .bind(uid)
                .bind(&id)
                .fetch_one(pool)
                .await
                .unwrap_or(0)
                    > 0
            } else {
                false
            };

            // 使用nickname如果存在，否则使用username
            let display_name = nickname.unwrap_or(username);
            let avatar_url = user_id
                .as_ref()
                .map(|id| format!("/api/avatar/{}", id))
                .unwrap_or_default();

            result.push(PostInfo {
                id: id.clone(),
                title,
                author: AuthorInfo {
                    name: display_name,
                    avatar: avatar_url,
                },
                content: PostContent {
                    description: Some(content),
                    audio_url: Some(format!("/api/post/audio/{}", id)),
                    audio_data: None,
                },
                meta: PostMeta {
                    likes: likes_count as i32,
                    comments: comments_count as i32,
                    is_liked,
                    time: created_at,
                    voice_info: VoiceInfo {
                        voice_type: voice_name,
                        voice_meta_id: None,
                    },
                },
            });
        }

        Ok(result)
    }

    async fn search_post(&self, query: &str) -> anyhow::Result<Vec<String>> {
        leptos::logging::debug_log!("搜索帖子: {}", query);

        let pool = &self.pool;

        // 支持高级查询语法
        // uid:xxx - 查询特定用户的帖子
        // 其他 - 搜索标题和内容

        if query.starts_with("uid:") {
            // 查询特定用户的帖子
            let user_id = &query[4..];
            leptos::logging::debug_log!("查询用户帖子: {}", user_id);

            let post_ids: Vec<(String,)> = sqlx::query_as(
                "SELECT id FROM posts
                WHERE status = 'normal' AND user_id = ?
                ORDER BY created_at DESC",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .context("查询用户帖子失败")?;

            Ok(post_ids.into_iter().map(|(id,)| id).collect())
        } else {
            // 按标题和内容搜索
            let search_pattern = format!("%{}%", query);

            let post_ids: Vec<(String,)> = sqlx::query_as(
                "SELECT id FROM posts
                WHERE status = 'normal' 
                AND (title LIKE ? OR content LIKE ?)
                ORDER BY created_at DESC",
            )
            .bind(&search_pattern)
            .bind(&search_pattern)
            .fetch_all(pool)
            .await
            .context("搜索帖子失败")?;

            Ok(post_ids.into_iter().map(|(id,)| id).collect())
        }
    }

    async fn get_post(&self, post_id: &str) -> anyhow::Result<PostInfo> {
        leptos::logging::debug_log!("获取帖子详情: {}", post_id);

        let pool = &self.pool;
        let current_user_id = get_user_id_from_session(pool).await.ok();

        let (
            id,
            title,
            content,
            user_id,
            username,
            nickname,
            _avatar_mime,
            likes_count,
            comments_count,
            created_at,
            voice_name,
        ) = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Option<String>,
                String,
                Option<String>,
                Option<String>,
                i64,
                i64,
                String,
                String,
            ),
        >(
            "SELECT 
                    p.id, p.title, p.content, p.user_id, 
                    ua.username, u.nickname, u.avatar_mime,
                    p.likes_count, p.comments_count, p.created_at,
                    vm.name as voice_name
                FROM posts p
                JOIN users u ON p.user_id = u.id
                JOIN user_auth ua ON u.id = ua.user_id
                JOIN voice_meta_infos vm ON p.voice_meta_id = vm.id
                WHERE p.id = ? AND p.status = 'normal'",
        )
        .bind(post_id)
        .fetch_one(pool)
        .await
        .context("帖子不存在")?;

        let is_liked = if let Some(ref uid) = current_user_id {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM post_likes WHERE user_id = ? AND post_id = ?",
            )
            .bind(uid)
            .bind(&id)
            .fetch_one(pool)
            .await
            .unwrap_or(0)
                > 0
        } else {
            false
        };

        let display_name = nickname.unwrap_or(username);
        let avatar_url = user_id
            .as_ref()
            .map(|id| format!("/api/avatar/{}", id))
            .unwrap_or_default();

        Ok(PostInfo {
            id: id.clone(),
            title,
            author: AuthorInfo {
                name: display_name,
                avatar: avatar_url,
            },
            content: PostContent {
                description: Some(content),
                audio_url: Some(format!("/api/post/audio/{}", id)),
                audio_data: None,
            },
            meta: PostMeta {
                likes: likes_count as i32,
                comments: comments_count as i32,
                is_liked,
                time: created_at,
                voice_info: VoiceInfo {
                    voice_type: voice_name,
                    voice_meta_id: None,
                },
            },
        })
    }

    async fn create_post(&self, post: &PostInfo) -> anyhow::Result<()> {
        let post_id = if post.id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            post.id.clone()
        };

        leptos::logging::debug_log!("创建新帖子: {}, ID: {}", post.title, post_id);

        let pool = &self.pool;
        let user_id = match get_user_id_from_session(pool).await {
            Ok(id) => {
                leptos::logging::debug_log!("用户ID: {}", id);
                id
            }
            Err(e) => {
                leptos::logging::error!("获取用户ID失败: {}", e);
                return Err(e);
            }
        };

        // 直接从 post 嵌套结构中获取数据
        let content = post.content.description.clone();

        let audio_data = post.content.audio_data.clone().unwrap_or_default();

        leptos::logging::debug_log!("音频数据长度: {} bytes", audio_data.len());

        // 从 post.meta.voice_info.voice_meta_id 获取 base_model_id
        let base_model_id = post
            .meta
            .voice_info
            .voice_meta_id
            .as_deref()
            .unwrap_or("default");

        let voice_meta_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM voice_meta_infos WHERE base_model_id = ? LIMIT 1")
                .bind(base_model_id)
                .fetch_optional(pool)
                .await
                .context("查询 voice_meta_id 失败")?;

        leptos::logging::debug_log!(
            "查询结果: base_model_id={}, voice_meta_id={:?}",
            base_model_id,
            voice_meta_id
        );

        let result = sqlx::query(
            "INSERT INTO posts (id, user_id, title, content, generated_audio_data, voice_meta_id, status, created_at)
             VALUES (?, ?, ?, ?, ?, ?, 'normal', datetime('now'))"
        )
        .bind(&post_id)
        .bind(&user_id)
        .bind(&post.title)
        .bind(content)
        .bind(&audio_data)
        .bind(&voice_meta_id)

        .execute(pool)
        .await;

        match result {
            Ok(_) => {
                leptos::logging::debug_log!("帖子创建成功: {}", post_id);
                Ok(())
            }
            Err(e) => {
                leptos::logging::error!("帖子创建失败: {}", e);
                Err(anyhow!("创建帖子失败: {}", e))
            }
        }
    }

    async fn update_post(&self, post: &PostInfo) -> anyhow::Result<()> {
        leptos::logging::debug_log!("更新帖子: {}", post.id);

        let pool = &self.pool;
        let user_id = get_user_id_from_session(pool).await?;

        // 检查帖子是否存在且属于当前用户
        let (owner_id,): (String,) = sqlx::query_as("SELECT user_id FROM posts WHERE id = ?")
            .bind(&post.id)
            .fetch_one(pool)
            .await
            .context("帖子不存在")?;

        if owner_id != user_id {
            return Err(anyhow!("无权限修改此帖子"));
        }

        // 直接使用 post 嵌套结构字段
        let content = post.content.description.as_deref();

        sqlx::query("UPDATE posts SET title = ?, content = ? WHERE id = ?")
            .bind(&post.title)
            .bind(content)
            .bind(&post.id)
            .execute(pool)
            .await
            .context("更新帖子失败")?;

        Ok(())
    }

    async fn delete_post(&self, post_id: &str) -> anyhow::Result<()> {
        leptos::logging::debug_log!("删除帖子: {}", post_id);

        let pool = &self.pool;
        let user_id = get_user_id_from_session(pool).await?;

        // 检查帖子是否存在且属于当前用户
        let (owner_id,): (String,) = sqlx::query_as("SELECT user_id FROM posts WHERE id = ?")
            .bind(post_id)
            .fetch_one(pool)
            .await
            .context("帖子不存在")?;

        if owner_id != user_id {
            return Err(anyhow!("无权限删除此帖子"));
        }

        // 软删除
        sqlx::query("UPDATE posts SET status = 'deleted' WHERE id = ?")
            .bind(post_id)
            .execute(pool)
            .await
            .context("删除帖子失败")?;

        Ok(())
    }

    async fn comment_on_post(&self, post_id: &str, content: &str) -> anyhow::Result<()> {
        leptos::logging::debug_log!("评论帖子: {}", post_id);

        let pool = &self.pool;
        let user_id = get_user_id_from_session(pool).await?;

        let comment_id = Uuid::new_v4().to_string();

        // 使用事务确保评论计数准确
        let mut tx = pool.begin().await?;

        // 插入评论
        sqlx::query(
            "INSERT INTO post_comments (id, post_id, user_id, content, status, created_at)
             VALUES (?, ?, ?, ?, 'normal', datetime('now'))",
        )
        .bind(&comment_id)
        .bind(post_id)
        .bind(&user_id)
        .bind(content)
        .execute(&mut *tx)
        .await
        .context("创建评论失败")?;

        // 更新评论计数
        sqlx::query("UPDATE posts SET comments_count = comments_count + 1 WHERE id = ?")
            .bind(post_id)
            .execute(&mut *tx)
            .await
            .context("更新评论计数失败")?;

        tx.commit().await?;

        Ok(())
    }

    async fn like_dislike_post(&self, post_id: &str) -> anyhow::Result<()> {
        leptos::logging::debug_log!("切换点赞状态: {}", post_id);

        let pool = &self.pool;
        let user_id = get_user_id_from_session(pool).await?;

        // 检查是否已点赞
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM post_likes WHERE user_id = ? AND post_id = ?")
                .bind(&user_id)
                .bind(post_id)
                .fetch_one(pool)
                .await?;

        let mut tx = pool.begin().await?;

        if count > 0 {
            // 取消点赞
            sqlx::query("DELETE FROM post_likes WHERE user_id = ? AND post_id = ?")
                .bind(&user_id)
                .bind(post_id)
                .execute(&mut *tx)
                .await
                .context("取消点赞失败")?;

            sqlx::query("UPDATE posts SET likes_count = likes_count - 1 WHERE id = ?")
                .bind(post_id)
                .execute(&mut *tx)
                .await
                .context("更新点赞计数失败")?;

            tx.commit().await?;
            Ok(())
        } else {
            // 点赞
            sqlx::query(
                "INSERT INTO post_likes (user_id, post_id, created_at)
                 VALUES (?, ?, datetime('now'))",
            )
            .bind(&user_id)
            .bind(post_id)
            .execute(&mut *tx)
            .await
            .context("点赞失败")?;

            sqlx::query("UPDATE posts SET likes_count = likes_count + 1 WHERE id = ?")
                .bind(post_id)
                .execute(&mut *tx)
                .await
                .context("更新点赞计数失败")?;

            tx.commit().await?;
            Ok(())
        }
    }
}
