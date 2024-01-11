use crate::encoder::bencode_encoder::EncodingParser;
use crate::encoder::bencode_parser::Bencode;

/******************************************************************************************/
/*                                  TRACKER RESPONSE                                     */
/******************************************************************************************/

const INTERVAL: usize = 10;

/// Estructura que modela la response del tracker.
#[derive(PartialEq, Debug, Clone)]
pub struct Response {
    pub tracker_id: String,
    pub interval: usize,
    pub complete: usize,
    pub incomplete: usize,
    pub bencode_peers: Vec<Bencode>,
}

impl Response {
    /// Inicializa la response.
    pub fn new(
        tracker_id: String,
        complete: usize,
        incomplete: usize,
        bencode_peers: Vec<Bencode>,
    ) -> Self {
        Response {
            tracker_id,
            interval: INTERVAL,
            complete,
            incomplete,
            bencode_peers,
        }
    }

    /// Bencodea la response en el formato correspondiente.
    fn bencode(&mut self) -> String {
        let complete = ("complete".to_string(), Bencode::Int(self.complete as i64));
        let incomplete = (
            "incomplete".to_string(),
            Bencode::Int(self.incomplete as i64),
        );
        let interval = ("interval".to_string(), Bencode::Int(self.interval as i64));
        let mut dict = vec![complete, incomplete, interval];
        let list = Bencode::List(self.bencode_peers.clone());
        dict.push(("peers".to_string(), list));

        let encoded = EncodingParser::encode(&EncodingParser, Bencode::Dictionary(dict));
        String::from_utf8_lossy(&encoded).to_string()
    }

    /// Genera el mensaje para devolver por la conexion
    pub fn make_message(&mut self) -> String {
        let bencode = self.bencode();
        format!(
            "HTTP/1.1 200 OK \r\nHost: 127.0.0.1:8080\r\nContent-Length:{}\r\nContent-Type: text/plain\r\n\r\n{}",
            bencode.chars().count(),
            bencode)
    }
}

#[cfg(test)]
mod response_should {
    use super::*;
    use crate::encoder::bencode_parser::DecodingParser;

    #[test]
    fn initilize() {
        let id = ("id".to_string(), Bencode::String("PEER-BACAN".to_string()));
        let ip = ("ip".to_string(), Bencode::String("127.0.0.1".to_string()));
        let port = ("port".to_string(), Bencode::String("420".to_string()));
        let dict1 = Bencode::Dictionary([id, ip, port].to_vec());

        let id = ("id".to_string(), Bencode::String("PEER-PANA".to_string()));
        let ip = ("ip".to_string(), Bencode::String("127.0.0.2".to_string()));
        let port = ("port".to_string(), Bencode::String("440".to_string()));
        let dict2 = Bencode::Dictionary([id, ip, port].to_vec());

        let list = [dict1, dict2].to_vec();
        let tracker_id = "TRACKER-BACAN".to_string();
        let response = Response::new(tracker_id.clone(), 0, 2, list.clone());

        assert_eq!(response.tracker_id, tracker_id);
        assert_eq!(response.complete, 0);
        assert_eq!(response.incomplete, 2);
        assert_eq!(response.bencode_peers, list);
    }

    #[test]
    fn generate_message() {
        let id = ("id".to_string(), Bencode::String("PEER-BACAN".to_string()));
        let ip = ("ip".to_string(), Bencode::String("127.0.0.1".to_string()));
        let port = ("port".to_string(), Bencode::String("420".to_string()));
        let dict1 = Bencode::Dictionary([id, ip, port].to_vec());

        let id = ("id".to_string(), Bencode::String("PEER-PANA".to_string()));
        let ip = ("ip".to_string(), Bencode::String("127.0.0.2".to_string()));
        let port = ("port".to_string(), Bencode::String("440".to_string()));
        let dict2 = Bencode::Dictionary([id, ip, port].to_vec());

        let list = [dict1, dict2].to_vec();
        let tracker_id = "TRACKER-BACAN".to_string();
        let mut response = Response::new(tracker_id.clone(), 0, 2, list.clone());

        let message = response.make_message();
        println!("{}", message);
        let split: Vec<&str> = message.split("\n").collect();

        let bencode =
            DecodingParser::decode_from_string(&DecodingParser, split[2].to_string()).unwrap();
        bencode.print();
    }
}
