/// 登录/注册/登出/获取当前用户 接口
/// - UserAuth: 用户认证信息类型（用户名、手机号、邮箱等）
#[async_trait::async_trait]
pub trait Auth {
    type UserAuth;

    async fn register(userauth: &Self::UserAuth, password: &str) -> anyhow::Result<()>;
    async fn login(&mut self, userauth: &str, password: &str) -> anyhow::Result<()>;
    async fn logout(&mut self) -> anyhow::Result<()>;
    async fn get_current_user(&mut self) -> anyhow::Result<()>;
}

/// 用户资料管理接口
/// - UserId: 用户唯一标识类型
/// - User: 用户资料
#[async_trait::async_trait]
pub trait UserProfile {
    type User;

    async fn get_user_profile(&self) -> anyhow::Result<Self::User>;
    async fn update_user_profile(&self, user: &Self::User) -> anyhow::Result<()>;
}

/// 语音模型管理接口
/// - VoiceId: 语音模型唯一标识类型
/// - VoiceModel: 语音模型信息类型
#[async_trait::async_trait]
pub trait VoiceModel {
    type VoiceId;
    type VoiceModel;

    async fn list_voice_models(&self) -> anyhow::Result<Vec<Self::VoiceModel>>;
    async fn get_voice_model(&self, voice_id: &Self::VoiceId) -> anyhow::Result<Self::VoiceModel>;
    async fn update_voice_model(&self, voice: &Self::VoiceModel) -> anyhow::Result<()>;
    async fn delete_voice_model(&self, voice_id: &Self::VoiceId) -> anyhow::Result<()>;
}

/// 语音元数据(声音滤镜)管理接口
/// - VoiceId: 语音唯一标识类型
/// - VoiceMetadata: 语音元数据类型
#[async_trait::async_trait]
pub trait VoiceMetadata {
    type VoiceId;
    type VoiceMetadata;

    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<Self::VoiceMetadata>>;
    async fn get_voice_metadata(
        &self,
        voice_id: &Self::VoiceId,
    ) -> anyhow::Result<Self::VoiceMetadata>;
    async fn update_voice_metadata(&self, metadata: &Self::VoiceMetadata) -> anyhow::Result<()>;
    async fn delete_voice_metadata(&self, voice_id: &Self::VoiceId) -> anyhow::Result<()>;
}

/// 帖子管理接口
/// - PostId: 帖子唯一标识类型
/// - Post: 帖子信息
#[async_trait::async_trait]
pub trait Posts {
    type PostId;
    type Post;

    async fn list_posts(&self) -> anyhow::Result<Vec<Self::Post>>;
    async fn search_post(&self, post_info: &str) -> anyhow::Result<Vec<Self::PostId>>;
    async fn get_post(&self, post_id: &Self::PostId) -> anyhow::Result<Self::Post>;
    async fn create_post(&self, post: &Self::Post) -> anyhow::Result<()>;
    async fn update_post(&self, post: &Self::Post) -> anyhow::Result<()>;
    async fn delete_post(&self, post_id: &Self::PostId) -> anyhow::Result<()>;
    async fn comment_on_post(&self, post_id: &Self::PostId, comment: &str) -> anyhow::Result<()>;
    async fn like_dislike_post(&self, post_id: &Self::PostId) -> anyhow::Result<()>;
}

pub mod post;
pub mod sql_struct;
pub mod user;
pub mod voicedata;
