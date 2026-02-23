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
