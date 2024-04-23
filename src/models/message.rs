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

    pub fn new_text(receiver_id: u16, content: String, udp_id: Option<u16>) -> Message {
        let content_bytes = content.as_bytes().to_vec();
        let timestamp = Utc::now();
        let metadata = MsgMetadata::new(
            0,
            receiver_id,
            timestamp,
            MessageType::Text,
            content_bytes.len() as u64,
            udp_id,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub fn new_list_clients(
        sender_id: u16,
        clients: Vec<(u16, String)>,
        udp_id: Option<u16>,
    ) -> Message {
        let timestamp = Utc::now();
        let content_json = serde_json::to_string(&clients).unwrap();
        let content_bytes = content_json.as_bytes().to_vec();
        let metadata = MsgMetadata::new(
            sender_id,
            sender_id,
            timestamp,
            MessageType::ListClients,
            content_bytes.len() as u64,
            udp_id,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub fn new_set_name(sender_id: u16, success: bool, udp_id: Option<u16>) -> Message {
        let timestamp = Utc::now();
        let content_bytes = success.to_string().as_bytes().to_vec();
        let metadata = MsgMetadata::new(
            sender_id,
            sender_id,
            timestamp,
            MessageType::SetName,
            content_bytes.len() as u64,
            udp_id,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub fn new_udp_packet(data: Vec<u8>) -> Result<Message, String> {
        let metadata = match MsgMetadata::deserialize(&data, true) {
            Ok(metadata) => metadata,
            Err(e) => {
                return Err(format!("Metadata inválida: \n{0}", e));
            }
        };
        Ok(Message {
            metadata,
            content: data,
        })
    }

    pub async fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.metadata.serialize().await.unwrap());
        bytes.extend(&self.content);
        bytes
    }

    pub fn is_complete(&self) -> bool {
        self.metadata.is_complete(self.content.len() as u64)
    }
}
