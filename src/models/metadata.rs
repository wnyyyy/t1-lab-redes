use std::io::{Error, ErrorKind};

use chrono::{DateTime, TimeZone, Utc};
use tokio::io;
use tokio::io::AsyncWriteExt;

use crate::utilities::enums::MessageType;

#[derive(Debug, Clone)]
pub struct MsgMetadata {
    pub sender_id: u16,
    pub receiver_id: u16,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub message_length: u64,
}

impl MsgMetadata {
    pub fn new(
        sender_id: u16,
        receiver_id: u16,
        timestamp: DateTime<Utc>,
        message_type: MessageType,
        message_length: u64,
    ) -> MsgMetadata {
        MsgMetadata {
            sender_id,
            receiver_id,
            timestamp,
            message_type,
            message_length,
        }
    }

    pub async fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let sender_id_bytes = self.sender_id.to_le_bytes();
        bytes.write_all(&sender_id_bytes).await?;
        let receiver_id_bytes = self.receiver_id.to_le_bytes();
        bytes.write_all(&receiver_id_bytes).await?;

        let timestamp_secs = self.timestamp.timestamp();
        let timestamp_bytes = timestamp_secs.to_le_bytes();
        bytes.write_all(&timestamp_bytes).await?;

        bytes.push(self.message_type as u8);

        let message_length_bytes = self.message_length.to_le_bytes();
        bytes.write_all(&message_length_bytes).await?;

        Ok(bytes)
    }

    pub fn deserialize(data: &[u8]) -> Result<MsgMetadata, Error> {
        let sender_id = u16::from_le_bytes(data[0..2].try_into().unwrap());

        let receiver_id = u16::from_le_bytes(data[2..4].try_into().unwrap());

        let timestamp_secs = match data[4..12].try_into() {
            Ok(bytes) => u64::from_le_bytes(bytes),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Timestamp inválido (Conversão de bits)",
                ))
            }
        };
        let timestamp_mapped = Utc.timestamp_opt(timestamp_secs as i64, 0);
        let timestamp = match timestamp_mapped {
            chrono::LocalResult::Single(timestamp) => timestamp,
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Timestamp inválido (Conversão para DateTime)",
                ))
            }
        };

        let message_type = match MessageType::try_from(data[12]) {
            Ok(message_type) => message_type,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Tipo de mensagem inválido",
                ))
            }
        };

        let message_length = match data[13..].try_into() {
            Ok(bytes) => u64::from_le_bytes(bytes),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Tamanho da mensagem inválido",
                ))
            }
        };

        Ok(MsgMetadata {
            sender_id,
            receiver_id,
            timestamp,
            message_type,
            message_length,
        })
    }

    pub fn string(&self) -> String {
        format!(
            "Remetente: {0}\nDestinatário: {1}\nTimestamp: {2}\nTipo de Mensagem: {3:?}\nTamanho da Mensagem: {4}",
            self.sender_id, self.receiver_id, self.timestamp, self.message_type, self.message_length
        )
    }
}
