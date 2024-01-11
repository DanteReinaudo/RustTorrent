/******************************************************************************************/
/*                                       PEER                                             */
/******************************************************************************************/

/// Estructura que modela a un peer.
#[derive(PartialEq, Debug, Clone)]
pub struct Peer {
    pub id: String,
    pub ip: String,
    pub port: String,
    pub bitfield: Vec<bool>,
    pub choked: bool,
    pub interested: bool,
}

impl Peer {
    pub fn new(id: String, ip: String, port: String) -> Peer {
        Peer {
            id,
            ip,
            port,
            bitfield: vec![],
            choked: false,
            interested: false,
        }
    }

    /// Almacena el bitmap, lo recibe en formato comprimido binario y lo convierte en un bitmap booleano.
    pub fn store_bitmap(&mut self, bytes: Vec<u8>, num_pieces: usize) {
        let mut bitfield: Vec<bool> = vec![false; num_pieces];
        for (i, _value) in bitfield.clone().iter().enumerate() {
            let bytes_index = i / 8;
            let index_into_byte = i % 8;
            let byte = bytes[bytes_index];
            let value = (byte & (1 << (7 - index_into_byte))) != 0;
            bitfield[i] = value;
        }
        self.bitfield = bitfield;
    }

    /// Realiza la operacion inversa, recibe un bitmap booleano y lo transforma en un formato binario comprimido.
    pub fn bytes_from_bitmap(bitmap: Vec<bool>) -> Vec<u8> {
        let mut init = 0;
        let mut finish = 8;
        let mut bitfield: Vec<u8> = vec![];
        while init < bitmap.len() {
            let byte = {
                if finish > bitmap.len() {
                    bitmap[init..].to_vec()
                } else {
                    bitmap[init..finish].to_vec()
                }
            };
            let mut num = 0;
            for (i, bit) in byte.into_iter().enumerate() {
                if bit {
                    num += Self::binary_value(i as u8);
                }
            }
            bitfield.push(num);
            init += 8;
            finish += 8;
        }

        bitfield
    }

    fn binary_value(offset: u8) -> u8 {
        match offset {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            4 => 16,
            5 => 32,
            6 => 64,
            7 => 128,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod peer_should {
    use super::*;

    #[test]
    fn initialize_connection() {
        let peer = Peer::new("id".to_string(), "ip".to_string(), "port".to_string());
        assert_eq!(peer.id, "id".to_string());
        assert_eq!(peer.ip, "ip".to_string());
        assert_eq!(peer.port, "port".to_string());
        assert_eq!(peer.bitfield, vec![]);
        assert_eq!(peer.choked, false);
        assert_eq!(peer.interested, false);
    }

    #[test]
    fn store_bitmap() {
        let mut peer = Peer::new("id".to_string(), "ip".to_string(), "port".to_string());
        peer.store_bitmap(vec![17], 8);
        assert_eq!(
            peer.bitfield,
            vec![false, false, false, true, false, false, false, true]
        );
    }

    #[test]
    fn return_bytes_from_bitmap() {
        let bytes = Peer::bytes_from_bitmap(vec![true; 16]);
        assert_eq!(bytes, vec![255, 255]);
    }
}
