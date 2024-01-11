use crate::encoder::url_encoder::URLEncoder;
use crate::errors::BitTrackerError;
use crate::metainfo::MetaInfo;
use crate::peer::Peer;
/******************************************************************************************/
/*                                        TORRENT                                         */
/******************************************************************************************/

/// Estructura que modela a un torrent del tracker.
/// Tiene su metainfo y la lista de peers conectados.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Torrent {
    pub peers: Vec<Peer>,
    pub metainfo: MetaInfo,
    pub info_hash_url: String,
}

#[allow(dead_code)]
impl Torrent {
    pub fn new(path: &str) -> Result<Self, BitTrackerError> {
        let metainfo = MetaInfo::new(path).or(Err(BitTrackerError::OpenFileError))?;
        let peers = vec![];
        let info_hash_url = URLEncoder
            .urlencode(metainfo.info_hash.clone())
            .or(Err(BitTrackerError::DecodingError))?;
        Ok(Torrent {
            metainfo,
            peers,
            info_hash_url,
        })
    }
}

#[cfg(test)]
mod torrent_should {
    use super::*;

    #[test]
    fn initialize_torrent() {
        let path = "./torrents/INFORME - BITTORRENT.pdf.torrent";
        let tracker = Torrent::new(path).unwrap();
        println!("Announce: {:?}", tracker.metainfo.announce);
        println!("Hash: {:?}", tracker.metainfo.info_hash);
        println!("Hash Url: {}", tracker.info_hash_url);
        println!("Name: {:?}", tracker.metainfo.info.name);
        println!("Num pieces: {:?}", tracker.metainfo.info.num_pieces);
        println!("Length: {:?}", tracker.metainfo.info.length);
        println!("Piece Length: {:?}", tracker.metainfo.info.piece_length);
    }
}
