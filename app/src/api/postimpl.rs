#![cfg(feature = "ssr")]
use {
    crate::api::{
        ServiceProvider,
        post::*,
        user::UserRole,
        userimpl::{get_authenticated_user, get_user_id_from_session},
    },
    anyhow::{Context, anyhow},
    sqlx::Postgres,
    uuid::Uuid,
};

// `table` is always one of the two internal literal table names below, never input.
async fn apply_action(
    pool: &sqlx::Pool<Postgres>,
    table: &str,
    post_id: Uuid,
    action: Action,
) -> anyhow::Result<LikeState> {
    let user_id = get_user_id_from_session(pool).await?;
    let mut tx = pool.begin().await?;
    // Serialize toggles (including the count triggers) and check visibility atomically.
    sqlx::query(&format!(
        "SELECT id FROM {table} WHERE id = $1 AND status IN ('Normal', 'Recommended') FOR UPDATE"
    ))
    .bind(post_id)
    .fetch_one(&mut *tx)
    .await
    .context("帖子不存在或不可见")?;
    match action {
        Action::LikeDislike => {
            let deleted = sqlx::query(&format!(
                "DELETE FROM {table}_like WHERE post_id = $1 AND user_id = $2"
            ))
            .bind(post_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
            if deleted.rows_affected() == 0 {
                sqlx::query(&format!(
                    "INSERT INTO {table}_like (post_id, user_id) VALUES ($1, $2)"
                ))
                .bind(post_id)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
            }
        }
        Action::Comment(content) => {
            sqlx::query(&format!(
                "INSERT INTO {table}_comment (post_id, user_id, content) VALUES ($1, $2, $3)"
            ))
            .bind(post_id)
            .bind(user_id)
            .bind(content)
            .execute(&mut *tx)
            .await?;
        }
    }
    let state = sqlx::query_as::<_, LikeState>(&format!(
        "SELECT EXISTS(SELECT 1 FROM {table}_like WHERE post_id = $1 AND user_id = $2) AS liked,
                likes_count FROM {table} WHERE id = $1"
    ))
    .bind(post_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(state)
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
        .bind(PostStatus::Normal)
        .fetch_one(&self.pool)
        .await
        .context("创建帖子失败")?;

        Ok(id)
    }

    async fn get(&self, post_id: VoiceMetaPostID) -> anyhow::Result<VoiceMetaPost> {
        let current_user_id = get_user_id_from_session(&self.pool).await.ok();

        let post = if let Some(user_id) = current_user_id {
            sqlx::query_as::<_, VoiceMetaPost>(
                "SELECT p.id, p.title, p.content, p.meta_id, p.author, p.status, p.comments_count, p.likes_count,
                        CASE WHEN l.user_id IS NOT NULL THEN true ELSE false END as is_liked_by_current_user
                 FROM voice_meta_post p
                 LEFT JOIN voice_meta_post_like l ON p.id = l.post_id AND l.user_id = $2
                  WHERE p.id = $1 AND p.status IN ('Normal', 'Recommended')",
            )
            .bind(post_id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .context("获取帖子失败")?
        } else {
            sqlx::query_as::<_, VoiceMetaPost>(
                "SELECT id, title, content, meta_id, author, status, comments_count, likes_count, false as is_liked_by_current_user
                 FROM voice_meta_post WHERE id = $1 AND status IN ('Normal', 'Recommended')",
            )
            .bind(post_id)
            .fetch_one(&self.pool)
            .await
            .context("获取帖子失败")?
        };

        Ok(post)
    }

    async fn update(&self, post: VoiceMetaPost) -> anyhow::Result<()> {
        let (user_id, role) = get_authenticated_user(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_meta_post SET title = $1, content = $2, meta_id = $3,
                 status = CASE WHEN $7 THEN $4::post_status ELSE status END
             WHERE id = $5 AND ($7 OR (author = $6 AND status IN ('Normal', 'Recommended') AND status = $4::post_status))"
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
        .bind(matches!(role, UserRole::Admin))
        .execute(&self.pool)
        .await
        .context("更新帖子失败")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("帖子不存在或无权修改"));
        }

        Ok(())
    }

    async fn delete(&self, post_id: VoiceMetaPostID) -> anyhow::Result<()> {
        let (user_id, role) = get_authenticated_user(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_meta_post SET status = 'Deleted'::post_status WHERE id = $1
             AND ($3 OR (author = $2 AND status IN ('Normal', 'Recommended')))",
        )
        .bind(post_id)
        .bind(user_id)
        .bind(matches!(role, UserRole::Admin))
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
        let current_user_id = get_user_id_from_session(&self.pool).await.ok();

        let posts = if let Some(user_id) = current_user_id {
            sqlx::query_as::<_, VoiceMetaPost>(
                "SELECT p.id, p.title, p.content, p.meta_id, p.author, p.status, p.comments_count, p.likes_count,
                        CASE WHEN l.user_id IS NOT NULL THEN true ELSE false END as is_liked_by_current_user
                 FROM voice_meta_post p
                 LEFT JOIN voice_meta_post_like l ON p.id = l.post_id AND l.user_id = $3
                  WHERE p.status IN ('Normal', 'Recommended')
                 AND (p.title ILIKE $1 OR p.content ILIKE $2)
                  ORDER BY p.created_at DESC LIMIT 100",
            )
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .context("搜索帖子失败")?
        } else {
            sqlx::query_as::<_, VoiceMetaPost>(
                "SELECT id, title, content, meta_id, author, status, comments_count, likes_count, false as is_liked_by_current_user
                 FROM voice_meta_post 
                  WHERE status IN ('Normal', 'Recommended')
                 AND (title ILIKE $1 OR content ILIKE $2)
                  ORDER BY created_at DESC LIMIT 100",
            )
            .bind(&search_pattern)
            .bind(&search_pattern)
            .fetch_all(&self.pool)
            .await
            .context("搜索帖子失败")?
        };

        Ok(posts)
    }

    async fn list(
        &self,
        status: Option<PostStatus>,
        number: u8,
    ) -> anyhow::Result<Vec<VoiceMetaPost>> {
        let current_user_id = get_user_id_from_session(&self.pool).await.ok();
        let posts = sqlx::query_as::<_, VoiceMetaPost>(
            "SELECT p.id, p.title, p.content, p.meta_id, p.author, p.status,
                    p.comments_count, p.likes_count,
                    EXISTS(SELECT 1 FROM voice_meta_post_like l WHERE l.post_id = p.id AND l.user_id = $1)
                        AS is_liked_by_current_user
             FROM voice_meta_post p
             WHERE p.status IN ('Normal', 'Recommended')
               AND ($2::post_status IS NULL OR p.status = $2::post_status)
             ORDER BY p.created_at DESC LIMIT $3"
        )
        .bind(current_user_id)
        .bind(status)
        .bind(i64::from(number.min(100)))
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    async fn action(&self, post_id: VoiceMetaPostID, action: Action) -> anyhow::Result<()> {
        apply_action(&self.pool, "voice_meta_post", post_id, action).await?;
        Ok(())
    }

    async fn toggle_like(&self, post_id: VoiceMetaPostID) -> anyhow::Result<LikeState> {
        apply_action(&self.pool, "voice_meta_post", post_id, Action::LikeDislike).await
    }

    async fn get_comments(&self, post_id: VoiceMetaPostID) -> anyhow::Result<Vec<PostComment>> {
        let comments = sqlx::query_as::<_, PostComment>(
            "SELECT c.user_id, c.content,
                    COALESCE(NULLIF(BTRIM(u.nick_name), ''), '用户' || LEFT(c.user_id::text, 4)) AS nick_name
             FROM voice_meta_post_comment c
             JOIN voice_meta_post p ON p.id = c.post_id
             JOIN \"user\" u ON u.id = c.user_id
             WHERE c.post_id = $1 AND p.status IN ('Normal', 'Recommended')
             ORDER BY c.created_at ASC LIMIT 100",
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
        .bind(PostStatus::Normal)
        .fetch_one(&self.pool)
        .await
        .context("创建帖子失败")?;

        Ok(id)
    }

    async fn get(&self, post_id: VoicePostID) -> anyhow::Result<VoicePost> {
        let post = sqlx::query_as::<_, VoicePost>(
            "SELECT id, title, content, library_id, author, status, comments_count, likes_count FROM voice_post
             WHERE id = $1 AND status IN ('Normal', 'Recommended')",
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await
        .context("获取帖子失败")?;

        Ok(post)
    }

    async fn update(&self, post: VoicePost) -> anyhow::Result<()> {
        let (user_id, role) = get_authenticated_user(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_post SET title = $1, content = $2, library_id = $3,
                 status = CASE WHEN $7 THEN $4::post_status ELSE status END
             WHERE id = $5 AND ($7 OR (author = $6 AND status IN ('Normal', 'Recommended') AND status = $4::post_status))"
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
        .bind(matches!(role, UserRole::Admin))
        .execute(&self.pool)
        .await
        .context("更新帖子失败")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("帖子不存在或无权修改"));
        }

        Ok(())
    }

    async fn delete(&self, post_id: VoicePostID) -> anyhow::Result<()> {
        let (user_id, role) = get_authenticated_user(&self.pool).await?;

        let result = sqlx::query(
            "UPDATE voice_post SET status = 'Deleted'::post_status WHERE id = $1
             AND ($3 OR (author = $2 AND status IN ('Normal', 'Recommended')))",
        )
        .bind(post_id)
        .bind(user_id)
        .bind(matches!(role, UserRole::Admin))
        .execute(&self.pool)
        .await
        .context("删除帖子失败")?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("帖子不存在或无权删除"));
        }

        Ok(())
    }

    async fn search(&self, query: &str) -> anyhow::Result<Vec<VoicePost>> {
        if let Some(user_id) = query.trim().strip_prefix("uid:") {
            let author = Uuid::parse_str(user_id.trim()).context("无效的用户ID搜索条件")?;

            let posts = sqlx::query_as::<_, VoicePost>(
                "SELECT id, title, content, library_id, author, status, comments_count, likes_count FROM voice_post 
                  WHERE status IN ('Normal', 'Recommended')
                 AND author = $1
                  ORDER BY created_at DESC LIMIT 100",
            )
            .bind(author)
            .fetch_all(&self.pool)
            .await
            .context("按用户搜索帖子失败")?;

            return Ok(posts);
        }

        let search_pattern = format!("%{}%", query);

        let posts = sqlx::query_as::<_, VoicePost>(
            "SELECT id, title, content, library_id, author, status, comments_count, likes_count FROM voice_post 
              WHERE status IN ('Normal', 'Recommended')
             AND (title ILIKE $1 OR content ILIKE $2)
              ORDER BY created_at DESC LIMIT 100",
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await
        .context("搜索帖子失败")?;

        Ok(posts)
    }

    async fn list(&self, status: Option<PostStatus>, number: u8) -> anyhow::Result<Vec<VoicePost>> {
        Ok(self
            .list_with_likes(status, number)
            .await?
            .into_iter()
            .map(|row| row.post)
            .collect())
    }

    async fn list_with_likes(
        &self,
        status: Option<PostStatus>,
        number: u8,
    ) -> anyhow::Result<Vec<VoicePostWithLike>> {
        let user_id = get_user_id_from_session(&self.pool).await.ok();
        let posts = sqlx::query_as::<_, VoicePostWithLike>(
            "SELECT p.id, p.title, p.content, p.library_id, p.author, p.status,
                    p.comments_count, p.likes_count,
                    EXISTS(SELECT 1 FROM voice_post_like l WHERE l.post_id = p.id AND l.user_id = $1)
                        AS is_liked_by_current_user
             FROM voice_post p
             WHERE p.status IN ('Normal', 'Recommended')
               AND ($2::post_status IS NULL OR p.status = $2::post_status)
             ORDER BY p.created_at DESC LIMIT $3"
        )
        .bind(user_id)
        .bind(status)
        .bind(i64::from(number.min(100)))
        .fetch_all(&self.pool)
        .await?;
        Ok(posts)
    }

    async fn action(&self, post_id: VoicePostID, action: Action) -> anyhow::Result<()> {
        apply_action(&self.pool, "voice_post", post_id, action).await?;
        Ok(())
    }

    async fn toggle_like(&self, post_id: VoicePostID) -> anyhow::Result<LikeState> {
        apply_action(&self.pool, "voice_post", post_id, Action::LikeDislike).await
    }

    async fn get_comments(&self, post_id: VoicePostID) -> anyhow::Result<Vec<PostComment>> {
        let comments = sqlx::query_as::<_, PostComment>(
            "SELECT c.user_id, c.content,
                    COALESCE(NULLIF(BTRIM(u.nick_name), ''), '用户' || LEFT(c.user_id::text, 4)) AS nick_name
             FROM voice_post_comment c
             JOIN voice_post p ON p.id = c.post_id
             JOIN \"user\" u ON u.id = c.user_id
             WHERE c.post_id = $1 AND p.status IN ('Normal', 'Recommended')
             ORDER BY c.created_at ASC LIMIT 100",
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .context("获取评论失败")?;

        Ok(comments)
    }
}
