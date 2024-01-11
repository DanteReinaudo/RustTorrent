use serde::{Deserialize, Serialize};

/******************************************************************************************/
/*                                  STATS                                                 */
/******************************************************************************************/

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats {
    pub cant_peers: usize,
    pub cant_seeders: usize,
    pub cant_torrents: usize,
    pub info: Vec<InfoPeer>,
}

impl Stats {
    pub fn new(
        cant_peers: usize,
        cant_seeders: usize,
        cant_torrents: usize,
        info: Vec<InfoPeer>,
    ) -> Self {
        Stats {
            cant_peers,
            cant_seeders,
            cant_torrents,
            info,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InfoPeer {
    pub id: String,
    pub time_last_request: String,
    pub completed: bool,
    pub torrent: String,
}

impl InfoPeer {
    pub fn new(id: String, time_last_request: String, completed: bool, torrent: String) -> Self {
        InfoPeer {
            id,
            time_last_request,
            completed,
            torrent,
        }
    }
}

#[cfg(test)]
mod stats_should {
    use super::*;

    #[test]
    fn serialize() {
        let info = InfoPeer::new(
            String::from("PANA-PEER"),
            String::from("Ayer"),
            false,
            String::from("sample.torrent"),
        );
        let mut vec = vec![];
        vec.push(info);
        let stats = Stats::new(0, 0, 0, vec);

        let serialized = serde_json::to_string(&stats).unwrap();
        println!("serialized = {}", serialized);
    }

    #[test]
    fn initialize_peer_info() {
        let info = InfoPeer::new(
            String::from("PANA-PEER"),
            String::from("Ayer"),
            false,
            String::from("sample.torrent"),
        );
        assert_eq!(info.id, String::from("PANA-PEER"));
        assert_eq!(info.time_last_request, String::from("Ayer"));
        assert_eq!(info.completed, false);
    }

    #[test]
    fn initialize_stats() {
        let info = InfoPeer::new(
            String::from("PANA-PEER"),
            String::from("Ayer"),
            false,
            String::from("sample.torrent"),
        );
        let mut vec = vec![];
        vec.push(info);
        let stats = Stats::new(0, 0, 0, vec.clone());
        assert_eq!(stats.cant_peers, 0);
        assert_eq!(stats.cant_seeders, 0);
        assert_eq!(stats.cant_torrents, 0);
        assert_eq!(stats.info, vec);
    }
}
