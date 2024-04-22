use std::collections::HashMap;
use std::sync::Arc;

use bimap::BiMap;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::net::unix::SocketAddr;
use tokio::sync::{Mutex, RwLock};
use tokio::task;

use crate::config::{HOST_ADDRESS, TCP_PORT, UDP_PORT};
use crate::network::tcp;

#[derive(Debug, Clone)]
pub struct Server {
    tcp_clients: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
    udp_clients: Arc<RwLock<HashMap<String, SocketAddr>>>,
    id_table: Arc<RwLock<BiMap<u16, String>>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            tcp_clients: Arc::new(RwLock::new(HashMap::new())),
            udp_clients: Arc::new(RwLock::new(HashMap::new())),
            id_table: Arc::new(RwLock::new(BiMap::new())),
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

        let udp_clients = self.udp_clients.clone();
        let tcp_clients = self.tcp_clients.clone();
        let id_table = self.id_table.clone();

        let tcp_task = task::spawn(async move {
            Self::listen_tcp(tcp_listener, tcp_clients, id_table).await;
        });

        let udp_task = task::spawn(async move {
            Self::listen_udp(udp_socket).await;
        });

        let _ = tokio::join!(tcp_task, udp_task);
    }

    async fn listen_tcp(
        listener: TcpListener,
        tcp_clients: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
        id_table: Arc<RwLock<BiMap<u16, String>>>,
    ) {
        loop {
            println!("Aguardando conexão TCP...");
            let (stream, addr) = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(e) => {
                    eprintln!("Falha ao aceitar conexão: {}", e);
                    continue;
                }
            };
            let addr = addr.to_string();
            let tcp_clients_clone = tcp_clients.clone();
            let id_table_clone = id_table.clone();
            let id = Self::assign_id(addr.clone(), id_table.clone()).await;
            {
                let mut tcp_clients_write = tcp_clients_clone.write().await;
                tcp_clients_write.insert(addr.clone(), Arc::new(Mutex::new(stream)));
            }
            println!("Nova conexão TCP: {0} - ID {1}", addr, 0);

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
                        message
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

    async fn listen_udp(socket: UdpSocket) {
        let mut buf = [0; 1024];
        loop {
            let (len, addr) = match socket.recv_from(&mut buf).await {
                Ok((len, addr)) => (len, addr),
                Err(e) => {
                    eprintln!("Falha ao receber dados: {}", e);
                    continue;
                }
            };

            println!("Recebendo dados de {}: {:?}", addr, &buf[..len]);
            if let Err(e) = socket.send_to(&buf[..len], &addr).await {
                eprintln!("Falha ao receber dados: {}", e);
            }
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
