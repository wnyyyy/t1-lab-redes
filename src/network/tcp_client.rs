use std::error::Error;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
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

    async fn set_name(&mut self, name: String) -> Result<u16, Box<dyn Error>> {
        let key = Message::generate_key();
        let message = Message::new_set_name_request(key, name.clone(), None, None);
        let message_bytes = message.serialize().await;
        self.send(message_bytes).await?;
        Ok(key)
    }

    async fn send_connection_request(&mut self, name: String) -> Result<(), Box<dyn Error>> {
        let key = Message::generate_key();
        let message = Message::new_connection_request(key, name);
        self.send(message.serialize().await).await?;
        Ok(())
    }

    async fn send_text(
        &mut self,
        content: String,
        destination_id: u16,
    ) -> Result<u16, Box<dyn Error>> {
        let key = Message::generate_key();
        let message = Message::new_text(key, destination_id, content, None, None);
        self.send(message.serialize().await).await?;
        Ok(key)
    }

    async fn send(&mut self, message_bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut stream = self.stream.write().await;
        stream.write_all(&message_bytes).await?;
        Ok(())
    }

    async fn listen(&mut self) -> Result<Message, Box<dyn Error>> {
        let mut stream = self.stream.write().await;
        let message = tcp::receive(&mut stream).await?;
        Ok(message)
    }

    fn create_command(&self, input: String) -> (MessageType, u16, String) {
        let input = input.trim();
        let mut command = input[0..3].to_string();
        let mut has_dest = false;
        let message_type = match command.as_str() {
            "msg" => {
                command = input[4..].trim().to_string();
                has_dest = true;
                MessageType::Text
            }
            _ => MessageType::Error,
        };
        let mut dest_index = 0;
        let mut destination_id = 0;
        if !has_dest {
            dest_index = command.find(' ').unwrap();
            destination_id = command[0..dest_index].parse::<u16>().unwrap();
            dest_index += 1;
        }
        let content = command[dest_index..].to_string();
        (message_type, destination_id, content)
    }
}

impl TcpClient {
    pub async fn new(name: String) -> Result<Self, Box<dyn Error>> {
        let addr = format!("{}:{}", HOST_ADDRESS, TCP_PORT);
        let mut stream = TcpStream::connect(addr).await?;
        let connection_request =
            Message::new_connection_request(Message::generate_key(), name.clone());
        stream
            .write_all(&connection_request.serialize().await)
            .await?;
        // let msg = loop {
        //     let msg = match tcp::receive(&mut stream).await {
        //         Ok(msg) => msg,
        //         Err(_) => {
        //             continue;
        //         }
        //     };
        //     if msg.metadata.message_type == MessageType::Connection {
        //         break msg;
        //     }
        // };
        // if msg.metadata.message_type != MessageType::Connection {
        //     return Err("Erro ao conectar".into());
        // }
        // let id = msg.metadata.receiver_id;
        let log = Arc::new(RwLock::new(String::new()));
        let stream = Arc::new(RwLock::new(stream));
        println!("Conectado com sucesso!\nID: {}", 0);
        Ok(TcpClient {
            name,
            log,
            id: 0,
            stream,
        })
    }
}
