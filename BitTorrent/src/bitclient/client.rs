use crate::bitclient::errors::ClientError;
use crate::bitclient::server;
use crate::downloads::downloader::Downloader;
use crate::downloads::errors::DownloaderError;
use crate::log::logger::Logger;
use crate::peer_connection::connection::Connection;
use crate::peers::peer::Peer;
use crate::pieces::errors::PiecesError;
use crate::pieces::piece::Piece;
use crate::torrent_file::errors::MetaInfoError;
use crate::torrent_file::metainfo::MetaInfo;
use crate::tracker::errors::TrackerError;
use crate::tracker::tracker_request::TrackerRequest;
use crate::tracker::tracker_response::TrackerResponse;
use gtk4::glib::Sender as gtkSender;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/******************************************************************************************/
/*                                      BITCLIENT                                         */
/******************************************************************************************/

static BLOCK_SIZE: u32 = 16384; // 2^14

pub enum Event {
    UpdateName(String),
    UpdateInfoHash(String),
    UpdateNumPieces(usize),
    DownloadedPiece(),
    UpdatePeerList(Vec<Peer>),
    UpdateSpeed(f64),
    Unchoked(usize),
    Choked(usize),
}

/// Estructura BitClient, encargada de hacer de cliente en la descarga del torrent.
/// Se inicializa con un archivo de torrent y un archivo de configuracion.
#[allow(dead_code)]
#[derive(Debug)]
pub struct BitClient {
    pub metainfo: MetaInfo,
    pub downloader: Downloader,
    pub log_path: String,
    pub log: Sender<String>,
    pub event_bus: gtkSender<Event>,
    pub port_to_peers: String,
    pub peer_id: String,
    pub peer: Peer, //Representa al cliente como peer
    pub pieces: Vec<Piece>,
}

type Result<T> = std::result::Result<T, ClientError>;

#[allow(dead_code)]
impl BitClient {
    pub fn new(configuration_path: &str, torrent_path: &str) -> Result<BitClient> {
        let config_parameters = Self::read_configuration_file(configuration_path)?;
        let id: String = Self::generate_id();
        let (log, _rx) = mpsc::channel();
        let (null_sender, _null_receiver) =
            gtk4::glib::MainContext::channel(gtk4::glib::PRIORITY_DEFAULT);
        let metainfo = MetaInfo::new(torrent_path).or(Err(ClientError::DecodingError(
            MetaInfoError::DecodingError,
        )))?;
        let downloader = Downloader::new(
            &config_parameters[2],
            &metainfo.info.name,
            metainfo.info.length,
        )
        .or(Err(ClientError::CreateDownloaderError(
            DownloaderError::FileCreationError,
        )))?;

        let pieces = Self::generate_pieces(&metainfo);
        let mut peer = Peer::new(id.clone(), String::from(""), config_parameters[0].clone());
        peer.bitfield = vec![false; metainfo.info.num_pieces];
        let client: BitClient = BitClient {
            port_to_peers: config_parameters[0].clone(),
            log_path: config_parameters[1].clone(),
            log,
            downloader,
            peer_id: id,
            peer,
            metainfo,
            pieces,
            event_bus: null_sender,
        };
        Ok(client)
    }

    /// Inicializa el vector de piezas con el tamaño estimado en la metainfo del torrent
    fn generate_pieces(metainfo: &MetaInfo) -> Vec<Piece> {
        let piece_length = metainfo.info.piece_length;
        let num_pieces = metainfo.info.num_pieces as u32;
        let size = metainfo.info.length as u32;

        println!("La cantidad de piezas a descargar es {:?}", num_pieces);
        println!("EL tamaño del archivo es {:?}", metainfo.info.length);
        let mut pieces: Vec<Piece> = vec![];
        let n = metainfo.info.pieces.len();
        let last_piece_dont_fix = size % piece_length != 0;

        for i in 0..n {
            let length = {
                if i == n - 1 && last_piece_dont_fix {
                    size % piece_length
                } else {
                    piece_length
                }
            };

            let piece = Piece::new(
                length,
                i as u32,
                piece_length,
                metainfo.info.pieces[i as usize].clone(),
                BLOCK_SIZE,
            );
            pieces.push(piece);
        }

        pieces
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

    /// Lee el archivo de configuracion y almacena las variables en el cliente.
    fn read_configuration_file(path: &str) -> Result<Vec<String>> {
        let file = File::open(path).or(Err(ClientError::OpenFileError))?;
        let reader = BufReader::new(file);
        let mut lineas: Vec<String> = Vec::new();
        for line in reader.lines() {
            let line = line.or(Err(ClientError::ReadFileError))?;
            let split_line: Vec<&str> = line.split(':').collect();
            lineas.push(split_line[1].to_string());
        }
        Ok(lineas)
    }

    /// Funcion que se encarga de anunciarse al tracker
    pub fn announce_to_tracker(&mut self) -> Result<TrackerResponse> {
        let mut tracker_request = TrackerRequest::new(
            self.metainfo.info_hash.clone(),
            self.peer_id.clone(),
            self.port_to_peers.clone(),
            self.metainfo.announce.clone(),
        );
        let response = tracker_request
            .announce()
            .or(Err(ClientError::TrackerError(
                TrackerError::FailToConnectError,
            )))?;

        Ok(response)
    }

    /// Funcion que se encarga de almacenar la data de un bloque especifico de una pieza en el vector de piezas
    pub fn store(&mut self, piece_index: u32, block_index: u32, data: Vec<u8>) -> Result<bool> {
        self.pieces[piece_index as usize]
            .store(
                &mut self.downloader,
                block_index,
                data,
                self.log.clone(),
                self.event_bus.clone(),
            )
            .or(Err(ClientError::StorageError(
                PiecesError::DownloadingError,
            )))?;
        Ok(self.is_complete())
    }

    /// Verifica si se completo la descarga del torrent, es decir si todas las piezas estan completas
    fn is_complete(&self) -> bool {
        for piece in self.pieces.iter() {
            if !piece.is_complete {
                return false;
            }
        }
        println!("[CLIENTE] El torrent se ha descargado completamente");
        self.log
            .send("- [INFO] El torrent se ha descargado completamente".to_string())
            .expect("panic");
        true
    }

    /// Busca en el vector de piezas cual es el siguiente bloque que debe descargar
    pub fn next_block_to_request(&self, peer_bitfield: &[bool]) -> Option<(u32, u32, u32)> {
        for piece in self.pieces.iter() {
            if peer_bitfield[piece.index as usize] {
                if let Some(block) = piece.next_block_to_request() {
                    return Some((piece.index, block.index, block.length));
                }
            }
        }
        None
    }

    /// Funcion que se llama desde el main, se encarga de inicializar el cliente, comunicarse con el tracker.
    /// Dispara un thread para el logger, un thread para el servidor y uno para la conexion por cada peer.

    pub fn download_torrent(
        configuration_path: &str,
        torrent_path: &str,
        app_sender: gtkSender<Event>,
    ) -> Result<()> {
        //Inicializo el Cliente
        let mut client = BitClient::new(configuration_path, torrent_path)?;
        client.event_bus = app_sender;
        client
            .event_bus
            .send(Event::UpdateName(client.metainfo.info.name.clone()))
            .or(Err(ClientError::WriteLogError))?;
        let info_hash = hex::encode(&(*client.metainfo.info_hash));
        client
            .event_bus
            .send(Event::UpdateInfoHash(info_hash))
            .or(Err(ClientError::WriteLogError))?;
        client
            .event_bus
            .send(Event::UpdateNumPieces(client.metainfo.info.num_pieces))
            .or(Err(ClientError::WriteLogError))?;
        let (tx, rx) = mpsc::channel();
        let cloned_sender = tx.clone();
        client.log = cloned_sender;

        //Inicializo el Logger
        let mut logger = Logger::new(
            &client.log_path.clone(),
            &client.metainfo.info.name.clone(),
            rx,
        )
        .or(Err(ClientError::OpenFileError))?;

        let log = thread::spawn(move || logger.listening());
        client
            .log
            .send("- [INFO] Cliente inicializado correctamente!".to_string())
            .or(Err(ClientError::WriteLogError))?;

        //Me comunico con el tracker
        match client.announce_to_tracker() {
            Err(error) => {
                let message = "- [ERROR] ".to_owned() + &error.to_string();
                client
                    .log
                    .send(message)
                    .or(Err(ClientError::WriteLogError))?;
            }
            Ok(response) => {
                client
                    .log
                    .send("- [INFO] Se obtuvo una respuesta correcta del tracker".to_string())
                    .or(Err(ClientError::WriteLogError))?;

                client
                    .event_bus
                    .send(Event::UpdatePeerList(response.peers.clone()))
                    .or(Err(ClientError::WriteLogError))?;

                //Clono data relevante
                let mutex = Arc::new(Mutex::new(client));

                //Inicio el server
                let server = server::start(mutex.clone())?;

                //Disparo un thread por conexion
                let mut connections = vec![];
                println!(
                    "[CLIENTE] Recibi {} peers del tracker",
                    response.peers.len()
                );
                for (id, peer) in response.peers.into_iter().enumerate() {
                    //println!("[CLIENTE] Por conectarme al peer {}, Peer: {:?} ",id, peer);
                    let mutex_clone = mutex.clone();
                    connections.push(thread::spawn(move || {
                        Connection::connect(id, peer, mutex_clone)
                    }));
                }

                for (id, connection) in connections.into_iter().enumerate() {
                    let result = connection.join();
                    match result {
                        Ok(result_connection) => {
                            if let Err(err) = result_connection {
                                println!("[ERROR] Conexión {}: {}", id, err);
                                let message = "- [ERROR] Conexión ".to_owned()
                                    + &id.to_string()
                                    + " : "
                                    + &err.to_string();
                                tx.send(message).or(Err(ClientError::WriteLogError))?;
                                //return Err(ClientError::ReadConnectionError(err));

                                //id conexion fallo, ya no esta activa
                            }
                        }
                        Err(_) => return Err(ClientError::FailToJoinThreadError),
                    }
                }
                if let Err(err) = server.join() {
                    println!("[Error] Fallo al joinear el server: {:?}", err)
                }
            }
        }

        let result = log.join();
        match result {
            Ok(result_log) => {
                if let Err(_error) = result_log {
                    return Err(ClientError::WriteLogError);
                }
            }
            Err(_) => return Err(ClientError::FailToJoinThreadError),
        }

        Ok(())
    }

    pub fn mark_as_requested(&mut self, piece_index: u32, block_index: u32) {
        self.pieces[piece_index as usize].mark_as_requested(block_index);
    }
}

/******************************************************************************************/
/*                                        TESTS                                           */
/******************************************************************************************/

#[cfg(test)]
mod client_should {
    use super::*;
    use gtk4::glib;
    use gtk4::glib::MainContext;
    use gtk4::glib::Receiver as gtkReceiver;

    #[test]
    fn initialize_bit_torrent() {
        let client =
            BitClient::new("./config/configuration_file", "./torrents/sample.torrent").unwrap();

        assert_eq!(client.port_to_peers, "6881");
        assert_eq!(client.downloader.path, "./downloads/sample.txt");
        assert_eq!(
            client.metainfo.announce,
            "udp://tracker.openbittorrent.com:80"
        );
    }

    #[test]
    fn fail_if_wrong_config_file_path() {
        assert_eq!(
            BitClient::new("./config/wrong_path.txt", "./torrents/sample.torrent")
                .unwrap_err()
                .to_string(),
            "No se pudo abrir el archivo"
        );
    }

    #[test]
    fn fail_if_wrong_torrent_file_path() {
        let client = BitClient::new("./config/configuration_file", "./torrents/bad.torrent");

        assert_eq!(
            client.unwrap_err().to_string(),
            "No se pudo parsear el archivo"
        );
    }

    #[test]
    fn store_torrent_announce_and_info() {
        let client = BitClient::new(
            "./config/configuration_file",
            "./torrents/kubuntu-16.04.6-desktop-amd64.iso.torrent",
        )
        .unwrap();

        let info_hash = vec![
            69, 179, 214, 147, 207, 242, 133, 151, 95, 98, 42, 202, 235, 117, 197, 98, 106, 202,
            255, 111,
        ];
        assert_eq!(
            client.metainfo.announce,
            "http://torrent.ubuntu.com:6969/announce"
        );
        assert_eq!(client.metainfo.info_hash, info_hash);
        assert_eq!(client.metainfo.info.length, 1676083200);
        assert_eq!(
            client.metainfo.info.name,
            "kubuntu-16.04.6-desktop-amd64.iso"
        );
        assert_eq!(client.metainfo.info.piece_length, 524288);
        assert_eq!(client.metainfo.info.num_pieces, 3197);
    }

    #[test]
    fn announce_to_ubuntu_tracker() {
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/debian-11.3.0-amd64-netinst.iso.torrent",
        )
        .unwrap();

        let response = client.announce_to_tracker();

        match response {
            Ok(_v) => assert_eq!(_v.peers.is_empty(), false),
            Err(_e) => assert_eq!(_e.to_string(), "Fallo en la comunicacion con el tracker"),
        }
    }

    #[test]
    #[ignore]
    fn announce_to_our_tracker() {
        let mut client = BitClient::new(
            "./config/configuration_file",
            "./torrents/INFORME.pdf.torrent",
        )
        .unwrap();

        let response = client.announce_to_tracker();

        match response {
            Ok(_v) => assert_eq!(_v.peers.is_empty(), true),
            Err(_e) => assert_eq!(_e.to_string(), "Fallo en la comunicacion con el tracker"),
        }
    }

    #[test]
    #[ignore]
    fn download_from_multiple_threads() {
        let path = "./torrents/debian-11.3.0-amd64-netinst.iso.torrent";
        let (fake_sender, _receiver): (gtkSender<Event>, gtkReceiver<Event>) =
            MainContext::channel(glib::PRIORITY_DEFAULT);

        BitClient::download_torrent("./config/configuration_file", path, fake_sender).unwrap();
    }

    #[test]
    #[ignore]
    fn download_from_our_traker_multiple_threads() {
        let path = "./torrents/DIAPOS.pdf.torrent";
        let (fake_sender, _receiver): (gtkSender<Event>, gtkReceiver<Event>) =
            MainContext::channel(glib::PRIORITY_DEFAULT);

        BitClient::download_torrent("./config/configuration_file", path, fake_sender).unwrap();
    }
}
