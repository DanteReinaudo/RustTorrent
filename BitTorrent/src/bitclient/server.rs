use crate::bitclient::client::BitClient;
use crate::bitclient::errors::ClientError;
use crate::peer_connection::errors::ConnectionError;
use crate::peer_protocol::handshake::Handshake;
use crate::peer_protocol::messages::{Message, MessageId};
use crate::peers::peer::Peer;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::TcpListener;
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
const HANDSHAKE_LEN: usize = 68;

const HOST: &str = "127.0.0.1";
const LEN: usize = 4;
use std::thread::JoinHandle;
type Result<T> = std::result::Result<T, ClientError>;
#[allow(dead_code)]
/// Estrucutura server encargada de escuchar las request de otros peers
pub struct Server {
    addr: String,
    client: Arc<Mutex<BitClient>>,
    connections: Vec<ServerConnection>,
    listener: TcpListener,
    log: Sender<String>,
}
#[allow(dead_code)]
impl Server {
    fn new(mutex: Arc<Mutex<BitClient>>) -> Result<Self> {
        let client = mutex.lock().or(Err(ClientError::MutexLockError))?;
        let port = client.port_to_peers.clone();
        let sender = client.log.clone();
        drop(client);

        let addr = HOST.to_owned() + ":" + &port;
        let listener =
            TcpListener::bind(addr.clone()).or(Err(ClientError::TpcBindAsServerError))?;
        println!("[SERVER] Estableci la conexion, listo para recibir pedidos!");
        sender
            .send("- [INFO] Server inicializado, listo para recibir pedidos!".to_string())
            .or(Err(ClientError::WriteLogError))?;

        Ok(Server {
            addr,
            client: mutex,
            log: sender,
            listener,
            connections: vec![],
        })
    }

    /// Envia el bitfield por la conexion
    fn send_bitfield(&mut self) -> Result<Vec<u8>> {
        let client = self.client.lock().or(Err(ClientError::MutexLockError))?;
        let bitfield = client.peer.bitfield.clone();
        drop(client);
        let mut bytes = Peer::bytes_from_bitmap(bitfield);
        let message =
            Message::send_bitfield(&mut bytes).or(Err(ClientError::InvalidMessageError))?;
        Ok(message)
    }

    /// Realiza el handshake con el otro peer, en caso de que no coincidan los info hash de ambos devuelve false
    fn attempt_handshake(&mut self, id: usize) -> Result<bool> {
        println!("[SERVER] Recibi una conexion, le asigno id :{}", id);
        self.log
            .send("- [INFO] Recibi una conexion desde el servidor".to_string())
            .or(Err(ClientError::WriteLogError))?;

        let client = self.client.lock().or(Err(ClientError::MutexLockError))?;
        let client_id = client.peer_id.clone();
        let info_hash = client.metainfo.info_hash.clone();
        drop(client);

        let handshake = Handshake::new(info_hash, client_id);
        let request = handshake.as_bytes();
        self.connections[id].stream.write_all(&request).or(Err(
            ClientError::WriteConnectionError(ConnectionError::WriteConnectionError),
        ))?;
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        self.connections[id].stream.read_exact(&mut buffer).or(Err(
            ClientError::ReadConnectionError(ConnectionError::ReadConnectionError),
        ))?;

        let handshake_response = Handshake::from_bytes(buffer.to_vec())
            .or(Err(ClientError::InvalidUTF8HandshakeError))?;
        if handshake_response.info_hash == handshake.info_hash {
            Ok(true)
        } else {
            Err(ClientError::BadPeerResponseError)
        }
    }

    /// Se comunica con el peer que acaba de conectarse y atiende sus pedidos.
    /// En caso de que el peer tenga ipv6 lo ignora, ya que no soporta ese tipo de conexion.
    pub fn attend_connection(&mut self, stream: TcpStream, id: usize) -> Result<()> {
        //println!("[SERVER] Estableci conexión nro {}", id);
        let peer_addr = stream.peer_addr().expect("Could not retrieve peer address");
        let ip = peer_addr.ip();
        match ip {
            IpAddr::V4(..) => {
                let peer_ip = peer_addr.ip().to_string();
                let peer_port = peer_addr.port().to_string();
                let peer_id = id.to_string();
                let peer = Peer::new(peer_id, peer_ip, peer_port);

                self.connections.push(ServerConnection { stream, peer, id });
                let valid = self.attempt_handshake(id)?;
                if !valid {
                    println!(
                        "[SERVER] El handshake con la conexion {} , fue invalido",
                        id
                    );
                    self.log
                        .send(
                            "- [INFO] El handshake con una conexion dsede el servidor fue invalido"
                                .to_string(),
                        )
                        .or(Err(ClientError::WriteLogError))?;
                    return Ok(());
                }
                println!("[SERVER] El handshake con la conexion {} , es valido", id);
                let bitfield = self.send_bitfield()?;
                self.write_messages(bitfield, id)?;
                println!("[SERVER] envie el bitfield a la conexion {}", id);

                let have = self.return_have()?;
                let message = Message::send_have(have as u32);
                self.write_messages(message, id)?;
                println!(
                    "[SERVER] envie have de la pieza {} a la conexion {}",
                    have, id
                );

                let mut done = false;
                while !done {
                    let messages =
                        self.read_stream(id)
                            .or(Err(ClientError::ReadConnectionError(
                                ConnectionError::ReadConnectionError,
                            )))?;
                    match self.handle_server_message(messages, id) {
                        Err(err) => {
                            return Err(err);
                        }
                        Ok(result) => {
                            done = result;
                        }
                    }
                }
                if let Err(err) = self.end(id) {
                    return Err(err);
                }
            }
            _ => println!("[SERVER] No soporto la ip del Peer nro {}", id),
        }
        Ok(())
    }

    /// Funcion encargada de leer desde la conexion y transformarlo en mensaje
    pub fn read_stream(&mut self, id: usize) -> Result<Message> {
        let len: usize = self.read_size(id)?;
        let mut byte_message: Vec<u8>;
        match len {
            0 => {
                byte_message = vec![];
            }
            _ => {
                //println!("[CONEXION {}] Por leer el mensaje",self.id);
                byte_message = vec![0; len];
                self.connections[id]
                    .stream
                    .read_exact(&mut byte_message)
                    .or(Err(ClientError::ReadConnectionError(
                        ConnectionError::ReadConnectionError,
                    )))?;
            }
        }
        let len = len.try_into().or(Err(ClientError::InvalidMessageError))?;
        let message = Message::new(len, byte_message).or(Err(ClientError::InvalidMessageError))?;
        Ok(message)
    }

    /// Lee el largo del mensaje, es decir los primeros 4 bytes
    fn read_size(&mut self, id: usize) -> Result<usize> {
        //println!("[CONEXION {}] Por leer el tamaño de del mensaje",self.id);
        let mut byte_len: [u8; LEN] = [0; LEN];
        self.connections[id]
            .stream
            .read_exact(&mut byte_len)
            .or(Err(ClientError::ReadConnectionError(
                ConnectionError::ReadConnectionError,
            )))?;
        let len: usize = u32::from_be_bytes(byte_len)
            .try_into()
            .or(Err(ClientError::InvalidMessageError))?;
        Ok(len)
    }

    /// Escribe un mensaje por la conexion
    pub fn write_messages(&mut self, bytes: Vec<u8>, id: usize) -> Result<()> {
        self.connections[id].stream.write_all(&bytes).or(Err(
            ClientError::WriteConnectionError(ConnectionError::WriteConnectionError),
        ))?;
        Ok(())
    }

    /// Busca en el bitfield de nuestro peer para saber que pieza tenemos y enviarla por el have.
    fn return_have(&mut self) -> Result<usize> {
        let client = self.client.lock().or(Err(ClientError::MutexLockError))?;
        let bitfield = client.peer.bitfield.clone();
        let mut have = 0;
        for (index, piece) in bitfield.into_iter().enumerate() {
            if piece {
                have = index;
                break;
            }
        }
        drop(client);
        Ok(have)
    }

    /// Funcion encargada de manejar los distintos mensajes y peticiones que puede recibir nuestro servidor.
    /// Matchea los mensajes por su id y en base a esto hace lo que debe.
    pub fn handle_server_message(&mut self, message: Message, id: usize) -> Result<bool> {
        match message.id {
            MessageId::KeepAlive => {}
            MessageId::Bitfield(bytes) => {
                println! {"[SERVER] Recibi un Bitfield de la conexion {}!",id};
                let client = self.client.lock().or(Err(ClientError::MutexLockError))?;
                let num_pieces = client.metainfo.info.num_pieces;
                drop(client);
                self.connections[id].peer.store_bitmap(bytes, num_pieces);
                let have = self.return_have()?;
                let message = Message::send_have(have as u32);
                self.write_messages(message, id)?;
            }
            MessageId::Unchoke => {
                println! {"[SERVER] Recibi un Unchoke de la conexion {}!",id};
                let choke = Message::send_choke();
                self.write_messages(choke, id)?;
                return Ok(true);
            }

            MessageId::Request(piece_index, begin, length) => {
                println! {"[SERVER] Recibi un Request de la conexion {}!",id};
                let mut client = self.client.lock().or(Err(ClientError::MutexLockError))?;
                let piece_length = client.metainfo.info.piece_length;
                let offset = piece_index * piece_length + begin;
                let mut block = client
                    .downloader
                    .upload(offset as u64, length as u64)
                    .or(Err(ClientError::UploadError))?;
                drop(client);
                let message = Message::send_piece(piece_index, begin, &mut block)
                    .or(Err(ClientError::InvalidMessageError))?;
                self.write_messages(message, id)?;
            }

            MessageId::Cancel(_piece_index, _begin, _length) => {
                println! {"[SERVER] Recibi un Cancel de la conexion {}!",id};
                let choke = Message::send_choke();
                self.write_messages(choke, id)?;
                return Ok(true);
            }
            MessageId::Choke => {
                println! {"[SERVER] Recibi un Choke de la conexion {}!",id};
                self.connections[id].peer.choked = true;
                return Ok(true);
            }
            MessageId::NotInterested => {
                println! {"[SERVER] Recibi un NotInerested de la conexion {}!",id};
                self.connections[id].peer.interested = true;
                let choke = Message::send_choke();
                self.write_messages(choke, id)?;
                return Ok(true);
            }
            MessageId::Interested => {
                println! {"[SERVER] Recibi un Interested de la conexion {}!",id};
                self.connections[id].peer.interested = false;
                let unchoke = Message::send_unchoke();
                self.write_messages(unchoke, id)?;
            }
            _ => {
                return Err(ClientError::InvalidMessageError);
            }
        }
        Ok(false)
    }

    /// Funcion para cerrar la conexion.
    pub fn end(&mut self, id: usize) -> Result<()> {
        println! {"[SERVER] Cerrando la conexion de {}!",id};
        self.connections[id]
            .stream
            .shutdown(Shutdown::Both)
            .or(Err(ClientError::FailToConnectError(
                ConnectionError::FailToConnectError,
            )))?;
        Ok(())
    }
}

#[allow(dead_code)]
pub struct ServerConnection {
    stream: TcpStream,
    peer: Peer,
    id: usize,
}

/// Funcion disparada desde un thread, establece un bind y escucha una a una las peticiones.
pub fn start(mutex: Arc<Mutex<BitClient>>) -> Result<JoinHandle<()>> {
    let mut server = Server::new(mutex)?;

    let listener = server.listener.try_clone().unwrap();
    let server_thread = thread::spawn(move || {
        for (id, stream) in listener.incoming().enumerate() {
            match stream {
                Ok(s) => {
                    let message = "- [INFO] Recibi una conexión desde el server, le asigno el id: "
                        .to_owned()
                        + &id.to_string();
                    if let Err(err) = server.log.send(message) {
                        println!("[SERVER] Error al intentar loguear: {:?}", err);
                    }
                    if let Err(err) = server.attend_connection(s, id) {
                        println!("[SERVER] Error en la conexion {} : {}", id, err);
                        continue;
                    }
                }
                Err(e) => {
                    println!("[SERVER] Error {:?}", e)
                }
            }
        }
    });
    Ok(server_thread)
}

#[cfg(test)]
mod server_should {
    use super::*;
    use crate::log::logger::Logger;
    use std::sync::mpsc;

    pub fn read_stream(stream: &mut TcpStream) -> Message {
        let len: usize = read_size(stream);
        let mut byte_message: Vec<u8>;
        match len {
            0 => {
                byte_message = vec![];
            }
            _ => {
                byte_message = vec![0; len];
                stream.read_exact(&mut byte_message).unwrap();
            }
        }
        let message = Message::new(len as u32, byte_message).unwrap();
        message
    }

    fn read_size(stream: &mut TcpStream) -> usize {
        //println!("[CONEXION {}] Por leer el tamaño de del mensaje",self.id);
        let mut byte_len: [u8; LEN] = [0; LEN];
        stream.read_exact(&mut byte_len).unwrap();
        let len: usize = u32::from_be_bytes(byte_len).try_into().unwrap();
        len
    }

    #[test]
    #[ignore]
    fn make_handshake() {
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/ubuntu-20.04.4-live-server-amd64.iso.torrent",
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let cloned_sender = tx.clone();
        client.log = cloned_sender;
        let port = client.port_to_peers.clone();
        let info_hash = client.metainfo.info_hash.clone();

        //Inicializo el Logger
        let _logger = Logger::new(
            &client.log_path.clone(),
            &client.metainfo.info.name.clone(),
            rx,
        )
        .unwrap();

        let _response = client.announce_to_tracker().unwrap();
        let mutex = Arc::new(Mutex::new(client));
        let _server = start(mutex).unwrap();

        //Me conecto
        let addr = HOST.to_string() + ":" + &port;
        let mut stream = TcpStream::connect(addr).unwrap();

        let id = BitClient::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();

        let handshake_response = Handshake::from_bytes(buffer.to_vec()).unwrap();

        assert_eq!(handshake_response.info_hash, handshake.info_hash);
    }

    #[test]
    #[ignore]
    fn make_invalid_handshake() {
        //Inicializo el Cliente
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/ubuntu-20.04.4-live-server-amd64.iso.torrent",
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let cloned_sender = tx.clone();
        client.log = cloned_sender;
        let port = client.port_to_peers.clone();
        let mut info_hash = client.metainfo.info_hash.clone();
        info_hash[0] = 123;
        //Inicializo el Logger
        let _logger = Logger::new(
            &client.log_path.clone(),
            &client.metainfo.info.name.clone(),
            rx,
        )
        .unwrap();

        let _response = client.announce_to_tracker().unwrap();
        let mutex = Arc::new(Mutex::new(client));
        let _server = start(mutex).unwrap();

        //Me conecto
        let addr = HOST.to_string() + ":" + &port;
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = BitClient::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();

        let handshake_response = Handshake::from_bytes(buffer.to_vec()).unwrap();

        assert!(handshake_response.info_hash != handshake.info_hash);
    }

    #[test]
    #[ignore]
    fn send_bitfield() {
        //Inicializo el Cliente
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/ubuntu-20.04.4-live-server-amd64.iso.torrent",
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let cloned_sender = tx.clone();
        client.log = cloned_sender;
        let num_pieces = client.metainfo.info.num_pieces;
        let port = client.port_to_peers.clone();
        let info_hash = client.metainfo.info_hash.clone();

        //Inicializo el Logger
        let _logger = Logger::new(
            &client.log_path.clone(),
            &client.metainfo.info.name.clone(),
            rx,
        )
        .unwrap();

        let _response = client.announce_to_tracker().unwrap();
        let mutex = Arc::new(Mutex::new(client));
        let _server = start(mutex).unwrap();

        //Me conecto
        let addr = HOST.to_string() + ":" + &port;
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = BitClient::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();

        println!("[Cliente] Recibi el bitfield");
        let message = read_stream(&mut stream);
        let bitmap_len: usize = ((num_pieces as f32) / 8.0).ceil() as usize;
        assert_eq!(message.id, MessageId::Bitfield(vec![0; bitmap_len]));
    }

    #[test]
    #[ignore]
    fn make_full_protocol() {
        //Inicializo el Cliente
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/ubuntu-20.04.4-live-server-amd64.iso.torrent",
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let cloned_sender = tx.clone();
        client.log = cloned_sender;
        let num_pieces = client.metainfo.info.num_pieces;
        let port = client.port_to_peers.clone();
        let info_hash = client.metainfo.info_hash.clone();
        client.peer.bitfield[5] = true;

        //Inicializo el Logger
        let _logger = Logger::new(
            &client.log_path.clone(),
            &client.metainfo.info.name.clone(),
            rx,
        )
        .unwrap();

        let _response = client.announce_to_tracker().unwrap();
        let mutex = Arc::new(Mutex::new(client));
        let _server = start(mutex).unwrap();

        //Me conecto
        let addr = HOST.to_string() + ":" + &port;
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = BitClient::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();

        let id = 0;
        let message = read_stream(&mut stream);
        let bitmap_len: usize = ((num_pieces as f32) / 8.0).ceil() as usize;
        let mut vec = vec![0; bitmap_len];
        vec[0] = 32;
        assert_eq!(message.id, MessageId::Bitfield(vec));
        println!("[Conexion {}] Recibi un bitfield", id);

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Have(5));
        println!("[Conexion {}] Recibi un have", id);

        println!("[Conexion {}] Envio interested", id);
        let message = Message::send_interested();
        stream.write_all(&message).unwrap();

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Unchoke);
        println!("[Conexion {}] Recibi un Unchoke", id);

        println!("[Conexion {}] Por pedir una pieza", id);
        let message = Message::send_request(5, 0, 25);
        stream.write_all(&message).unwrap();

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Piece(5, 0, vec![0; 25]));
        println!("[Conexion {}] Recibi una pieza", id);

        println!("[Conexion {}] Por enviar choke", id);
        let message = Message::send_choke();
        stream.write_all(&message).unwrap();
    }

    #[test]
    #[ignore]
    fn make_full_protocol_twice() {
        //Inicializo el Cliente
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/ubuntu-20.04.4-live-server-amd64.iso.torrent",
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let cloned_sender = tx.clone();
        client.log = cloned_sender;
        let num_pieces = client.metainfo.info.num_pieces;
        let port = client.port_to_peers.clone();
        let info_hash = client.metainfo.info_hash.clone();
        client.peer.bitfield[5] = true;

        //Inicializo el Logger
        let _logger = Logger::new(
            &client.log_path.clone(),
            &client.metainfo.info.name.clone(),
            rx,
        )
        .unwrap();

        let _response = client.announce_to_tracker().unwrap();
        let mutex = Arc::new(Mutex::new(client));
        let _server = start(mutex).unwrap();

        //Me conecto
        let addr = HOST.to_string() + ":" + &port;
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = BitClient::generate_id();
        let handshake = Handshake::new(info_hash.clone(), id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();

        let id = 0;
        let message = read_stream(&mut stream);
        let bitmap_len: usize = ((num_pieces as f32) / 8.0).ceil() as usize;
        let mut vec = vec![0; bitmap_len];
        vec[0] = 32;
        assert_eq!(message.id, MessageId::Bitfield(vec));
        println!("[Conexion {}] Recibi un bitfield", id);

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Have(5));
        println!("[Conexion {}] Recibi un have", id);

        println!("[Conexion {}] Envio not interested", id);
        let message = Message::send_not_interested();
        stream.write_all(&message).unwrap();

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Choke);
        println!("[Conexion {}] Recibi un Choke", id);

        //Me conecto de nuevo
        let addr = HOST.to_string() + ":" + &port;
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = BitClient::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();

        let id = 0;
        let message = read_stream(&mut stream);
        let bitmap_len: usize = ((num_pieces as f32) / 8.0).ceil() as usize;
        let mut vec = vec![0; bitmap_len];
        vec[0] = 32;
        assert_eq!(message.id, MessageId::Bitfield(vec));
        println!("[Conexion {}] Recibi un bitfield", id);

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Have(5));
        println!("[Conexion {}] Recibi un have", id);

        println!("[Conexion {}] Envio cancel", id);
        let message = Message::send_cancel(5, 10, 25);
        stream.write_all(&message).unwrap();

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Choke);
        println!("[Conexion {}] Recibi un Choke", id);
    }

    #[test]
    #[ignore]
    fn ignore_invalid_messages() {
        //Inicializo el Cliente
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/ubuntu-20.04.4-live-server-amd64.iso.torrent",
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let cloned_sender = tx.clone();
        client.log = cloned_sender;
        let num_pieces = client.metainfo.info.num_pieces;
        let port = client.port_to_peers.clone();
        let info_hash = client.metainfo.info_hash.clone();
        client.peer.bitfield[5] = true;

        //Inicializo el Logger
        let _logger = Logger::new(
            &client.log_path.clone(),
            &client.metainfo.info.name.clone(),
            rx,
        )
        .unwrap();

        let _response = client.announce_to_tracker().unwrap();
        let mutex = Arc::new(Mutex::new(client));
        let _server = start(mutex).unwrap();

        //Me conecto
        let addr = HOST.to_string() + ":" + &port;
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = BitClient::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();

        let id = 0;
        let message = read_stream(&mut stream);
        let bitmap_len: usize = ((num_pieces as f32) / 8.0).ceil() as usize;
        let mut vec = vec![0; bitmap_len];
        vec[0] = 32;
        assert_eq!(message.id, MessageId::Bitfield(vec));
        println!("[Conexion {}] Recibi un bitfield", id);

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Have(5));
        println!("[Conexion {}] Recibi un have", id);

        println!("[Conexion {}] Envio piece o.o", id);
        let mut vec = vec![4, 2, 0];
        let message = Message::send_piece(5, 10, &mut vec).unwrap();
        stream.write_all(&message).unwrap();

        _server.join().unwrap();
    }
}
