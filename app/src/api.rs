use async_trait::async_trait;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthInfo {
    username: Option<String>,
    email: Option<String>,
    phone: Option<String>,
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
    id: String,
    username: String,
    avatar_url: String,
    status: UserStatus,
    meta: String, // JSON 格式的用户元数据
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
    id: String,
    name: String,
    metadata: String, // JSON 格式的元数据
}
/// 语音元数据(声音滤镜)管理接口
#[async_trait]
pub trait VoiceMetadataService: Send + Sync {
    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<VoiceMetaInfo>>;
    async fn get_voice_metadata(&self, voice_id: &str) -> anyhow::Result<VoiceMetaInfo>;
    async fn update_voice_metadata(&self, metadata: &VoiceMetaInfo) -> anyhow::Result<()>;
    async fn delete_voice_metadata(&self, voice_id: &str) -> anyhow::Result<()>;
}
pub type VoiceMetadataProvider = Arc<dyn VoiceMetadataService>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInfo {
    id: String,
    title: String,
    metadata: String, // JSON 格式的帖子元数据
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
pub mod sql_struct;
pub mod user;
pub mod voicedata;

#[server]
pub async fn register(userauth: UserAuthInfo, password: String) -> Result<(), ServerFnError> {
    use_context::<AuthProvider>()
        .ok_or(ServerFnError::new("AuthProvider not found in context"))?
        .register(userauth, &password)
        .await
        .map_err(|e| ServerFnError::new(format!("Register failed: {}", e)))?;
}

#[server]
pub async fn login(userauth: UserAuthInfo, password: String) -> Result<(), ServerFnError> {
    use_context::<AuthProvider>()
        .ok_or_else(|| ServerFnError::new("Auth provider missing"))?
        .login(userauth, &password)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use_context::<AuthProvider>()
        .ok_or_else(|| ServerFnError::new("Auth provider missing"))?
        .logout()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_current_user() -> Result<(), ServerFnError> {
    // 注意：Trait定义中返回的是()，通常这里应该返回具体的 UserInfo 或 UserID
    use_context::<AuthProvider>()
        .ok_or_else(|| ServerFnError::new("Auth provider missing"))?
        .get_current_user()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_my_profile() -> Result<UserInfo, ServerFnError> {
    use_context::<UserServiceProvider>()
        .ok_or_else(|| ServerFnError::new("User provider missing"))?
        .get_user_profile()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_my_profile(user: UserInfo) -> Result<(), ServerFnError> {
    use_context::<UserServiceProvider>()
        .ok_or_else(|| ServerFnError::new("User provider missing"))?
        .update_user_profile(&user) // 注意这里传引用
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_voice_models() -> Result<Vec<VoiceModelInfo>, ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("Voice model provider missing"))?
        .list_voice_models()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_voice_model(voice_id: String) -> Result<VoiceModelInfo, ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("Voice model provider missing"))?
        .get_voice_model(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_voice_model(voice: VoiceModelInfo) -> Result<(), ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("Voice model provider missing"))?
        .update_voice_model(&voice)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_voice_model(voice_id: String) -> Result<(), ServerFnError> {
    use_context::<VoiceModelProvider>()
        .ok_or_else(|| ServerFnError::new("Voice model provider missing"))?
        .delete_voice_model(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_voice_metadata() -> Result<Vec<VoiceMetaInfo>, ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("Metadata provider missing"))?
        .list_voice_metadata()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_voice_metadata(voice_id: String) -> Result<VoiceMetaInfo, ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("Metadata provider missing"))?
        .get_voice_metadata(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_voice_metadata(metadata: VoiceMetaInfo) -> Result<(), ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("Metadata provider missing"))?
        .update_voice_metadata(&metadata)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_voice_metadata(voice_id: String) -> Result<(), ServerFnError> {
    use_context::<VoiceMetadataProvider>()
        .ok_or_else(|| ServerFnError::new("Metadata provider missing"))?
        .delete_voice_metadata(&voice_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn list_posts() -> Result<Vec<PostInfo>, ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .list_posts()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn search_posts(query: String) -> Result<Vec<String>, ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .search_post(&query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn get_post(post_id: String) -> Result<PostInfo, ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .get_post(&post_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_post(post: PostInfo) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .create_post(&post)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_post(post: PostInfo) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .update_post(&post)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_post(post_id: String) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .delete_post(&post_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn comment_on_post(post_id: String, comment: String) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .comment_on_post(&post_id, &comment)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn like_dislike_post(post_id: String) -> Result<(), ServerFnError> {
    use_context::<PostProvider>()
        .ok_or_else(|| ServerFnError::new("Post provider missing"))?
        .like_dislike_post(&post_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
