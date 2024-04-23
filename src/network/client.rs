use std::error::Error;

use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::models::message::Message;

pub struct Client {
    stream: Option<TcpStream>,
}

impl Client {
    pub async fn connect_tcp(addr: String) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Client {
            stream: Some(stream),
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
