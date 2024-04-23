use std::convert::TryFrom;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    Connection = 0,
    Text = 1,
    File = 2,
    ListClients = 3,
    SetName = 4,
    Broadcast = 5,
    Disconnect = 6,
    Error = 7,
    Success = 8,
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
            0 => Ok(MessageType::Connection),
            1 => Ok(MessageType::Text),
            2 => Ok(MessageType::File),
            3 => Ok(MessageType::ListClients),
            4 => Ok(MessageType::SetName),
            5 => Ok(MessageType::Broadcast),
            6 => Ok(MessageType::Disconnect),
            7 => Ok(MessageType::Error),
            8 => Ok(MessageType::Success),
            _ => Err("Tipo de mensagem inválido".to_string()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    TCP,
    UDP,
}
