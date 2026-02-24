#![cfg(feature = "ssr")]
use {
    crate::api::{ServiceProvider, voice::*, voice_backend::generate_audio},
    anyhow::Context,
    sqlx::Postgres,
    uuid::Uuid,
};

// 临时结构体，用于从数据库中读取展平的字段
#[derive(sqlx::FromRow)]
struct VoiceModelRow {
    id: Uuid,
    voice_id: String,
    name: String,
    description: String,
    category: String, // 数据库中是 enum，这里先读为 String
    supported_model: String,
    user_id: Option<Uuid>,
    voice_generate_data_id: Option<Uuid>,
    voice_clone: bool,
    voice_design: bool,
    ssml: bool,
    latex: bool,
    parametric_control: bool,
    instruction_control: bool,
}

impl From<VoiceModelRow> for VoiceModel {
    fn from(row: VoiceModelRow) -> Self {
        let category = match row.category.as_str() {
            "Official" => VoiceModelCategory::Official(row.supported_model),
            "User" => VoiceModelCategory::User {
                supported_model: row.supported_model,
                user_id: row.user_id.unwrap_or_default(),
                voice_generate_data_id: row.voice_generate_data_id.unwrap_or_default(),
            },
            _ => VoiceModelCategory::Official(String::new()), // Fallback
        };

        VoiceModel {
            id: row.id,
            voice_id: row.voice_id,
            info: VoiceModelInfo {
                name: row.name,
                description: row.description,
            },
            avatar_url: format!("/api/voice_avatar/{}", row.id),
            category,
            ability: VoiceModelAbility {
                voice_clone: row.voice_clone,
                voice_design: row.voice_design,
                ssml: row.ssml,
                latex: row.latex,
                parametric_control: row.parametric_control,
                instruction_control: row.instruction_control,
            },
        }
    }
}

#[derive(sqlx::FromRow)]
struct VoiceMetaRow {
    voice_model_id: Uuid,
    pitch: Option<f32>,
    speed: Option<f32>,
    volume: Option<f32>,
    instruction: Option<String>,
}

impl From<VoiceMetaRow> for VoiceMeta {
    fn from(row: VoiceMetaRow) -> Self {
        let parametric = if row.pitch.is_some() || row.speed.is_some() || row.volume.is_some() {
            Some(Parametic {
                pitch: row.pitch.unwrap_or(1.0),
                speed: row.speed.unwrap_or(1.0),
                volume: row.volume.unwrap_or(1.0),
            })
        } else {
            None
        };

        VoiceMeta {
            voice_model_id: row.voice_model_id,
            parametric,
            instruction: row.instruction,
        }
    }
}

#[async_trait::async_trait]
impl VoiceService for ServiceProvider<Postgres> {
    async fn list_voice_models(&self) -> anyhow::Result<Vec<VoiceModel>> {
        let rows: Vec<VoiceModelRow> = sqlx::query_as(
            r#"
            SELECT 
                id, voice_id, name, description, 
                category::text, supported_model, user_id, voice_generate_data_id,
                voice_clone, voice_design, ssml, latex, parametric_control, instruction_control
            FROM voice_model
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("获取语音模型列表失败")?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn search_voice_model(&self, info: &str) -> anyhow::Result<Vec<VoiceModel>> {
        let search_pattern = format!("%{}%", info);
        let rows: Vec<VoiceModelRow> = sqlx::query_as(
            r#"
            SELECT 
                id, voice_id, name, description, 
                category::text, supported_model, user_id, voice_generate_data_id,
                voice_clone, voice_design, ssml, latex, parametric_control, instruction_control
            FROM voice_model
            WHERE name ILIKE $1 OR description ILIKE $1 OR voice_id ILIKE $1
            "#,
        )
        .bind(search_pattern)
        .fetch_all(&self.pool)
        .await
        .context("搜索语音模型失败")?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn generate_voice_model(
        &self,
        _voice_generate_meta: VoiceModelInfo,
        _voice_data: VoiceGenerateData,
    ) -> anyhow::Result<VoiceModelID> {
        unimplemented!("语音模型生成接口尚未实现，等待后端完善相关逻辑")
    }

    async fn update_voice_model(
        &self,
        voice_model_id: VoiceModelID,
        voice_model_info: VoiceModelInfo,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE voice_model 
            SET name = $1, description = $2
            WHERE id = $3
            "#,
        )
        .bind(&voice_model_info.name)
        .bind(&voice_model_info.description)
        .bind(voice_model_id)
        .execute(&self.pool)
        .await
        .context("更新语音模型信息失败")?;

        Ok(())
    }

    async fn generate_meta(&self, voice_meta: VoiceMeta) -> anyhow::Result<VoiceMetaID> {
        let meta_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO voice_meta (voice_model_id, pitch, speed, volume, instruction)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(&voice_meta.voice_model_id)
        .bind(voice_meta.parametric.as_ref().map(|p| p.pitch))
        .bind(voice_meta.parametric.as_ref().map(|p| p.speed))
        .bind(voice_meta.parametric.as_ref().map(|p| p.volume))
        .bind(&voice_meta.instruction)
        .fetch_one(&self.pool)
        .await
        .context("插入 voice_meta 失败")?;

        Ok(meta_id)
    }

    async fn generate_voice(
        &self,
        voice_meta_id: VoiceMetaID,
        text: &str,
    ) -> anyhow::Result<VoiceLibraryID> {
        // 1. 获取 voice_meta
        let voice_meta: VoiceMetaRow = sqlx::query_as(
            "SELECT voice_model_id, pitch, speed, volume, instruction FROM voice_meta WHERE id = $1"
        )
        .bind(voice_meta_id)
        .fetch_one(&self.pool)
        .await
        .context("未找到对应的语音元数据")?;
        let voice_meta: VoiceMeta = voice_meta.into();

        // 2. 获取 voice_model 的外部 voice_id 和 supported_model
        let (voice_id, supported_model): (String, String) =
            sqlx::query_as("SELECT voice_id, supported_model FROM voice_model WHERE id = $1")
                .bind(voice_meta.voice_model_id)
                .fetch_one(&self.pool)
                .await
                .context("未找到对应的语音模型")?;

        // 3. 解析 supported_model 为枚举
        let model = supported_model
            .parse::<crate::api::voice_backend::SupportedModel>()
            .context("不支持的语音模型类型")?;

        // 4. 调用外部 TTS API 生成音频
        let audio_bytes = generate_audio(&model, &voice_id, text, &voice_meta).await?;

        // 5. 将生成的记录保存到 voice_library 表中
        let library_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO voice_library (voice_meta_id, text_content, audio_data)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(voice_meta_id)
        .bind(text)
        .bind(audio_bytes)
        .fetch_one(&self.pool)
        .await
        .context("插入 voice_library 失败")?;

        Ok(library_id)
    }
}
