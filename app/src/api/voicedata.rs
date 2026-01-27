#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl VoiceModelService for ServiceProvider<Sqlite> {
    async fn list_voice_models(&self) -> anyhow::Result<Vec<VoiceModelInfo>> {
        // 从数据库读取 voice_models 表
        let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
            r#"SELECT id, name, category, description
               FROM voice_models
               WHERE status = 'normal'"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let models = rows
            .into_iter()
            .map(|(id, name, category, description)| VoiceModelInfo {
                id: id.clone(),
                name,
                // 使用 DiceBear 生成占位图标，避免额外静态资源依赖
                icon_url: format!(
                    "https://api.dicebear.com/7.x/shapes/svg?seed={}&backgroundType=gradientLinear&size=64",
                    id
                ),
                metadata: serde_json::json!({
                    "category": category,
                    "description": description.unwrap_or_default()
                })
                .to_string(),
            })
            .collect();
        Ok(models)
    }

    async fn get_voice_model(&self, voice_id: &str) -> anyhow::Result<VoiceModelInfo> {
        let row: (String, String, String, Option<String>) = sqlx::query_as(
            r#"SELECT id, name, category, description
               FROM voice_models
               WHERE id = ? LIMIT 1"#,
        )
        .bind(voice_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| anyhow::anyhow!("Voice model not found: {}", voice_id))?;

        let (id, name, category, description) = row;
        Ok(VoiceModelInfo {
            id: id.clone(),
            name,
            icon_url: format!(
                "https://api.dicebear.com/7.x/shapes/svg?seed={}&backgroundType=gradientLinear&size=64",
                id
            ),
            metadata: serde_json::json!({
                "category": category,
                "description": description.unwrap_or_default()
            })
            .to_string(),
        })
    }

    async fn update_voice_model(&self, voice: &VoiceModelInfo) -> anyhow::Result<()> {
        // 从 metadata 中提取可更新字段
        let meta: serde_json::Value = serde_json::from_str(&voice.metadata).unwrap_or_default();
        let category = meta.get("category").and_then(|v| v.as_str()).unwrap_or("");
        let description = meta
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        sqlx::query(
            r#"UPDATE voice_models
               SET name = ?, category = ?, description = ?
               WHERE id = ?"#,
        )
        .bind(&voice.name)
        .bind(category)
        .bind(description)
        .bind(&voice.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_voice_model(&self, voice_id: &str) -> anyhow::Result<()> {
        // 软删除：隐藏模型
        sqlx::query("UPDATE voice_models SET status = 'hidden' WHERE id = ?")
            .bind(voice_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl VoiceMetadataService for ServiceProvider<Sqlite> {
    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<VoiceMetaInfo>> {
        // 公开且正常的滤镜，按使用量倒序
        let rows: Vec<(
            String,         // id
            String,         // name
            Option<String>, // description
            String,         // base_model_id
            f64,            // pitch
            f64,            // speed
            f64,            // volume
            String,         // emotion
            i64,            // usage_count
            i64,            // is_public
            Option<String>, // nickname
            Option<String>, // username
        )> = sqlx::query_as(
            r#"SELECT vm.id, vm.name, vm.description, vm.base_model_id,
                      vm.pitch, vm.speed, vm.volume, vm.emotion,
                      vm.usage_count, vm.is_public,
                      u.nickname, ua.username
                 FROM voice_meta_infos vm
                 JOIN users u ON vm.user_id = u.id
                 JOIN user_auth ua ON ua.user_id = u.id
                 WHERE vm.status = 'normal' AND vm.is_public = 1
                 ORDER BY vm.usage_count DESC, vm.created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let list = rows
            .into_iter()
            .map(
                |(
                    id,
                    name,
                    description,
                    base_model_id,
                    pitch,
                    speed,
                    volume,
                    emotion,
                    usage,
                    is_public,
                    nickname,
                    username,
                )| {
                    let author = nickname.or(username).unwrap_or_else(|| "未知".to_string());
                    let meta = serde_json::json!({
                        "description": description.unwrap_or_default(),
                        "base_model_id": base_model_id,
                        "pitch": pitch,
                        "speed": speed,
                        "volume": volume,
                        "emotion": emotion,
                        "usage_count": usage as i32,
                        "is_public": is_public == 1,
                        "tags": [],
                        "author": author,
                        "is_official": false
                    });
                    VoiceMetaInfo {
                        id,
                        name,
                        metadata: meta.to_string(),
                    }
                },
            )
            .collect();
        Ok(list)
    }

    async fn get_voice_metadata(&self, voice_id: &str) -> anyhow::Result<VoiceMetaInfo> {
        let row: (
            String,
            String,
            Option<String>,
            String,
            f64,
            f64,
            f64,
            String,
            i64,
            i64,
            Option<String>,
            Option<String>,
        ) = sqlx::query_as(
            r#"SELECT vm.id, vm.name, vm.description, vm.base_model_id,
                      vm.pitch, vm.speed, vm.volume, vm.emotion,
                      vm.usage_count, vm.is_public,
                      u.nickname, ua.username
                 FROM voice_meta_infos vm
                 JOIN users u ON vm.user_id = u.id
                 JOIN user_auth ua ON ua.user_id = u.id
                 WHERE vm.id = ? LIMIT 1"#,
        )
        .bind(voice_id)
        .fetch_one(&self.pool)
        .await?;

        let (
            id,
            name,
            description,
            base_model_id,
            pitch,
            speed,
            volume,
            emotion,
            usage,
            is_public,
            nickname,
            username,
        ) = row;
        let author = nickname.or(username).unwrap_or_else(|| "未知".to_string());
        let meta = serde_json::json!({
            "description": description.unwrap_or_default(),
            "base_model_id": base_model_id,
            "pitch": pitch,
            "speed": speed,
            "volume": volume,
            "emotion": emotion,
            "usage_count": usage as i32,
            "is_public": is_public == 1,
            "tags": [],
            "author": author,
            "is_official": false
        });

        Ok(VoiceMetaInfo {
            id,
            name,
            metadata: meta.to_string(),
        })
    }

    async fn update_voice_metadata(&self, metadata: &VoiceMetaInfo) -> anyhow::Result<()> {
        // 从 metadata JSON 提取字段并更新
        let meta: serde_json::Value = serde_json::from_str(&metadata.metadata).unwrap_or_default();
        let description = meta
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let base_model_id = meta
            .get("base_model_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let pitch = meta.get("pitch").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let speed = meta.get("speed").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let volume = meta.get("volume").and_then(|v| v.as_f64()).unwrap_or(1.0);
        let emotion = meta
            .get("emotion")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        let is_public = meta
            .get("is_public")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // 约束范围：pitch/speed/volume between 0.5~2.5
        let clamp = |v: f64| v.max(0.5).min(2.5);
        let pitch = clamp(pitch);
        let speed = clamp(speed);
        let volume = clamp(volume);

        sqlx::query(
            r#"UPDATE voice_meta_infos
               SET name = ?, description = ?, base_model_id = ?,
                   pitch = ?, speed = ?, volume = ?, emotion = ?, is_public = ?
               WHERE id = ?"#,
        )
        .bind(&metadata.name)
        .bind(description)
        .bind(base_model_id)
        .bind(pitch)
        .bind(speed)
        .bind(volume)
        .bind(emotion)
        .bind(if is_public { 1 } else { 0 })
        .bind(&metadata.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_voice_metadata(&self, voice_id: &str) -> anyhow::Result<()> {
        // 软删除
        sqlx::query("UPDATE voice_meta_infos SET status = 'deleted' WHERE id = ?")
            .bind(voice_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn generate_voice(
        &self,
        voice_info: &VoiceMetaInfo,
        text: &str,
    ) -> anyhow::Result<Vec<u8>> {
        // 从 metadata 解析所需参数
        let meta: serde_json::Value =
            serde_json::from_str(&voice_info.metadata).unwrap_or_default();

        leptos::logging::debug_log!("generate_voice metadata: {}", voice_info.metadata);

        let base_model_id = meta
            .get("base_model_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 如果 base_model_id 为空，尝试使用默认语音
        let voice_id = if base_model_id.is_empty() {
            leptos::logging::error!("base_model_id is empty, using default voice");
            "longanhuan" // 使用默认语音
        } else {
            base_model_id
        };

        let pitch = meta.get("pitch").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        let speed = meta.get("speed").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

        leptos::logging::debug_log!(
            "Calling CosyVoice with voice_id={}, pitch={}, speed={}",
            voice_id,
            pitch,
            speed
        );

        // 调用 CosyVoice 后端生成，失败直接返回错误
        crate::api::voice_backend_api::cosyvoice_generate(text, voice_id, speed, pitch).await
    }
}
