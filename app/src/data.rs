#[cfg(feature = "ssr")]
use chrono::{DateTime, Utc}; // 引入 chrono 处理时间
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::prelude::FromRow;
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceData {
    pub voice_id: String,
    pub voice_params: VoiceParams,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceParams {
    pub pitch: f32,
    pub speed: f32,
    pub emotion: Emotion,
}

impl Default for VoiceParams {
    fn default() -> Self {
        VoiceParams {
            pitch: 0.0,
            speed: 1.0,
            emotion: Emotion::Normal,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(feature = "ssr", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
pub enum Emotion {
    Normal,
    Angry,
    Calm,
    Excited,
    Happy,
    Peaceful,
    Sad,
    Suprised,
}

impl From<String> for Emotion {
    fn from(s: String) -> Self {
        match s.as_str() {
            "angry" => Emotion::Angry,
            "calm" => Emotion::Calm,
            "excited" => Emotion::Excited,
            "happy" => Emotion::Happy,
            "peaceful" => Emotion::Peaceful,
            "sad" => Emotion::Sad,
            "suprised" => Emotion::Suprised,
            _ => Emotion::Normal,
        }
    }
}

impl fmt::Display for Emotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Emotion::Normal => "正常",
            Emotion::Angry => "生气",
            Emotion::Calm => "冷静",
            Emotion::Excited => "激动",
            Emotion::Happy => "开心",
            Emotion::Peaceful => "平静",
            Emotion::Sad => "悲伤",
            Emotion::Suprised => "惊讶",
        }
        .fmt(f)
    }
}

impl Emotion {
    pub fn all() -> Vec<Emotion> {
        vec![
            Emotion::Normal,
            Emotion::Angry,
            Emotion::Calm,
            Emotion::Excited,
            Emotion::Happy,
            Emotion::Peaceful,
            Emotion::Sad,
            Emotion::Suprised,
        ]
    }
}

// --- 应用层模型 ---
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceFilter {
    pub id: i64,
    pub name: String,
    pub desc: String,
    pub voice_data: VoiceData,
    pub tags: Vec<String>,
    pub usage_count: i32,
    pub author: String,
    pub state: DisplayState,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct VoiceOption {
    pub id: String,
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(feature = "ssr", sqlx(type_name = "TEXT", rename_all = "snake_case"))]
pub enum DisplayState {
    Visible,
    Hidden,
    Disabled,
    Official,
    Recommended,
}

// --- 数据库模型 ---
#[cfg(feature = "ssr")]
#[derive(FromRow, Debug)]
pub struct VoiceFilterDb {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub voice_id: String,
    pub pitch: f32,
    pub speed: f32,
    pub emotion: String,
    pub tags: String,
    pub usage_count: i32,
    pub author: String,
    pub state: String,
}

#[cfg(feature = "ssr")]
impl VoiceFilterDb {
    pub fn to_domain(&self) -> VoiceFilter {
        VoiceFilter {
            id: self.id,
            name: self.name.clone(),
            desc: self.description.clone(),
            voice_data: VoiceData {
                voice_id: self.voice_id.clone(),
                voice_params: VoiceParams {
                    pitch: self.pitch,
                    speed: self.speed,
                    emotion: Emotion::from(self.emotion.clone()),
                },
            },
            tags: self
                .tags
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect(),
            usage_count: self.usage_count,
            author: self.author.clone(),
            state: match self.state.as_str() {
                "official" => DisplayState::Official,
                "recommended" => DisplayState::Recommended,
                _ => DisplayState::Visible,
            },
        }
    }
}

// --- 声音作品模型 (Voice Plaza) ---

// 前端使用的模型 (保持不变，time 字段接收格式化后的字符串)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VoiceWork {
    pub id: i64,
    pub author: String,
    pub avatar: String,
    pub time: String, // 这里存的是 "2小时前" 这样的展示字符串
    pub title: String,
    pub description: String,
    pub likes: i32,
    pub comments: i32,
    pub voice_type: String,
    pub audio_url: String,
}

// 后端数据库模型
#[cfg(feature = "ssr")]
#[derive(FromRow, Debug)]
pub struct VoiceWorkDb {
    pub id: i64,
    pub author: String,
    pub avatar: String,
    // 使用 chrono 类型直接映射数据库的 DATETIME
    pub created_at: DateTime<Utc>,
    pub title: String,
    pub description: String,
    pub likes: i32,
    pub comments: i32,
    pub voice_type: String,
    pub audio_url: String,
    pub is_featured: bool,
}

#[cfg(feature = "ssr")]
impl VoiceWorkDb {
    pub fn to_domain(&self) -> VoiceWork {
        // 计算相对时间
        let now = Utc::now();
        let diff = now.signed_duration_since(self.created_at);

        let time_desc = if diff.num_seconds() < 60 {
            "刚刚".to_string()
        } else if diff.num_minutes() < 60 {
            format!("{}分钟前", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}小时前", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}天前", diff.num_days())
        } else if diff.num_days() < 30 {
            format!("{}周前", diff.num_days() / 7)
        } else {
            // 超过一个月显示具体日期 (YYYY-MM-DD)
            self.created_at.format("%Y-%m-%d").to_string()
        };

        VoiceWork {
            id: self.id,
            author: self.author.clone(),
            avatar: self.avatar.clone(),
            time: time_desc, // 使用计算出的时间描述
            title: self.title.clone(),
            description: self.description.clone(),
            likes: self.likes,
            comments: self.comments,
            voice_type: self.voice_type.clone(),
            audio_url: self.audio_url.clone(),
        }
    }
}

// --- 新增：用户相关模型 ---

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
}

#[cfg(feature = "ssr")]
#[derive(FromRow, Debug)]
pub struct UserDb {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
}
