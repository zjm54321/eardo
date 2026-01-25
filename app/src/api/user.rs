use crate::api::{Auth, UserProfile, sql_struct::*};
use anyhow::Result;
use async_trait::async_trait;
pub struct UserMeta {
    id: Option<u128>,
    pub icon_url: String,
}

pub struct User {
    pub user_meta: UserMeta,
    pub nickname: String,
    pub bio: String,
    pub level: i32,
}

struct UserAuth {
    username: String,
    email: Option<String>,
    phone: Option<String>,
    password: String,
}

impl UserAuth {
    pub fn new(
        username: String,
        password: String,
        email: Option<String>,
        phone: Option<String>,
    ) -> UserAuth {
        UserAuth {
            username,
            email,
            phone,
            password,
        }
    }
}

#[async_trait]
impl Auth for UserMeta {
    type UserAuth = UserAuth;

    async fn register(userauth: &Self::UserAuth, password: &str) -> Result<()> {
        todo!()
    }
    async fn login(&mut self, userauth: &str, password: &str) -> Result<()> {
        todo!()
    }
    async fn logout(&mut self) -> Result<()> {
        todo!()
    }
    async fn get_current_user(&mut self) -> Result<()> {
        todo!()
    }
}

#[async_trait]
impl UserProfile for UserMeta {
    type User = u8;

    async fn get_user_profile(&self) -> Result<Self::User> {
        todo!()
    }
    async fn update_user_profile(&self, user: &Self::User) -> Result<()> {
        todo!()
    }
}

impl UserMeta {
    pub fn new() -> UserMeta {
        UserMeta {
            id: None,
            icon_url: String::new(),
        }
    }
}
