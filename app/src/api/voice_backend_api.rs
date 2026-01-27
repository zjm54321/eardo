use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use leptos::logging::{debug_log, error};

#[cfg(feature = "ssr")]
use uuid::Uuid;

#[cfg(feature = "ssr")]
use futures_util::{SinkExt, StreamExt};
#[cfg(feature = "ssr")]
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};

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

/// Generate audio bytes via CosyVoice WebSocket API
pub async fn cosyvoice_generate(
    input_text: &str,
    voice_id: &str,
    rate: f32,
    pitch: f32,
) -> Result<Vec<u8>> {
    #[cfg(not(feature = "ssr"))]
    {
        return Err(anyhow!("CosyVoice generation not available on client"));
    }

    #[cfg(feature = "ssr")]
    {
        let api_key = std::env::var("ALIYUN_API_KEY").unwrap_or_default();
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
                model: Some("cosyvoice-v3-flash".to_string()),
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
}
