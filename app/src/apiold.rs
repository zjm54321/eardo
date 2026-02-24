//! 定义应用程序的各种服务接口和数据结构
//! 并且给出了实现，可以直接在页面中应用
//! api 文件夹中有具体实现
use async_trait::async_trait;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(feature = "ssr")]
pub struct ServiceProvider<DB>
where
    DB: sqlx::Database,
{
    // 2. 字段明确为 sqlx 的连接池
    pub pool: sqlx::Pool<DB>,
}

#[cfg(feature = "ssr")]
impl<DB> Clone for ServiceProvider<DB>
where
    DB: sqlx::Database,
{
    fn clone(&self) -> Self {
        Self {
            // Pool 内部是 Arc，这里只是增加引用计数，开销极小
            pool: self.pool.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceModelInfo {
    pub id: String,
    pub name: String,
    pub icon_url: String,
    pub category: VoiceModelCategory,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum VoiceModelCategory {
    // 官方模型，string字段为可用的后端接口
    Official(String),
    // 用户自定义模型，string字段为用户id
    UserDesigned,
    // 语音生成模型，string字段为对应用户id
    VoiceGenerated,
}

// --- 语音参数结构 ---
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceParams {
    pub pitch: f32,
    pub speed: f32,
    pub volume: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        VoiceParams {
            pitch: 0.0,
            speed: 1.0,
            volume: 1.0,
        }
    }
}

/// 用于声音生成的最小信息集合（仅包含生成所需的参数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMetaInfo {
    pub base_model: VoiceModelInfo,
    pub metadata: VoiceMetadata, // 枚举类型：控制方式
}

/// 用于声音滤镜列表/详情展示（包含POST所需的完整信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMetaPost {
    pub id: String,
    pub name: String,
    pub base_model: VoiceModelInfo, // 关联的基础模型信息
    pub metadata: VoiceMetadata,    // 枚举类型：控制方式
    pub author: String,             // 作者
    pub description: String,        // 描述
    pub tags: Vec<String>,          // 标签
    pub usage_count: i32,           // 使用次数s
    pub is_public: bool,            // 是否公开
    pub is_official: bool,          // 是否官方推荐
    pub created_at: String,         // 创建日期
    pub updated_at: String,         // 更新日期
}

// VoiceMetadata 枚举：支持参数控制和指令控制两种方式
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum VoiceMetadata {
    // 参数控制：使用 VoiceParams 的方式
    Parametric(VoiceParams),
    // 指令控制：使用自然语言指令的方式
    Instruction(String),
}

impl Default for VoiceMetadata {
    fn default() -> Self {
        VoiceMetadata::Parametric(VoiceParams::default())
    }
}
/// 语音元数据(声音滤镜)管理接口
#[async_trait]
pub trait VoiceMetadataService: Send + Sync {
    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<VoiceMetaPost>>;
    async fn get_voice_metadata(&self, voice_id: &str) -> anyhow::Result<VoiceMetaPost>;
    async fn update_voice_metadata(&self, metadata: &VoiceMetaPost) -> anyhow::Result<()>;
    async fn delete_voice_metadata(&self, voice_id: &str) -> anyhow::Result<()>;
    async fn generate_voice(
        &self,
        voice_info: &VoiceMetaInfo,
        input_text: &str,
    ) -> anyhow::Result<Vec<u8>>; // 返回音频数据
}
pub type VoiceMetadataProvider = Arc<dyn VoiceMetadataService>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInfo {
    pub id: String,
    pub title: String,
    pub author: AuthorInfo,
    pub content: PostContent,
    pub meta: PostMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub name: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostContent {
    pub description: Option<String>,
    pub audio_url: Option<String>,
    pub audio_data: Option<Vec<u8>>, // 用于创建帖子时传递音频数据
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMeta {
    pub likes: i32,
    pub comments: i32,
    pub is_liked: bool,
    pub time: String,
    pub voice_info: VoiceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub voice_type: String,
    pub voice_meta_id: Option<String>, // 用于创建帖子时指定voice meta
}
/// 帖子管理接口
#[async_trait]
pub trait PostService: Send + Sync {
    async fn list_posts(&self) -> anyhow::Result<Vec<PostInfo>>;
    async fn search_post(&self, post_info: &str) -> anyhow::Result<Vec<String>>;
    async fn get_post(&self, post_id: &str) -> anyhow::Result<PostInfo>;
    async fn create_post(&self, post: &PostInfo) -> anyhow::Result<()>;
    async fn update_post(&self, post: &PostInfo) -> anyhow::Result<()>;
    async fn delete_post(&self, post_id: &str) -> anyhow::Result<()>;
    async fn comment_on_post(&self, post_id: &str, comment: &str) -> anyhow::Result<()>;
    async fn like_dislike_post(&self, post_id: &str) -> anyhow::Result<()>;
}
pub type PostProvider = Arc<dyn PostService>;

pub mod post;
pub mod voice_backend_api;
pub mod voicedata;

// === 语音元数据模块 ===

#[server]
pub async fn list_voice_metadata() -> Result<Vec<VoiceMetaPost>, ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音元数据服务组件(VoiceMetadataProvider)"))?
        .list_voice_metadata()
        .await
        .map_err(|e| ServerFnError::new(format!("获取语音元数据列表失败: {}", e)))
}

#[server]
pub async fn get_voice_metadata(voice_id: String) -> Result<VoiceMetaPost, ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音元数据服务组件(VoiceMetadataProvider)"))?
        .get_voice_metadata(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(format!("获取语音元数据详情失败: {}", e)))
}

#[server]
pub async fn update_voice_metadata(metadata: VoiceMetaPost) -> Result<(), ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音元数据服务组件(VoiceMetadataProvider)"))?
        .update_voice_metadata(&metadata)
        .await
        .map_err(|e| ServerFnError::new(format!("更新语音元数据失败: {}", e)))
}

#[server]
pub async fn delete_voice_metadata(voice_id: String) -> Result<(), ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音元数据服务组件(VoiceMetadataProvider)"))?
        .delete_voice_metadata(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(format!("删除语音元数据失败: {}", e)))
}

// === 帖子模块 ===

#[server]
pub async fn list_posts() -> Result<Vec<PostInfo>, ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .list_posts()
        .await
        .map_err(|e| ServerFnError::new(format!("获取帖子列表失败: {}", e)))
}

#[server]
pub async fn search_post(query: String) -> Result<Vec<String>, ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .search_post(&query)
        .await
        .map_err(|e| ServerFnError::new(format!("搜索帖子失败: {}", e)))
}

#[server]
pub async fn get_post(post_id: String) -> Result<PostInfo, ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .get_post(&post_id)
        .await
        .map_err(|e| ServerFnError::new(format!("获取帖子详情失败: {}", e)))
}

#[server]
pub async fn create_post(post: PostInfo) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .create_post(&post)
        .await
        .map_err(|e| ServerFnError::new(format!("创建帖子失败: {}", e)))
}

#[server]
pub async fn update_post(post: PostInfo) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .update_post(&post)
        .await
        .map_err(|e| ServerFnError::new(format!("更新帖子失败: {}", e)))
}

#[server]
pub async fn delete_post(post_id: String) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .delete_post(&post_id)
        .await
        .map_err(|e| ServerFnError::new(format!("删除帖子失败: {}", e)))
}

#[server]
pub async fn comment_on_post(post_id: String, comment: String) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .comment_on_post(&post_id, &comment)
        .await
        .map_err(|e| ServerFnError::new(format!("评论帖子失败: {}", e)))
}

#[server]
pub async fn like_dislike_post(post_id: String) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("未找到帖子服务组件(PostProvider)"))?
        .like_dislike_post(&post_id)
        .await
        .map_err(|e| ServerFnError::new(format!("点赞/取消作失败: {}", e)))
}
