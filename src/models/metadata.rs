use std::io::{Error, ErrorKind};

use chrono::{DateTime, Utc};
use tokio::io;
use tokio::io::AsyncWriteExt;

use crate::utilities::enums::MessageType;

#[derive(Debug, Clone)]
pub struct MsgMetadata {
    pub key: u16,
    pub receiver_id: u16,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub message_length: u64,
    pub udp_id: Option<u16>,
    pub udp_seq: Option<u16>,
}

impl MsgMetadata {
    pub fn new(
        key: u16,
        receiver_id: u16,
        message_type: MessageType,
        message_length: u64,
        udp_id: Option<u16>,
        udp_seq: Option<u16>,
    ) -> MsgMetadata {
        MsgMetadata {
            key,
            receiver_id,
            timestamp: Utc::now(),
            message_type,
            message_length,
            udp_id,
            udp_seq,
        }
    }

    pub async fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let key = self.key.to_le_bytes();
        bytes.write_all(&key).await?;
        let receiver_id_bytes = self.receiver_id.to_le_bytes();
        bytes.write_all(&receiver_id_bytes).await?;

        bytes.push(self.message_type as u8);

        let message_length_bytes = self.message_length.to_le_bytes();
        bytes.write_all(&message_length_bytes).await?;

        if let Some(udp_id) = self.udp_id {
            let udp_id_bytes = udp_id.to_le_bytes();
            bytes.write_all(&udp_id_bytes).await?;
        }
        if let Some(udp_seq) = self.udp_seq {
            let udp_seq_bytes = udp_seq.to_le_bytes();
            bytes.write_all(&udp_seq_bytes).await?;
        }

        Ok(bytes)
    }

    pub fn deserialize(data: &[u8], is_udp: bool) -> Result<MsgMetadata, Error> {
        let key = u16::from_le_bytes(data[0..2].try_into().unwrap());

        let receiver_id = u16::from_le_bytes(data[2..4].try_into().unwrap());

        let message_type = match MessageType::try_from(data[4]) {
            Ok(message_type) => message_type,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Tipo de mensagem inválido",
                ))
            }
        };

        let message_length = match data[5..13].try_into() {
            Ok(bytes) => u64::from_le_bytes(bytes),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Tamanho da mensagem inválido",
                ))
            }
        };

        let udp_id;
        let udp_seq;
        if is_udp {
            udp_id = match data[13..15].try_into() {
                Ok(bytes) => Some(u16::from_le_bytes(bytes)),
                Err(_) => return Err(Error::new(ErrorKind::InvalidData, "ID UDP inválido")),
            };
            udp_seq = match data[15..17].try_into() {
                Ok(bytes) => Some(u16::from_le_bytes(bytes)),
                Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Sequência UDP inválida")),
            };
        } else {
            udp_id = None;
            udp_seq = None;
        }

        Ok(MsgMetadata {
            key,
            receiver_id,
            timestamp: Utc::now(),
            message_type,
            message_length,
            udp_id,
            udp_seq,
        })
    }

    pub fn string(&self) -> String {
        format!(
            "ID: {0}\nDestinatário: {1}\nTimestamp: {2}\nTipo de Mensagem: {3:?}\nTamanho da Mensagem: {4}",
            self.key, self.receiver_id, self.timestamp, self.message_type, self.message_length
        )
    }

    pub(crate) fn is_complete(&self, content_size: u64) -> bool {
        self.message_length == content_size
    }
}
