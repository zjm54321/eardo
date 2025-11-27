use serde::{Deserialize, Serialize};
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

impl fmt::Display for Emotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Emotion::Normal => "正常",
            Emotion::Angry => "生气",
            Emotion::Calm => "冷静",
            Emotion::Excited => "激动",
            Emotion::Happy => "开心",
            Emotion::Peaceful => "平静",
            Emotion::Sad => "悲伤",
            Emotion::Suprised => "惊讶",
        };
        write!(f, "{}", s)
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

// --- 新增：滤镜模型 ---
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceFilter {
    pub id: u128,
    pub name: String,
    pub desc: String,
    // 该滤镜对应的参数预设
    pub voice_data: VoiceData,
    // UI 展示用
    pub tags: Vec<String>,
    pub usage_count: u32,
    pub author: String,
    pub state: DisplayState,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VoiceOption {
    pub id: String,
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DisplayState {
    Visible,
    Hidden,
    Disabled,
    Official,
    Recommended,
}
