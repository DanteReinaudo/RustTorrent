use bittorrent::bitclient::client::{BitClient, Event};
use gtk4::glib;
use gtk4::glib::{MainContext, Receiver, Sender};
use std::env::args;
use std::thread;

fn descargar_torrent(config: String, torrent: String) -> Result<(), String> {
    let (sender, _receiver): (Sender<Event>, Receiver<Event>) =
        MainContext::channel(glib::PRIORITY_DEFAULT);
    if let Err(error) = BitClient::download_torrent(&config, &torrent, sender) {
        return Err(error.to_string());
    }
    Ok(())
}

fn main() {
    let args = args().collect::<Vec<String>>();
    if args.len() < 3 {
        println!("[ERROR] Cantidad de argumentos inválido");
        return;
    }
    let config = args[1].clone();
    let torrents = args[2..].to_vec();
    let mut descargas = vec![];
    for torrent in torrents {
        let config_clone = config.clone();
        descargas.push(thread::spawn(move || {
            descargar_torrent(config_clone, torrent)
        }))
    }
    for (index, descarga) in descargas.into_iter().enumerate() {
        let resultado_join = descarga.join();
        match resultado_join {
            Ok(resultado_descarga) => {
                if let Err(error) = resultado_descarga {
                    println!("[ERROR] Descarga {}: {}", index, error);
                }
            }
            Err(_) => println!("[ERROR] Fallo el join de la descarga numero {}", index),
        }
    }
}
