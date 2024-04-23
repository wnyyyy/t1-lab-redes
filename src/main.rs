use std::{env, io};
use std::collections::HashMap;
use std::io::{BufRead, stdin};
use std::sync::Arc;
use std::time::Duration;

use bimap::BiMap;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use tokio::io::AsyncBufReadExt;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::time;
use tui::{Frame, Terminal};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, ListItem};

use t1_lab_redes::network::client::Client;
use t1_lab_redes::network::server::Server;
use t1_lab_redes::network::tcp_client::TcpClient;
use t1_lab_redes::utilities::enums::MessageType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("server") => {
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen)?;
            let server = Server::new();
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;
            let id_table = server.id_table.clone();
            let name_table = server.name_table.clone();
            let logs = server.log.clone();
            tokio::spawn(async move {
                server.start().await;
            });
            let ui_result = draw_server_ui(&mut terminal, id_table, name_table, logs).await;
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen)?;

            ui_result
        }
        Some("client") => {
            let udp = if let Some(arg) = args.get(2) {
                arg == "udp"
            } else {
                false
            };
            let client = if udp {
                TcpClient::new("Client UDP".to_string()).await.unwrap()
            } else {
                TcpClient::new("Client TCP".to_string()).await.unwrap()
            };
            let client_arc = Arc::new(Mutex::new(client));
            let sender = {
                let client = Arc::clone(&client_arc);
                tokio::spawn(async move {
                    let mut input = String::new();
                    let mut stdin = stdin();
                    println!("\nComandos disponiveis:\nmsg <id destino> <conteúdo>\n\n");
                    while stdin.read_line(&mut input)? > 0 {
                        let mut client = client.lock().await;
                        let (message_type, destination_id, content) =
                            client.create_command(input.clone());
                        match message_type {
                            MessageType::SetName => {
                                client.set_name(content).await.unwrap();
                            }
                            MessageType::Text => {
                                println!("Enviando mensagem para {}\n", destination_id);
                                client.send_text(content, destination_id).await.unwrap();
                            }
                            _ => {}
                        }
                        input.clear();
                    }
                    Ok::<(), io::Error>(())
                })
            };

            let receiver = {
                let client = Arc::clone(&client_arc);
                tokio::spawn(async move {
                    loop {
                        let mut client = client.lock().await;
                        let message = client.listen().await.unwrap();
                        println!(
                            "\nMensagem Recebida de {}:\n{}\n",
                            message.metadata.receiver_id,
                            String::from_utf8(message.content).unwrap()
                        );
                    }
                })
            };

            let _ = tokio::try_join!(sender, receiver);
            Ok(())
        }
        _ => {
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen)?;
            Err(Box::try_from(io::Error::new(io::ErrorKind::Other, "Erro")).unwrap())
        }
    }
}

async fn draw_server_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    id_table: Arc<RwLock<BiMap<u16, String>>>,
    name_table: Arc<RwLock<HashMap<u16, String>>>,
    logs: Arc<RwLock<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let id_table = id_table.write().await;
        let name_table = name_table.write().await;
        let logs = logs.read().await;
        terminal.draw(|f| {
            render_clients(f, id_table, name_table, logs);
        })?;
        time::sleep(Duration::from_millis(500)).await;
    }
}

fn render_clients<B: Backend>(
    f: &mut Frame<B>,
    id_table: RwLockWriteGuard<BiMap<u16, String>>,
    name_table: RwLockWriteGuard<HashMap<u16, String>>,
    logs: RwLockReadGuard<String>,
) {
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new("\n"));
    for (id, addr) in id_table.iter() {
        let name = match name_table.get(id) {
            Some(name) => name,
            None => "Sem nome",
        };
        let item = ListItem::new(format!("Client ID:{0}\n\"{1}\" - {2}\n", id, name, addr));
        items.push(item);
    }
    let client_list =
        List::new(items).block(Block::default().title("Server").borders(Borders::ALL));

    let lines = logs.lines();
    let log_items: Vec<ListItem> = lines.map(|log| ListItem::new(log.to_string())).collect();
    let log_list = List::new(log_items).block(Block::default().title("Logs").borders(Borders::ALL));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());

    f.render_widget(client_list, chunks[0]);
    f.render_widget(log_list, chunks[1]);
}
