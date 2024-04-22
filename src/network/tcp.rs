use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::config::METADATA_BYTES;
use crate::models::message::Message;
use crate::models::metadata::MsgMetadata;

pub async fn receive(stream: &mut TcpStream) -> Result<Message, String> {
    let mut metadata_buffer = vec![0u8; METADATA_BYTES];
    if stream.read_exact(&mut metadata_buffer).await.is_ok() {
        println!("Lendo metadata...");
        let metadata = match MsgMetadata::deserialize(&metadata_buffer) {
            Ok(meta) => meta,
            Err(e) => {
                return Err(format!("Metadata inválida: \n{0}", e));
            }
        };
        println!("Metadata válida: \n{:?}\n", metadata);

        let message_length = metadata.message_length as usize;
        let mut message_buffer = vec![0u8; message_length];

        if stream.read_exact(&mut message_buffer).await.is_ok() {
            Ok(Message::new(metadata, message_buffer))
        } else {
            Err("Mensagem corrompida".to_string())
        }
    } else {
        Err("Metadata não encontrada".to_string())
    }
}
