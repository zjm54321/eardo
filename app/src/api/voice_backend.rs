#![cfg(feature = "ssr")]

use crate::api::voice::VoiceMeta;
use anyhow::{Result, anyhow};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{str::FromStr, sync::OnceLock, time::Duration};
use tokio::{sync::Semaphore, time::timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};
use uuid::Uuid;

const MAX_AUDIO_BYTES: usize = 20 * 1024 * 1024;
// Accommodate a base64-encoded 20 MiB audio response plus JSON metadata.
const MAX_TTS_RESPONSE_BYTES: usize = 28 * 1024 * 1024;

fn tts_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("TTS_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60)
            .clamp(1, 600),
    )
}

/// One connection pool for TTS, signed audio downloads, and LLM requests.
pub(crate) fn http_client() -> Result<&'static reqwest::Client> {
    static CLIENT: OnceLock<Result<reqwest::Client, reqwest::Error>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(tts_timeout())
                .build()
        })
        .as_ref()
        .map_err(|_| anyhow!("Provider HTTP client initialization failed"))
}

/// Read incrementally: Content-Length is optional and cannot be trusted alone.
pub(crate) async fn read_limited(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>> {
    if response.content_length().is_some_and(|len| len > limit as u64) {
        return Err(anyhow!("Provider response exceeds size limit"));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow!("Provider response read failed"))?
    {
        if chunk.len() > limit.saturating_sub(body.len()) {
            return Err(anyhow!("Provider response exceeds size limit"));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Provider messages may echo user text or URLs. Only expose a bounded code.
pub(crate) fn sanitized_code(code: Option<&str>) -> String {
    code.unwrap_or("unknown")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .take(80)
        .collect()
}

fn generation_semaphore() -> &'static Semaphore {
    static SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();
    SEMAPHORE.get_or_init(|| {
        let permits = std::env::var("TTS_MAX_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(2)
            .clamp(1, 64);
        Semaphore::new(permits)
    })
}

fn aliyun_key() -> Result<String> {
    std::env::var("ALIYUN_API_KEY")
        .or_else(|_| std::env::var("DASHSCOPE_API_KEY"))
        .ok()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| anyhow!("ALIYUN_API_KEY or DASHSCOPE_API_KEY missing"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedModel {
    CosyVoiceV3Plus,
    CosyVoiceV3Flash,
    Qwen3TtsFlash,
    QwenAudio31TtsFlash,
    // Keep legacy database IDs readable; synthesis gives an actionable error.
    Qwen3TtsVd20260126,
    Qwen3TtsVc20260122,
}

impl FromStr for SupportedModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cosyvoice-v3-plus" => Ok(Self::CosyVoiceV3Plus),
            "cosyvoice-v3-flash" => Ok(Self::CosyVoiceV3Flash),
            "qwen3-tts-flash"
            | "qwen3-tts-instruct-flash"
            | "qwen3-tts-instruct-flash-2026-01-26" => Ok(Self::Qwen3TtsFlash),
            "qwen-audio-3.1-tts-flash" => Ok(Self::QwenAudio31TtsFlash),
            "qwen3-tts-vd-2026-01-26" => Ok(Self::Qwen3TtsVd20260126),
            "qwen3-tts-vc-2026-01-22" => Ok(Self::Qwen3TtsVc20260122),
            "cosyvoice-v3.5-flash" | "cosyvoice-v3.5-plus" => Err(anyhow!(
                "CosyVoice v3.5 requires an enrolled voice; voice enrollment is not supported yet"
            )),
            _ => Err(anyhow!("Unsupported TTS model")),
        }
    }
}

impl SupportedModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CosyVoiceV3Plus => "cosyvoice-v3-plus",
            Self::CosyVoiceV3Flash => "cosyvoice-v3-flash",
            Self::Qwen3TtsFlash => "qwen3-tts-flash",
            Self::QwenAudio31TtsFlash => "qwen-audio-3.1-tts-flash",
            Self::Qwen3TtsVd20260126 => "qwen3-tts-vd-2026-01-26",
            Self::Qwen3TtsVc20260122 => "qwen3-tts-vc-2026-01-22",
        }
    }
}

/// The deadline covers queueing, synthesis, and the audio URL download.
pub async fn generate_audio(
    model: &SupportedModel,
    voice_id: &str,
    text: &str,
    meta: &VoiceMeta,
) -> Result<Vec<u8>> {
    if matches!(model, SupportedModel::Qwen3TtsVd20260126 | SupportedModel::Qwen3TtsVc20260122) {
        return Err(anyhow!(
            "This legacy voice-design/cloning model requires enrollment, which is not supported yet; select an official voice"
        ));
    }
    timeout(tts_timeout(), async {
        let _permit = generation_semaphore()
            .acquire()
            .await
            .map_err(|_| anyhow!("TTS generation unavailable"))?;
        match model {
            SupportedModel::CosyVoiceV3Plus | SupportedModel::CosyVoiceV3Flash => {
                let rate = meta.parametric.as_ref().map(|p| p.speed).unwrap_or(1.0);
                let pitch = meta.parametric.as_ref().map(|p| p.pitch).unwrap_or(1.0);
                // App volume is a multiplier (1.0 normal); CosyVoice uses 0..100 (50 normal).
                let volume = meta.parametric.as_ref().map(|p| p.volume).unwrap_or(1.0);
                if !volume.is_finite() || !rate.is_finite() || !pitch.is_finite() {
                    return Err(anyhow!("Invalid voice parameters"));
                }
                cosyvoice_generate(text, voice_id, rate, pitch, (volume * 50.0).clamp(0.0, 100.0).round() as u32, model.as_str()).await
            }
            _ => qwen_generate(model, voice_id, text, meta).await,
        }
    })
    .await
    .map_err(|_| anyhow!("TTS generation timed out"))?
}

async fn cosyvoice_generate(
    input_text: &str,
    voice_id: &str,
    rate: f32,
    pitch: f32,
    volume: u32,
    model_name: &str,
) -> Result<Vec<u8>> {
    let key = aliyun_key()?;
    let task_id = Uuid::new_v4().simple().to_string();
    let mut request = "wss://dashscope.aliyuncs.com/api-ws/v1/inference"
        .into_client_request()
        .map_err(|_| anyhow!("TTS WebSocket request build failed"))?;
    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {key}").parse().map_err(|_| anyhow!("Invalid TTS API key header"))?,
    );
    let (mut socket, _) = timeout(Duration::from_secs(10), connect_async(request))
        .await
        .map_err(|_| anyhow!("TTS WebSocket connection timed out"))?
        .map_err(|_| anyhow!("TTS WebSocket connection failed"))?;
    let message = |action: &str, payload: Value| {
        Message::Text(json!({
            "header": {"action": action, "task_id": task_id, "streaming": "duplex"},
            "payload": payload,
        }).to_string())
    };
    socket.send(message("run-task", json!({
        "task_group": "audio", "task": "tts", "function": "SpeechSynthesizer",
        "model": model_name,
        "parameters": {"text_type": "PlainText", "voice": voice_id, "format": "mp3",
            "sample_rate": 22050, "volume": volume, "rate": rate, "pitch": pitch},
        "input": {},
    }))).await.map_err(|_| anyhow!("TTS task submission failed"))?;

    let mut audio = Vec::new();
    let mut started = false;
    let mut finished = false;
    loop {
        let next = timeout(Duration::from_secs(30), socket.next())
            .await
            .map_err(|_| anyhow!("TTS WebSocket read timed out"))?;
        let Some(next) = next else { break };
        match next.map_err(|_| anyhow!("TTS WebSocket read failed"))? {
            Message::Text(text) => {
                let event: Value = serde_json::from_str(&text)
                    .map_err(|_| anyhow!("Invalid TTS WebSocket event"))?;
                match event["header"]["event"].as_str() {
                    Some("task-started") if !started => {
                        started = true;
                        socket.send(message("continue-task", json!({"input": {"text": input_text}})))
                            .await.map_err(|_| anyhow!("TTS text submission failed"))?;
                        socket.send(message("finish-task", json!({"input": {}})))
                            .await.map_err(|_| anyhow!("TTS finish submission failed"))?;
                    }
                    Some("task-finished") => { finished = true; break; }
                    Some("task-failed") => return Err(anyhow!("TTS task failed ({})",
                        sanitized_code(event["header"]["error_code"].as_str()))),
                    _ => {}
                }
            }
            Message::Binary(bytes) => {
                if bytes.len() > MAX_AUDIO_BYTES.saturating_sub(audio.len()) {
                    return Err(anyhow!("TTS audio exceeds 20 MiB limit"));
                }
                audio.extend_from_slice(&bytes);
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    if !started || !finished {
        return Err(anyhow!("TTS connection closed before task-finished"));
    }
    if audio.is_empty() {
        return Err(anyhow!("TTS returned no audio"));
    }
    Ok(audio)
}

async fn qwen_generate(
    model: &SupportedModel,
    voice_id: &str,
    text: &str,
    meta: &VoiceMeta,
) -> Result<Vec<u8>> {
    if voice_id.is_empty() {
        return Err(anyhow!("TTS voice ID is empty"));
    }
    let is_audio = matches!(model, SupportedModel::QwenAudio31TtsFlash);
    let (url, mut input) = if is_audio {
        ("https://dashscope.aliyuncs.com/api/v1/services/audio/tts/SpeechSynthesizer",
            json!({"text": text, "voice": voice_id, "format": "mp3", "sample_rate": 24000}))
    } else {
        ("https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation",
            json!({"text": text, "voice": voice_id, "language_type": "Auto"}))
    };
    // Qwen3 Flash has no instruction control. New Qwen-Audio uses singular instruction.
    if is_audio && let Some(instruction) = &meta.instruction {
        input["instruction"] = json!(instruction);
    }
    let client = http_client()?;
    let response = client.post(url)
        .bearer_auth(aliyun_key()?)
        .json(&json!({"model": model.as_str(), "input": input}))
        .send().await.map_err(|_| anyhow!("DashScope TTS request failed"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!("DashScope TTS returned HTTP {}", status.as_u16()));
    }
    let body = read_limited(response, MAX_TTS_RESPONSE_BYTES).await?;
    let parsed: Value = serde_json::from_slice(&body)
        .map_err(|_| anyhow!("Invalid DashScope TTS response"))?;
    if parsed.get("code").is_some_and(|code| !code.is_null())
        || parsed.get("status_code").and_then(Value::as_u64).is_some_and(|code| code != 200)
    {
        return Err(anyhow!("DashScope TTS failed ({})", sanitized_code(parsed["code"].as_str())));
    }
    let audio = &parsed["output"]["audio"];
    let bytes = if let Some(url) = audio["url"].as_str() {
        // DashScope result buckets (`<bucket>.oss-<region>.aliyuncs.com`) are IPv4-only.
        // The production VM is IPv6-only, so try OSS's dual-stack host first
        // (`<bucket>.<region>.oss.aliyuncs.com`, same signature) and fall back to the original.
        let mut response = None;
        if let Some(dual_stack) = oss_dual_stack_url(url) {
            response = client.get(&dual_stack).send().await.ok()
                .filter(|r| r.status().is_success());
        }
        let response = match response {
            Some(r) => r,
            None => client.get(url).send().await
                .map_err(|_| anyhow!("TTS audio download failed"))?,
        };
        if !response.status().is_success() {
            return Err(anyhow!("TTS audio download returned HTTP {}", response.status().as_u16()));
        }
        read_limited(response, MAX_AUDIO_BYTES).await?
    } else if let Some(data) = audio["data"].as_str() {
        let bytes = base64::engine::general_purpose::STANDARD.decode(data)
            .map_err(|_| anyhow!("Invalid TTS audio encoding"))?;
        if bytes.len() > MAX_AUDIO_BYTES {
            return Err(anyhow!("TTS audio exceeds 20 MiB limit"));
        }
        bytes
    } else {
        return Err(anyhow!("DashScope TTS response missing audio"));
    };
    if bytes.is_empty() {
        return Err(anyhow!("TTS returned no audio"));
    }
    Ok(bytes)
}

/// `http(s)://<bucket>.oss-<region>.aliyuncs.com/<path>` → `https://<bucket>.<region>.oss.aliyuncs.com/<path>`
fn oss_dual_stack_url(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;
    let (host, path) = rest.split_once('/')?;
    let host = host.strip_suffix(".aliyuncs.com")?;
    let (bucket, region) = host.split_once(".oss-")?;
    if bucket.is_empty() || region.is_empty() || region.contains('.') {
        return None;
    }
    Some(format!("https://{bucket}.{region}.oss.aliyuncs.com/{path}"))
}

#[cfg(test)]
mod oss_tests {
    #[test]
    fn rewrites_dashscope_result_url() {
        assert_eq!(
            super::oss_dual_stack_url("http://dashscope-result-bj.oss-cn-beijing.aliyuncs.com/prod/a.wav?Expires=1&Signature=x").as_deref(),
            Some("https://dashscope-result-bj.cn-beijing.oss.aliyuncs.com/prod/a.wav?Expires=1&Signature=x")
        );
        assert_eq!(super::oss_dual_stack_url("https://example.com/a.wav"), None);
    }
}
