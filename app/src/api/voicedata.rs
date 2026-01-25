#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl VoiceModelService for ServiceProvider<Sqlite> {
    async fn list_voice_models(&self) -> anyhow::Result<Vec<VoiceModelInfo>> {
        // 假实现：返回示例语音模型
        let models = vec![
            VoiceModelInfo {
                id: "model_001".to_string(),
                name: "基础女声".to_string(),
                icon_url: "https://cdn.example.com/models/001.png".to_string(),
                metadata: serde_json::json!({
                    "version": "1.0",
                    "language": "zh-CN",
                    "gender": "female",
                    "quality": "high",
                    "latency": "low",
                    "description": "高质量的标准女性声音模型"
                })
                .to_string(),
            },
            VoiceModelInfo {
                id: "model_002".to_string(),
                name: "基础男声".to_string(),
                icon_url: "https://cdn.example.com/models/002.png".to_string(),
                metadata: serde_json::json!({
                    "version": "1.0",
                    "language": "zh-CN",
                    "gender": "male",
                    "quality": "high",
                    "latency": "low",
                    "description": "高质量的标准男性声音模型"
                })
                .to_string(),
            },
            VoiceModelInfo {
                id: "model_003".to_string(),
                name: "儿童声".to_string(),
                icon_url: "https://cdn.example.com/models/003.png".to_string(),
                metadata: serde_json::json!({
                    "version": "1.0",
                    "language": "zh-CN",
                    "gender": "child",
                    "quality": "high",
                    "latency": "low",
                    "description": "可爱的儿童声音模型"
                })
                .to_string(),
            },
        ];
        Ok(models)
    }

    async fn get_voice_model(&self, voice_id: &str) -> anyhow::Result<VoiceModelInfo> {
        // 假实现：根据 ID 返回对应的模型
        match voice_id {
            "model_001" => Ok(VoiceModelInfo {
                id: "model_001".to_string(),
                name: "基础女声".to_string(),
                icon_url: "https://cdn.example.com/models/001.png".to_string(),
                metadata: serde_json::json!({
                    "version": "1.0",
                    "language": "zh-CN",
                    "gender": "female",
                    "quality": "high",
                    "latency": "low",
                    "description": "高质量的标准女性声音模型"
                })
                .to_string(),
            }),
            "model_002" => Ok(VoiceModelInfo {
                id: "model_002".to_string(),
                name: "基础男声".to_string(),
                icon_url: "https://cdn.example.com/models/002.png".to_string(),
                metadata: serde_json::json!({
                    "version": "1.0",
                    "language": "zh-CN",
                    "gender": "male",
                    "quality": "high",
                    "latency": "low",
                    "description": "高质量的标准男性声音模型"
                })
                .to_string(),
            }),
            "model_003" => Ok(VoiceModelInfo {
                id: "model_003".to_string(),
                name: "儿童声".to_string(),
                icon_url: "https://cdn.example.com/models/003.png".to_string(),
                metadata: serde_json::json!({
                    "version": "1.0",
                    "language": "zh-CN",
                    "gender": "child",
                    "quality": "high",
                    "latency": "low",
                    "description": "可爱的儿童声音模型"
                })
                .to_string(),
            }),
            _ => Err(anyhow::anyhow!("Voice model not found: {}", voice_id)),
        }
    }

    async fn update_voice_model(&self, voice: &VoiceModelInfo) -> anyhow::Result<()> {
        // 假实现：仅记录更新操作
        eprintln!("Mock: Updating voice model: {:?}", voice.id);
        Ok(())
    }

    async fn delete_voice_model(&self, voice_id: &str) -> anyhow::Result<()> {
        // 假实现：仅记录删除操作
        eprintln!("Mock: Deleting voice model: {}", voice_id);
        Ok(())
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl VoiceMetadataService for ServiceProvider<Sqlite> {
    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<VoiceMetaInfo>> {
        // 假实现：返回示例数据
        let sample_metadata = vec![
            VoiceMetaInfo {
                id: "1".to_string(),
                name: "甜美少女音".to_string(),
                metadata: serde_json::json!({
                    "description": "温柔甜美的少女声音，适合温馨场景",
                    "base_model_id": "model_001",
                    "pitch": 5.0,
                    "speed": 1.1,
                    "volume": 1.0,
                    "emotion": "happy",
                    "usage_count": 1250,
                    "is_public": true,
                    "tags": ["甜美", "少女", "温柔"],
                    "author": "官方",
                    "is_official": true
                })
                .to_string(),
            },
            VoiceMetaInfo {
                id: "2".to_string(),
                name: "磁性男声".to_string(),
                metadata: serde_json::json!({
                    "description": "低沉磁性的男性声音，充满魅力",
                    "base_model_id": "model_002",
                    "pitch": -3.0,
                    "speed": 0.95,
                    "volume": 1.2,
                    "emotion": "calm",
                    "usage_count": 980,
                    "is_public": true,
                    "tags": ["磁性", "男声", "低沉"],
                    "author": "官方",
                    "is_official": true
                })
                .to_string(),
            },
            VoiceMetaInfo {
                id: "3".to_string(),
                name: "活力少年".to_string(),
                metadata: serde_json::json!({
                    "description": "充满活力的少年声音，元气满满",
                    "base_model_id": "model_003",
                    "pitch": 3.0,
                    "speed": 1.15,
                    "volume": 1.1,
                    "emotion": "excited",
                    "usage_count": 750,
                    "is_public": true,
                    "tags": ["活力", "少年", "元气"],
                    "author": "官方",
                    "is_official": true
                })
                .to_string(),
            },
            VoiceMetaInfo {
                id: "4".to_string(),
                name: "御姐音".to_string(),
                metadata: serde_json::json!({
                    "description": "成熟性感的女性声音",
                    "base_model_id": "model_004",
                    "pitch": -1.0,
                    "speed": 0.9,
                    "volume": 1.0,
                    "emotion": "calm",
                    "usage_count": 520,
                    "is_public": true,
                    "tags": ["御姐", "成熟", "性感"],
                    "author": "用户小明",
                    "is_official": false
                })
                .to_string(),
            },
            VoiceMetaInfo {
                id: "5".to_string(),
                name: "正太音".to_string(),
                metadata: serde_json::json!({
                    "description": "可爱的小男孩声音",
                    "base_model_id": "model_005",
                    "pitch": 8.0,
                    "speed": 1.2,
                    "volume": 0.9,
                    "emotion": "happy",
                    "usage_count": 430,
                    "is_public": true,
                    "tags": ["正太", "可爱", "童声"],
                    "author": "用户小红",
                    "is_official": false
                })
                .to_string(),
            },
            VoiceMetaInfo {
                id: "6".to_string(),
                name: "冷酷杀手".to_string(),
                metadata: serde_json::json!({
                    "description": "冷酷无情的声音，适合反派角色",
                    "base_model_id": "model_006",
                    "pitch": -2.0,
                    "speed": 0.85,
                    "volume": 1.1,
                    "emotion": "angry",
                    "usage_count": 320,
                    "is_public": true,
                    "tags": ["冷酷", "反派", "低沉"],
                    "author": "用户暗影",
                    "is_official": false
                })
                .to_string(),
            },
        ];

        Ok(sample_metadata)
    }

    async fn get_voice_metadata(&self, voice_id: &str) -> anyhow::Result<VoiceMetaInfo> {
        // 模拟实现：返回对应 ID 的元数据
        let all_metadata = self.list_voice_metadata().await?;
        all_metadata
            .into_iter()
            .find(|m| m.id == voice_id)
            .ok_or_else(|| anyhow::anyhow!("Voice metadata not found: {}", voice_id))
    }

    async fn update_voice_metadata(&self, metadata: &VoiceMetaInfo) -> anyhow::Result<()> {
        // 模拟实现：记录更新操作
        leptos::logging::log!("Mock: Updating voice metadata: {}", metadata.name);
        Ok(())
    }

    async fn delete_voice_metadata(&self, voice_id: &str) -> anyhow::Result<()> {
        // 模拟实现：记录删除操作
        leptos::logging::log!("Mock: Deleting voice metadata: {}", voice_id);
        Ok(())
    }
    async fn generate_voice(
        &self,
        voice_info: &VoiceMetaInfo,
        text: &str,
    ) -> anyhow::Result<Vec<u8>> {
        println!("{:?}", voice_info);
        // 模拟实现：生成一个简单的 MP3 格式音频数据
        // 这是一个真实的 MP3 文件头 + 一些音频帧数据
        // 实际的 TTS 引擎应该替换这个实现

        use std::io::Write;
        let mut audio_data = Vec::new();

        // MP3 文件头 (ID3v2)
        let id3_header = b"ID3\x04\x00\x00\x00\x00\x00\x00";
        audio_data.write_all(id3_header)?;

        // MP3 帧头 - 创建多个帧来模拟音频长度
        // MPEG-1 Layer III, 44.1kHz, 128kbps, stereo
        let frame_header = [0xFF, 0xFB, 0x90, 0x00]; // 同步字 + 帧头

        // 根据文本长度生成不同长度的音频
        // 假设每个汉字对应约 1 秒的语音
        let estimated_seconds = if text.is_empty() {
            1
        } else {
            (text.len() / 3).max(1).min(30) // 限制在 1-30 秒之间
        };

        // 每帧约 26ms (44100Hz, 1152 样本)
        let frames_needed = (estimated_seconds * 1000) / 26;

        // 生成音频帧数据
        for _ in 0..frames_needed {
            audio_data.write_all(&frame_header)?;

            // 填充一些随机的音频数据来模拟真实的 MP3 帧
            // 实际的 MP3 帧应该包含压缩的音频数据
            let mut frame_data = vec![0; 417]; // MP3 帧的典型大小
            for i in 0..frame_data.len() {
                // 生成一些伪随机数据，但保持可重复性
                frame_data[i] =
                    (((text.len() as u16) ^ (i as u16) ^ (frames_needed as u16)) % 256) as u8;
            }
            audio_data.write_all(&frame_data)?;
        }

        // 添加 ID3v1 标签 (可选)
        let id3v1_tag = format!(
            "TAG{:<30}{:<30}{:<30}{}{}{}",
            "生成的音频", // 标题
            "耳朵 TTS",   // 艺术家
            "合成音频",   // 相册
            "2026",       // 年份
            "0",          // 注释
            0xFF          // 流派
        );
        audio_data.write_all(id3v1_tag.as_bytes())?;

        Ok(audio_data)
    }
}
