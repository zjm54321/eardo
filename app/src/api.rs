use crate::pages::homepage::GenerateParams;
#[cfg(not(target_arch = "wasm32"))]
use base64::{engine::general_purpose, Engine as _};
#[cfg(not(target_arch = "wasm32"))]
use leptos::logging::debug_log;
use leptos::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct VoiceOption {
    pub id: String,
    pub name: String,
    pub desc: String,
}
#[server]
pub async fn get_voices() -> Result<Vec<VoiceOption>, ServerFnError> {
    // 这里是服务器端代码
    // 模拟数据库查询，返回硬编码数据
    let voices = vec![
        VoiceOption {
            id: "Cherry".to_string(),
            name: "芊悦".to_string(),
            desc: "阳光积极、亲切自然小姐姐。".to_string(),
        },
        VoiceOption {
            id: "Ethan".to_string(),
            name: "晨煦".to_string(),
            desc: "标准普通话，带部分北方口音。阳光、温暖、活力、朝气。".to_string(),
        },
        VoiceOption {
            id: "Elias".to_string(),
            name: "墨讲师".to_string(),
            desc: "既保持学科严谨性，又通过叙事技巧将复杂知识转化为可消化的认知模块。".to_string(),
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
