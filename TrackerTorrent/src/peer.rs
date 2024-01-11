use crate::{errors::BitTrackerError, request::Request};
use chrono::{DateTime, Local};
/******************************************************************************************/
/*                                       PEER                                             */
/******************************************************************************************/
#[derive(PartialEq, Debug, Clone)]
pub enum Event {
    Started,
    Stopped,
    Completed,
}
/// Estructura que modela a un peer.
#[derive(PartialEq, Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: String,
    pub event: Event,
    pub uploaded: String,
    pub downloaded: String,
    pub left: String,
    pub time_last_request: DateTime<Local>,
}

#[allow(dead_code)]
impl Peer {
    /// Inicializa un peer con los datos recibidos en la request.
    pub fn new(request: &Request) -> Result<Peer, BitTrackerError> {
        let id = request.peer_id.clone();
        let ip = request.ip.clone();
        let port = request.port.clone();
        let event = Self::match_event(&request.event)?;
        let uploaded = request.uploaded.clone();
        let downloaded = request.downloaded.clone();
        let left = request.left.clone();
        let time_last_request = request.time;

        Ok(Peer {
            id,
            ip,
            port,
            event,
            uploaded,
            downloaded,
            left,
            time_last_request,
        })
    }

    /// Actualiza la informacion si el peer envia otro announce.
    pub fn actualize_request(&mut self, request: &Request) -> Result<(), BitTrackerError> {
        if request.peer_id != self.id {
            return Err(BitTrackerError::InvalidSyntaxError);
        }
        self.event = Self::match_event(&request.event)?;
        self.uploaded = request.uploaded.clone();
        self.downloaded = request.downloaded.clone();
        self.left = request.left.clone();
        self.time_last_request = request.time;
        Ok(())
    }

    fn match_event(event: &str) -> Result<Event, BitTrackerError> {
        match event {
            "started" => Ok(Event::Started),
            "completed" => Ok(Event::Completed),
            "stopped" => Ok(Event::Stopped),
            _ => Err(BitTrackerError::InvalidPeerState),
        }
    }
}

#[cfg(test)]
mod peer_should {
    use super::*;

    #[test]
    fn initialize_peer() {
        let time = Local::now();
        let request = Request {
            info_hash_url: "%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g".to_string(),
            peer_id: "PEER-KBRON".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
            compact: 0.to_string(),
            event: "started".to_string(),
            uploaded: 0.to_string(),
            downloaded: 0.to_string(),
            left: 0.to_string(),
            time: time,
        };

        //println!("Request : {:?}",request);

        let peer = Peer::new(&request).unwrap();
        //println!("Peer : {:?}",peer);
        assert_eq!(peer.id, "PEER-KBRON".to_string());
        assert_eq!(peer.ip, "127.0.0.1".to_string());
        assert_eq!(peer.port, "12345".to_string());
        assert_eq!(peer.uploaded, "0".to_string());
        assert_eq!(peer.downloaded, "0".to_string());
        assert_eq!(peer.left, "0".to_string());
        assert_eq!(peer.event, Event::Started);
        assert_eq!(peer.time_last_request, time);
    }

    #[test]
    fn actualize_peer() {
        let time = Local::now();
        let request = Request {
            info_hash_url: "%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g".to_string(),
            peer_id: "PEER-KBRON".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
            compact: 0.to_string(),
            event: "started".to_string(),
            uploaded: 0.to_string(),
            downloaded: 0.to_string(),
            left: 100.to_string(),
            time: time,
        };
        let mut peer = Peer::new(&request).unwrap();

        let time = Local::now();
        let new_request = Request {
            info_hash_url: "%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g".to_string(),
            peer_id: "PEER-KBRON".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
            compact: 0.to_string(),
            event: "completed".to_string(),
            uploaded: 10.to_string(),
            downloaded: 100.to_string(),
            left: 0.to_string(),
            time: time,
        };

        peer.actualize_request(&new_request).unwrap();
        assert_eq!(peer.id, "PEER-KBRON".to_string());
        assert_eq!(peer.ip, "127.0.0.1".to_string());
        assert_eq!(peer.port, "12345".to_string());
        assert_eq!(peer.uploaded, "10".to_string());
        assert_eq!(peer.downloaded, "100".to_string());
        assert_eq!(peer.left, "0".to_string());
        assert_eq!(peer.event, Event::Completed);
        assert_eq!(peer.time_last_request, time);
    }

    #[test]
    fn cant_actualize_peer_with_different_id() {
        let time = Local::now();
        let request = Request {
            info_hash_url: "%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g".to_string(),
            peer_id: "PEER-KBRON".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
            compact: 0.to_string(),
            event: "started".to_string(),
            uploaded: 0.to_string(),
            downloaded: 0.to_string(),
            left: 100.to_string(),
            time: time,
        };
        let mut peer = Peer::new(&request).unwrap();

        let time = Local::now();
        let new_request = Request {
            info_hash_url: "%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g".to_string(),
            peer_id: "PEER-No-Tan-KBRON".to_string(),
            ip: "127.0.0.1".to_string(),
            port: "12345".to_string(),
            compact: 0.to_string(),
            event: "completed".to_string(),
            uploaded: 10.to_string(),
            downloaded: 100.to_string(),
            left: 0.to_string(),
            time: time,
        };

        if let Err(_) = peer.actualize_request(&new_request) {
            assert!(true);
        }
    }

    #[test]
    fn match_event() {
        assert_eq!(Event::Started, Peer::match_event("started").unwrap());
        assert_eq!(Event::Stopped, Peer::match_event("stopped").unwrap());
        assert_eq!(Event::Completed, Peer::match_event("completed").unwrap());
        if let Err(_) = Peer::match_event("basado") {
            assert!(true);
        }
    }
}
