use crate::bitclient::client::{BitClient, Event};
use crate::peer_connection::errors::ConnectionError;
use crate::peer_protocol::handshake::Handshake;
use crate::peer_protocol::messages::{Message, MessageId};
use crate::peers::peer::Peer;
use gtk4::glib::Sender as gtkSender;
use std::fmt::Debug;
use std::io::Read;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

static BLOCK_SIZE: u32 = 16384; // 2^14
const HANDSHAKE_LEN: usize = 68;
const LEN: usize = 4;
/******************************************************************************************/
/*                                 CONNECTION                                             */
/******************************************************************************************/

type Result<T> = std::result::Result<T, ConnectionError>;

/// Estructura encargada de manejar la comunicacion entre el cliente y otro peer.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Connection {
    pub id: usize,
    pub peer: Peer,
    pub stream: TcpStream,
    pub client: Arc<Mutex<BitClient>>,
    pub log: Sender<String>,
    pub event_bus: gtkSender<Event>,
    pub num_pieces: usize,
    pub bitfield: bool,
}

#[allow(dead_code)]
impl Connection {
    /// Se inicializa con un id, el peer y una referencia mutable a un lock del cliente.
    /// Ademas se encarga de realizar el handshake entre estos.
    /// En caso de que el info hash no sea igual, cierra la conexion
    pub fn new(id: usize, peer: Peer, client: Arc<Mutex<BitClient>>) -> Result<Connection> {
        let lock = client.lock().or(Err(ConnectionError::MutexLockError))?;
        let client_id = lock.peer_id.clone();
        let info_hash = lock.metainfo.info_hash.clone();
        let num_pieces = lock.metainfo.info.num_pieces;
        let log = lock.log.clone();
        let event_bus = lock.event_bus.clone();
        drop(lock);

        let stream = Self::attempt_handshake(id, client_id, info_hash, &peer)?;
        println!("[CONEXION {}] Conexion establecida!", id);

        Ok(Connection {
            id,
            peer,
            stream,
            client,
            log,
            num_pieces,
            event_bus,
            bitfield: false,
        })
    }

    /// Se connecta al peer con un tcpstream.
    fn connect_to_peer(peer: &Peer) -> Result<TcpStream> {
        let message = peer.ip.clone() + ":" + &peer.port;
        let stream = TcpStream::connect(message).or(Err(ConnectionError::FailToConnectError))?;
        Ok(stream)
    }

    /// Realiza el handshake con el otro peer .
    fn attempt_handshake(
        _id: usize,
        client_id: String,
        info_hash: Vec<u8>,
        peer: &Peer,
    ) -> Result<TcpStream> {
        let handshake = Handshake::new(info_hash, client_id);
        let request = handshake.as_bytes();
        let mut stream = Self::connect_to_peer(peer)?;
        stream
            .write_all(&request)
            .or(Err(ConnectionError::WriteConnectionError))?;
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream
            .read_exact(&mut buffer)
            .or(Err(ConnectionError::ReadConnectionError))?;
        let handshake_response = Handshake::from_bytes(buffer.to_vec())
            .or(Err(ConnectionError::InvalidUTF8HandshakeError))?;
        if handshake_response.info_hash == handshake.info_hash {
            Ok(stream)
        } else {
            Err(ConnectionError::BadPeerResponseError)
        }
    }

    /// Lee desde la conexion y lo traduce en un mensaje.
    pub fn read_stream(&mut self) -> Result<Message> {
        let len: usize = self.read_size()?;
        let mut byte_message: Vec<u8>;
        match len {
            0 => {
                byte_message = vec![];
            }
            _ => {
                //println!("[CONEXION {}] Por leer el mensaje",self.id);
                byte_message = vec![0; len];
                self.stream
                    .read_exact(&mut byte_message)
                    .or(Err(ConnectionError::ReadConnectionError))?;
            }
        }
        let len = len.try_into().or(Err(ConnectionError::UsizeFromU32Error))?;
        let message =
            Message::new(len, byte_message).or(Err(ConnectionError::InvalidMessageError))?;
        Ok(message)
    }

    /// Lee el tamaño del mensaje desde la conexion.
    fn read_size(&mut self) -> Result<usize> {
        //println!("[CONEXION {}] Por leer el tamaño de del mensaje",self.id);
        let mut byte_len: [u8; LEN] = [0; LEN];
        self.stream
            .read_exact(&mut byte_len)
            .or(Err(ConnectionError::ReadConnectionError))?;
        let len: usize = u32::from_be_bytes(byte_len)
            .try_into()
            .or(Err(ConnectionError::UsizeFromU32Error))?;
        Ok(len)
    }

    /// Escribe en la conexion.
    pub fn write_messages(&mut self, bytes: Vec<u8>) -> Result<()> {
        self.stream
            .write_all(&bytes)
            .or(Err(ConnectionError::WriteConnectionError))?;
        Ok(())
    }

    /// Funcion llamada desde afuera cuando se dispara un thread.
    /// Inicializa la conexion y se queda leyendo.
    pub fn connect(id: usize, peer: Peer, client: Arc<Mutex<BitClient>>) -> Result<bool> {
        let mut connection = Connection::new(id, peer, client)?;
        let mut done = false;
        while !done {
            let messages = connection.read_stream()?;
            done = connection.handle_message(messages)?;
        }

        Ok(true)
    }

    /// Maneja los mensajes que recibe del cliente y le responde en base a nuestros intereses.
    fn handle_message(&mut self, message: Message) -> Result<bool> {
        match message.id {
            MessageId::KeepAlive => {}
            MessageId::Bitfield(bytes) => {
                self.handle_bitfield(bytes)?;
            }
            MessageId::Have(index) => {
                self.handle_have(index)?;
            }
            MessageId::Unchoke => {
                self.handle_unchoke()?;
            }

            MessageId::Piece(piece_index, offset, data) => {
                let completed = self.handle_piece(piece_index, offset, data)?;
                if completed {
                    self.event_bus
                        .send(Event::DownloadedPiece())
                        .or(Err(ConnectionError::StorageError))?;
                    return Ok(true);
                } else {
                    let _ = self.request_next_block()?;
                }
            }
            MessageId::Choke => {
                self.handle_choke()?;
            }
            _ => {
                return Err(ConnectionError::InvalidMessageError);
            }
        }
        Ok(false)
    }

    /// Le solicita al cliente que le diga cual es el proximo bloque que necesita.
    /// Dado que contamos con muchos threads los cuales estan descargando piezas para el mismo cliente, el pedido
    /// del proximo bloque de la pieza se hace tomando el lock del cliente. De esta manera se evita que el cliente
    /// le pida el mismo bloque de una misma pieza a varios peers.
    fn request_next_block(&mut self) -> Result<()> {
        let mut client = self
            .client
            .lock()
            .or(Err(ConnectionError::MutexLockError))?;
        let next_block = client.next_block_to_request(&self.peer.bitfield);
        match next_block {
            Some((piece_index, block_index, block_length)) => {
                let offset = block_index * BLOCK_SIZE;
                client.mark_as_requested(piece_index, block_index);
                println!(
                    "[CONEXION {}] Estoy pidiendo el bloque {} de la pieza {}",
                    self.id, block_index, piece_index
                );
                let message = Message::send_request(piece_index, offset, block_length);
                drop(client);
                self.write_messages(message)?;
            }
            None => {
                println!(
                    "[CONEXION {}] Descargamos todas las piezas posibles de este peer",
                    self.id
                );
            }
        }
        Ok(())
    }

    /// Maneja el mensaje en caso de recibir un bitfield.
    fn handle_bitfield(&mut self, bytes: Vec<u8>) -> Result<()> {
        println! {"[CONEXION {}] Bitfield!",self.id};
        self.bitfield = true;
        self.peer.store_bitmap(bytes, self.num_pieces);
        let message = Message::send_interested();
        self.write_messages(message)?;
        Ok(())
    }

    /// Maneja el mensaje en caso de recibirun have.
    fn handle_have(&mut self, index: u32) -> Result<()> {
        println! {"[CONEXION {}] Have piece: {}", self.id, index};
        if !self.bitfield {
            self.peer.bitfield = vec![true; self.num_pieces];
        }
        let mut client = self
            .client
            .lock()
            .or(Err(ConnectionError::MutexLockError))?;
        client.peer.bitfield[index as usize] = true;
        drop(client);
        let message = Message::send_interested();
        self.write_messages(message)?;
        Ok(())
    }

    /// Maneja el mensaje en caso de recibir un unchoke.
    fn handle_unchoke(&mut self) -> Result<()> {
        println! {"[CONEXION {}] Unchoke!",self.id};
        if !self.bitfield {
            self.peer.bitfield = vec![true; self.num_pieces];
        }
        let mut client = self
            .client
            .lock()
            .or(Err(ConnectionError::MutexLockError))?;
        client.peer.choked = false;
        client
            .event_bus
            .send(Event::Unchoked(self.id))
            .or(Err(ConnectionError::WriteConnectionError))?;
        drop(client);
        self.request_next_block()?;
        Ok(())
    }

    /// Maneja el mensaje en caso de recibir un piece.
    fn handle_piece(&self, piece_index: u32, offset: u32, data: Vec<u8>) -> Result<bool> {
        println!(
            "[CONEXION {}] Recibi una pieza: {}, offset: {}",
            self.id, piece_index, offset
        );
        let mut client = self
            .client
            .lock()
            .or(Err(ConnectionError::MutexLockError))?;
        let block_index = offset / BLOCK_SIZE;
        let completed = client
            .store(piece_index, block_index, data)
            .or(Err(ConnectionError::MutexLockError))?;
        drop(client);
        Ok(completed)
    }

    /// Maneja el mensaje en caso de recibir un choke.
    fn handle_choke(&self) -> Result<()> {
        println!("[CONEXION {}] Choked u.u", self.id);
        let mut client = self
            .client
            .lock()
            .or(Err(ConnectionError::MutexLockError))?;
        client.peer.choked = true;
        client
            .event_bus
            .send(Event::Choked(self.id))
            .or(Err(ConnectionError::WriteConnectionError))?;
        drop(client);
        Ok(())
    }

    /// Maneja el mensaje en caso de recibir un request.
    fn handle_request(&self) -> Result<()> {
        println!("[CONEXION {}] Recibi un request", self.id);
        let mut client = self
            .client
            .lock()
            .or(Err(ConnectionError::MutexLockError))?;
        client.peer.choked = true;
        drop(client);
        Ok(())
    }

    /// Maneja el mensaje en caso de recibir un have.
    fn return_have(&mut self) -> Result<usize> {
        let client = self
            .client
            .lock()
            .or(Err(ConnectionError::MutexLockError))?;
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

    /// Cierra la conexion
    pub fn end(&mut self) -> Result<()> {
        self.stream
            .shutdown(Shutdown::Both)
            .or(Err(ConnectionError::FailToConnectError))?;
        Ok(())
    }
}

#[cfg(test)]
mod connection_should {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    #[ignore] //hay que buscar los peers en el momento
    fn initialize_connection() {
        let client = BitClient::new(
            "./config/configuration_file",
            "./torrents/kubuntu-16.04.6-desktop-amd64.iso.torrent",
        )
        .unwrap();

        let client_mutex = Arc::new(Mutex::new(client));
        let peer = get_random_peer();
        let connection = Connection::new(1, peer.clone(), client_mutex).unwrap();

        assert_eq!(connection.id, 1);
        assert_eq!(connection.peer.id, peer.id);
        assert_eq!(connection.peer.ip, peer.ip);
        assert_eq!(connection.peer.port, peer.port);
    }

    #[test]
    fn fail_to_connection_with_bad_peer() {
        let client = BitClient::new(
            "./config/configuration_file",
            "./torrents/kubuntu-16.04.6-desktop-amd64.iso.torrent",
        )
        .unwrap();
        let client_mutex = Arc::new(Mutex::new(client));
        let id = String::from("peer malvado");
        let ip = String::from("maldito peer");
        let port = String::from("420420");
        let peer = Peer::new(id, ip, port);
        assert_eq!(
            Connection::new(1, peer.clone(), client_mutex,)
                .unwrap_err()
                .to_string(),
            "No se pudo establecer la conexion"
        );
    }

    #[ignore]
    #[test]
    fn attempt_handshake() {
        let client = BitClient::new(
            "./config/configuration_file",
            "./torrents/kubuntu-16.04.6-desktop-amd64.iso.torrent",
        )
        .unwrap();
        let client_mutex = Arc::new(Mutex::new(client));
        let peer = get_random_peer();
        let connection = Connection::new(1, peer.clone(), client_mutex);
        match connection {
            Ok(_v) => assert_eq!(true, true),
            Err(_e) => assert_eq!(false, true),
        }
    }

    fn get_random_peer() -> Peer {
        /*
        let peer1 = Peer::new(
            String::from("T03I--00TlLMMyMk1Zja"),
            String::from("91.189.95.21"),
            String::from("6884"),
        );

        let peer2 = Peer::new(
            String::from("-TR2940-e9w81dl6jzrv"),
            String::from("95.146.226.42"),
            String::from("5846"),
        );
        */

        let peer3 = Peer::new(
            String::from("-TR2930-4bnc63hb6jam"),
            String::from("82.64.150.155"),
            String::from("16881"),
        );

        let peer4 = Peer::new(
            String::from("-TR2940-t33f2d21lnyy"),
            String::from("167.248.7.31"),
            String::from("51413"),
        );
        /*
              let peer5 = Peer::new(
                  String::from("-TR3000-1vcdpilmmb5z"),
                  String::from("37.113.3.52"),
                  String::from("50083"),
              );
        */
        let vec = vec![peer3, peer4];
        let mut rng = thread_rng();
        let random: usize = rng.gen();
        let x: usize = random % 2;
        println!("Entro al peer : {}", x);
        return vec[x].clone();
    }
}
