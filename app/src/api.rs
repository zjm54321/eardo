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
pub struct UserAuthInfo {
    pub username: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}
/// 登录/注册/登出/获取当前用户 接口
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register(&self, userauth: UserAuthInfo, password: &str) -> anyhow::Result<()>;
    async fn login(&self, userauth: UserAuthInfo, password: &str) -> anyhow::Result<()>;
    async fn logout(&self) -> anyhow::Result<()>;
    async fn get_current_user(&self) -> anyhow::Result<()>;
}
pub type AuthProvider = Arc<dyn AuthService>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserStatus {
    Normal,
    Deleted,
    Banned,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub avatar_url: String,
    pub status: UserStatus,
    pub meta: String, // JSON 格式的用户元数据
}
/// 用户资料管理接口
#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_user_profile(&self) -> anyhow::Result<UserInfo>;
    async fn update_user_profile(&self, user: &UserInfo) -> anyhow::Result<()>;
}
pub type UserServiceProvider = Arc<dyn UserService>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceModelInfo {
    id: String,
    name: String,
    icon_url: String,
    metadata: String, // JSON 格式的模型元数据
}
/// 语音模型管理接口
#[async_trait]
pub trait VoiceModelService: Send + Sync {
    async fn list_voice_models(&self) -> anyhow::Result<Vec<VoiceModelInfo>>;
    async fn get_voice_model(&self, voice_id: &str) -> anyhow::Result<VoiceModelInfo>;
    async fn update_voice_model(&self, voice: &VoiceModelInfo) -> anyhow::Result<()>;
    async fn delete_voice_model(&self, voice_id: &str) -> anyhow::Result<()>;
}
pub type VoiceModelProvider = Arc<dyn VoiceModelService>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMetaInfo {
    pub id: String,
    pub name: String,
    pub metadata: String, // JSON 格式的元数据
}
/// 语音元数据(声音滤镜)管理接口
#[async_trait]
pub trait VoiceMetadataService: Send + Sync {
    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<VoiceMetaInfo>>;
    async fn get_voice_metadata(&self, voice_id: &str) -> anyhow::Result<VoiceMetaInfo>;
    async fn update_voice_metadata(&self, metadata: &VoiceMetaInfo) -> anyhow::Result<()>;
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
    pub metadata: String, // JSON 格式的帖子元数据
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
pub mod user;
pub mod voicedata;

#[server]
pub async fn register(userauth: UserAuthInfo, password: String) -> Result<(), ServerFnError> {
    use_context::<AuthProvider>()
        .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
        .register(userauth, &password)
        .await
        .map_err(|e| ServerFnError::new(format!("注册失败: {}", e)))
}

#[server]
pub async fn login(userauth: UserAuthInfo, password: String) -> Result<(), ServerFnError> {
    use_context::<AuthProvider>()
        .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
        .login(userauth, &password)
        .await
        .map_err(|e| ServerFnError::new(format!("登录失败: {}", e)))
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use_context::<AuthProvider>()
        .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
        .logout()
        .await
        .map_err(|e| ServerFnError::new(format!("登出失败: {}", e)))
}

#[server]
pub async fn get_current_user() -> Result<(), ServerFnError> {
    use_context::<AuthProvider>()
        .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
        .get_current_user()
        .await
        .map_err(|e| ServerFnError::new(format!("获取当前登录用户失败: {}", e)))
}

// === 用户资料模块 ===

#[server]
pub async fn get_user_profile() -> Result<UserInfo, ServerFnError> {
    use_context::<UserServiceProvider>()
        .ok_or_else(|| ServerFnError::new("未找到用户服务组件(UserServiceProvider)"))?
        .get_user_profile()
        .await
        .map_err(|e| ServerFnError::new(format!("获取个人资料失败: {}", e)))
}

#[server]
pub async fn update_user_profile(user: UserInfo) -> Result<(), ServerFnError> {
    use_context::<UserServiceProvider>()
        .ok_or_else(|| ServerFnError::new("未找到用户服务组件(UserServiceProvider)"))?
        .update_user_profile(&user)
        .await
        .map_err(|e| ServerFnError::new(format!("更新个人资料失败: {}", e)))
}

// === 语音模型模块 ===

#[server]
pub async fn list_voice_models() -> Result<Vec<VoiceModelInfo>, ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音模型服务组件(VoiceModelProvider)"))?
        .list_voice_models()
        .await
        .map_err(|e| ServerFnError::new(format!("获取语音模型列表失败: {}", e)))
}

#[server]
pub async fn get_voice_model(voice_id: String) -> Result<VoiceModelInfo, ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音模型服务组件(VoiceModelProvider)"))?
        .get_voice_model(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(format!("获取语音模型详情失败: {}", e)))
}

#[server]
pub async fn update_voice_model(voice: VoiceModelInfo) -> Result<(), ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音模型服务组件(VoiceModelProvider)"))?
        .update_voice_model(&voice)
        .await
        .map_err(|e| ServerFnError::new(format!("更新语音模型失败: {}", e)))
}

#[server]
pub async fn delete_voice_model(voice_id: String) -> Result<(), ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音模型服务组件(VoiceModelProvider)"))?
        .delete_voice_model(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(format!("删除语音模型失败: {}", e)))
}

// === 语音元数据模块 ===

#[server]
pub async fn list_voice_metadata() -> Result<Vec<VoiceMetaInfo>, ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音元数据服务组件(VoiceMetadataProvider)"))?
        .list_voice_metadata()
        .await
        .map_err(|e| ServerFnError::new(format!("获取语音元数据列表失败: {}", e)))
}

#[server]
pub async fn get_voice_metadata(voice_id: String) -> Result<VoiceMetaInfo, ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音元数据服务组件(VoiceMetadataProvider)"))?
        .get_voice_metadata(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(format!("获取语音元数据详情失败: {}", e)))
}

#[server]
pub async fn update_voice_metadata(metadata: VoiceMetaInfo) -> Result<(), ServerFnError> {
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

#[server]
pub async fn generate_audio(
    voice_meta: VoiceMetaInfo,
    input_text: String,
) -> Result<Vec<u8>, ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("未找到语音元数据服务组件(VoiceMetadataProvider)"))?
        .generate_voice(&voice_meta, &input_text)
        .await
        .map_err(|e| ServerFnError::new(format!("生成音频失败: {}", e)))
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
