//! SQL 中的结构体实现
//! 包括了结构体和其获取方法`get()`的实现
//! 具体内容可以参考 ../../../sql/new.sql
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 用户核心资料表 (Users - Public Profile)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserDB {
    pub id: String,
    pub nickname: Option<String>,
    pub bio: Option<String>,
    pub level: i32,
    pub avatar_data: Option<Vec<u8>>,
    pub avatar_mime: Option<String>,
    pub status: String, // normal, deleted, banned
    pub role: String,   // user, admin
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 用户鉴权表 (User Auth - Private)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAuthDB {
    pub user_id: String,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/// 会话管理表 (Sessions)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSessionDB {
    pub token: String,
    pub user_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_name: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
}

/// 用户上传源声音 (User Source Voices)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserVoiceDB {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub audio_data: Vec<u8>,
    pub visualization_data: Option<Vec<u8>>,
    pub status: String, // normal
    pub created_at: DateTime<Utc>,
}

/// 声音模型表 (Voice Models)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoiceModelDB {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub icon_data: Option<Vec<u8>>,
    pub status: String, // normal, hidden
    pub created_at: DateTime<Utc>,
}

/// 元信息表 (Voice Meta Infos)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VoiceMetaInfoDB {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub base_model_id: String,
    pub pitch: f64,
    pub speed: f64,
    pub volume: f64,
    pub emotion: String, // normal
    pub usage_count: i32,
    pub is_public: bool,
    pub status: String, // normal
    pub created_at: DateTime<Utc>,
}

/// 帖子表 (Posts)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostDB {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: Option<String>,
    pub generated_audio_data: Vec<u8>,
    pub voice_meta_id: String,
    pub likes_count: i32,
    pub comments_count: i32,
    pub status: String, // normal
    pub created_at: DateTime<Utc>,
}

/// 评论表 (Post Comments)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostCommentDB {
    pub id: String,
    pub post_id: String,
    pub user_id: String,
    pub content: String,
    pub status: String, // normal
    pub created_at: DateTime<Utc>,
}

/// 点赞表 (Post Likes)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PostLikeDB {
    pub user_id: String,
    pub post_id: String,
    pub created_at: DateTime<Utc>,
}
