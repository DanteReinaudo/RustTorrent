use super::errors::TrackerError;
use crate::encoder::bencode_parser::{Bencode, DecodingParser};
use crate::peers::peer::Peer;

/******************************************************************************************/
/*                               TRACKER RESPONSE                                         */
/******************************************************************************************/

#[derive(PartialEq, Debug, Clone)]
pub struct TrackerResponse {
    interval: String,
    complete: String,
    incomplete: String,
    pub peers: Vec<Peer>,
}

impl Default for TrackerResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackerResponse {
    pub fn new() -> TrackerResponse {
        TrackerResponse {
            interval: "".to_string(),
            complete: "".to_string(),
            incomplete: "".to_string(),
            peers: vec![],
        }
    }

    /// Esta funcion recibe la respuesta del tracker como un Vector de u8
    /// y  la devuelve como una TrackerResponse
    pub fn from(&mut self, vec: Vec<u8>) -> Result<TrackerResponse, TrackerError> {
        let dict = DecodingParser
            .decode_from_u8(vec)
            .or(Err(TrackerError::InvalidSyntaxError))?;
        let mut response = TrackerResponse::new();

        if let Bencode::Dictionary(dict) = dict {
            for (key, value) in dict {
                match value {
                    Bencode::Int(value) => {
                        if key == "interval" {
                            response.interval = value.clone().to_string()
                        }
                        if key == "complete" {
                            response.complete = value.clone().to_string()
                        }
                        if key == "incomplete" {
                            response.incomplete = value.clone().to_string()
                        }
                    }
                    Bencode::List(value) => {
                        response.peers = self.get_peers(value);
                    }
                    _ => {}
                }
            }
        }
        Ok(response)
    }

    /// Esta funcion recibe una lista de Bencodes e interpreta la misma,
    /// devolviendo los Peers que contiene
    fn get_peers(&mut self, list: Vec<Bencode>) -> Vec<Peer> {
        let mut id = "".to_string();
        let mut ip = "".to_string();
        let mut port = "".to_string();
        let mut peers = vec![];
        for i in list {
            if let Bencode::Dictionary(i) = i {
                for (key, value) in i {
                    match value {
                        Bencode::String(value) => {
                            if key == "peer id" {
                                id = value.clone()
                            }
                            if key == "ip" {
                                ip = value.clone()
                            }
                            if key == "port" {
                                port = value.clone()
                            }
                        }
                        Bencode::Int(value) => {
                            if key == "port" {
                                port = value.to_string()
                            }
                        }
                        _ => {}
                    }
                }
                let peer = Peer::new(id.clone(), ip.clone(), port.clone());
                peers.push(peer)
            }
        }
        peers
    }
}

#[cfg(test)]
mod tracker_response_should {
    use super::*;

    #[test]
    fn get_peers_info() {
        let mut list: Vec<Bencode> = vec![];
        let mut dict: Vec<(String, Bencode)> = vec![];
        dict.push((
            String::from("ip"),
            Bencode::String(String::from("91.189.95.21")),
        ));
        dict.push((
            String::from("peer id"),
            Bencode::String(String::from("T03I--00TiFSaYzPDIpT")),
        ));
        dict.push((String::from("port"), Bencode::Int(6891)));
        list.push(Bencode::Dictionary(dict));

        let mut tracker_response = TrackerResponse::new();

        let response = tracker_response.get_peers(list);
        assert_eq!(response[0].id, "T03I--00TiFSaYzPDIpT");
        assert_eq!(response[0].ip, "91.189.95.21");
        assert_eq!(response[0].port, "6891");
    }
}
