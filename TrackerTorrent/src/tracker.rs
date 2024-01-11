use crate::connection::Connection;
use crate::encoder::bencode_parser::Bencode;
use crate::errors::BitTrackerError;
use crate::peer::Event;
use crate::peer::Peer;
use crate::request::Request;
use crate::response::Response;
use crate::stats::InfoPeer;
use crate::stats::Stats;
use crate::torrent::Torrent;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
/******************************************************************************************/
/*                                  BIT TRACKER                                           */
/******************************************************************************************/

const ADDRESS: &str = "127.0.0.1:8080";

/// Estructura que modela nuestro Tracker
/// Tiene un id, una lista de torrents que puede hostear y un sender para loguear.
#[derive(Debug)]
#[allow(dead_code)]
pub struct BitTracker {
    pub id: String,
    pub torrents: Vec<Torrent>,
    pub log: Sender<String>,
}

#[allow(unused_assignments)]
#[allow(dead_code)]
impl BitTracker {
    /// Inicializa el tracker, recibe un archivo de configuracion con los nombres de los torrents a hostear.
    pub fn new(config: &str, tx: Sender<String>) -> Result<Self, BitTrackerError> {
        let file = File::open(config).or(Err(BitTrackerError::OpenFileError))?;
        let reader = BufReader::new(file);
        let mut torrents: Vec<Torrent> = Vec::new();
        for line in reader.lines() {
            let line = line.or(Err(BitTrackerError::ReadFileError))?;
            let torrent = Torrent::new(&line)?;
            torrents.push(torrent);
        }
        let id = Self::generate_id();
        Ok(BitTracker {
            id,
            torrents,
            log: tx,
        })
    }

    /// Genera aleatoriamente el id de nuestro Tracker.
    pub fn generate_id() -> String {
        let id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        id
    }

    ///Calcula las estadisticas de nuestro Tracker.
    pub fn get_stats(&mut self) -> Result<Stats, BitTrackerError> {
        let mut cant_peers = 0;
        let mut cant_seeders = 0;
        let mut cant_torrents = 0;
        let mut list = vec![];
        for torrent in &self.torrents {
            if !torrent.peers.is_empty() {
                cant_torrents += 1;
            }
            for peer in &torrent.peers {
                let mut completed = false;
                cant_peers += 1;
                if let Event::Completed = peer.event {
                    completed = true;
                    cant_seeders += 1;
                }
                let info_peer = InfoPeer::new(
                    peer.id.clone(),
                    peer.time_last_request.clone().to_string(),
                    completed,
                    torrent.metainfo.info.name.clone(),
                );
                list.push(info_peer);
            }
        }

        Ok(Stats::new(cant_peers, cant_seeders, cant_torrents, list))
    }

    pub fn receive_request(&mut self, request: &Request) -> Result<bool, BitTrackerError> {
        let mut has_torrent = false;
        for torrent in &mut self.torrents {
            if torrent.info_hash_url == request.info_hash_url {
                has_torrent = true;
                let mut has_peer = false;
                for peer in &mut torrent.peers {
                    if peer.id == request.peer_id {
                        has_peer = true;
                        peer.actualize_request(request)?;
                        return Ok(has_torrent);
                    }
                }
                if !has_peer {
                    let peer = Peer::new(request)?;
                    torrent.peers.push(peer);
                }
            }
        }
        Ok(has_torrent)
    }

    pub fn make_response(&mut self, request: &Request) -> Result<Response, BitTrackerError> {
        let mut complete = 0;
        let mut incomplete = 0;
        let mut list = vec![];
        let id = self.id.clone();
        for torrent in &mut self.torrents {
            if torrent.info_hash_url == request.info_hash_url {
                for peer in &torrent.peers {
                    if request.peer_id != peer.id {
                        let dict = Self::make_dict(peer, request);
                        list.push(dict);
                        match peer.event {
                            Event::Completed => {
                                complete += 1;
                            }
                            _ => {
                                incomplete += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(Response::new(id, complete, incomplete, list))
    }

    fn make_dict(peer: &Peer, request: &Request) -> Bencode {
        let ip = ("ip".to_string(), Bencode::String(peer.ip.clone()));
        let port = ("port".to_string(), Bencode::String(peer.port.clone()));
        let mut dict = vec![ip, port];
        if request.compact == 0.to_string() {
            dict.push(("id".to_string(), Bencode::String(peer.id.clone())));
        }
        Bencode::Dictionary(dict)
    }

    pub fn start(config: &str, tx: Sender<String>) -> Result<(), BitTrackerError> {
        let tracker = BitTracker::new(config, tx)?;
        println!("[TRACKER] Inicializado correctamente");
        let mut connections = vec![];
        let listener = TcpListener::bind(ADDRESS).or(Err(BitTrackerError::FailToConnectError))?;
        tracker
            .log
            .send("- [INFO] Tracker inicializado correctamente!".to_string())
            .or(Err(BitTrackerError::WriteLogError))?;
        let mutex = Arc::new(Mutex::new(tracker));
        for (id, stream) in listener.incoming().into_iter().enumerate() {
            match stream {
                Ok(stream) => {
                    let clone = mutex.clone();
                    connections.push(thread::spawn(move || handle_connection(stream, id, clone)))
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

pub fn handle_connection(
    stream: TcpStream,
    id: usize,
    mutex: Arc<Mutex<BitTracker>>,
) -> Result<(), BitTrackerError> {
    println!("[TRACKER] Recibi una conexion le asigno id: {}", id);
    Connection::connect(id, stream, mutex)?;
    Ok(())
}

#[cfg(test)]
mod bittracker_should {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn initialize_tracker() {
        let path = "./config_file";
        let (tx, _rx) = mpsc::channel();
        let tracker = BitTracker::new(path, tx).unwrap();

        for torrent in tracker.torrents {
            println!();
            println!("Announce: {:?}", torrent.metainfo.announce);
            println!("Url Hash: {:?}", torrent.info_hash_url);
            println!("Hash: {:?}", torrent.metainfo.info_hash);
            println!("Name: {:?}", torrent.metainfo.info.name);
            println!("Num pieces: {:?}", torrent.metainfo.info.num_pieces);
            println!("Length: {:?}", torrent.metainfo.info.length);
            println!("Piece Length: {:?}", torrent.metainfo.info.piece_length);
        }
    }

    #[test]
    #[ignore]
    fn tracker() {
        let path = "./config_file";
        let (tx, _rx) = mpsc::channel();
        BitTracker::start(path, tx).unwrap();
    }
}
