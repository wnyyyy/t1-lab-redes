use chrono::Utc;

use crate::models::metadata::MsgMetadata;
use crate::utilities::enums::MessageType;

#[derive(Debug, Clone)]
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

    pub fn new_list_clients(sender_id: u16, clients: Vec<(u16, String)>) -> Message {
        let timestamp = Utc::now();
        let content_json = serde_json::to_string(&clients).unwrap();
        let content_bytes = content_json.as_bytes().to_vec();
        let metadata = MsgMetadata::new(
            sender_id,
            sender_id,
            timestamp,
            MessageType::ListClients,
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
