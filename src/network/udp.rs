use crate::models::message::Message;

pub async fn build_udp_message(bytes: Vec<u8>, owned_ids: Vec<u16>, current_packets: &mut Vec<Message> ) -> Option<Message> {
    let message = Message::new_udp_packet(bytes);
    if let Err(e) = message {
        eprintln!("Erro processando pacote UDP:\n{:?}", e);
        return None;
    }
    let message = message.unwrap();
    if owned_ids.contains(&message.metadata.udp_id.unwrap()) {
        eprintln!("Mensagem não pertence ao remetente. Ignorando...");
        return None;
    }
    if current_packets.iter().any(|x| x.metadata.timestamp == message.metadata.timestamp) {
        eprintln!("Pacote UDP já recebido. Ignorando...");
        return None;
    }
    current_packets.push(message.clone());
    let is_complete = message.is_complete();
    if !is_complete {
        eprint!("Pacote UDP incompleto. Aguardando mais pacotes...");
        return None;
    }
    println!("Pacote UDP completo!");
    let message = rebuild_message(current_packets, message.metadata.udp_id.unwrap());
    if let Some(message) = message {
        return Some(message);
    }
    println!("Erro ao reconstruir mensagem.");
    return None;

}

fn rebuild_message(current_packets: &mut Vec<Message>, udp_id: u16) -> Option<Message> {
    current_packets.sort_by(|a, b| a.metadata.timestamp.cmp(&b.metadata.timestamp));
    let metadata = current_packets[0].metadata.clone();
    let mut content = Vec::new();
    for packet in current_packets {
        content.extend(&packet.content);
    }
    let message = Message {
        metadata,
        content,
    };
    Some(message)
}