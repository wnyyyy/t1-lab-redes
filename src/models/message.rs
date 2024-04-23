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

    pub fn new_text(
        key: u16,
        receiver_id: u16,
        content: String,
        udp_id: Option<u16>,
        udp_seq: Option<u16>,
    ) -> Message {
        let content_bytes = content.as_bytes().to_vec();
        let metadata = MsgMetadata::new(
            key,
            receiver_id,
            MessageType::Text,
            content_bytes.len() as u64,
            udp_id,
            udp_seq,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub fn new_connection_request(key: u16, name: String) -> Message {
        let content_bytes = name.as_bytes().to_vec();
        let metadata = MsgMetadata::new(
            key,
            0,
            MessageType::Connection,
            content_bytes.len() as u64,
            None,
            None,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub fn new_list_clients_request(key: u16) -> Message {
        let metadata = MsgMetadata::new(key, 0, MessageType::ListClients, 0, None, None);
        Message {
            metadata,
            content: Vec::new(),
        }
    }

    pub fn new_list_clients_response(
        key: u16,
        receiver_id: u16,
        clients: Vec<(u16, String)>,
        udp_id: Option<u16>,
        udp_seq: Option<u16>,
    ) -> Message {
        let content_json = serde_json::to_string(&clients).unwrap();
        let content_bytes = content_json.as_bytes().to_vec();
        let metadata = MsgMetadata::new(
            key,
            receiver_id,
            MessageType::Success,
            content_bytes.len() as u64,
            udp_id,
            udp_seq,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub fn new_set_name_request(
        key: u16,
        name: String,
        udp_id: Option<u16>,
        udp_seq: Option<u16>,
    ) -> Message {
        let content_bytes = name.as_bytes().to_vec();
        let metadata = MsgMetadata::new(
            key,
            0,
            MessageType::SetName,
            content_bytes.len() as u64,
            udp_id,
            udp_seq,
        );
        Message {
            metadata,
            content: content_bytes,
        }
    }

    pub fn new_generic_response(key: u16, receiver_id: u16, success: bool) -> Message {
        let metadata = MsgMetadata::new(
            key,
            receiver_id,
            if success {
                MessageType::Success
            } else {
                MessageType::Error
            },
            0,
            None,
            None,
        );
        Message {
            metadata,
            content: Vec::new(),
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

    pub fn generate_key() -> u16 {
        rand::random::<u16>()
    }
}
