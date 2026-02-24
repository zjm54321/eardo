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
/**
# 用户系统设计
用户id为UUID，登录方式使用`UserAuth`枚举区分不同的登录方式，
支持账户密码、第三方登录（OAuth 2.0 / OIDC）和Passkey方式。

用户信息存储在`UserMeta`结构体中，
包括昵称、头像、简介、角色和等级等字段。
角色分为管理员和普通用户。

提供了`AuthService`和`UserService`两个异步接口，分别用于用户认证和用户信息管理。

# 未完成部分 [todo)
- OAuth和Passkey的具体实现细节（如字段设计、第三方登录流程等）需要根据实际需求进一步完善。

*/
pub mod user {
    use email_address::EmailAddress;
    use leptos::{prelude::*, server_fn::codec::Json};
    use phonenumber::PhoneNumber;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use uuid::Uuid;

    pub type UserID = Uuid;

    // 以下为用户认证相关的设计
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum UserAuth {
        Password(PasswordAuth),
        OAuth(OAuthProvider),
        Passkey(),
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PasswordAuth {
        pub auth_id: AuthID,
        /// 此处的哈希是简单的SHA256哈希，仅用于前端传输安全，后端使用bcrypt等更安全的哈希算法存储密码
        pub password_hash: String,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AuthID {
        Email(EmailAddress),
        Phone(PhoneNumber),
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum OAuthProvider {
        Github(),
        WeChat(),
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum UpdateAuthInfo {
        ChangePassword(String, String), // 旧密码，新密码
        AddAuth(UserAuth),
        RemoveAuth(AuthID),
        ChangePassAuth(AuthID),
    }

    /// 用户信息结构体
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: UserID,
        pub usermeta: UserMeta,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
    pub struct UserMeta {
        pub nick_name: String,
        #[cfg_attr(feature = "ssr", sqlx(default))]
        pub avatar_url: String,
        pub bio: String,
        pub role: UserRole,
        pub level: i32,
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "ssr", derive(sqlx::Type))]
    #[cfg_attr(feature = "ssr", sqlx(type_name = "user_role"))]
    pub enum UserRole {
        Admin,
        User,
        FakeUser,
        BannedUser,
        DeletedUser,
        Bot,
    }

    #[async_trait::async_trait]
    pub trait AuthService: Send + Sync {
        async fn register(&self, userauth: UserAuth) -> anyhow::Result<UserID>;
        async fn login(&self, userauth: UserAuth) -> anyhow::Result<UserID>;
        async fn logout(&self) -> anyhow::Result<()>;
        async fn get_authinfo(&self) -> anyhow::Result<Vec<UserAuth>>;
        /// 用于更新或添加用户登录信息（如绑定邮箱/手机号、修改密码等）
        async fn update_authinfo(&self, update_info: UpdateAuthInfo) -> anyhow::Result<()>;
    }
    pub type AuthProvider = Arc<dyn AuthService>;
    #[async_trait::async_trait]
    pub trait UserService: Send + Sync {
        async fn get_user_profile(&self) -> anyhow::Result<User>;
        async fn update_user_profile(&self, user: &User) -> anyhow::Result<()>;
        /// 用于更新用户头像，接收二进制数据
        async fn update_user_avatar(&self, avatar_data: Vec<u8>) -> anyhow::Result<()>;
    }
    pub type UserProvider = Arc<dyn UserService>;

    //以下为server实现，可以直接在前端使用
    #[server(input = Json)]
    pub async fn register(userauth: UserAuth) -> Result<UserID, ServerFnError> {
        use_context::<AuthProvider>()
            .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
            .register(userauth)
            .await
            .map_err(|e| ServerFnError::new(format!("注册失败: {}", e)))
    }
    #[server(input = Json)]
    pub async fn login(userauth: UserAuth) -> Result<UserID, ServerFnError> {
        use_context::<AuthProvider>()
            .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
            .login(userauth)
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
    #[server(input = Json)]
    pub async fn get_authinfo() -> Result<Vec<UserAuth>, ServerFnError> {
        use_context::<AuthProvider>()
            .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
            .get_authinfo()
            .await
            .map_err(|e| ServerFnError::new(format!("获取认证信息失败: {}", e)))
    }
    #[server(input = Json)]
    pub async fn update_authinfo(update_info: UpdateAuthInfo) -> Result<(), ServerFnError> {
        use_context::<AuthProvider>()
            .ok_or_else(|| ServerFnError::new("未找到认证服务组件(AuthProvider)"))?
            .update_authinfo(update_info)
            .await
            .map_err(|e| ServerFnError::new(format!("更新认证信息失败: {}", e)))
    }
    #[server]
    pub async fn get_user_profile() -> Result<User, ServerFnError> {
        use_context::<UserProvider>()
            .ok_or_else(|| ServerFnError::new("未找到用户服务组件(UserProvider)"))?
            .get_user_profile()
            .await
            .map_err(|e| ServerFnError::new(format!("获取用户信息失败: {}", e)))
    }
    #[server]
    pub async fn update_user_profile(user: User) -> Result<(), ServerFnError> {
        use_context::<UserProvider>()
            .ok_or_else(|| ServerFnError::new("未找到用户服务组件(UserProvider)"))?
            .update_user_profile(&user)
            .await
            .map_err(|e| ServerFnError::new(format!("更新用户信息失败: {}", e)))
    }

    #[server]
    pub async fn update_user_avatar(avatar_data: Vec<u8>) -> Result<(), ServerFnError> {
        use_context::<UserProvider>()
            .ok_or_else(|| ServerFnError::new("未找到用户服务组件(UserProvider)"))?
            .update_user_avatar(avatar_data)
            .await
            .map_err(|e| ServerFnError::new(format!("更新用户头像失败: {}", e)))
    }
}
mod userimpl;

pub mod voice {
    use leptos::{prelude::*, server_fn::codec::Json};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use uuid::Uuid;

    pub type VoiceModelID = Uuid;
    pub type VoiceMetaID = Uuid;
    pub type VoiceLibraryID = Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VoiceModel {
        pub id: VoiceModelID,
        pub voice_id: String,
        pub info: VoiceModelInfo,
        pub avatar_url: String,
        pub category: VoiceModelCategory,
        pub ability: VoiceModelAbility,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VoiceModelInfo {
        pub name: String,
        pub description: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum VoiceModelCategory {
        Official(String),
        User {
            supported_model: String,
            user_id: crate::api::user::UserID,
            voice_generate_data_id: Uuid,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VoiceModelAbility {
        pub voice_clone: bool,
        pub voice_design: bool,
        pub ssml: bool,
        pub latex: bool,
        pub parametric_control: bool,
        pub instruction_control: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "ssr", derive(sqlx::Type))]
    pub struct VoiceMeta {
        pub voice_model_id: VoiceModelID,
        pub parametric: Option<Parametic>,
        pub instruction: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "ssr", derive(sqlx::Type))]
    pub struct Parametic {
        pub pitch: f32,
        pub speed: f32,
        pub volume: f32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum VoiceGenerateData {
        VoiceClone(Vec<u8>),
        VoiceDesign(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VoiceLibrary {
        pub id: VoiceLibraryID,
        pub meta_id: VoiceMetaID,
        pub text: String,
        pub audio_url: String,
    }

    #[async_trait::async_trait]
    pub trait VoiceService: Send + Sync {
        async fn list_voice_models(&self) -> anyhow::Result<Vec<VoiceModel>>;
        async fn search_voice_model(&self, info: &str) -> anyhow::Result<Vec<VoiceModel>>;
        async fn generate_voice_model(
            &self,
            voice_generate_meta: VoiceModelInfo,
            voice_data: VoiceGenerateData,
        ) -> anyhow::Result<VoiceModelID>;
        async fn update_voice_model(
            &self,
            voice_model_id: VoiceModelID,
            voice_model_info: VoiceModelInfo,
        ) -> anyhow::Result<()>;
        async fn generate_meta(&self, voice_meta: VoiceMeta) -> anyhow::Result<VoiceMetaID>;
        async fn get_meta(&self, voice_meta_id: VoiceMetaID) -> anyhow::Result<VoiceMeta>;
        /// 返回生成的音频文件URL
        async fn generate_voice(
            &self,
            voice_meta_id: VoiceMetaID,
            text: &str,
        ) -> anyhow::Result<VoiceLibraryID>;
    }

    pub type VoiceProvider = Arc<dyn VoiceService>;

    #[server]
    pub async fn list_voice_models() -> Result<Vec<VoiceModel>, ServerFnError> {
        use_context::<VoiceProvider>()
            .ok_or_else(|| ServerFnError::new("未找到语音服务组件(VoiceProvider)"))?
            .list_voice_models()
            .await
            .map_err(|e| ServerFnError::new(format!("获取语音模型列表失败: {}", e)))
    }

    #[server]
    pub async fn search_voice_model(info: String) -> Result<Vec<VoiceModel>, ServerFnError> {
        use_context::<VoiceProvider>()
            .ok_or_else(|| ServerFnError::new("未找到语音服务组件(VoiceProvider)"))?
            .search_voice_model(&info)
            .await
            .map_err(|e| ServerFnError::new(format!("搜索语音模型失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn generate_voice_model(
        voice_generate_meta: VoiceModelInfo,
        voice_data: VoiceGenerateData,
    ) -> Result<VoiceModelID, ServerFnError> {
        use_context::<VoiceProvider>()
            .ok_or_else(|| ServerFnError::new("未找到语音服务组件(VoiceProvider)"))?
            .generate_voice_model(voice_generate_meta, voice_data)
            .await
            .map_err(|e| ServerFnError::new(format!("生成语音模型失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn update_voice_model(
        voice_model_id: VoiceModelID,
        voice_model_info: VoiceModelInfo,
    ) -> Result<(), ServerFnError> {
        use_context::<VoiceProvider>()
            .ok_or_else(|| ServerFnError::new("未找到语音服务组件(VoiceProvider)"))?
            .update_voice_model(voice_model_id, voice_model_info)
            .await
            .map_err(|e| ServerFnError::new(format!("更新语音模型失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn generate_meta(voice_meta: VoiceMeta) -> Result<VoiceMetaID, ServerFnError> {
        use_context::<VoiceProvider>()
            .ok_or_else(|| ServerFnError::new("未找到语音服务组件(VoiceProvider)"))?
            .generate_meta(voice_meta)
            .await
            .map_err(|e| ServerFnError::new(format!("生成语音元数据失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn get_meta(voice_meta_id: VoiceMetaID) -> Result<VoiceMeta, ServerFnError> {
        use_context::<VoiceProvider>()
            .ok_or_else(|| ServerFnError::new("未找到语音服务组件(VoiceProvider)"))?
            .get_meta(voice_meta_id)
            .await
            .map_err(|e| ServerFnError::new(format!("获取语音元数据失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn generate_voice(
        voice_meta_id: VoiceMetaID,
        text: String,
    ) -> Result<VoiceLibraryID, ServerFnError> {
        use_context::<VoiceProvider>()
            .ok_or_else(|| ServerFnError::new("未找到语音服务组件(VoiceProvider)"))?
            .generate_voice(voice_meta_id, &text)
            .await
            .map_err(|e| ServerFnError::new(format!("生成语音失败: {}", e)))
    }
}
mod voice_backend;
mod voiceimpl;

pub mod post {
    use crate::api::{
        user::UserID,
        voice::{VoiceLibraryID, VoiceMetaID},
    };
    use leptos::{prelude::*, server_fn::codec::Json};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use uuid::Uuid;

    pub type VoiceMetaPostID = Uuid;
    pub type VoicePostID = Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
    pub struct VoiceMetaPost {
        pub id: VoiceMetaPostID,
        pub title: String,
        pub content: String,
        /// 指向VoiceMeta的id
        pub meta_id: VoiceMetaID,
        pub author: UserID,
        pub status: PostStatus,
        pub comments_count: i32,
        pub likes_count: i32,
        pub is_liked_by_current_user: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
    pub struct VoicePost {
        pub id: VoicePostID,
        pub title: String,
        pub content: String,
        /// 指向VoiceLibrary的id
        pub library_id: VoiceLibraryID,
        pub author: UserID,
        pub status: PostStatus,
        pub comments_count: i32,
        pub likes_count: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[cfg_attr(feature = "ssr", derive(sqlx::Type))]
    #[cfg_attr(feature = "ssr", sqlx(type_name = "post_status"))]
    pub enum PostStatus {
        Normal,
        Deleted,
        Banned,
        Recommended,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Action {
        LikeDislike,
        Comment(String), // 用户id和评论内容
    }

    #[async_trait::async_trait]
    pub trait VoiceMetaPostService: Send + Sync {
        async fn create(&self, post: VoiceMetaPost) -> anyhow::Result<VoiceMetaPostID>;
        async fn get(&self, post_id: VoiceMetaPostID) -> anyhow::Result<VoiceMetaPost>;
        async fn update(&self, post: VoiceMetaPost) -> anyhow::Result<()>;
        async fn delete(&self, post_id: VoiceMetaPostID) -> anyhow::Result<()>;
        async fn search(&self, query: &str) -> anyhow::Result<Vec<VoiceMetaPost>>;
        async fn list(
            &self,
            status: Option<PostStatus>,
            number: u8,
        ) -> anyhow::Result<Vec<VoiceMetaPost>>;
        async fn action(&self, post_id: VoiceMetaPostID, action: Action) -> anyhow::Result<()>;
        async fn get_comments(
            &self,
            post_id: VoiceMetaPostID,
        ) -> anyhow::Result<Vec<(UserID, String)>>;
    }

    pub type VoiceMetaPostProvider = Arc<dyn VoiceMetaPostService>;

    #[async_trait::async_trait]
    pub trait VoicePostService: Send + Sync {
        async fn create(&self, post: VoicePost) -> anyhow::Result<VoicePostID>;
        async fn get(&self, post_id: VoicePostID) -> anyhow::Result<VoicePost>;
        async fn update(&self, post: VoicePost) -> anyhow::Result<()>;
        async fn delete(&self, post_id: VoicePostID) -> anyhow::Result<()>;
        async fn search(&self, query: &str) -> anyhow::Result<Vec<VoicePost>>;
        async fn list(
            &self,
            status: Option<PostStatus>,
            number: u8,
        ) -> anyhow::Result<Vec<VoicePost>>;
        async fn action(&self, post_id: VoicePostID, action: Action) -> anyhow::Result<()>;
        async fn get_comments(&self, post_id: VoicePostID)
        -> anyhow::Result<Vec<(UserID, String)>>;
    }

    pub type VoicePostProvider = Arc<dyn VoicePostService>;

    #[server(input = Json)]
    pub async fn create_voice_meta_post(
        post: VoiceMetaPost,
    ) -> Result<VoiceMetaPostID, ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .create(post)
            .await
            .map_err(|e| ServerFnError::new(format!("创建失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn get_voice_meta_post(
        post_id: VoiceMetaPostID,
    ) -> Result<VoiceMetaPost, ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .get(post_id)
            .await
            .map_err(|e| ServerFnError::new(format!("获取失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn update_voice_meta_post(post: VoiceMetaPost) -> Result<(), ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .update(post)
            .await
            .map_err(|e| ServerFnError::new(format!("更新失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn delete_voice_meta_post(post_id: VoiceMetaPostID) -> Result<(), ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .delete(post_id)
            .await
            .map_err(|e| ServerFnError::new(format!("删除失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn search_voice_meta_post(
        query: String,
    ) -> Result<Vec<VoiceMetaPost>, ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .search(&query)
            .await
            .map_err(|e| ServerFnError::new(format!("搜索失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn list_voice_meta_post(
        status: Option<PostStatus>,
        number: u8,
    ) -> Result<Vec<VoiceMetaPost>, ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .list(status, number)
            .await
            .map_err(|e| ServerFnError::new(format!("获取列表失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn action_voice_meta_post(
        post_id: VoiceMetaPostID,
        action: Action,
    ) -> Result<(), ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .action(post_id, action)
            .await
            .map_err(|e| ServerFnError::new(format!("操作失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn get_voice_meta_post_comments(
        post_id: VoiceMetaPostID,
    ) -> Result<Vec<(UserID, String)>, ServerFnError> {
        use_context::<VoiceMetaPostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoiceMetaPostProvider"))?
            .get_comments(post_id)
            .await
            .map_err(|e| ServerFnError::new(format!("获取评论失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn create_voice_post(post: VoicePost) -> Result<VoicePostID, ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .create(post)
            .await
            .map_err(|e| ServerFnError::new(format!("创建失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn get_voice_post(post_id: VoicePostID) -> Result<VoicePost, ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .get(post_id)
            .await
            .map_err(|e| ServerFnError::new(format!("获取失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn update_voice_post(post: VoicePost) -> Result<(), ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .update(post)
            .await
            .map_err(|e| ServerFnError::new(format!("更新失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn delete_voice_post(post_id: VoicePostID) -> Result<(), ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .delete(post_id)
            .await
            .map_err(|e| ServerFnError::new(format!("删除失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn search_voice_post(query: String) -> Result<Vec<VoicePost>, ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .search(&query)
            .await
            .map_err(|e| ServerFnError::new(format!("搜索失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn list_voice_post(
        status: Option<PostStatus>,
        number: u8,
    ) -> Result<Vec<VoicePost>, ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .list(status, number)
            .await
            .map_err(|e| ServerFnError::new(format!("获取列表失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn action_voice_post(
        post_id: VoicePostID,
        action: Action,
    ) -> Result<(), ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .action(post_id, action)
            .await
            .map_err(|e| ServerFnError::new(format!("操作失败: {}", e)))
    }

    #[server(input = Json)]
    pub async fn get_voice_post_comments(
        post_id: VoicePostID,
    ) -> Result<Vec<(UserID, String)>, ServerFnError> {
        use_context::<VoicePostProvider>()
            .ok_or_else(|| ServerFnError::new("未找到VoicePostProvider"))?
            .get_comments(post_id)
            .await
            .map_err(|e| ServerFnError::new(format!("获取评论失败: {}", e)))
    }
}
mod postimpl;
