use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    Text = 0,
    File = 1,
    ListClients = 2,
}
impl From<MessageType> for u8 {
    fn from(message_type: MessageType) -> Self {
        message_type as u8
    }
}
impl TryFrom<u8> for MessageType {
    type Error = String;

    fn try_from(value: u8) -> Result<MessageType, String> {
        match value {
            0 => Ok(MessageType::Text),
            1 => Ok(MessageType::File),
            2 => Ok(MessageType::ListClients),
            _ => Err("Tipo de mensagem inválido".to_string()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    TCP,
    UDP,
}
