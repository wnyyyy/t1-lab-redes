use std::error::Error;

pub trait Client {
    fn get_id(&self) -> u16;
    fn get_name(&self) -> String;
    async fn get_log(&self) -> String;
    async fn set_name(&mut self, name: String);
    async fn send_text(
        &mut self,
        content: String,
        destination_id: u16,
    ) -> Result<(), Box<dyn Error>>;
    async fn send(&mut self, message_bytes: Vec<u8>) -> Result<(), Box<dyn Error>>;
    fn listen(&mut self) -> Result<String, Box<dyn Error>>;
}
