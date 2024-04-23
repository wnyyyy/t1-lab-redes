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
    tcp_clients: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    id_table: Arc<RwLock<BiMap<u16, String>>>,
    name_table: Arc<RwLock<HashMap<u16, String>>>,
    udp_id_map: Arc<RwLock<HashMap<u16, Vec<u16>>>>,
    udp_data_map: Arc<RwLock<HashMap<u16, Vec<Message>>>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            tcp_clients: Arc::new(RwLock::new(HashMap::new())),
            id_table: Arc::new(RwLock::new(BiMap::new())),
            name_table: Arc::new(RwLock::new(HashMap::new())),
            udp_id_map: Arc::new(RwLock::new(HashMap::new())),
            udp_data_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self) {
        let tcp_listener = TcpListener::bind(format!("{0}:{1}", HOST_ADDRESS, TCP_PORT))
            .await
            .unwrap();
        let udp_socket = UdpSocket::bind(format!("{0}:{1}", HOST_ADDRESS, UDP_PORT))
            .await
            .unwrap();

        println!("Servidor executando TCP na porta 8080 e UDP porta 8081");

        let tcp_clients = self.tcp_clients.clone();
        let id_table = self.id_table.clone();
        let name_table = self.name_table.clone();

        let tcp_task = task::spawn(async move {
            Self::listen_tcp(
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
        listener: TcpListener,
        tcp_clients: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
        id_table: Arc<RwLock<BiMap<u16, String>>>,
        name_table: Arc<RwLock<HashMap<u16, String>>>,
    ) {
        loop {
            let tcp_clients_clone = tcp_clients.clone();
            let name_table_clone = name_table.clone();
            let id_table_clone = id_table.clone();
            println!("Aguardando conexão TCP...");
            let (stream, addr) = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(e) => {
                    eprintln!("Falha ao aceitar conexão: {}", e);
                    continue;
                }
            };
            let addr = addr.to_string();
            let id = Self::assign_id(addr.clone(), id_table.clone()).await;
            {
                let mut tcp_clients_write = tcp_clients.write().await;
                tcp_clients_write.insert(addr.clone(), Arc::new(Mutex::new(stream)));
            }
            println!("Nova conexão TCP: {0} - ID {1}", addr, id);

            tokio::spawn(async move {
                loop {
                    let message = {
                        let client_stream = {
                            let tcp_clients_read = tcp_clients_clone.read().await;
                            tcp_clients_read.get(&addr).unwrap().clone()
                        };
                        let mut stream = client_stream.lock().await;
                        println!(
                            "Recebendo dados do Client: {} - ID {}",
                            stream.peer_addr().unwrap(),
                            &id
                        );
                        let mut message = match tcp::receive(&mut stream).await {
                            Ok(msg) => msg,
                            Err(e) => {
                                eprintln!("{}", e);
                                return;
                            }
                        };
                        let sender_addr = stream.peer_addr().unwrap().to_string();
                        message.metadata.sender_id = *id_table_clone
                            .read()
                            .await
                            .get_by_right(&sender_addr)
                            .unwrap();
                        let mut id_table = id_table_clone.write().await;
                        let mut name_table = name_table_clone.write().await;
                        Self::process_message(&mut message, &mut id_table, &mut name_table, None)
                            .await
                    };

                    let dest_client_addr = {
                        let id_table_read = id_table_clone.read().await;
                        match id_table_read.get_by_left(&message.metadata.receiver_id) {
                            Some(addr) => addr.clone(),
                            None => {
                                eprintln!("Destinatário não conectado.");
                                continue;
                            }
                        }
                    };
                    let dest_client = {
                        let tcp_clients_read = tcp_clients_clone.read().await;
                        match tcp_clients_read.get(&dest_client_addr) {
                            Some(client) => client.clone(),
                            None => {
                                eprintln!("Falha ao comunicar com destinatário.");
                                continue;
                            }
                        }
                    };
                    if message.metadata.message_type == MessageType::Broadcast {
                        let tcp_clients_read = tcp_clients_clone.read().await;
                        for (client_addr, client_stream) in tcp_clients_read.iter() {
                            if client_addr == &addr {
                                continue;
                            }
                            let mut stream = client_stream.lock().await;
                            if let Err(e) = stream.write_all(&message.content).await {
                                eprintln!("Erro ao enviar mensagem: {}", e);
                                continue;
                            }
                            println!(
                                "Mensagem enviada para Client: {0} - ID {1}",
                                client_addr, message.metadata.receiver_id
                            );
                        }
                        continue;
                    }
                    let mut dest_client_stream = dest_client.lock().await;
                    if let Err(e) = dest_client_stream.write_all(&message.content).await {
                        eprintln!("Erro ao enviar mensagem: {}", e);
                        continue;
                    }

                    println!(
                        "Mensagem enviada para Client: {0} - ID {1}",
                        dest_client_addr, message.metadata.receiver_id
                    );
                }
            });
        }
    }

    async fn process_message(
        message: &mut Message,
        id_table: &mut BiMap<u16, String>,
        name_table: &mut HashMap<u16, String>,
        udp_id: Option<u16>,
    ) -> Message {
        match message.metadata.message_type {
            MessageType::File | MessageType::Text => {
                let content = String::from_utf8_lossy(&message.content);
                println!("Mensagem de dados: {}", content);
                message.clone()
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
                Message::new_list_clients(message.metadata.sender_id, clients, udp_id)
            }
            MessageType::SetName => {
                let client_id = message.metadata.sender_id;
                let client_name = String::from_utf8_lossy(&message.content).trim().to_string();
                let success = !name_table.values().any(|name| name == &client_name);
                if success {
                    name_table.insert(client_id, client_name.clone());
                }
                println!("Clientes ID {0} - Novo nome: {1}", client_id, client_name);
                let message = Message::new_set_name(client_id, success, udp_id);
                message.clone()
            }
            MessageType::Disconnect => {
                let client_id = message.metadata.sender_id;
                name_table.remove(&client_id);
                id_table.remove_by_left(&client_id);
                println!("Cliente ID {0} desconectado", client_id);
                message.clone()
            }
            MessageType::Broadcast => {
                let content = String::from_utf8_lossy(&message.content);
                println!("Mensagem de broadcast: {}", content);
                message.clone()
            }
        }
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
