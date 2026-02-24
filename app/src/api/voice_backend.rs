#![cfg(feature = "ssr")]

use crate::api::voice::VoiceMeta;
use anyhow::{Result, anyhow};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use leptos::logging::{debug_log, error};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct CosyHeader {
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<String>,
    task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    streaming: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CosyParams {
    text_type: String,
    voice: String,
    format: String,
    sample_rate: u32,
    volume: u32,
    rate: f32,
    pitch: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CosyInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CosyPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    task_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<CosyParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<CosyInput>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CosyMessage {
    header: CosyHeader,
    payload: CosyPayload,
}

#[derive(Serialize, Deserialize, Debug)]
struct DashScopeTtsRequest {
    model: String,
    input: DashScopeTtsPayload,
}

#[derive(Serialize, Deserialize, Debug)]
struct DashScopeTtsPayload {
    text: String,
    voice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    optimize_instructions: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DashScopeTtsAudio {
    #[serde(default)]
    data: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DashScopeTtsOutput {
    #[serde(default)]
    finish_reason: Option<String>,
    #[serde(default)]
    audio: Option<DashScopeTtsAudio>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DashScopeTtsResponse {
    #[serde(default = "default_status_code")]
    status_code: i32,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    output: Option<DashScopeTtsOutput>,
}

fn default_status_code() -> i32 {
    200
}

/// 支持的语音模型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedModel {
    CosyVoiceV3Plus,
    CosyVoiceV3Flash,
    Qwen3TtsInstructFlash,
    Qwen3TtsVd20260126,
    Qwen3TtsVc20260122,
    Qwen3TtsFlash,
}

impl FromStr for SupportedModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cosyvoice-v3-plus" => Ok(SupportedModel::CosyVoiceV3Plus),
            "cosyvoice-v3-flash" => Ok(SupportedModel::CosyVoiceV3Flash),
            "qwen3-tts-instruct-flash" => Ok(SupportedModel::Qwen3TtsInstructFlash),
            "qwen3-tts-vd-2026-01-26" => Ok(SupportedModel::Qwen3TtsVd20260126),
            "qwen3-tts-vc-2026-01-22" => Ok(SupportedModel::Qwen3TtsVc20260122),
            "qwen3-tts-flash" => Ok(SupportedModel::Qwen3TtsFlash),
            _ => Err(anyhow!("Unsupported model: {}", s)),
        }
    }
}

impl SupportedModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedModel::CosyVoiceV3Plus => "cosyvoice-v3-plus",
            SupportedModel::CosyVoiceV3Flash => "cosyvoice-v3-flash",
            SupportedModel::Qwen3TtsInstructFlash => "qwen3-tts-instruct-flash",
            SupportedModel::Qwen3TtsVd20260126 => "qwen3-tts-vd-2026-01-26",
            SupportedModel::Qwen3TtsVc20260122 => "qwen3-tts-vc-2026-01-22",
            SupportedModel::Qwen3TtsFlash => "qwen3-tts-flash",
        }
    }
}

/// 根据模型枚举分发到对应的生成逻辑
pub async fn generate_audio(
    model: &SupportedModel,
    voice_id: &str,
    text: &str,
    meta: &VoiceMeta,
) -> Result<Vec<u8>> {
    match model {
        SupportedModel::CosyVoiceV3Plus | SupportedModel::CosyVoiceV3Flash => {
            // 从 meta 中提取参数
            let rate = meta.parametric.as_ref().map(|p| p.speed).unwrap_or(1.0);
            let pitch = meta.parametric.as_ref().map(|p| p.pitch).unwrap_or(1.0);

            cosyvoice_generate(text, voice_id, rate, pitch, model.as_str()).await
        }
        SupportedModel::Qwen3TtsInstructFlash
        | SupportedModel::Qwen3TtsVd20260126
        | SupportedModel::Qwen3TtsVc20260122
        | SupportedModel::Qwen3TtsFlash => qwen_generate(model, voice_id, text, meta).await,
    }
}

/// Generate audio bytes via CosyVoice WebSocket API
async fn cosyvoice_generate(
    input_text: &str,
    voice_id: &str,
    rate: f32,
    pitch: f32,
    model_name: &str,
) -> Result<Vec<u8>> {
    let api_key = std::env::var("ALIYUN_API_KEY")
        .or_else(|_| std::env::var("DASHSCOPE_API_KEY"))
        .unwrap_or_default();
    if api_key.is_empty() {
        return Err(anyhow!("ALIYUN_API_KEY missing"));
    }

    let ws_url = "wss://dashscope.aliyuncs.com/api-ws/v1/inference";
    let task_id = Uuid::new_v4().to_string().replace("-", "");

    // Build request with headers
    let mut request = ws_url
        .into_client_request()
        .map_err(|e| anyhow!("WS Req build error: {}", e))?;
    let headers = request.headers_mut();
    headers.insert(
        "Authorization",
        format!("bearer {}", api_key).parse().unwrap(),
    );
    headers.insert("X-DashScope-DataInspection", "enable".parse().unwrap());

    // Connect
    let (mut ws_stream, _) = connect_async(request)
        .await
        .map_err(|e| anyhow!("WS Connect failed: {}", e))?;

    // Send run-task
    let run_task_msg = CosyMessage {
        header: CosyHeader {
            action: Some("run-task".to_string()),
            task_id: task_id.clone(),
            streaming: Some("duplex".to_string()),
            event: None,
            error_code: None,
            error_message: None,
        },
        payload: CosyPayload {
            task_group: Some("audio".to_string()),
            task: Some("tts".to_string()),
            function: Some("SpeechSynthesizer".to_string()),
            model: Some(model_name.to_string()),
            parameters: Some(CosyParams {
                text_type: "PlainText".to_string(),
                voice: voice_id.to_string(),
                format: "mp3".to_string(),
                sample_rate: 22050,
                volume: 50,
                rate,
                pitch,
            }),
            input: Some(CosyInput { text: None }),
        },
    };
    let json_str = serde_json::to_string(&run_task_msg)?;
    debug_log!("Sending run-task: {}", json_str);
    ws_stream
        .send(Message::Text(json_str))
        .await
        .map_err(|e| anyhow!("Send run-task failed: {}", e))?;

    let mut audio_buffer: Vec<u8> = Vec::new();
    let mut task_started = false;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.map_err(|e| anyhow!("WS Read error: {}", e))?;
        match msg {
            Message::Text(text) => {
                let event: CosyMessage = match serde_json::from_str(&text) {
                    Ok(e) => e,
                    Err(e) => {
                        debug_log!("Parse event error: {}, text: {}", e, text);
                        continue;
                    }
                };
                if let Some(event_type) = event.header.event.clone() {
                    match event_type.as_str() {
                        "task-started" => {
                            task_started = true;
                            let continue_msg = CosyMessage {
                                header: CosyHeader {
                                    action: Some("continue-task".to_string()),
                                    task_id: task_id.clone(),
                                    streaming: Some("duplex".to_string()),
                                    event: None,
                                    error_code: None,
                                    error_message: None,
                                },
                                payload: CosyPayload {
                                    input: Some(CosyInput {
                                        text: Some(input_text.to_string()),
                                    }),
                                    task_group: None,
                                    task: None,
                                    function: None,
                                    model: None,
                                    parameters: None,
                                },
                            };
                            let continue_json = serde_json::to_string(&continue_msg)?;
                            debug_log!("Sending continue-task: {}", continue_json);
                            ws_stream
                                .send(Message::Text(continue_json))
                                .await
                                .map_err(|e| anyhow!("Send continue-task failed: {}", e))?;

                            let finish_msg = CosyMessage {
                                header: CosyHeader {
                                    action: Some("finish-task".to_string()),
                                    task_id: task_id.clone(),
                                    streaming: Some("duplex".to_string()),
                                    event: None,
                                    error_code: None,
                                    error_message: None,
                                },
                                payload: CosyPayload {
                                    input: Some(CosyInput { text: None }),
                                    task_group: None,
                                    task: None,
                                    function: None,
                                    model: None,
                                    parameters: None,
                                },
                            };
                            let finish_json = serde_json::to_string(&finish_msg)?;
                            debug_log!("Sending finish-task: {}", finish_json);
                            ws_stream
                                .send(Message::Text(finish_json))
                                .await
                                .map_err(|e| anyhow!("Send finish-task failed: {}", e))?;
                        }
                        "task-finished" => {
                            debug_log!("CosyVoice finished, bytes: {}", audio_buffer.len());
                            break;
                        }
                        "task-failed" => {
                            let err_msg = event
                                .header
                                .error_message
                                .unwrap_or("Unknown error".to_string());
                            error!("CosyVoice failed: {}", err_msg);
                            return Err(anyhow!("Task failed: {}", err_msg));
                        }
                        _ => {}
                    }
                }
            }
            Message::Binary(bin) => {
                audio_buffer.extend_from_slice(&bin);
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }

    if !task_started {
        return Err(anyhow!("Connection closed before task started"));
    }
    if audio_buffer.is_empty() {
        return Err(anyhow!("No audio data received"));
    }
    Ok(audio_buffer)
}

/// Generate audio bytes via Qwen TTS HTTP API
async fn qwen_generate(
    model: &SupportedModel,
    voice_id: &str,
    text: &str,
    meta: &VoiceMeta,
) -> Result<Vec<u8>> {
    let api_key = std::env::var("DASHSCOPE_API_KEY")
        .or_else(|_| std::env::var("ALIYUN_API_KEY"))
        .unwrap_or_default();
    if api_key.is_empty() {
        return Err(anyhow!("DASHSCOPE_API_KEY missing"));
    }

    let model_name = model.as_str().to_string();

    let base_url =
        "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation";

    let client = reqwest::Client::new();
    let request = client
        .post(base_url)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json");

    if voice_id.is_empty() {
        return Err(anyhow!("base_model_id is empty"));
    }

    let instructions = meta.instruction.clone();
    let optimize_instructions = if instructions.is_some() && model_name.contains("instruct") {
        Some(true)
    } else {
        None
    };

    let payload = DashScopeTtsRequest {
        model: model_name,
        input: DashScopeTtsPayload {
            text: text.to_string(),
            voice: voice_id.to_string(),
            language_type: Some("Auto".to_string()),
            instructions,
            optimize_instructions,
        },
    };

    debug_log!("DashScope TTS request payload: {:?}", payload);

    let response = request
        .json(&payload)
        .send()
        .await
        .map_err(|e| anyhow!("DashScope request failed: {}", e))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| anyhow!("DashScope read response failed: {}", e))?;

    let parsed: DashScopeTtsResponse = serde_json::from_str(&body)
        .map_err(|e| anyhow!("DashScope parse response failed: {}, body: {}", e, body))?;

    if !status.is_success() || parsed.status_code != 200 {
        let code = parsed.code.unwrap_or_else(|| "unknown".to_string());
        let message = parsed
            .message
            .unwrap_or_else(|| "unknown error".to_string());
        return Err(anyhow!(
            "DashScope error: status={}, code={}, message={}",
            parsed.status_code,
            code,
            message
        ));
    }

    let audio = parsed
        .output
        .and_then(|output| output.audio)
        .ok_or_else(|| anyhow!("DashScope response missing audio"))?;

    if let Some(url) = audio.url {
        debug_log!("DashScope audio url: {}", url);
        let audio_response = client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("DashScope audio fetch failed: {}", e))?;
        let audio_status = audio_response.status();
        if !audio_status.is_success() {
            return Err(anyhow!(
                "DashScope audio fetch failed with status {}",
                audio_status
            ));
        }
        let bytes = audio_response
            .bytes()
            .await
            .map_err(|e| anyhow!("DashScope audio read failed: {}", e))?;
        return Ok(bytes.to_vec());
    }

    if let Some(data) = audio.data {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(data.as_bytes())
            .map_err(|e| anyhow!("DashScope audio decode failed: {}", e))?;
        return Ok(decoded);
    }

    error!("DashScope response missing audio url/data");
    Err(anyhow!("DashScope response missing audio content"))
}
