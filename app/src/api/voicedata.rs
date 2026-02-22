#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

fn parse_voice_category(raw: &str, _model_name: &str) -> VoiceModelCategory {
    match raw.to_lowercase().as_str() {
        "user_designed" | "user" | "用户设计" => VoiceModelCategory::UserDesigned,
        "voice_generated" | "generated" | "声音生成" => VoiceModelCategory::VoiceGenerated,
        _ => VoiceModelCategory::Official(raw.to_string()),
    }
}

fn category_to_db(category: &VoiceModelCategory) -> String {
    match category {
        VoiceModelCategory::Official(content) => content.clone(),
        VoiceModelCategory::UserDesigned => "user_designed".to_string(),
        VoiceModelCategory::VoiceGenerated => "voice_generated".to_string(),
    }
}

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
            .map(|(id, name, category, description)| {
                let model_category = parse_voice_category(&category, &name);
                VoiceModelInfo {
                    id: id.clone(),
                    name,
                    // 使用 DiceBear 生成占位图标，避免额外静态资源依赖
                    icon_url: format!(
                        "https://api.dicebear.com/7.x/shapes/svg?seed={}&backgroundType=gradientLinear&size=64",
                        id
                    ),
                    category: model_category,
                    description: description.unwrap_or_default(),
                }
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
        let model_category = parse_voice_category(&category, &name);
        Ok(VoiceModelInfo {
            id: id.clone(),
            name,
            icon_url: format!(
                "https://api.dicebear.com/7.x/shapes/svg?seed={}&backgroundType=gradientLinear&size=64",
                id
            ),
            category: model_category,
            description: description.unwrap_or_default(),
        })
    }

    async fn update_voice_model(&self, voice: &VoiceModelInfo) -> anyhow::Result<()> {
        sqlx::query(
            r#"UPDATE voice_models
               SET name = ?, category = ?, description = ?
               WHERE id = ?"#,
        )
        .bind(&voice.name)
        .bind(category_to_db(&voice.category))
        .bind(&voice.description)
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
    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<VoiceMetaPost>> {
        // 公开且正常的滤镜，按使用量倒序
        let rows: Vec<(
            String,         // id
            String,         // name
            Option<String>, // description
            String,         // base_model_id
            String,         // model_name
            String,         // model_category
            Option<String>, // model_description
            f64,            // pitch
            f64,            // speed
            f64,            // volume
            i64,            // usage_count
            i64,            // is_public
            String,         // status
            Option<String>, // nickname
            Option<String>, // username
            String,         // created_at
        )> = sqlx::query_as(
            r#"SELECT vm.id, vm.name, vm.description, vm.base_model_id,
                      m.name as model_name, m.category as model_category, m.description as model_description,
                      vm.pitch, vm.speed, vm.volume,
                      vm.usage_count, vm.is_public, vm.status,
                      u.nickname, ua.username, vm.created_at
                 FROM voice_meta_infos vm
                 JOIN voice_models m ON vm.base_model_id = m.id
                 JOIN users u ON vm.user_id = u.id
                 JOIN user_auth ua ON ua.user_id = u.id
                 WHERE (vm.status = 'normal' OR vm.status = 'official') AND vm.is_public = 1
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
                    model_name,
                    model_category,
                    model_description,
                    pitch,
                    speed,
                    volume,
                    usage,
                    is_public,
                    status,
                    nickname,
                    username,
                    created_at,
                )| {
                    let author = nickname.or(username).unwrap_or_else(|| "未知".to_string());
                    let metadata = VoiceMetadata::Parametric(VoiceParams {
                        pitch: pitch as f32,
                        speed: speed as f32,
                        volume: volume as f32,
                    });
                    let model_category = parse_voice_category(&model_category, &model_name);
                    let base_model = VoiceModelInfo {
                        id: base_model_id.clone(),
                        name: model_name,
                        icon_url: format!(
                            "https://api.dicebear.com/7.x/shapes/svg?seed={}&backgroundType=gradientLinear&size=64",
                            base_model_id
                        ),
                        category: model_category,
                        description: model_description.unwrap_or_default(),
                    };
                    VoiceMetaPost {
                        id,
                        name,
                        base_model,
                        metadata,
                        author,
                        description: description.unwrap_or_default(),
                        tags: vec![],
                        usage_count: usage as i32,
                        is_public: is_public == 1,
                        is_official: status == "official",
                        created_at: created_at.clone(),
                        updated_at: created_at,
                    }
                },
            )
            .collect();
        Ok(list)
    }

    async fn get_voice_metadata(&self, voice_id: &str) -> anyhow::Result<VoiceMetaPost> {
        let row: (
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            Option<String>,
            f64,
            f64,
            f64,
            i64,
            i64,
            String,
            Option<String>,
            Option<String>,
            String,
        ) = sqlx::query_as(
            r#"SELECT vm.id, vm.name, vm.description, vm.base_model_id,
                      m.name as model_name, m.category as model_category, m.description as model_description,
                      vm.pitch, vm.speed, vm.volume,
                      vm.usage_count, vm.is_public, vm.status,
                      u.nickname, ua.username, vm.created_at
                 FROM voice_meta_infos vm
                 JOIN voice_models m ON vm.base_model_id = m.id
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
            model_name,
            model_category,
            model_description,
            pitch,
            speed,
            volume,
            usage,
            is_public,
            status,
            nickname,
            username,
            created_at,
        ) = row;
        let author = nickname.or(username).unwrap_or_else(|| "未知".to_string());
        let metadata = VoiceMetadata::Parametric(VoiceParams {
            pitch: pitch as f32,
            speed: speed as f32,
            volume: volume as f32,
        });
        let model_category = parse_voice_category(&model_category, &model_name);
        let base_model = VoiceModelInfo {
            id: base_model_id.clone(),
            name: model_name,
            icon_url: format!(
                "https://api.dicebear.com/7.x/shapes/svg?seed={}&backgroundType=gradientLinear&size=64",
                base_model_id
            ),
            category: model_category,
            description: model_description.unwrap_or_default(),
        };

        Ok(VoiceMetaPost {
            id,
            name,
            base_model,
            metadata,
            author,
            description: description.unwrap_or_default(),
            tags: vec![],
            usage_count: usage as i32,
            is_public: is_public == 1,
            is_official: status == "official",
            created_at: created_at.clone(),
            updated_at: created_at,
        })
    }

    async fn update_voice_metadata(&self, metadata: &VoiceMetaPost) -> anyhow::Result<()> {
        let description = metadata.description.as_str();
        let base_model_id = metadata.base_model.id.as_str();
        let is_public = metadata.is_public;

        let (pitch, speed, volume) = match &metadata.metadata {
            VoiceMetadata::Parametric(params) => (
                params.pitch as f64,
                params.speed as f64,
                params.volume as f64,
            ),
            VoiceMetadata::Instruction(_) => (0.0, 1.0, 1.0),
        };

        // 约束范围：pitch/speed/volume between 0.5~2.5
        let clamp = |v: f64| v.max(0.5).min(2.5);
        let pitch = clamp(pitch);
        let speed = clamp(speed);
        let volume = clamp(volume);

        sqlx::query(
            r#"UPDATE voice_meta_infos
               SET name = ?, description = ?, base_model_id = ?,
                   pitch = ?, speed = ?, volume = ?, is_public = ?
               WHERE id = ?"#,
        )
        .bind(&metadata.name)
        .bind(description)
        .bind(base_model_id)
        .bind(pitch)
        .bind(speed)
        .bind(volume)
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
        let base_model_id = voice_info.base_model.id.as_str();

        // 如果 base_model_id 为空，返回错误
        if base_model_id.is_empty() {
            return Err(anyhow::anyhow!("base_model_id is empty"));
        }
        let voice_id = base_model_id;

        leptos::logging::log!(
            "使用以下参数创建了音频请求：\n {:?}",
            voice_info
        );

        crate::api::voice_backend_api::qwen_generate(
            voice_info,
            text,
        )
        .await
    }
}
