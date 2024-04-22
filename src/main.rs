use std::env;
use std::io::stdin;

use chrono::DateTime;
use t1_lab_redes::config::{HOST_ADDRESS, TCP_PORT};
use t1_lab_redes::models::message::Message;
use t1_lab_redes::models::metadata::MsgMetadata;
use t1_lab_redes::network::{client::Client, server::Server};
use t1_lab_redes::utilities::enums::MessageType;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("server") => {
            let server = Server::new();
            server.start().await;
        }
        Some("client") => {
            let addr = format!("{}:{}", HOST_ADDRESS, TCP_PORT);
            let mut client = Client::connect(addr).await.unwrap();
            loop {
                println!("Mensagem");
                let mut content_str = String::new();
                stdin().read_line(&mut content_str).unwrap();
                let content = content_str.trim().to_string().as_bytes().to_vec();
                println!("id");
                let mut id_str = String::new();
                stdin().read_line(&mut id_str).unwrap();            
                let id = id_str.trim().parse().unwrap();
                println!("tipo");
                let mut tipo_str = String::new();
                stdin().read_line(&mut tipo_str).unwrap();
                let tipo = MessageType::try_from(tipo_str.trim().parse::<u8>().unwrap()).unwrap();
                let timestamp = chrono::Utc::now();
                let metadata = MsgMetadata::new(id, id, timestamp, tipo, content.len() as u64);
                let message = Message::new(metadata, content);
                client.send_raw(message).await.unwrap();
                let response = client.receive().await.unwrap();
                println!("Mensagem recebida: {}", response);
            }
        }
        _ => {
            eprintln!("Usage: program [server|client]");
        }
    }
}
