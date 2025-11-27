use crate::data;
use crate::pages::homepage::GenerateParams;
#[cfg(not(target_arch = "wasm32"))]
use base64::{engine::general_purpose, Engine as _};
#[cfg(not(target_arch = "wasm32"))]
use leptos::logging::debug_log;
use leptos::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[server]
pub async fn get_voices() -> Result<Vec<data::VoiceOption>, ServerFnError> {
    // 这里是服务器端代码
    // 模拟数据库查询，返回硬编码数据
    let voices = vec![
        data::VoiceOption {
            id: "Cherry".to_string(),
            name: "芊悦".to_string(),
            desc: "阳光积极、亲切自然小姐姐。".to_string(),
        },
        data::VoiceOption {
            id: "Ethan".to_string(),
            name: "晨煦".to_string(),
            desc: "标准普通话，带部分北方口音。阳光、温暖、活力、朝气。".to_string(),
        },
        data::VoiceOption {
            id: "Nofish".to_string(),
            name: "不吃鱼".to_string(),
            desc: "不会翘舌音的设计师。".to_string(),
        },
        data::VoiceOption {
            id: "Jennifer".to_string(),
            name: "詹妮弗".to_string(),
            desc: "品牌级、电影质感般美语女声。".to_string(),
        },
        data::VoiceOption {
            id: "Ryan".to_string(),
            name: "甜茶".to_string(),
            desc: "节奏拉满，戏感炸裂，真实与张力共舞。".to_string(),
        },
        data::VoiceOption {
            id: "Katerina".to_string(),
            name: "卡捷琳娜".to_string(),
            desc: "御姐音色，韵律回味十足。".to_string(),
        },
        data::VoiceOption {
            id: "Elias".to_string(),
            name: "墨讲师".to_string(),
            desc: "既保持学科严谨性，又通过叙事技巧将复杂知识转化为可消化的认知模块。".to_string(),
        },
        data::VoiceOption {
            id: "Jada".to_string(),
            name: "上海-阿珍".to_string(),
            desc: "风风火火的沪上阿姐。".to_string(),
        },
        data::VoiceOption {
            id: "Dylan".to_string(),
            name: "北京-晓东".to_string(),
            desc: "北京胡同里长大的少年。".to_string(),
        },
        data::VoiceOption {
            id: "Sunny".to_string(),
            name: "四川-晴儿".to_string(),
            desc: "甜到你心里的川妹子。".to_string(),
        },
        data::VoiceOption {
            id: "Li".to_string(),
            name: "南京-老李".to_string(),
            desc: "耐心的瑜伽老师".to_string(),
        },
        data::VoiceOption {
            id: "Marcus".to_string(),
            name: "陕西-秦川".to_string(),
            desc: "面宽话短，心实声沉——老陕的味道。".to_string(),
        },
        data::VoiceOption {
            id: "Roy".to_string(),
            name: "闽南-阿杰".to_string(),
            desc: "诙谐直爽、市井活泼的台湾哥仔形象。".to_string(),
        },
        data::VoiceOption {
            id: "Peter".to_string(),
            name: "天津-李彼得".to_string(),
            desc: "天津相声，专业捧人。".to_string(),
        },
        data::VoiceOption {
            id: "Rocky".to_string(),
            name: "粤语-阿强".to_string(),
            desc: "幽默风趣的阿强，在线陪聊。".to_string(),
        },
        data::VoiceOption {
            id: "Kiki".to_string(),
            name: "粤语-阿清".to_string(),
            desc: "甜美的港妹闺蜜。".to_string(),
        },
        data::VoiceOption {
            id: "Eric".to_string(),
            name: "四川-程川".to_string(),
            desc: "一个跳脱市井的四川成都男子。".to_string(),
        },
    ];

    Ok(voices)
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize)]
struct DashScopeRequest {
    model: String,
    input: DashScopeInput,
    parameters: DashScopeParameters,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize)]
struct DashScopeInput {
    text: String,
    voice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language_type: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize)]
struct DashScopeParameters {
    // 这里的参数根据模型不同而不同，qwen3-tts-flash 文档主要强调 input
    // 我们可以预留 sample_rate 或 format，但在文档示例中未强制要求
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Deserialize, Debug)]
struct DashScopeResponse {
    // status_code 在 HTTP 层处理，这里解析 body 里的字段
    code: Option<String>,
    message: Option<String>,
    _request_id: Option<String>,
    output: Option<DashScopeOutput>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Deserialize, Debug)]
struct DashScopeOutput {
    audio: Option<DashScopeAudio>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Deserialize, Debug)]
struct DashScopeAudio {
    url: Option<String>,
}

// --- 新增：生成音频 API ---
#[server]
pub async fn generate_audio(params: GenerateParams) -> Result<String, ServerFnError> {
    let api_key = std::env::var("ALIYUN_API_KEY").unwrap_or("".into());
    debug_log!("使用阿里云 API Key: {}", &api_key);

    let url =
        "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation";

    // 1. 构造请求 Payload
    // 注意：阿里云 Qwen-TTS 模型暂时可能忽略 pitch/speed/emotion 参数，
    // 这里我们仅传递核心的 text 和 voice
    let request_body = DashScopeRequest {
        model: "qwen3-tts-flash".to_string(),
        input: DashScopeInput {
            text: params.text,
            voice: params.voice_id,
            language_type: Some("Auto".to_string()),
        },
        parameters: DashScopeParameters {},
    };

    let client = Client::new();

    // 2. 发送 POST 请求到阿里云
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key)) // 注意：阿里云是 Bearer Space Token
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| -> ServerFnError {
            ServerFnError::ServerError(format!("Request failed: {}", e))
        })?;

    // 检查 HTTP 状态码
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ServerFnError::ServerError(format!(
            "API HTTP Error {}: {}",
            status, text
        )));
    }

    // 3. 解析 JSON 响应
    let dash_res: DashScopeResponse = response.json().await.map_err(|e| -> ServerFnError {
        ServerFnError::ServerError(format!("Parse JSON failed: {}", e))
    })?;

    // 检查业务错误码 (code 字段非空通常表示错误)
    if let Some(code) = &dash_res.code {
        if !code.is_empty() {
            let msg = dash_res.message.unwrap_or_default();
            return Err(ServerFnError::ServerError(format!(
                "DashScope Error {}: {}",
                code, msg
            )));
        }
    }

    // 4. 获取音频 URL 并下载
    // 阿里云非流式接口返回的是一个临时的 OSS URL
    if let Some(output) = dash_res.output {
        if let Some(audio) = output.audio {
            if let Some(audio_url) = audio.url {
                // 后端下载音频文件，避免前端跨域问题，并保持接口返回格式一致
                let audio_resp =
                    client
                        .get(&audio_url)
                        .send()
                        .await
                        .map_err(|e| -> ServerFnError {
                            ServerFnError::ServerError(format!("Download audio failed: {}", e))
                        })?;

                let audio_bytes = audio_resp.bytes().await.map_err(|e| -> ServerFnError {
                    ServerFnError::ServerError(format!("Read audio bytes failed: {}", e))
                })?;

                // 5. 转换为 Base64 Data URI
                // 阿里云返回的 URL 通常包含扩展名，或者默认为 wav
                let content_type = if audio_url.contains(".mp3") {
                    "audio/mp3"
                } else {
                    "audio/wav"
                };
                let base64_data = general_purpose::STANDARD.encode(&audio_bytes);
                let data_uri = format!("data:{};base64,{}", content_type, base64_data);

                return Ok(data_uri);
            }
        }
    }

    Err(ServerFnError::ServerError(
        "No audio URL found in response".to_string(),
    ))
}

// --- 新增：获取滤镜列表 API ---
#[server]
pub async fn get_voice_filters() -> Result<Vec<data::VoiceFilter>, ServerFnError> {
    // 模拟数据
    let filters = vec![
        // --- 官方推荐 ---
        data::VoiceFilter {
            id: 1,
            name: "清澈高音".to_string(),
            desc: "提升音调，适合欢快的场景".to_string(),
            voice_data: data::VoiceData {
                voice_id: "Cherry".to_string(),
                voice_params: data::VoiceParams {
                    pitch: 1.8,
                    speed: 1.1,
                    emotion: data::Emotion::Happy,
                },
            },
            tags: vec!["明亮".to_string(), "高音".to_string()],
            usage_count: 1205,
            author: "官方".to_string(),
            state: data::DisplayState::Official,
        },
        data::VoiceFilter {
            id: 2,
            name: "低沉叙述".to_string(),
            desc: "压低声线，适合讲故事".to_string(),
            voice_data: data::VoiceData {
                voice_id: "Ethan".to_string(),
                voice_params: data::VoiceParams {
                    pitch: -2.0,
                    speed: 0.9,
                    emotion: data::Emotion::Calm,
                },
            },

            tags: vec!["低沉".to_string(), "磁性".to_string()],
            usage_count: 890,
            author: "官方".to_string(),
            state: data::DisplayState::Official,
        },
        // --- 用户推荐 ---
        data::VoiceFilter {
            id: 3,
            name: "悲伤独白".to_string(),
            desc: "慢速低沉，充满忧伤".to_string(),
            voice_data: data::VoiceData {
                voice_id: "Nofish".to_string(),
                voice_params: data::VoiceParams {
                    pitch: -1.5,
                    speed: 0.8,
                    emotion: data::Emotion::Sad,
                },
            },

            tags: vec!["悲伤".to_string(), "慢速".to_string()],
            usage_count: 342,
            author: "林间鹿".to_string(),
            state: data::DisplayState::Recommended,
        },
        data::VoiceFilter {
            id: 4,
            name: "激昂演讲".to_string(),
            desc: "快速有力，充满激情".to_string(),
            voice_data: data::VoiceData {
                voice_id: "Ryan".to_string(),
                voice_params: data::VoiceParams {
                    pitch: 1.0,
                    speed: 1.3,
                    emotion: data::Emotion::Excited,
                },
            },
            tags: vec!["激昂".to_string(), "演讲".to_string()],
            usage_count: 567,
            author: "雨夜听风".to_string(),
            state: data::DisplayState::Recommended,
        },
        data::VoiceFilter {
            id: 5,
            name: "温柔晚安".to_string(),
            desc: "轻柔舒缓，助眠专用".to_string(),
            voice_data: data::VoiceData {
                voice_id: "Sunny".to_string(),
                voice_params: data::VoiceParams {
                    pitch: -0.5,
                    speed: 0.75,
                    emotion: data::Emotion::Peaceful,
                },
            },
            tags: vec!["治愈".to_string(), "晚安".to_string()],
            usage_count: 128,
            author: "星辰大海".to_string(),
            state: data::DisplayState::Recommended,
        },
    ];

    Ok(filters)
}
