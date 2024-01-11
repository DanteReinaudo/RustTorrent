use crate::peer_protocol::errors::PeerProtocolError;
use std::fmt::Debug;

const LEN: u8 = 19;
const BTPROTOCOL: &str = "BitTorrent protocol";

/******************************************************************************************/
/*                                 HANDSHAKE                                     */
/******************************************************************************************/

/// Estructura que almacena los parametros para el handshake de los peers.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Handshake {
    pub len: u8,
    pub pstr: String,
    pub reserved: Vec<u8>,
    pub info_hash: Vec<u8>,
    pub peer_id: String,
}

#[allow(dead_code)]
impl Handshake {
    /// Se inicializa con el vector info_hash y el id del peer.
    pub fn new(info_hash: Vec<u8>, peer_id: String) -> Handshake {
        Handshake {
            len: LEN,
            pstr: BTPROTOCOL.to_string(),
            reserved: [0; 8].to_vec(),
            info_hash,
            peer_id,
        }
    }

    /// Devuelve el Hanshake en forma de vector de u8 para enviarlo por la conexion.
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec: Vec<u8> = vec![self.len];
        vec.append(&mut self.pstr.as_bytes().to_vec());
        vec.append(&mut self.reserved.clone());
        vec.append(&mut self.info_hash.clone());
        vec.append(&mut self.peer_id.as_bytes().to_vec());

        vec
    }

    /// Crea el handshake a partir de un vector de u8.
    pub fn from_bytes(message: Vec<u8>) -> Result<Handshake, PeerProtocolError> {
        let len = message[0];
        //println!("Recibi el mensaje {:?}", message);
        let pstr = String::from_utf8(message[1..20].to_vec())
            .or(Err(PeerProtocolError::HandshakeInvalidUTF8CharError))?;
        //println!("Pstr: {}", pstr);
        let reserved: Vec<u8> = message[20..28].to_vec();
        let info_hash: Vec<u8> = message[28..48].to_vec();
        let peer_id: String = String::from_utf8_lossy(&message[48..]).to_string();
        //.or(Err(PeerProtocolError::HandshakeInvalidUTF8CharError))?;
        //println!("peer_id: {}", peer_id);
        Ok(Handshake {
            len,
            pstr,
            reserved,
            info_hash,
            peer_id,
        })
    }
}

#[cfg(test)]
mod handshake_should {
    use super::*;

    #[test]
    fn initialize() {
        let info: Vec<u8> = vec![];
        let peer_id = String::from("-4R0001-D23T25F26S27");
        let _handshake = Handshake::new(info.clone(), peer_id);
        assert_eq!(_handshake.len, 19);
        assert_eq!(_handshake.pstr, "BitTorrent protocol".to_string());
        assert_eq!(_handshake.reserved, [0; 8].to_vec());
        assert_eq!(_handshake.info_hash, info);
        assert_eq!(_handshake.peer_id, String::from("-4R0001-D23T25F26S27"));
    }

    #[test]
    fn into_bytes() {
        let info: Vec<u8> = [
            69, 179, 214, 147, 207, 242, 133, 151, 95, 98, 42, 202, 235, 117, 197, 98, 106, 202,
            255, 111,
        ]
        .to_vec();
        let peer_id = String::from("-4R0001-D23T25F26S27");
        let handshake = Handshake::new(info, peer_id);
        let result = handshake.as_bytes();
        let expected: Vec<u8> = [
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 0, 0, 0, 69, 179, 214, 147, 207, 242, 133, 151, 95, 98, 42,
            202, 235, 117, 197, 98, 106, 202, 255, 111, 45, 52, 82, 48, 48, 48, 49, 45, 68, 50, 51,
            84, 50, 53, 70, 50, 54, 83, 50, 55,
        ]
        .to_vec();
        assert_eq!(result, expected);
    }

    #[test]
    fn from_bytes() {
        let message: Vec<u8> = [
            19, 66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99,
            111, 108, 0, 0, 0, 0, 0, 0, 0, 0, 69, 179, 214, 147, 207, 242, 133, 151, 95, 98, 42,
            202, 235, 117, 197, 98, 106, 202, 255, 111, 45, 52, 82, 48, 48, 48, 49, 45, 68, 50, 51,
            84, 50, 53, 70, 50, 54, 83, 50, 55,
        ]
        .to_vec();
        let info: Vec<u8> = [
            69, 179, 214, 147, 207, 242, 133, 151, 95, 98, 42, 202, 235, 117, 197, 98, 106, 202,
            255, 111,
        ]
        .to_vec();
        let peer_id = String::from("-4R0001-D23T25F26S27");
        let new_handshake: Handshake = Handshake::from_bytes(message).unwrap();

        assert_eq!(new_handshake.len, 19);
        assert_eq!(new_handshake.pstr, "BitTorrent protocol".to_string());
        assert_eq!(new_handshake.info_hash, info);
        assert_eq!(new_handshake.peer_id, peer_id);
    }
}
