use crate::errors::BitTrackerError;
use crate::peer::Peer;
use crate::request::Request;
use crate::tracker::BitTracker;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

/******************************************************************************************/
/*                                 CONNECTION                                             */
/******************************************************************************************/

pub enum Endpoint {
    Announce,
    Stats,
    End,
    BadRequest,
}

/// Estructura encargada de manejar la comunicacion entre el tracker y otro peer.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Connection {
    pub id: usize,
    pub peer: Option<Peer>,
    pub stream: TcpStream,
    pub reader: BufReader<TcpStream>,
    pub tracker: Arc<Mutex<BitTracker>>,
    pub log: Sender<String>,
}

#[allow(dead_code)]
#[allow(clippy::unused_io_amount)]
impl Connection {
    /// Inicializa la conexion
    fn new(
        id: usize,
        stream: TcpStream,
        mutex: Arc<Mutex<BitTracker>>,
    ) -> Result<Self, BitTrackerError> {
        let stream_clone = stream
            .try_clone()
            .or(Err(BitTrackerError::FailToConnectError))?;
        let reader = BufReader::new(stream_clone);
        let tracker = mutex.lock().or(Err(BitTrackerError::MutexLockError))?;
        let log = tracker.log.clone();
        drop(tracker);
        Ok(Connection {
            id,
            peer: None,
            stream,
            reader,
            tracker: mutex,
            log,
        })
    }

    /// Lee de la conexion, utiliza un readline, leera hasta obtener un \n
    fn read_stream(&mut self) -> Result<String, BitTrackerError> {
        let mut buffer = [0_u8; 1024];
        self.stream
            .read(&mut buffer)
            .or(Err(BitTrackerError::ReadConnectionError))?;
        let message = String::from_utf8_lossy(&buffer).to_string();
        Ok(message)
    }

    /// Obtiene un mensaje de la conexion y lo convierte en un enum del tipo Edpoint
    fn match_endpoint(message: String) -> Result<Endpoint, BitTrackerError> {
        let split: Vec<&str> = message.split('?').collect();
        if split[0].contains("announce") {
            Ok(Endpoint::Announce)
        } else if split[0].contains("stats") {
            Ok(Endpoint::Stats)
        } else if split[0].contains("end") {
            Ok(Endpoint::End)
        } else {
            Ok(Endpoint::BadRequest)
        }
    }

    /// Maneja los mensajes recibidos desde la conexion, estos puden ser Announce, Stats o End.
    /// En caso de recibir otro mensaje, devuelve un bad request.
    fn handle_message(&mut self, message: String) -> Result<bool, BitTrackerError> {
        let endpoint = Self::match_endpoint(message.to_lowercase())?;
        match endpoint {
            Endpoint::Announce => {
                println!("[TRACKER] Recibi un Announce de la conexion {}", self.id);
                let log_message = "- [INFO] Recibi un announce de la conexion : ".to_string()
                    + &self.id.to_string();
                self.log
                    .send(log_message)
                    .or(Err(BitTrackerError::WriteLogError))?;
                self.handle_announce(message)?;
            }
            Endpoint::Stats => {
                println!("[TRACKER] Recibi un Stats de la conexion {}", self.id);
                let log_message =
                    "- [INFO] Recibi un Stats de la conexion : ".to_string() + &self.id.to_string();
                self.log
                    .send(log_message)
                    .or(Err(BitTrackerError::WriteLogError))?;
                self.handle_stats()?;
            }
            Endpoint::End => {
                println!("[TRACKER] Recibi un END de la conexion {}", self.id);
                let log_message =
                    "- [INFO] Recibi un END de la conexion : ".to_string() + &self.id.to_string();
                self.log
                    .send(log_message)
                    .or(Err(BitTrackerError::WriteLogError))?;
                return Ok(true);
            }
            Endpoint::BadRequest => {
                println!(
                    "[TRACKER] Recibi un mensaje incomprensible de la conexion {}",
                    self.id
                );
                let log_message = "- [INFO] Recibi un mensaje incomprensible de la conexion : "
                    .to_string()
                    + &self.id.to_string();
                self.log
                    .send(log_message)
                    .or(Err(BitTrackerError::WriteLogError))?;
                self.send_bad_request()?;
            }
        }
        Ok(false)
    }

    /// Si recibe una mensaje de Statas, calcula las estadisticas y devuelve un json por la conexion
    fn handle_stats(&mut self) -> Result<(), BitTrackerError> {
        let mut tracker = self
            .tracker
            .lock()
            .or(Err(BitTrackerError::MutexLockError))?;
        let stats = tracker.get_stats()?;
        drop(tracker);
        let serialized = serde_json::to_string(&stats).unwrap() + "\n";
        let message = format!(
            "HTTP/1.1 200 OK \r\nContent-Length:{}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
            serialized.chars().count(),
            serialized
          );

        self.stream
            .write_all((message + "\n").as_bytes())
            .or(Err(BitTrackerError::WriteConnectionError))?;
        Ok(())
    }

    /// En caso de recibir un announce, almacena la informacion en el tracker y genera la response.
    fn handle_announce(&mut self, message: String) -> Result<(), BitTrackerError> {
        let mut request = Request::new();
        match request.parse_request(message) {
            Ok(request) => {
                println!("[TRACKER CONEXION {}]: {:?}", self.id, request);
                let mut tracker = self
                    .tracker
                    .lock()
                    .or(Err(BitTrackerError::MutexLockError))?;

                let result = tracker.receive_request(request);
                drop(tracker);
                match result {
                    Ok(has_torrent) => {
                        // Debo leer dos veces mas de la Conexión ya que un mensaje announce tiene 3 \n
                        /*
                        let mut buffer = String::new();
                        self.reader
                            .read_line(&mut buffer)
                            .or(Err(BitTrackerError::ReadConnectionError))?;
                        self.reader
                            .read_line(&mut buffer)
                            .or(Err(BitTrackerError::ReadConnectionError))?;
                        */
                        if has_torrent {
                            self.make_response(request)?;
                        } else {
                            self.send_bad_request()?;
                        }
                    }
                    Err(error) => {
                        let message = "- [ERROR] Conexion ".to_string()
                            + &self.id.to_string()
                            + ": "
                            + &error.to_string();
                        println!("{}", message);
                        self.log
                            .send(message)
                            .or(Err(BitTrackerError::WriteLogError))?;
                        self.send_bad_request()?;
                    }
                }

                Ok(())
            }
            Err(_) => {
                println!(
                    "[ERROR] Parametro invalido en la request de la conexion {}",
                    self.id
                );
                self.send_bad_request()?;
                Ok(())
            }
        }
    }

    /// Recibe una requqest y genera una response.
    fn make_response(&mut self, request: &Request) -> Result<(), BitTrackerError> {
        let mut tracker = self
            .tracker
            .lock()
            .or(Err(BitTrackerError::MutexLockError))?;
        let mut response = tracker.make_response(request)?;
        drop(tracker);
        let message = response.make_message();
        self.stream
            .write_all(message.as_bytes())
            .or(Err(BitTrackerError::WriteConnectionError))?;
        Ok(())
    }

    /// Escribe un mensaje de Bad Request por la conexion.
    fn send_bad_request(&mut self) -> Result<(), BitTrackerError> {
        let bad_request = "HTTP/1.1 400 Bad Request \r\n";
        self.stream
            .write_all(bad_request.as_bytes())
            .or(Err(BitTrackerError::WriteConnectionError))?;
        Ok(())
    }

    /// Inicializa la conexion y se queda leyendo.
    pub fn connect(
        id: usize,
        stream: TcpStream,
        tracker: Arc<Mutex<BitTracker>>,
    ) -> Result<(), BitTrackerError> {
        let mut connection = Connection::new(id, stream, tracker)?;

        let mut done = false;
        while !done {
            let messages = connection.read_stream()?;
            if !messages.is_empty() {
                println!("[TRACKER] Recibi el mensaje : {}", messages);
                done = connection.handle_message(messages)?;
            }
        }

        println!("[TRACKER] Finalizo la conexion {}", id);
        let message = "- [INFO] Finalizo la conexion : ".to_string() + &connection.id.to_string();
        connection
            .log
            .send(message)
            .or(Err(BitTrackerError::WriteLogError))?;
        Ok(())
    }
}

#[cfg(test)]
mod connection_should {
    use super::*;
    use std::io::BufRead;
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    #[ignore]
    fn initialize_connection() {
        let path = "./config_file";
        let (tx, _rx) = mpsc::channel();
        let tracker = BitTracker::new(path, tx).unwrap();
        let tracker = Arc::new(Mutex::new(tracker));
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        let connection = thread::spawn(move || {
            println!("Por enviar basado");
            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /BASADO?\n";
            stream.write_all(message.as_bytes()).unwrap();
            println!("Esperando respuesta");
            let stream_clone = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream_clone);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /END\n";
            stream.write_all(message.as_bytes()).unwrap();
        });

        let stream = listener.accept().unwrap();
        Connection::connect(1, stream.0, tracker).unwrap();

        connection.join().unwrap();
    }

    #[test]
    #[ignore]
    fn get_stats() {
        let path = "./config_file";
        let (tx, _rx) = mpsc::channel();
        let tracker = BitTracker::new(path, tx).unwrap();
        let tracker = Arc::new(Mutex::new(tracker));
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        let connection = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /STATS\n";
            stream.write_all(message.as_bytes()).unwrap();

            let stream_clone = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream_clone);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /END\n";
            stream.write_all(message.as_bytes()).unwrap();
        });

        let stream = listener.accept().unwrap();
        Connection::connect(1, stream.0, tracker).unwrap();

        connection.join().unwrap();
    }

    #[test]
    #[ignore]
    fn get_response() {
        let path = "./config_file";
        let (tx, _rx) = mpsc::channel();
        let tracker = BitTracker::new(path, tx).unwrap();
        let tracker = Arc::new(Mutex::new(tracker));
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        let connection = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-4R01010-D23T24S25F26&port=6881&uploaded=0&downloaded=0&left=0&compact=1&event=started&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();

            let stream_clone = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream_clone);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-SeHaceElOtro-D23T24S25F26&port=6831&uploaded=1&downloaded=2&left=0&compact=0&event=completed&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /STATS\n";
            stream.write_all(message.as_bytes()).unwrap();
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Stats : {}", buffer);

            let message = "GET /END\n";
            stream.write_all(message.as_bytes()).unwrap();
        });

        let stream = listener.accept().unwrap();
        Connection::connect(1, stream.0, tracker).unwrap();

        connection.join().unwrap();
    }

    #[test]
    #[ignore]
    fn send_bad_announce() {
        let path = "./config_file";
        let (tx, _rx) = mpsc::channel();
        let tracker = BitTracker::new(path, tx).unwrap();
        let tracker = Arc::new(Mutex::new(tracker));
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        let connection = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /announce?infohash=7f%ff%c4g&peer_id=-4R01010-D23T24S25F26&port=6881&event=started&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();

            let stream_clone = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream_clone);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-SeHaceElOtro-D23T24S25F26&port=6831&uploaded=1&downloaded=2&left=0&compact=0&event=completed&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /STATS\n";
            stream.write_all(message.as_bytes()).unwrap();
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Stats : {}", buffer);

            let message = "GET /END\n";
            stream.write_all(message.as_bytes()).unwrap();
        });

        let stream = listener.accept().unwrap();
        Connection::connect(1, stream.0, tracker).unwrap();

        connection.join().unwrap();
    }

    #[test]
    #[ignore]
    fn send_double_announce() {
        let path = "./config_file";
        let (tx, _rx) = mpsc::channel();
        let tracker = BitTracker::new(path, tx).unwrap();
        let tracker = Arc::new(Mutex::new(tracker));
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        let connection = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-4R01010-D23T24S25F26&port=6881&uploaded=0&downloaded=0&left=0&compact=1&event=started&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();

            let stream_clone = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream_clone);
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-4R01010-D23T24S25F26&port=6831&uploaded=1&downloaded=2&left=0&compact=1&event=completed&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Response : {}", buffer);

            let message = "GET /STATS\n";
            stream.write_all(message.as_bytes()).unwrap();
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            reader.read_line(&mut buffer).unwrap();
            println!("[Peer] recibi Stats : {}", buffer);

            let message = "GET /END\n";
            stream.write_all(message.as_bytes()).unwrap();
        });

        let stream = listener.accept().unwrap();
        Connection::connect(1, stream.0, tracker).unwrap();

        connection.join().unwrap();
    }

    #[test]
    #[ignore]
    fn send_multiple_announce() {
        let connection1 = thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-4R01010-D23T24S25F26&port=6881&uploaded=0&downloaded=0&left=0&compact=1&event=started&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();

            //let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-420-D23T24S25F26&port=6881&uploaded=0&downloaded=0&left=0&compact=1&event=started&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();

            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-415-D23T24S25F26&port=6881&uploaded=0&downloaded=0&left=0&compact=1&event=completed&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();

            //let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            let message = "GET /announce?infohash=%a9%b3%27cNr%21~%e0%1a%de%8emw%16%10%7f%ff%c4g&peer_id=-470-D23T24S25F26&port=6881&uploaded=0&downloaded=0&left=0&compact=1&event=completed&ip=186.189.238.5 HTTP/1.0\r\nHost: torrent.ubuntu.com\r\n\r\n";
            stream.write_all(message.as_bytes()).unwrap();
        });

        connection1.join().unwrap();
    }

    #[test]
    #[ignore]
    fn request_stats() {
        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let message = "GET /STATS\n";
        stream.write_all(message.as_bytes()).unwrap();

        let stream_clone = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream_clone);
        let mut buffer = String::new();
        reader.read_line(&mut buffer).unwrap();
        reader.read_line(&mut buffer).unwrap();
        reader.read_line(&mut buffer).unwrap();
        reader.read_line(&mut buffer).unwrap();
        reader.read_line(&mut buffer).unwrap();
        reader.read_line(&mut buffer).unwrap();
        println!("[Peer] recibi Response : {}", buffer);

        let message = "GET /END\n";
        stream.write_all(message.as_bytes()).unwrap();
    }
}
