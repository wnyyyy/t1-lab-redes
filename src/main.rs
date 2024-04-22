use std::env;
use std::io::stdin;

use t1_lab_redes::config::{HOST_ADDRESS, TCP_PORT};
use t1_lab_redes::network::{client::Client, server::Server};

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
                let mut message = String::new();
                stdin().read_line(&mut message).unwrap();
                println!("id");
                let mut id_str = String::new();
                stdin().read_line(&mut id_str).unwrap();
                let id = id_str.trim().parse().unwrap();
                client.send_text(message, id).await.unwrap();
                let response = client.receive().await.unwrap();
                println!("Mensagem recebida: {}", response);
            }
        }
        _ => {
            eprintln!("Usage: program [server|client]");
        }
    }
}
