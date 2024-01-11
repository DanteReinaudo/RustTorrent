use crate::encoder::bencode_encoder::EncodingParser;
use crate::encoder::bencode_parser::{Bencode, DecodingParser};
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Read;

use crate::errors::BitTrackerError;

/******************************************************************************************/
/*                                     METAINFO                                          */
/******************************************************************************************/

/// Estructura que representa el campo metainfo del torrent file.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MetaInfo {
    pub announce: String,
    pub info: Info,
    pub info_hash: Vec<u8>,
}

type Result<T> = std::result::Result<T, BitTrackerError>;

/// Estructura que representa el campo info del metainfo del torrent file.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub piece_length: u32,
    pub pieces: Vec<Vec<u8>>,
    pub num_pieces: usize,
    pub name: String,
    pub length: u64,
}
#[allow(dead_code)]
impl MetaInfo {
    /// Recibe el torrent path, e inicializa la estructura Metainfo.
    pub fn new(torrent_path: &str) -> Result<MetaInfo> {
        let mut info = Info::new(vec![(String::from(""), Bencode::Int(0))])?;
        let mut info_hash: Vec<u8> = vec![];
        let mut announce = String::from("");
        let bencode = Self::decode_torrent_file(torrent_path)?;
        if let Bencode::Dictionary(dict) = bencode {
            for (key, value) in dict {
                match key.as_str() {
                    "info" => {
                        info_hash = Self::hashing(&EncodingParser.encode(value.clone()));
                        if let Bencode::Dictionary(dict) = value {
                            info = Info::new(dict)?;
                        }
                    }
                    "announce" => {
                        if let Bencode::String(string) = value {
                            announce = string.clone();
                        }
                    }
                    _ => continue,
                }
            }
        }
        Ok(MetaInfo {
            announce,
            info,
            info_hash,
        })
    }

    /// Abre el archivo de torren y lo parsea.
    fn decode_torrent_file(torrent_path: &str) -> Result<Bencode> {
        let mut file = File::open(torrent_path).or(Err(BitTrackerError::OpenFileError))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .or(Err(BitTrackerError::ReadFileError))?;
        let parser = DecodingParser;
        let result: Bencode = parser
            .decode_from_u8(data)
            .or(Err(BitTrackerError::DecodingError))?;
        Ok(result)
    }

    /// Realiza el SHA1 del campo info del torrent file para obtener el hash_info
    pub fn hashing(info: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(info);
        let hashed_info = &hasher.finalize()[..];
        hashed_info.to_owned()
    }
}

#[allow(dead_code)]
impl Info {
    fn new(dict: Vec<(String, Bencode)>) -> Result<Info> {
        let mut piece_length: u32 = 0;
        let mut pieces: Vec<Vec<u8>> = vec![vec![]];
        let mut num_pieces: usize = 0;
        let mut name: String = String::from("");
        let mut length: u64 = 0;

        for (key, value) in dict {
            match key.as_str() {
                "piece length" => {
                    piece_length = Self::get_piece_length(value)?;
                }
                "pieces" => {
                    pieces = Self::get_pieces(value)?;
                    num_pieces = pieces.len();
                }
                "name" => {
                    if let Bencode::String(string) = value {
                        name = string.clone();
                    }
                }
                "length" => {
                    length = Self::get_length(value)?;
                }
                _ => continue,
            }
        }
        Ok(Info {
            piece_length,
            pieces,
            num_pieces,
            name,
            length,
        })
    }

    fn get_piece_length(bencode: Bencode) -> Result<u32> {
        if let Bencode::Int(num) = bencode {
            let piece_length = num
                .try_into()
                .or(Err(BitTrackerError::IntegerConvertionError))?;
            return Ok(piece_length);
        }
        Err(BitTrackerError::DecodingError)
    }

    fn get_length(bencode: Bencode) -> Result<u64> {
        if let Bencode::Int(num) = bencode {
            let length = num
                .try_into()
                .or(Err(BitTrackerError::IntegerConvertionError))?;
            return Ok(length);
        }
        Err(BitTrackerError::DecodingError)
    }

    fn get_pieces(bencode: Bencode) -> Result<Vec<Vec<u8>>> {
        if let Bencode::ByteString(bytes) = bencode {
            let mut vec: Vec<Vec<u8>> = vec![];
            let len = bytes.len();
            let mut i: usize = 0;
            while i < len {
                vec.push(bytes[i..i + 20].to_vec());
                i += 20;
            }
            return Ok(vec);
        }
        Err(BitTrackerError::DecodingError)
    }
}

#[cfg(test)]
mod metainfo_should {
    use super::*;
    use crate::encoder::url_encoder::URLEncoder;

    #[test]
    fn initialize_metainfo_with_sample_torrent() {
        let meta = MetaInfo::new("./torrents/sample.torrent").unwrap();
        let announce = String::from("udp://tracker.openbittorrent.com:80");
        let length = 20;
        let name = String::from("sample.txt");
        let piece_length = 65536;
        let num_pieces = 1;
        let pieces = vec![vec![
            92, 197, 230, 82, 190, 13, 230, 242, 120, 5, 179, 4, 100, 255, 155, 0, 244, 137, 240,
            201,
        ]];
        let info_hash = vec![
            208, 209, 76, 146, 110, 110, 153, 118, 26, 47, 220, 255, 39, 180, 3, 217, 99, 118, 239,
            246,
        ];
        assert_eq!(meta.announce, announce);
        assert_eq!(meta.info_hash, info_hash);
        assert_eq!(meta.info.length, length);
        assert_eq!(meta.info.name, name);
        assert_eq!(meta.info.piece_length, piece_length);
        assert_eq!(meta.info.num_pieces, num_pieces);
        assert_eq!(meta.info.pieces, pieces);
    }

    #[test]
    fn initialize_metainfo_with_example_torrent() {
        let meta = MetaInfo::new("./torrents/example.torrent").unwrap();
        let announce = String::from("udp://tracker.opentrackr.org:1337/announce");
        let length = 361969;
        let name = String::from("Proyecto_2022_1C_BitTorrent.pdf");
        let piece_length = 16384;

        let pieces = vec![
            140, 42, 142, 36, 165, 93, 46, 186, 25, 9, 70, 47, 90, 26, 93, 191, 93, 30, 89, 34,
            190, 166, 39, 176, 113, 0, 249, 132, 241, 121, 2, 195, 169, 212, 222, 84, 51, 186, 130,
            34, 207, 98, 51, 185, 111, 254, 252, 70, 154, 146, 63, 75, 22, 147, 246, 101, 223, 8,
            198, 117, 82, 48, 165, 177, 210, 205, 53, 148, 167, 178, 57, 255, 128, 162, 84, 219,
            123, 87, 124, 235, 234, 138, 111, 107, 76, 102, 1, 116, 28, 138, 187, 201, 74, 32, 67,
            116, 2, 60, 93, 133, 85, 215, 116, 81, 210, 141, 78, 57, 166, 82, 208, 135, 175, 13,
            234, 188, 148, 111, 53, 172, 32, 16, 140, 52, 52, 225, 31, 80, 150, 224, 139, 216, 158,
            101, 162, 6, 91, 105, 84, 158, 76, 223, 112, 115, 96, 179, 226, 171, 200, 235, 73, 0,
            174, 229, 15, 102, 73, 159, 170, 249, 218, 16, 104, 5, 199, 94, 23, 187, 47, 92, 229,
            229, 202, 3, 152, 133, 245, 156, 64, 182, 89, 50, 59, 135, 79, 236, 94, 134, 182, 122,
            64, 216, 217, 163, 6, 53, 162, 106, 215, 91, 63, 47, 2, 101, 222, 49, 81, 181, 241,
            128, 69, 199, 233, 215, 86, 205, 115, 246, 41, 0, 44, 252, 53, 63, 203, 36, 253, 35, 9,
            174, 255, 172, 7, 191, 15, 206, 107, 220, 134, 13, 33, 142, 224, 53, 14, 224, 173, 209,
            20, 59, 245, 221, 47, 63, 25, 224, 14, 187, 163, 28, 30, 144, 20, 40, 140, 69, 113, 90,
            48, 3, 156, 131, 254, 205, 210, 2, 228, 239, 77, 236, 87, 34, 238, 82, 233, 193, 0,
            213, 68, 91, 221, 242, 50, 85, 55, 251, 253, 35, 94, 225, 67, 203, 125, 109, 147, 8,
            253, 252, 217, 235, 223, 242, 144, 245, 160, 126, 112, 219, 210, 134, 96, 47, 119, 147,
            49, 139, 62, 189, 182, 13, 47, 19, 198, 114, 215, 58, 241, 53, 155, 231, 132, 8, 228,
            184, 224, 147, 53, 125, 41, 90, 128, 230, 28, 247, 19, 79, 20, 171, 67, 242, 87, 168,
            29, 55, 67, 9, 41, 4, 172, 131, 172, 69, 76, 94, 134, 128, 90, 87, 78, 109, 89, 75,
            185, 138, 90, 146, 111, 41, 227, 68, 20, 150, 143, 211, 139, 54, 244, 79, 176, 250, 89,
            176, 237, 223, 103, 148, 92, 19, 145, 112, 250, 36, 136, 93, 80, 62, 228, 141, 62, 5,
            74, 14, 138, 116, 7, 204, 171, 124, 170, 6, 91, 179, 100, 27, 89, 172, 74, 11, 153, 18,
            189, 13, 45, 247, 246, 202, 200, 178, 220, 200, 100, 107, 185, 73, 230, 80, 205, 200,
            72, 168,
        ];
        let num_pieces = pieces.len() / 20;
        let len = pieces.len();
        let mut i: usize = 0;
        let mut j = 0;
        while i < len {
            assert_eq!(meta.info.pieces[j].to_vec(), pieces[i..i + 20].to_vec());
            i += 20;
            j += 1;
        }

        let info_hash = vec![
            93, 81, 240, 224, 157, 149, 245, 50, 143, 202, 31, 157, 22, 129, 156, 155, 124, 5, 70,
            223,
        ];
        assert_eq!(meta.announce, announce);
        assert_eq!(meta.info_hash, info_hash);
        assert_eq!(meta.info.length, length);
        assert_eq!(meta.info.name, name);
        assert_eq!(meta.info.piece_length, piece_length);
        assert_eq!(meta.info.num_pieces, num_pieces);
    }

    #[test]
    fn hashing_parameter() {
        let meta = MetaInfo::new("./torrents/kubuntu-16.04.6-desktop-amd64.iso.torrent").unwrap();

        assert_eq!(
            hex::encode(meta.info_hash),
            "45b3d693cff285975f622acaeb75c5626acaff6f"
        );
    }

    #[test]
    fn hash_sentence() {
        let expected = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";
        let output = MetaInfo::hashing(&String::from("hello world").into_bytes());

        assert_eq!(hex::encode(output), expected);
    }

    #[test]
    fn url_encode_info_hash() {
        let meta = MetaInfo::new("./torrents/kubuntu-16.04.6-desktop-amd64.iso.torrent").unwrap();

        let urlencoded_string = URLEncoder.urlencode(meta.info_hash).unwrap();

        assert_eq!(
            urlencoded_string,
            "E%b3%d6%93%cf%f2%85%97_b%2a%ca%ebu%c5bj%ca%ffo"
        );
    }
}
