use std::collections::HashMap;
use std::sync::Arc;

use bimap::BiMap;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{Mutex, RwLock};
use tokio::task;

use crate::config::{HOST_ADDRESS, TCP_PORT, UDP_PORT};
use crate::models::message::Message;
use crate::network::{tcp, udp};
use crate::utilities::enums::MessageType;

#[derive(Debug, Clone)]
pub struct Server {
    pub tcp_clients: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    pub id_table: Arc<RwLock<BiMap<u16, String>>>,
    polling_table: Arc<RwLock<HashMap<u16, Vec<u16>>>>,
    pub name_table: Arc<RwLock<HashMap<u16, String>>>,
    pub log: Arc<RwLock<String>>,
    udp_id_map: Arc<RwLock<HashMap<u16, Vec<u16>>>>,
    udp_data_map: Arc<RwLock<HashMap<u16, Vec<Message>>>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            tcp_clients: Arc::new(RwLock::new(HashMap::new())),
            id_table: Arc::new(RwLock::new(BiMap::new())),
            polling_table: Arc::new(RwLock::new(HashMap::new())),
            name_table: Arc::new(RwLock::new(HashMap::new())),
            udp_id_map: Arc::new(RwLock::new(HashMap::new())),
            udp_data_map: Arc::new(RwLock::new(HashMap::new())),
            log: Arc::new(RwLock::new(String::new())),
        }
    }

    pub async fn start(&self) {
        let tcp_listener = TcpListener::bind(format!("{0}:{1}", HOST_ADDRESS, TCP_PORT))
            .await
            .unwrap();
        let udp_socket = UdpSocket::bind(format!("{0}:{1}", HOST_ADDRESS, UDP_PORT))
            .await
            .unwrap();

        self.log
            .write()
            .await
            .push_str("\nServidor executando TCP na porta 8080 e UDP porta 8081");

        let tcp_clients = self.tcp_clients.clone();
        let id_table = self.id_table.clone();
        let name_table = self.name_table.clone();
        let log = self.log.clone();

        let tcp_task = task::spawn(async move {
            Self::listen_tcp(
                log,
                tcp_listener,
                tcp_clients,
                id_table.clone(),
                name_table.clone(),
            )
            .await;
        });

        let id_table = self.id_table.clone();
        let udp_id_map = self.udp_id_map.clone();
        let udp_data_map = self.udp_data_map.clone();

        let udp_task = task::spawn(async move {
            Self::listen_udp(udp_socket, id_table.clone(), udp_id_map, udp_data_map).await;
        });

        let _ = tokio::join!(tcp_task, udp_task);
    }

    async fn listen_tcp(
        log: Arc<RwLock<String>>,
        listener: TcpListener,
        tcp_clients: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
        id_table: Arc<RwLock<BiMap<u16, String>>>,
        name_table: Arc<RwLock<HashMap<u16, String>>>,
    ) {
        loop {
            let log_clone = log.clone();
            let tcp_clients_clone = tcp_clients.clone();
            let name_table_clone = name_table.clone();
            let id_table_clone = id_table.clone();
            let (stream, addr) = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(e) => {
                    log_clone
                        .write()
                        .await
                        .push_str(&format!("\nFalha ao aceitar conexão: {0}", e));
                    continue;
                }
            };
            let addr = addr.to_string();
            let id = Self::assign_id(addr.clone(), id_table.clone()).await;
            {
                let mut tcp_clients_write = tcp_clients.write().await;
                tcp_clients_write.insert(addr.clone(), Arc::new(Mutex::new(stream)));
            }
            log_clone
                .write()
                .await
                .push_str(&format!("\nNova conexão TCP: {0} - ID {1}", addr, id));

            tokio::spawn(async move {
                loop {
                    let sender_id;
                    let messages = {
                        let client_stream = {
                            let tcp_clients_read = tcp_clients_clone.read().await;
                            tcp_clients_read.get(&addr).unwrap().clone()
                        };
                        let mut stream = client_stream.lock().await;
                        log_clone.write().await.push_str(&format!(
                            "\nRecebendo dados do Client: {} - ID {}, Nome: {}",
                            stream.peer_addr().unwrap(),
                            &id,
                            name_table_clone
                                .read()
                                .await
                                .get(&id)
                                .unwrap_or(&String::from("Sem nome"))
                        ));
                        let mut message = match tcp::receive(&mut stream).await {
                            Ok(msg) => msg,
                            Err(e) => {
                                eprintln!("{}", e);
                                return;
                            }
                        };
                        let sender_addr = stream.peer_addr().unwrap().to_string();
                        sender_id = *id_table_clone
                            .read()
                            .await
                            .get_by_right(&sender_addr)
                            .unwrap();
                        let mut id_table = id_table_clone.write().await;
                        let mut name_table = name_table_clone.write().await;
                        Self::process_message(
                            &mut message,
                            sender_id,
                            &mut id_table,
                            &mut name_table,
                            &log_clone,
                        )
                        .await
                    };

                    for (dest_id, message) in messages {
                        log_clone
                            .write()
                            .await
                            .push_str(&format!("\nEnviando mensagem para ID {0}", dest_id));
                        let dest_client_addr = {
                            let id_table_read = id_table_clone.read().await;
                            match id_table_read.get_by_left(&dest_id) {
                                Some(addr) => addr.clone(),
                                None => {
                                    log_clone
                                        .write()
                                        .await
                                        .push_str("\nFalha ao encontrar destinatário.");
                                    continue;
                                }
                            }
                        };
                        if dest_client_addr.starts_with("udp") {
                            continue;
                        } else {
                            let stream = {
                                let tcp_clients_read = tcp_clients_clone.read().await;
                                match tcp_clients_read.get(&dest_client_addr) {
                                    Some(client) => client.clone(),
                                    None => {
                                        log_clone
                                            .write()
                                            .await
                                            .push_str("\nFalha ao encontrar destinatário.");
                                        continue;
                                    }
                                }
                            };
                            let mut stream = stream.lock().await;
                            Self::send_tcp(&mut stream, message, log_clone.clone())
                                .await
                                .unwrap();
                        }
                    }
                }
            });
        }
    }

    async fn process_message(
        message: &mut Message,
        from: u16,
        id_table: &mut BiMap<u16, String>,
        name_table: &mut HashMap<u16, String>,
        log: &Arc<RwLock<String>>,
    ) -> Vec<(u16, Message)> {
        let mut messages = Vec::new();
        match message.metadata.message_type {
            MessageType::File | MessageType::Text => {
                let content = String::from_utf8_lossy(&message.content);
                let dest_message = Message::new_text(
                    message.metadata.key,
                    message.metadata.receiver_id,
                    content.to_string(),
                    None,
                    None,
                );
                let response_message = Message::new_generic_response(
                    message.metadata.key,
                    message.metadata.receiver_id,
                    true,
                );
                messages.push((message.metadata.receiver_id, dest_message));
                messages.push((from, response_message));
                log.write().await.push_str(&format!(
                    "\nMensagem de {0} para {1}:\n{2}\n",
                    from.to_string(),
                    message.metadata.receiver_id,
                    content
                ));
            }
            MessageType::Connection => {
                let client_name = String::from_utf8_lossy(&message.content).trim().to_string();
                let success = !name_table.values().any(|name| name == &client_name);
                if success {
                    name_table.insert(from, client_name.clone());
                }
                let response_message =
                    Message::new_generic_response(message.metadata.key, from, success);
                messages.push((from, response_message));
                log.write().await.push_str(&format!(
                    "\nClient ID {0} - Nome: {1}\nConectado\n",
                    from, client_name
                ));
            }
            MessageType::ListClients => {
                println!("Listando clientes conectados...");
                let mut clients = Vec::<(u16, String)>::new();
                for client in id_table.iter() {
                    let id = *client.0;
                    let name = match name_table.get(&id) {
                        Some(name) => name.clone(),
                        None => String::from("Sem nome"),
                    };
                    clients.push((id, name));
                }
                //Message::new_list_clients(message.metadata.sender_id, clients, udp_id)
            }
            MessageType::SetName => {
                let client_name = String::from_utf8_lossy(&message.content).trim().to_string();
                let success = !name_table.values().any(|name| name == &client_name);
                if success {
                    name_table.insert(from, client_name.clone());
                }
                log.write().await.push_str(&format!(
                    "\nClient ID {0} - Novo nome: {1}",
                    from, client_name
                ));
            }
            _ => {}
        }
        messages
    }

    async fn send_tcp(
        stream: &mut TcpStream,
        message: Message,
        log: Arc<RwLock<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message_bytes = message.serialize().await;
        stream.write_all(&message_bytes).await?;
        log.write().await.push_str(&format!(
            "\nMensagem enviada para Client: {0} - ID {1}",
            stream.peer_addr().unwrap(),
            message.metadata.receiver_id
        ));
        Ok(())
    }

    async fn listen_udp(
        socket: UdpSocket,
        id_table: Arc<RwLock<BiMap<u16, String>>>,
        udp_id_map: Arc<RwLock<HashMap<u16, Vec<u16>>>>,
        udp_data_map: Arc<RwLock<HashMap<u16, Vec<Message>>>>,
    ) {
        let mut buf = [0; 1024];
        loop {
            let (len, addr) = match socket.recv_from(&mut buf).await {
                Ok((len, addr)) => (len, addr),
                Err(e) => {
                    eprintln!("Falha ao receber dados: {}", e);
                    continue;
                }
            };
            let addr_str = addr.to_string();
            let id = {
                let id_table_read = id_table.read().await;
                match id_table_read.get_by_right(&addr_str) {
                    Some(&id) => id,
                    None => {
                        drop(id_table_read);
                        Self::assign_id(addr_str.clone(), id_table.clone()).await
                    }
                }
            };
            let mut udp_id_map_read = udp_id_map.write().await;
            let owned_ids = udp_id_map_read.entry(id).or_insert(Vec::new());
            let mut udp_data_map_write = udp_data_map.write().await;
            let current_packets = udp_data_map_write.entry(id).or_insert(Vec::new());
            udp::build_udp_message(buf[..len].to_vec(), owned_ids.clone(), current_packets).await;
            println!("Recebendo dados UDP de {0} - ID {1}", addr, id);
        }
    }

    async fn assign_id(addr: String, id_table: Arc<RwLock<BiMap<u16, String>>>) -> u16 {
        loop {
            let id = rand::random::<u16>();
            let mut id_table_write = id_table.write().await;
            if !id_table_write.contains_left(&id) {
                id_table_write.insert(id, addr.clone());
                return id;
            }
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
