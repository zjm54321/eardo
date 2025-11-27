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
#[cfg_attr(feature = "ssr", derive(sqlx::Type))] // 允许 sqlx 自动处理枚举映射（存储为文本时）
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

// 手动实现 From<String> 以便从数据库文本转换（如果 sqlx::Type 不起作用的备选方案）
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
    pub id: i64, // 数据库 ID 通常是 i64
    pub name: String,
    pub desc: String,
    pub voice_data: VoiceData,
    pub tags: Vec<String>,
    pub usage_count: i32, // 数据库 Integer
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

// --- 数据库模型 (用于从扁平化的 SQL 表读取) ---
// 因为 SQL 不方便直接存嵌套结构体，我们定义一个 DB 专用的结构体
#[cfg(feature = "ssr")]
#[derive(FromRow, Debug)]
pub struct VoiceFilterDb {
    pub id: i64,
    pub name: String,
    pub description: String,
    // 扁平化的 VoiceData
    pub voice_id: String,
    pub pitch: f32,
    pub speed: f32,
    pub emotion: String, // 存文本
    // 扁平化的 Tags (JSON 字符串或逗号分隔)
    pub tags: String,
    pub usage_count: i32,
    pub author: String,
    pub state: String, // 存文本
}

// 提供转换方法
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
            // 假设 tags 用逗号分隔
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
