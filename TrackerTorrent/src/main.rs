use std::env::args;
use std::sync::mpsc;
use std::thread;
use trackertorrent::logger::Logger;
use trackertorrent::tracker::BitTracker;

const LOG_PATH: &str = "./logs";
const LOG_NAME: &str = "BitTracker";

fn main() {
    let args = args().collect::<Vec<String>>();
    if args.len() != 2 {
        println!("[ERROR] Cantidad de argumentos inválido");
        return;
    }
    let config_file = args[1].clone();

    let (tx, rx) = mpsc::channel();

    //Inicializo el Logger
    match Logger::new(LOG_PATH, LOG_NAME, rx) {
        Ok(mut logger) => {
            let log = thread::spawn(move || logger.listening());
            if let Err(error) = BitTracker::start(&config_file, tx) {
                println!("[ERROR] {}", error);
            }
            if let Err(error) = log.join() {
                println!("[ERROR] {:?}", error);
            }
        }
        Err(error) => {
            println!("[ERROR] {}", error);
        }
    }
}
