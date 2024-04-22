use chrono::Utc;

use crate::models::metadata::MsgMetadata;
use crate::utilities::enums::MessageType;

#[derive(Debug)]
pub struct Message {
    pub metadata: MsgMetadata,
    pub content: Vec<u8>,
}

impl Message {
    pub fn new(metadata: MsgMetadata, content: Vec<u8>) -> Message {
        Message { metadata, content }
    }

    pub fn new_text(receiver_id: u16, content: String) -> Message {
        let content_bytes = content.as_bytes().to_vec();
        let timestamp = Utc::now();
        let metadata = MsgMetadata::new(
            0,
            receiver_id,
            timestamp,
            MessageType::Text,
            content_bytes.len() as u64,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub async fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.metadata.serialize().await.unwrap());
        bytes.extend(&self.content);
        bytes
    }
}
