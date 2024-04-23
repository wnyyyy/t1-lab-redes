use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    Text = 0,
    File = 1,
    ListClients = 2,
    SetName = 3,
    Disconnect = 4,
    Broadcast = 5,
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
            3 => Ok(MessageType::SetName),
            4 => Ok(MessageType::Disconnect),
            5 => Ok(MessageType::Broadcast),
            _ => Err("Tipo de mensagem inválido".to_string()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    TCP,
    UDP,
}
