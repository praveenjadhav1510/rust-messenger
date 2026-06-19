use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextPayload {
    pub text: String,
    pub reply_to: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadataPayload {
    pub file_id: Uuid,
    pub filename: String,
    pub size: u64,
    pub mime_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileChunkPayload {
    pub file_id: Uuid,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub data: String, // Base64-encoded chunk data
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProtocolPayload {
    Text(TextPayload),
    FileMetadata(FileMetadataPayload),
    FileChunk(FileChunkPayload),
}
