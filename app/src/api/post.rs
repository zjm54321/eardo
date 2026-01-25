#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl PostService for ServiceProvider<Sqlite> {
    async fn list_posts(&self) -> anyhow::Result<Vec<PostInfo>> {
        todo!()
    }
    async fn search_post(&self, post_info: &str) -> anyhow::Result<Vec<String>> {
        todo!()
    }
    async fn get_post(&self, post_id: &str) -> anyhow::Result<PostInfo> {
        todo!()
    }
    async fn create_post(&self, post: &PostInfo) -> anyhow::Result<()> {
        todo!()
    }
    async fn update_post(&self, post: &PostInfo) -> anyhow::Result<()> {
        todo!()
    }
    async fn delete_post(&self, post_id: &str) -> anyhow::Result<()> {
        todo!()
    }
    async fn comment_on_post(&self, post_id: &str, comment: &str) -> anyhow::Result<()> {
        todo!()
    }
    async fn like_dislike_post(&self, post_id: &str) -> anyhow::Result<()> {
        todo!()
    }
}
