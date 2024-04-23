use std::error::Error;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::config::{HOST_ADDRESS, TCP_PORT};
use crate::models::message::Message;
use crate::network::client::Client;
use crate::network::tcp;
use crate::utilities::enums::MessageType;

pub struct TcpClient {
    pub name: String,
    pub id: u16,
    pub log: Arc<RwLock<String>>,
    stream: Arc<RwLock<TcpStream>>,
}

impl Client for TcpClient {
    fn get_id(&self) -> u16 {
        self.id
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    async fn get_log(&self) -> String {
        self.log.read().await.clone()
    }

    async fn set_name(&mut self, name: String) {
        let message = Message::new_set_name_request(name.clone(), None);
        let message_bytes = message.serialize().await;
        let response = self.send(message_bytes);
        if response.is_err() {
            self.log.write().await.push_str(&format!(
                "Erro ao enviar mensagem de mudança de nome: {0}\n",
                response.err().unwrap()
            ));
        }
    }

    async fn send_text(
        &mut self,
        content: String,
        destination_id: u16,
    ) -> Result<(), Box<dyn Error>> {
        let message = Message::new(destination_id, content, None);
        let _ = self.send(message).await?;
        Ok(())
    }

    async fn send(&mut self, message_bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut stream = self.stream.write().await;
        stream.write_all(&message_bytes).await?;
        Ok(())
    }

    fn listen(&mut self) -> Result<String, Box<dyn Error>> {
        let message = self.receive()?;
        Ok(message)
    }
}

impl TcpClient {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let addr = format!("{}:{}", HOST_ADDRESS, TCP_PORT);
        let mut stream = TcpStream::connect(addr).await?;
        let msg = loop {
            let msg = match tcp::receive(&mut stream).await {
                Ok(msg) => msg,
                Err(_) => {
                    continue;
                }
            };
            if msg.metadata.message_type == MessageType::Connection {
                break msg;
            }
        };
        let id = u16::from_le_bytes(msg.content[0..2].try_into().unwrap());
        let log = Arc::new(RwLock::new(String::new()));
        let stream = Arc::new(RwLock::new(stream));
        Ok(TcpClient {
            name: "Sem nome".to_string(),
            log,
            id,
            stream,
        })
    }

    pub async fn send_text(&mut self, content: String, destination_id: u16) -> io::Result<()> {
        if self.stream.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client is not connected",
            ));
        }
        let stream = self.stream.as_mut().unwrap();
        let message = Message::new_text(destination_id, content, None);
        let message_bytes = message.serialize().await;
        stream.write_all(&message_bytes).await
    }

    pub async fn send_raw(&mut self, message: Message) -> io::Result<()> {
        if self.stream.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client is not connected",
            ));
        }
        let stream = self.stream.as_mut().unwrap();
        let message_bytes = message.serialize().await;
        stream.write_all(&message_bytes).await
    }

    pub async fn receive(&mut self) -> Result<String, Box<dyn Error>> {
        if self.stream.is_none() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client is not connected",
            )));
        }
        let stream = self.stream.as_mut().unwrap();
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await?;
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }
}
