use crate::encoder::url_encoder::URLEncoder;
use crate::tracker::errors::TrackerError;
use crate::tracker::tracker_response::TrackerResponse;
use native_tls::{TlsConnector, TlsStream};
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

/******************************************************************************************/
/*                                  TRACKER REQUEST                                      */
/******************************************************************************************/
static HTTPS_PORT: &str = ":443";
static HTTP_PORT: &str = ":6969";
static OUR_PORT: &str = ":8080";
static OUR_HOST: &str = "127.0.0.1";

static HTTPS: &str = "https";

#[allow(dead_code)]
#[derive(Debug)]
pub struct TrackerRequest {
    info_hash: Vec<u8>,
    peer_id: String,
    port_to_peers: String,
    protocol: String,
    host: String,
}

pub enum Connector {
    Http(TcpStream),
    HttpNuestro(TcpStream),
    Https(TlsStream<TcpStream>),
}

#[allow(dead_code)]
impl Connector {
    /// Esta funcion inicializa el Conector en base al protocolo inidicado por la url del Torrent.
    /// De ser HTTP utiliza el puerto reservado para conexiones HTTP, y utiliza tan solo un flujo de tipo TcpStram.
    /// De ser HTTPS utiliza otro puerto y además monta una conexión Tls sobre la conexión Tcp.
    /// En caso de haber un error al realizar la conexión devuelve error de conexión.
    fn new(tracker: &TrackerRequest) -> Result<Connector, TrackerError> {
        if tracker.protocol == HTTPS {
            let connector = TlsConnector::new().or(Err(TrackerError::FailToConnectError))?;
            let stream = TcpStream::connect(tracker.host.to_owned() + HTTPS_PORT)
                .or(Err(TrackerError::FailToConnectError))?;

            let result: TlsStream<TcpStream> = connector
                .connect(&*tracker.host, stream)
                .or(Err(TrackerError::FailToConnectError))?;
            Ok(Connector::Https(result))
        } else {
            //println!("{}", tracker.host);
            if tracker.host == OUR_HOST {
                let stream = TcpStream::connect(tracker.host.to_owned() + OUR_PORT)
                    .or(Err(TrackerError::FailToConnectError))?;
                return Ok(Connector::HttpNuestro(stream));
            }

            let stream = TcpStream::connect(tracker.host.to_owned() + HTTP_PORT)
                .or(Err(TrackerError::FailToConnectError))?;
            Ok(Connector::Http(stream))
        }
    }

    /// Esta función recibe un &str con la Request para el Tracker
    /// y utiliza el flujo abierto para enviar dicha Request y leer su respuesta.
    /// Devuelve la Respuesta dada por el Tracker, un Vec<u8>.
    /// En caso de haber un error al enviar la request al Tracker devuelve error de escritura.
    /// En caso de haber un error al recibir la respuesta del Tracker devuelve error de lectura.
    fn stream(&mut self, req: &str) -> Result<Vec<u8>, TrackerError> {
        match self {
            Connector::Https(stream) => {
                stream
                    .write_all(req.as_bytes())
                    .or(Err(TrackerError::WriteConnectionError))?;
                let mut res = vec![];
                stream
                    .read_to_end(&mut res)
                    .or(Err(TrackerError::ReadConnectionError))?;
                Ok(res)
            }
            Connector::Http(stream) => {
                stream
                    .write_all(req.as_bytes())
                    .or(Err(TrackerError::WriteConnectionError))?;
                println!("Pasé el write");
                let mut res = vec![];
                stream
                    .read_to_end(&mut res)
                    .or(Err(TrackerError::ReadConnectionError))?;
                println!("Pasé el read");

                Ok(res)
            }

            Connector::HttpNuestro(stream) => {
                stream
                    .write_all(req.as_bytes())
                    .or(Err(TrackerError::WriteConnectionError))?;
                println!("Pasé el write");
                let mut buffer = [0_u8; 1024];

                let _ = stream
                    .read(&mut buffer)
                    .or(Err(TrackerError::ReadConnectionError))?;
                println!("Pasé el read");
                Ok(buffer.to_vec())
            }
        }
    }
}

#[allow(dead_code)]
impl TrackerRequest {
    /// Esta funcion inicializa el TrackerRequest. Recibe los datos necesarios para hacerlo.
    pub fn new(
        info_hash: Vec<u8>,
        peer_id: String,
        port_to_peers: String,
        url: String,
    ) -> TrackerRequest {
        let announce: Vec<&str> = url.split('/').collect();
        let split_protocol: Vec<&str> = announce[0].split(':').collect();
        let split_host: Vec<&str> = announce[2].split(':').collect();
        let protocol = String::from(split_protocol[0]);
        let host = String::from(split_host[0]);
        println!("{}", host);
        TrackerRequest {
            info_hash,
            peer_id,
            port_to_peers,
            protocol,
            host,
        }
    }

    /// Esta funcion recibe un vector con los parametros para la request y los junta.
    /// Devuelve un String con la request final a ser enviada.
    fn join_parameter_request(parameters: Vec<(&str, &str)>, host: &str) -> String {
        let query_params: Vec<String> = parameters
            .iter()
            .map(|&parameter| format!("{}={}", parameter.0, parameter.1))
            .collect();
        let string_query_params = query_params.join("&");

        let mut request_string = String::from("GET /announce?");
        request_string.push_str(&*string_query_params);
        request_string.push_str(&*(" HTTP/1.0"));
        request_string.push_str(&*("\r\n"));
        request_string.push_str(&*("Host: ".to_owned() + &*host));
        request_string.push_str(&*("\r\n\r\n"));

        request_string
    }

    /// Esta funcion genera la request al tracker
    fn generate_tracker_request(&self) -> Result<String, TrackerError> {
        let info_url_encoding = URLEncoder
            .urlencode(self.info_hash.clone())
            .or(Err(TrackerError::URLEncodingError))?;

        let id_url_encoding = URLEncoder
            .urlencode(self.peer_id.clone().into_bytes().to_vec())
            .or(Err(TrackerError::URLEncodingError))?;

        let query_params: Vec<(&str, &str)> = vec![
            ("info_hash", &info_url_encoding),
            ("peer_id", &id_url_encoding),
            ("ip", "186.189.238.5"), //Agrego ip a la implementacion
            ("port", &self.port_to_peers),
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("left", "0"),
            ("event", "started"),
        ];
        let request_string = TrackerRequest::join_parameter_request(query_params, &self.host);

        Ok(request_string)
    }

    ///Esta funcion recibe un Vector con la respuesta del Tracker
    /// y calcula a partir de donde inicia la parte a ser interpretada como un Diccionario Bencode.
    /// Devuelve el indice a esa posicion.
    fn index(vec: Vec<u8>) -> usize {
        let string = String::from_utf8_lossy(vec.as_slice());
        let mut str_vec: Vec<&str> = string.split("\r\n").collect();
        str_vec.pop().unwrap();

        let mut joined_str_vec = str_vec.join("\r\n");
        joined_str_vec.push_str("\r\n");

        let bytes_vec = joined_str_vec.into_bytes();
        bytes_vec.len() - 1
    }

    ///Esta funcion recibe la respuesta del Tracker como vector
    /// y parsea el diccionario a partir del indice devuelto por Index.
    /// Devuelve un TrackerResponse.
    fn parse_response(response: &mut [u8]) -> Result<TrackerResponse, TrackerError> {
        //print!("Desde Parse Response: {}", String::from_utf8_lossy(response));
        let index = TrackerRequest::index(response.to_owned());
        let info = &response[index + 1..];

        let response = TrackerResponse::new()
            .from(info.to_owned())
            .or(Err(TrackerError::ReadConnectionError))?;

        //print!("Desde Parse Response: {:?}", response);
        Ok(response)
    }

    fn parse_our_response(response: &mut [u8]) -> Result<TrackerResponse, TrackerError> {
        let string = String::from_utf8_lossy(response).to_string();
        //println!("Desde Our Response:{}", string);
        let str_vec: Vec<&str> = string.split("\r\n").collect();

        let info = str_vec[5];
        //println!("Desde Our Response:{:?}", info);

        let response = TrackerResponse::new()
            .from(info.as_bytes().to_owned())
            .or(Err(TrackerError::ReadConnectionError))?;
        Ok(response)
    }

    /// Esta funcion realiza las conexiones correspondientes con el Tracker, se anuncia.
    /// Genera la request y recibe la respuesta leída. Por ultimo manda a parsear
    /// y devuelve la respuesta como TrackerResponse.
    pub fn announce(&mut self) -> Result<TrackerResponse, TrackerError> {
        let mut connection = Connector::new(self).or(Err(TrackerError::FailToConnectError))?;
        println!("Me conecté con el Tracker");
        let req = self.generate_tracker_request()?;
        let mut response = connection
            .stream(&req)
            .or(Err(TrackerError::RequestError))?;

        if self.host == OUR_HOST {
            let tracker_response = TrackerRequest::parse_our_response(&mut response)
                .or(Err(TrackerError::InvalidSyntaxError))?;
            println!("{:?}", tracker_response);
            return Ok(tracker_response);
        }

        //println!("{}", String::from_utf8_lossy(&*response));
        let tracker_response = TrackerRequest::parse_response(&mut response)
            .or(Err(TrackerError::InvalidSyntaxError))?;
        Ok(tracker_response)
    }
}

/******************************************************************************************/
/*                                        TESTS                                           */
/******************************************************************************************/

#[cfg(test)]
mod tracker_request_should {
    use super::*;
    use sha1::Digest;
    use sha1::Sha1;

    #[test]
    fn initialize() {
        let peer_id = String::from("-4R01010-D23T24S25F26");

        let mut hasher = Sha1::new();
        hasher.update("hello world");
        let hashed_info = &hasher.finalize()[..];

        let tracker_request = TrackerRequest::new(
            hashed_info.to_vec(),
            peer_id.clone(),
            String::from("6881"),
            String::from("http://torrent.ubuntu.com:6969/announce"),
        );

        assert_eq!(tracker_request.info_hash, hashed_info);
        assert_eq!(tracker_request.peer_id, peer_id);
        assert_eq!(tracker_request.port_to_peers, String::from("6881"));
        assert_eq!(tracker_request.protocol, String::from("http"),);
        assert_eq!(tracker_request.host, String::from("torrent.ubuntu.com"),);
    }

    #[test]
    fn generate_request() {
        let peer_id = String::from("-4R01010-D23T24S25F26");

        let mut hasher = Sha1::new();
        hasher.update("hello world");
        let hashed_info = &hasher.finalize()[..];

        let tracker_request = TrackerRequest::new(
            hashed_info.to_vec(),
            peer_id.clone(),
            String::from("6881"),
            String::from("http://torrent.ubuntu.com:6969/announce"),
        );
        let request = tracker_request
            .generate_tracker_request()
            .unwrap()
            .to_owned();

        let expected = "GET /announce?info_hash=%2a%ael5%c9O%cf%b4%15%db%e9_%40%8b%9c%e9%1e%e8F%ed&peer_id=".to_owned() + &peer_id + "&port=6881&uploaded=0&downloaded=0&left=0&event=started HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
        assert_eq!(request, expected);
    }

    #[test]
    fn connect_to_tracker_http_protocol() {
        let peer_id = String::from("-4R01010-D23T24S25F26");

        let mut hasher = Sha1::new();
        hasher.update("hello world");
        let hashed_info = &hasher.finalize()[..];

        let tracker_request = TrackerRequest::new(
            hashed_info.to_vec(),
            peer_id.clone(),
            String::from("6881"),
            String::from("http://torrent.ubuntu.com:6969/announce"),
        );

        let connection = Connector::new(&tracker_request);
        match connection {
            Ok(_v) => assert_eq!(true, true),
            Err(_e) => assert_eq!(false, true),
        }
    }

    #[test]
    fn connect_to_tracker_https_protocol() {
        let peer_id = String::from("-4R01010-D23T24S25F26");

        let mut hasher = Sha1::new();
        hasher.update("hello world");
        let hashed_info = &hasher.finalize()[..];

        let tracker_request = TrackerRequest::new(
            hashed_info.to_vec(),
            peer_id.clone(),
            String::from("6881"),
            String::from("https://torrent.ubuntu.com/announce"),
        );

        let connection = Connector::new(&tracker_request);
        match connection {
            Ok(_v) => assert_eq!(true, true),
            Err(_e) => assert_eq!(false, true),
        }
    }

    #[ignore]
    #[test]
    fn connect_to_ubuntu_16_tracker() {
        let info_hash = vec![
            69, 179, 214, 147, 207, 242, 133, 151, 95, 98, 42, 202, 235, 117, 197, 98, 106, 202,
            255, 111,
        ];
        let port_to_peers = String::from("6881");
        let peer_id = String::from("zpkbYZrkUAShNERx06u7");
        let url = String::from("http://torrent.ubuntu.com:6969/announce");
        let mut request = TrackerRequest::new(info_hash, peer_id, port_to_peers, url);

        match request.announce() {
            Ok(_v) => assert_eq!(true, true),
            Err(_e) => assert_eq!(false, true),
        }
    }

    #[ignore]
    #[test]
    fn connect_to_ubuntu_18_tracker() {
        let info_hash = "592c557efa5ef8115aa1ce4f8ef0735d44a98357".as_bytes();

        let port_to_peers = String::from("6881");
        let peer_id = String::from("zpkbYZrkUAShNERx06u7");
        let url = String::from("http://torrent.ubuntu.com:6969/announce");
        let mut request = TrackerRequest::new(Vec::from(info_hash), peer_id, port_to_peers, url);

        match request.announce() {
            Ok(_v) => assert_eq!(true, true),
            Err(_e) => assert_eq!(false, true),
        }
    }
}
