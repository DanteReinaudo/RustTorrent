use bittorrent::bitclient::client::{BitClient, Event};
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4::glib;
use gtk4::glib::{MainContext, Receiver, Sender};
use std::ops::Add;
use std::thread;

const CONFIG_PATH: &str = "./config/configuration_file";
//const TORRENT_PATH: &str = "./torrents/debian-11.3.0-amd64-netinst.iso.torrent";
const TORRENT_PATH: &str = "./torrents/INFORME.pdf.torrent";

fn main() {
    let application = gtk::Application::new(Some("com.github.taller"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let app_xml = include_str!("app.xml");
    let builder = gtk::Builder::from_string(app_xml);

    let window: gtk::Window = match builder.object("main-window") {
        Some(window) => window,
        None => panic!("Error creating main window"),
    };

    let name: gtk::Label = builder.object("info-name").expect("error rendering name");
    let hash: gtk::Label = builder.object("info-hash").expect("error rendering hash");
    let bar: gtk::ProgressBar = builder
        .object("info-progress")
        .expect("error rendering progress");
    let speed: gtk::Label = builder.object("speed").expect("error rendering speed");
    let list: gtk::ListStore = builder.object("peer").expect("error rendering list");

    let mut total_pieces = 0;
    let mut downloaded_pieces = 0;

    let (sender, receiver): (Sender<Event>, Receiver<Event>) =
        MainContext::channel(glib::PRIORITY_DEFAULT);
    thread::spawn(move || {
        BitClient::download_torrent(CONFIG_PATH, TORRENT_PATH, sender).ok();
    });

    receiver.attach(None, move |msg| {
        match msg {
            Event::UpdateName(text) => name.set_text(text.as_str()),
            Event::UpdateInfoHash(text) => hash.set_text(text.as_str()),
            Event::UpdateNumPieces(num) => total_pieces = num,
            Event::DownloadedPiece() => {
                downloaded_pieces += 1;
                let progress = downloaded_pieces as f64 / total_pieces as f64;
                bar.set_fraction(progress as f64);
                bar.set_text(Some(&*progress.to_string()));
            }
            Event::UpdateSpeed(num) => {
                let text = num.to_string().add(" MB/s");
                speed.set_text(&*text);
            }
            Event::UpdatePeerList(peers) => {
                for peer in peers {
                    let iter = list.append();
                    list.set(
                        &iter,
                        &[
                            (0, &peer.id),
                            (1, &peer.ip),
                            (2, &peer.port),
                            (3, &"choked"),
                        ],
                    );
                }
            }
            Event::Unchoked(id) => {
                let iter = list.iter_from_string(&*id.to_string()).unwrap();
                list.set(&iter, &[(3, &"unchoked")])
            }
            Event::Choked(id) => {
                let iter = list.iter_from_string(&*id.to_string()).unwrap();
                list.set(&iter, &[(3, &"unchoked")])
            }
        }
        Continue(true)
    });

    window.set_application(Some(application));
    window.show();
}
