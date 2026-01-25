#[cfg(feature = "ssr")]
use sqlx::Sqlite;

use crate::api::*;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl VoiceModelService for ServiceProvider<Sqlite> {
    async fn list_voice_models(&self) -> anyhow::Result<Vec<VoiceModelInfo>> {
        todo!()
    }
    async fn get_voice_model(&self, voice_id: &str) -> anyhow::Result<VoiceModelInfo> {
        todo!()
    }
    async fn update_voice_model(&self, voice: &VoiceModelInfo) -> anyhow::Result<()> {
        todo!()
    }
    async fn delete_voice_model(&self, voice_id: &str) -> anyhow::Result<()> {
        todo!()
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl VoiceMetadataService for ServiceProvider<Sqlite> {
    async fn list_voice_metadata(&self) -> anyhow::Result<Vec<VoiceMetaInfo>> {
        todo!()
    }
    async fn get_voice_metadata(&self, voice_id: &str) -> anyhow::Result<VoiceMetaInfo> {
        todo!()
    }
    async fn update_voice_metadata(&self, metadata: &VoiceMetaInfo) -> anyhow::Result<()> {
        todo!()
    }
    async fn delete_voice_metadata(&self, voice_id: &str) -> anyhow::Result<()> {
        todo!()
    }
}
