use std::{env, io};
use std::collections::HashMap;
use std::io::stdin;
use std::sync::Arc;
use std::time::Duration;

use bimap::BiMap;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use tokio::sync::{RwLock, RwLockWriteGuard};
use tokio::time;
use tui::{Frame, Terminal};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, ListItem};

use t1_lab_redes::config::{HOST_ADDRESS, TCP_PORT};
use t1_lab_redes::network::client::Client;
use t1_lab_redes::network::server::Server;

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
            let msg = server.log.clone();
            tokio::spawn(async move {
                server.start().await;
            });
            let ui_result = draw_server_ui(&mut terminal, msg, id_table, name_table).await;
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen)?;

            ui_result
        }
        Some("client") => {
            let mut client = Client::connect_tcp(format!("{}:{}", HOST_ADDRESS, TCP_PORT))
                .await
                .unwrap();
            tokio::spawn(async move {
                loop {
                    let message = client.receive().await.unwrap();
                    println!("{}", message);
                }
            });
            print!("Mensagem");
            let mut content_str = String::new();
            stdin().read_line(&mut content_str).unwrap();
            let content = content_str.trim().to_string().as_bytes().to_vec();
            println!("id");
            let mut id_str = String::new();
            stdin().read_line(&mut id_str).unwrap();
            if let Some(arg) = args.get(2) {
                if arg == "udp" {
                    print!("Mensagem");
                    let mut content_str = String::new();
                    stdin().read_line(&mut content_str).unwrap();
                    let content = content_str.trim().to_string().as_bytes().to_vec();
                    println!("id");
                    let mut id_str = String::new();
                    stdin().read_line(&mut id_str).unwrap();
                }
            }
            Err(Box::try_from(io::Error::new(io::ErrorKind::Other, "Erro")).unwrap())
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
    msg: Arc<RwLock<String>>,
    id_table: Arc<RwLock<BiMap<u16, String>>>,
    name_table: Arc<RwLock<HashMap<u16, String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let id_table = id_table.write().await;
        let name_table = name_table.write().await;
        let msg = msg.read().await;
        terminal.draw(|f| {
            render_clients(f, msg.as_str(), id_table, name_table);
        })?;
        time::sleep(Duration::from_millis(500)).await;
    }
}

fn render_clients<B: Backend>(
    f: &mut Frame<B>,
    msg: &str,
    id_table: RwLockWriteGuard<BiMap<u16, String>>,
    name_table: RwLockWriteGuard<HashMap<u16, String>>,
) {
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(msg));
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    f.render_widget(client_list, chunks[0]);
}
