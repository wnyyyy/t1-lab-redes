use std::error::Error;

use crate::models::message::Message;
use crate::utilities::enums::MessageType;

pub trait Client {
    fn get_id(&self) -> u16;
    fn get_name(&self) -> String;
    async fn get_log(&self) -> String;
    async fn set_name(&mut self, name: String) -> Result<u16, Box<dyn Error>>;
    async fn send_text(
        &mut self,
        content: String,
        destination_id: u16,
    ) -> Result<u16, Box<dyn Error>>;
    async fn send_connection_request(&mut self, name: String) -> Result<(), Box<dyn Error>>;
    async fn send(&mut self, message_bytes: Vec<u8>) -> Result<(), Box<dyn Error>>;
    async fn listen(&mut self) -> Result<Message, Box<dyn Error>>;
    fn create_command(&self, command: String) -> (MessageType, u16, String);
}
