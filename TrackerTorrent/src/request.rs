use crate::errors::RequestError;
use chrono::DateTime;
use chrono::Local;
/******************************************************************************************/
/*                                  TRACKER REQUEST                                      */
/******************************************************************************************/

/// Estructura que modela la request de un peer.
#[derive(PartialEq, Debug, Clone)]
pub struct Request {
    pub info_hash_url: String,
    pub peer_id: String,
    pub ip: String,
    pub port: String,
    pub compact: String,
    pub event: String,
    pub uploaded: String,
    pub downloaded: String,
    pub left: String,
    pub time: DateTime<Local>,
}

#[allow(clippy::needless_range_loop)]
#[allow(clippy::new_without_default)]
impl Request {
    pub fn new() -> Request {
        Request {
            info_hash_url: "".to_string(),
            peer_id: "".to_string(),
            port: "".to_string(),
            uploaded: "".to_string(),
            downloaded: "".to_string(),
            left: "".to_string(),
            compact: "".to_string(),
            event: "".to_string(),
            ip: "".to_string(),
            time: Local::now(),
        }
    }

    /// Esta funcion recibe 1 parametro cualquiera de la request y lo guarda donde corresponde
    pub fn parse_param(&mut self, param_string: &str) -> Result<&mut Request, RequestError> {
        let param: Vec<&str> = param_string.split('=').collect();

        match param[0] {
            "peer_id" => self.peer_id = param[1].to_string(),
            "port" => self.port = param[1].to_string(),
            "uploaded" => self.uploaded = param[1].to_string(),
            "downloaded" => self.downloaded = param[1].to_string(),
            "left" => self.left = param[1].to_string(),
            "compact" => self.compact = param[1].to_string(),
            "event" => {
                //Separamos el parametro final del Host y el Protocol
                let final_params: Vec<&str> = param[1].split(' ').collect();
                self.event = final_params[0].to_string();
            }
            "ip" => {
                //Separamos el parametro final del Host y el Protocol
                let final_params: Vec<&str> = param[1].split(' ').collect();
                self.ip = final_params[0].to_string();
            }

            _ => return Err(RequestError::InvalidParameterError),
        }

        Ok(self) //Preguntar
    }

    /// Esta funcion recibe el request string del tracker,
    /// parsea e inicializa la estructura Request con los parametros recibidos.
    pub fn parse_request(&mut self, request_string: String) -> Result<&mut Request, RequestError> {
        let query_params: Vec<&str> = request_string.split('&').collect();

        let request_and_info: Vec<&str> = query_params[0].split('?').collect();
        //println!("request type: {}", request_and_info[0]);
        //println!("info hash: {}", request_and_info[1]);
        let split: Vec<&str> = request_and_info[1].split('=').collect();
        self.info_hash_url = split[1].to_string();

        let query_params_length = query_params.len();
        //println!("{}",query_params_length);

        for i in 1..query_params_length {
            Request::parse_param(self, query_params[i])
                .or(Err(RequestError::InvalidParameterError))?;
        }

        Ok(self)
    }
}

#[cfg(test)]
mod request_should {
    use crate::request::Request;
    use sha1::Digest;
    use sha1::Sha1;

    #[test]
    fn save_split_parameters() {
        let mut hasher = Sha1::new();
        hasher.update("hello world");

        let mut request = Request::new();

        let expected = "GET /announce?info_hash=%2a%ael5%c9O%cf%b4%15%db%e9_%40%8b%9c%e9%1e%e8F%ed&peer_id=-4R01010-D23T24S25F26&port=6881&uploaded=0&downloaded=0&left=0&compact=1&event=started&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";

        request.parse_request(expected.to_string()).unwrap();

        println!("Info Hash Url: {}", request.info_hash_url);
        //assert_eq!(request.info_hash, hashed_info);
        assert_eq!(request.peer_id, "-4R01010-D23T24S25F26");
        assert_eq!(request.port, "6881");
        assert_eq!(request.uploaded, "0");
        assert_eq!(request.downloaded, "0");
        assert_eq!(request.left, "0");
        assert_eq!(request.compact, "1");
        assert_eq!(request.event, "started");
        assert_eq!(request.ip, "186.189.238.5");
    }
}
