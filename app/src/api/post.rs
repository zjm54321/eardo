#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl PostService for ServiceProvider<Sqlite> {
    async fn list_posts(&self) -> anyhow::Result<Vec<PostInfo>> {
        // 模拟实现：返回示例帖子数据
        let posts = vec![
            PostInfo {
                id: "1".to_string(),
                title: "我的第一个声音作品".to_string(),
                metadata: serde_json::json!({
                    "author": "小明",
                    "avatar": "https://i.pravatar.cc/150?img=1",
                    "time": "2小时前",
                    "description": "这是我第一次尝试使用 AI 生成声音，效果还不错！",
                    "likes": 156,
                    "comments": 23,
                    "voice_type": "甜美少女音",
                    "audio_url": "https://example.com/audio1.mp3"
                })
                .to_string(),
            },
            PostInfo {
                id: "2".to_string(),
                title: "有声小说配音练习".to_string(),
                metadata: serde_json::json!({
                    "author": "声音艺术家",
                    "avatar": "https://i.pravatar.cc/150?img=2",
                    "time": "5小时前",
                    "description": "尝试用 AI 为小说片段配音，感觉很有趣",
                    "likes": 289,
                    "comments": 45,
                    "voice_type": "磁性男声",
                    "audio_url": "https://example.com/audio2.mp3"
                })
                .to_string(),
            },
            PostInfo {
                id: "3".to_string(),
                title: "游戏角色语音包".to_string(),
                metadata: serde_json::json!({
                    "author": "游戏开发者",
                    "avatar": "https://i.pravatar.cc/150?img=3",
                    "time": "1天前",
                    "description": "为我的独立游戏制作的角色语音包，欢迎试听",
                    "likes": 421,
                    "comments": 67,
                    "voice_type": "活力少年",
                    "audio_url": "https://example.com/audio3.mp3"
                })
                .to_string(),
            },
            PostInfo {
                id: "4".to_string(),
                title: "新闻播报练习".to_string(),
                metadata: serde_json::json!({
                    "author": "主播小李",
                    "avatar": "https://i.pravatar.cc/150?img=4",
                    "time": "1天前",
                    "description": "使用 AI 生成标准普通话新闻播报",
                    "likes": 198,
                    "comments": 31,
                    "voice_type": "御姐音",
                    "audio_url": "https://example.com/audio4.mp3"
                })
                .to_string(),
            },
            PostInfo {
                id: "5".to_string(),
                title: "诗歌朗诵".to_string(),
                metadata: serde_json::json!({
                    "author": "文学爱好者",
                    "avatar": "https://i.pravatar.cc/150?img=5",
                    "time": "2天前",
                    "description": "用温柔的声音朗诵古诗词",
                    "likes": 345,
                    "comments": 52,
                    "voice_type": "甜美少女音",
                    "audio_url": "https://example.com/audio5.mp3"
                })
                .to_string(),
            },
            PostInfo {
                id: "6".to_string(),
                title: "广告配音示范".to_string(),
                metadata: serde_json::json!({
                    "author": "配音师张三",
                    "avatar": "https://i.pravatar.cc/150?img=6",
                    "time": "3天前",
                    "description": "商业广告配音效果展示",
                    "likes": 512,
                    "comments": 89,
                    "voice_type": "磁性男声",
                    "audio_url": "https://example.com/audio6.mp3"
                })
                .to_string(),
            },
        ];
        Ok(posts)
    }

    async fn search_post(&self, query: &str) -> anyhow::Result<Vec<String>> {
        // 模拟实现：根据查询词返回匹配的帖子 ID
        let all_posts = self.list_posts().await?;
        let matched_ids: Vec<String> = all_posts
            .into_iter()
            .filter(|p| p.title.to_lowercase().contains(&query.to_lowercase()))
            .map(|p| p.id)
            .collect();
        Ok(matched_ids)
    }

    async fn get_post(&self, post_id: &str) -> anyhow::Result<PostInfo> {
        // 模拟实现：返回对应 ID 的帖子
        let all_posts = self.list_posts().await?;
        all_posts
            .into_iter()
            .find(|p| p.id == post_id)
            .ok_or_else(|| anyhow::anyhow!("Post not found: {}", post_id))
    }

    async fn create_post(&self, post: &PostInfo) -> anyhow::Result<()> {
        // 模拟实现：记录创建操作
        leptos::logging::log!("Mock: Creating post: {}", post.title);
        Ok(())
    }

    async fn update_post(&self, post: &PostInfo) -> anyhow::Result<()> {
        // 模拟实现：记录更新操作
        leptos::logging::log!("Mock: Updating post: {}", post.title);
        Ok(())
    }

    async fn delete_post(&self, post_id: &str) -> anyhow::Result<()> {
        // 模拟实现：记录删除操作
        leptos::logging::log!("Mock: Deleting post: {}", post_id);
        Ok(())
    }

    async fn comment_on_post(&self, post_id: &str, comment: &str) -> anyhow::Result<()> {
        // 模拟实现：记录评论操作
        leptos::logging::log!("Mock: Commenting on post {}: {}", post_id, comment);
        Ok(())
    }

    async fn like_dislike_post(&self, post_id: &str) -> anyhow::Result<()> {
        // 模拟实现：记录点赞操作
        leptos::logging::log!("Mock: Toggling like on post: {}", post_id);
        Ok(())
    }
}
