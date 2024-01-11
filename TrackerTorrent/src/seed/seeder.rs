use crate::errors::BitTrackerError;
use crate::metainfo::MetaInfo;
use crate::peer_protocol::handshake::Handshake;
use crate::peer_protocol::messages::Message;
use crate::peer_protocol::messages::MessageId;
use crate::seed::uploader::Uploader;

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
//use std::fs::File;
//use std::io::BufReader;
//use std::env::args;
//use std::io::BufRead;

const HANDSHAKE_LEN: usize = 68;
const LEN: usize = 4;

#[allow(dead_code)]

/// Estrucutura server encargada de seedear
pub struct Seeder {
    pub uploader: Uploader,
    pub metainfo: MetaInfo,
    pub id: String,
}

#[allow(dead_code)]
impl Seeder {
    fn new(torrent_path: String, file_path: String, id: String) -> Result<Self, BitTrackerError> {
        let uploader = Uploader::new(&file_path)?;
        let metainfo = MetaInfo::new(&torrent_path)?;
        Ok(Seeder {
            uploader,
            metainfo,
            id,
        })
    }

    /// Genera aleatoriamente el id de nuestro peer.
    pub fn generate_id() -> String {
        let id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        id
    }

    pub fn start(
        torrent_path: String,
        file_path: String,
        id: String,
        addr: String,
    ) -> Result<(), BitTrackerError> {
        let seeder = Seeder::new(torrent_path, file_path, id)?;
        let listener = TcpListener::bind(addr).or(Err(BitTrackerError::FailToConnectError))?;
        println!("[SEEDER] Estableci la conexion, listo para recibir pedidos!");
        let mut connections = vec![];
        let mutex = Arc::new(Mutex::new(seeder));
        for (id, stream) in listener.incoming().into_iter().enumerate() {
            match stream {
                Ok(stream) => {
                    let clone = mutex.clone();
                    connections.push(thread::spawn(move || attend_connection(stream, clone, id)))
                }
                Err(error) => {
                    println!("[ERROR] {}", error);
                    continue;
                }
            }
        }
        for (index, connection) in connections.into_iter().enumerate() {
            match connection.join() {
                Ok(result_connection) => {
                    if let Err(error) = result_connection {
                        println!("[ERROR] Conexion {}: {}", index, error);
                    }
                }
                Err(_) => println!("[ERROR] Fallo el join de la conexion numero {}", index),
            }
        }

        Ok(())
    }
}

pub fn attend_connection(
    stream: TcpStream,
    seeder: Arc<Mutex<Seeder>>,
    id: usize,
) -> Result<(), BitTrackerError> {
    let mut connection = SeederConnection::new(stream, seeder, id);
    connection.attend_connection()?;
    Ok(())
}

#[allow(dead_code)]
pub struct SeederConnection {
    stream: TcpStream,
    seeder: Arc<Mutex<Seeder>>,
    id: usize,
}
#[allow(dead_code)]
impl SeederConnection {
    pub fn new(stream: TcpStream, seeder: Arc<Mutex<Seeder>>, id: usize) -> Self {
        SeederConnection { stream, seeder, id }
    }

    pub fn attend_connection(&mut self) -> Result<(), BitTrackerError> {
        let valid = self.attempt_handshake()?;
        if !valid {
            println!(
                "[SEEDER] El handshake con la conexion {} , fue invalido",
                self.id
            );
            return Ok(());
        }
        println!(
            "[SEEDER] El handshake con la conexion {} , es valido",
            self.id
        );

        let _bitfield = self.send_bitfield()?;
        //println!("Bitfield: {:?}",bitfield);
        //println!("Bitfield len: {}",bitfield.len());
        //self.write_messages(bitfield)?;
        println!("[SERVER] envie el bitfield a la conexion {}", self.id);

        let message = Message::send_have(0);
        //println!("Server : envie mensjae {:?}" , message);
        self.write_messages(message)?;
        println!(
            "[SEEDER] envie have de la pieza 0 a la conexion {}",
            self.id
        );

        let mut done = false;
        while !done {
            let messages = self
                .read_stream()
                .or(Err(BitTrackerError::ReadConnectionError))?;
            match self.handle_server_message(messages, self.id) {
                Err(err) => {
                    return Err(err);
                }
                Ok(result) => {
                    done = result;
                }
            }
        }
        if let Err(err) = self.end(self.id) {
            return Err(err);
        }
        Ok(())
    }

    // Realiza la operacion inversa, recibe un bitmap booleano y lo transforma en un formato binario comprimido.
    pub fn bytes_from_bitmap(bitmap: Vec<bool>) -> Vec<u8> {
        let mut init = 0;
        let mut finish = 8;
        let mut bitfield: Vec<u8> = vec![];
        while init < bitmap.len() {
            let byte = {
                if finish > bitmap.len() {
                    bitmap[init..].to_vec()
                } else {
                    bitmap[init..finish].to_vec()
                }
            };
            let mut num = 0;
            for (i, bit) in byte.into_iter().enumerate() {
                if bit {
                    num += Self::binary_value(i as u8);
                }
            }
            bitfield.push(num);
            init += 8;
            finish += 8;
        }

        bitfield
    }

    fn binary_value(offset: u8) -> u8 {
        match offset {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            4 => 16,
            5 => 32,
            6 => 64,
            7 => 128,
            _ => 0,
        }
    }

    /// Envia el bitfield por la conexion
    fn send_bitfield(&mut self) -> Result<Vec<u8>, BitTrackerError> {
        let seeder = self
            .seeder
            .lock()
            .or(Err(BitTrackerError::MutexLockError))?;
        let num_pieces = seeder.metainfo.info.num_pieces;
        drop(seeder);
        //let bitmap_len: usize = ((num_pieces as f32) / 8.0).ceil() as usize;
        //println!("Bitmap_len {}:",bitmap_len);
        let bitfield: Vec<bool> = vec![true; num_pieces];
        //println!("Bitfield {:?}:",bitfield);
        let mut bytes = Self::bytes_from_bitmap(bitfield);
        //println!("COmpress Bitfield {:?}:",bytes);
        let message =
            Message::send_bitfield(&mut bytes).or(Err(BitTrackerError::InvalidMessageError))?;
        Ok(message)
    }

    /// Realiza el handshake con el otro peer, en caso de que no coincidan los info hash de ambos devuelve false
    fn attempt_handshake(&mut self) -> Result<bool, BitTrackerError> {
        let seeder = self
            .seeder
            .lock()
            .or(Err(BitTrackerError::MutexLockError))?;
        let seeder_id = seeder.id.clone();
        let info_hash = seeder.metainfo.info_hash.clone();
        drop(seeder);

        let handshake = Handshake::new(info_hash, seeder_id);
        let request = handshake.as_bytes();
        self.stream
            .write_all(&request)
            .or(Err(BitTrackerError::WriteConnectionError))?;
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        self.stream
            .read_exact(&mut buffer)
            .or(Err(BitTrackerError::ReadConnectionError))?;

        let handshake_response = Handshake::from_bytes(buffer.to_vec())
            .or(Err(BitTrackerError::InvalidUTF8HandshakeError))?;
        if handshake_response.info_hash == handshake.info_hash {
            Ok(true)
        } else {
            Err(BitTrackerError::BadPeerResponseError)
        }
    }

    /// Funcion para cerrar la conexion.
    pub fn end(&mut self, id: usize) -> Result<(), BitTrackerError> {
        println! {"[SERVER] Cerrando la conexion de {}!",id};
        self.stream
            .shutdown(Shutdown::Both)
            .or(Err(BitTrackerError::FailToConnectError))?;
        Ok(())
    }

    /// Funcion encargada de leer desde la conexion y transformarlo en mensaje
    pub fn read_stream(&mut self) -> Result<Message, BitTrackerError> {
        let len: usize = self.read_size()?;
        let mut byte_message: Vec<u8>;
        match len {
            0 => {
                byte_message = vec![];
            }
            _ => {
                byte_message = vec![0; len];
                self.stream
                    .read_exact(&mut byte_message)
                    .or(Err(BitTrackerError::ReadConnectionError))?;
            }
        }
        let len = len
            .try_into()
            .or(Err(BitTrackerError::InvalidMessageError))?;
        let message =
            Message::new(len, byte_message).or(Err(BitTrackerError::InvalidMessageError))?;
        Ok(message)
    }

    /// Lee el largo del mensaje, es decir los primeros 4 bytes
    fn read_size(&mut self) -> Result<usize, BitTrackerError> {
        let mut byte_len: [u8; LEN] = [0; LEN];
        self.stream
            .read_exact(&mut byte_len)
            .or(Err(BitTrackerError::ReadConnectionError))?;
        let len: usize = u32::from_be_bytes(byte_len)
            .try_into()
            .or(Err(BitTrackerError::InvalidMessageError))?;
        Ok(len)
    }

    /// Escribe un mensaje por la conexion
    pub fn write_messages(&mut self, bytes: Vec<u8>) -> Result<(), BitTrackerError> {
        self.stream
            .write_all(&bytes)
            .or(Err(BitTrackerError::WriteConnectionError))?;
        Ok(())
    }

    /// Funcion encargada de manejar los distintos mensajes y peticiones que puede recibir nuestro servidor.
    /// Matchea los mensajes por su id y en base a esto hace lo que debe.
    pub fn handle_server_message(
        &mut self,
        message: Message,
        id: usize,
    ) -> Result<bool, BitTrackerError> {
        match message.id {
            MessageId::KeepAlive => {}
            MessageId::Bitfield(_bytes) => {
                println! {"[SERVER] Recibi un Bitfield de la conexion {}!",id};
                let message = Message::send_have(0);
                self.write_messages(message)?;
            }
            MessageId::Unchoke => {
                println! {"[SERVER] Recibi un Unchoke de la conexion {}!",id};
                let choke = Message::send_choke();
                self.write_messages(choke)?;
                return Ok(true);
            }

            MessageId::Request(piece_index, begin, length) => {
                println! {"[SERVER] Recibi un Request de la conexion {}!",id};
                let mut seeder = self
                    .seeder
                    .lock()
                    .or(Err(BitTrackerError::MutexLockError))?;
                let piece_length = seeder.metainfo.info.piece_length;
                let offset = piece_index * piece_length + begin;
                let mut block = seeder
                    .uploader
                    .upload(offset as u64, length as u64)
                    .or(Err(BitTrackerError::UploadError))?;
                drop(seeder);
                let message = Message::send_piece(piece_index, begin, &mut block)
                    .or(Err(BitTrackerError::InvalidMessageError))?;
                self.write_messages(message)?;
            }

            MessageId::Cancel(_piece_index, _begin, _length) => {
                println! {"[SERVER] Recibi un Cancel de la conexion {}!",id};
                let choke = Message::send_choke();
                self.write_messages(choke)?;
                return Ok(true);
            }
            MessageId::Choke => {
                println! {"[SERVER] Recibi un Choke de la conexion {}!",id};
                return Ok(true);
            }
            MessageId::NotInterested => {
                println! {"[SERVER] Recibi un NotInerested de la conexion {}!",id};
                let choke = Message::send_choke();
                self.write_messages(choke)?;
                return Ok(true);
            }
            MessageId::Interested => {
                println! {"[SERVER] Recibi un Interested de la conexion {}!",id};
                let unchoke = Message::send_unchoke();
                self.write_messages(unchoke)?;
            }
            _ => {
                return Err(BitTrackerError::InvalidMessageError);
            }
        }
        Ok(false)
    }
}

/*
fn main() {
    let args = args().collect::<Vec<String>>();
    if args.len() != 2 {
        println!("[ERROR] Cantidad de argumentos inválido");
        return;
    }
    let config_file = args[1].clone();
    if let Err(error) = start_seeder(&config_file){
        println!("{}",error);
    };

}
*/
/*
fn start_seeder(config_file: &str ) -> Result<(),BitTrackerError>{
    let file = File::open(config_file).or(Err(BitTrackerError::OpenFileError))?;
    let reader = BufReader::new(file);
    let mut lines= Vec::new();
    for line in reader.lines() {
        let line = line.or(Err(BitTrackerError::ReadFileError))?;
        lines.push(line);
    }
    let addr = HOST.to_string() + &lines[0];
    let file_path = lines[1].clone();
    let torrent_path = lines[2].clone();
    let id = Seeder::generate_id();
    Seeder::start(torrent_path, file_path, id,addr)
}
*/

#[cfg(test)]
mod seeder_should {
    use super::*;
    //use std::time::Duration;
    use std::thread;

    pub fn read_stream(stream: &mut TcpStream) -> Message {
        //println!("Por leer mensaje");
        let len: usize = read_size(stream);
        //println!("Lei tamaño del mensaje: {}",len);
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
        //println!("Lei mensaje: {:?}",message);
        message
    }

    fn read_size(stream: &mut TcpStream) -> usize {
        //println!("[CONEXION {}] Por leer el tamaño de del mensaje",self.id);
        let mut byte_len: [u8; LEN] = [0; LEN];
        stream.read_exact(&mut byte_len).unwrap();
        //println!("LEi byte len: {:?}",byte_len);
        let len: usize = u32::from_be_bytes(byte_len).try_into().unwrap();
        len
    }

    #[test]
    #[ignore]
    fn start_seeder() {
        let addr = "127.0.0.1:1234".to_string();
        let download_path = "./downloads/INFORME - BITTORRENT.pdf".to_string();
        let torrent_path = "./torrents/INFORME - BITTORRENT.pdf.torrent".to_string();
        let id = "-4R1010-D23T24S25F26".to_string();
        Seeder::start(torrent_path, download_path, id, addr).unwrap();
    }

    #[test]
    #[ignore]
    fn start_seeder_and_announce() {
        let download_path = "./downloads/INFORME - BITTORRENT.pdf".to_string();
        let torrent_path = "./torrents/INFORME - BITTORRENT.pdf.torrent".to_string();
        let id = Seeder::generate_id();
        let cloned = id.clone();
        let addr = "127.0.0.1:1234".to_string();
        let seed = thread::spawn(move || Seeder::start(torrent_path, download_path, cloned, addr));

        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let message = format!(
            "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id={}&port=1234&uploaded=0&downloaded=0&left=0&compact=1&event=completed&ip=127.0.0.1 HTTP/1.0\r\nHost: 127.0.0.2:8080\r\n\r\n",
            id,
        );

        stream.write_all(message.as_bytes()).unwrap();
        seed.join().unwrap().unwrap();
    }

    #[test]
    #[ignore]
    fn make_handshake() {
        let addr = "127.0.0.1:1234".to_string();
        let info_hash = vec![
            169, 179, 39, 99, 78, 114, 33, 126, 224, 26, 222, 142, 109, 119, 22, 16, 127, 255, 196,
            103,
        ];
        //Me conecto
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = Seeder::generate_id();
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
        let addr = "127.0.0.1:1234".to_string();
        let info_hash = vec![
            169, 10, 10, 99, 78, 114, 33, 126, 224, 26, 222, 142, 109, 119, 22, 16, 127, 255, 196,
            103,
        ];
        //Me conecto
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = Seeder::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();
        let handshake_response = Handshake::from_bytes(buffer.to_vec()).unwrap();
        assert_ne!(handshake_response.info_hash, handshake.info_hash);
    }

    #[test]
    #[ignore]
    fn make_full_protocol() {
        let addr = "127.0.0.1:1234".to_string();
        let info_hash = vec![
            169, 179, 39, 99, 78, 114, 33, 126, 224, 26, 222, 142, 109, 119, 22, 16, 127, 255, 196,
            103,
        ];
        //Me conecto
        let mut stream = TcpStream::connect(addr).unwrap();
        let id = Seeder::generate_id();
        let handshake = Handshake::new(info_hash, id);
        let request = handshake.as_bytes();
        stream.write_all(&request).unwrap();
        let mut buffer: [u8; HANDSHAKE_LEN] = [0; HANDSHAKE_LEN];
        stream.read_exact(&mut buffer).unwrap();
        let handshake_response = Handshake::from_bytes(buffer.to_vec()).unwrap();
        println!("Recibi response : {:?}", handshake_response);
        assert_eq!(handshake_response.info_hash, handshake.info_hash);

        let id = 0;
        println!("[Conexion {}] Recibi un handshake valido", id);

        //let message = read_stream(&mut stream);
        //let bitmap_len: usize = ((133 as f32) / 8.0).ceil() as usize;
        //let mut vec = vec![1; bitmap_len];
        //assert_eq!(message.id, MessageId::Bitfield(vec));
        println!("[Conexion {}] Recibi un bitfield", id);

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Have(0));
        println!("[Conexion {}] Recibi un have", id);

        println!("[Conexion {}] Envio interested", id);
        let message = Message::send_interested();
        stream.write_all(&message).unwrap();

        let message = read_stream(&mut stream);
        assert_eq!(message.id, MessageId::Unchoke);
        println!("[Conexion {}] Recibi un Unchoke", id);

        println!("[Conexion {}] Por pedir una pieza", id);
        let message = Message::send_request(0, 0, 10);
        stream.write_all(&message).unwrap();

        let message = read_stream(&mut stream);
        assert_eq!(
            message.id,
            MessageId::Piece(0, 0, vec![37, 80, 68, 70, 45, 49, 46, 52, 10, 37])
        );
        println!("[Conexion {}] Recibi una pieza", id);

        println!("[Conexion {}] Por enviar choke", id);
        let message = Message::send_choke();
        stream.write_all(&message).unwrap();
    }

    #[test]
    #[ignore]
    fn start_seeder_and_announce1() {
        let download_path = "./downloads/INFORME - BITTORRENT.pdf".to_string();
        let torrent_path = "./torrents/INFORME - BITTORRENT.pdf.torrent".to_string();
        let id = Seeder::generate_id();
        let cloned = id.clone();
        let addr = "127.0.0.1:1234".to_string();
        let seed = thread::spawn(move || Seeder::start(torrent_path, download_path, cloned, addr));

        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let message = format!(
            "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id={}&port=1234&uploaded=0&downloaded=0&left=0&compact=1&event=completed&ip=127.0.0.1 HTTP/1.0\r\nHost: 127.0.0.2:8080\r\n\r\n",
            id,
        );

        stream.write_all(message.as_bytes()).unwrap();
        seed.join().unwrap().unwrap();
    }

    #[test]
    #[ignore]
    fn start_seeder_and_announce2() {
        let download_path = "./downloads/INFORME - BITTORRENT.pdf".to_string();
        let torrent_path = "./torrents/INFORME - BITTORRENT.pdf.torrent".to_string();
        let id = Seeder::generate_id();
        let cloned = id.clone();
        let addr = "127.0.0.1:1235".to_string();
        let seed = thread::spawn(move || Seeder::start(torrent_path, download_path, cloned, addr));

        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let message = format!(
            "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id={}&port=1235&uploaded=0&downloaded=0&left=0&compact=1&event=completed&ip=127.0.0.1 HTTP/1.0\r\nHost: 127.0.0.2:8080\r\n\r\n",
            id,
        );

        stream.write_all(message.as_bytes()).unwrap();
        seed.join().unwrap().unwrap();
    }

    #[test]
    #[ignore]
    fn start_seeder_and_announce3() {
        let download_path =
            "./downloads/DIAPOS - Proyecto BitTorrent - 4Rustasticos.pdf".to_string();
        let torrent_path =
            "./torrents/DIAPOS - Proyecto BitTorrent - 4Rustasticos.pdf.torrent".to_string();
        let id = Seeder::generate_id();
        let cloned = id.clone();
        let addr = "127.0.0.1:1236".to_string();
        let seed = thread::spawn(move || Seeder::start(torrent_path, download_path, cloned, addr));

        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let message = format!(
            "GET /announce?infohash=%c3%14X%f3%04p%deY%d5%b1e%f8%87%0b%b9%14D%27%a0%9c&peer_id={}&port=1236&uploaded=0&downloaded=0&left=0&compact=1&event=completed&ip=127.0.0.1 HTTP/1.0\r\nHost: 127.0.0.2:8080\r\n\r\n",
            id,
        );

        stream.write_all(message.as_bytes()).unwrap();
        seed.join().unwrap().unwrap();
    }
}
