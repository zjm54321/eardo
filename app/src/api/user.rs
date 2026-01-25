#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl AuthService for ServiceProvider<Sqlite> {
    async fn register(&self, _userauth: UserAuthInfo, _password: &str) -> anyhow::Result<()> {
        // 模拟实现：假设注册总是成功
        leptos::logging::log!("Mock: User registered successfully");
        Ok(())
    }
    
    async fn login(&self, _userauth: UserAuthInfo, _password: &str) -> anyhow::Result<()> {
        // 模拟实现：假设登录总是成功
        leptos::logging::log!("Mock: User logged in successfully");
        Ok(())
    }
    
    async fn logout(&self) -> anyhow::Result<()> {
        // 模拟实现：假设登出总是成功
        leptos::logging::log!("Mock: User logged out successfully");
        Ok(())
    }
    
    async fn get_current_user(&self) -> anyhow::Result<()> {
        // 模拟实现：返回成功
        leptos::logging::log!("Mock: Getting current user");
        Ok(())
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl UserService for ServiceProvider<Sqlite> {
    async fn get_user_profile(&self) -> anyhow::Result<UserInfo> {
        // 模拟实现：返回示例用户信息
        Ok(UserInfo {
            id: "user_001".to_string(),
            username: "Demo用户".to_string(),
            avatar_url: "https://i.pravatar.cc/150?img=10".to_string(),
            status: UserStatus::Normal,
            meta: serde_json::json!({
                "bio": "这是一个演示账号，欢迎使用白昼聆夏！",
                "created_at": "2026-01-01",
                "preferences": {
                    "theme": "light",
                    "notifications": true
                }
            }).to_string(),
        })
    }
    
    async fn update_user_profile(&self, user: &UserInfo) -> anyhow::Result<()> {
        // 模拟实现：记录更新操作
        leptos::logging::log!("Mock: Updating user profile for {}", user.username);
        Ok(())
    }
}
