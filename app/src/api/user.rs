#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl AuthService for ServiceProvider<Sqlite> {
    async fn register(&self, userauth: UserAuthInfo, password: &str) -> anyhow::Result<()> {
        todo!()
    }
    async fn login(&self, userauth: UserAuthInfo, password: &str) -> anyhow::Result<()> {
        todo!()
    }
    async fn logout(&self) -> anyhow::Result<()> {
        todo!()
    }
    async fn get_current_user(&self) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl UserService for ServiceProvider<Sqlite> {
    async fn get_user_profile(&self) -> anyhow::Result<UserInfo> {
        Ok(UserInfo {
            id: "".to_string(),
            username: "ming".to_string(),
            avatar_url: "".to_string(),
            status: UserStatus::Normal,
            meta: "".to_string(),
        })
    }
    async fn update_user_profile(&self, user: &UserInfo) -> anyhow::Result<()> {
        todo!()
    }
}
