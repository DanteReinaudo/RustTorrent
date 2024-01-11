use super::errors::PeerProtocolError;

/******************************************************************************************/
/*                                  Messages P2P                                         */
/******************************************************************************************/

/// Estructura que modela el mensaje, con su largo y su id
#[allow(dead_code)]
#[derive(Debug)]
pub struct Message {
    pub len: u32,
    pub id: MessageId,
}

/// Enum que modela los distintos tipos de mensajes.
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum MessageId {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request(u32, u32, u32),
    Piece(u32, u32, Vec<u8>),
    Cancel(u32, u32, u32),
}

#[allow(dead_code)]
impl Message {
    pub fn new(len: u32, bytes: Vec<u8>) -> Result<Message, PeerProtocolError> {
        if len == 0 {
            return Ok(Self::generate_keep_alive());
        }
        match bytes[0] {
            0 => Ok(Self::generate_choke()),
            1 => Ok(Self::generate_unchoke()),
            2 => Ok(Self::generate_interested()),
            3 => Ok(Self::generate_not_interested()),
            4 => Ok(Self::generate_have(bytes)),
            5 => Self::generate_bitfield(len, bytes),
            6 => Ok(Self::generate_request(bytes)),
            7 => Self::generate_piece(len, bytes),
            8 => Ok(Self::generate_cancel(bytes)),
            _ => Err(PeerProtocolError::InvalidMessageFormatError),
        }
    }

    /// Genera el mensaje keep alive
    fn generate_keep_alive() -> Message {
        Message {
            len: 0,
            id: MessageId::KeepAlive,
        }
    }

    /// genera el mensaje choke
    fn generate_choke() -> Message {
        Message {
            len: 1,
            id: MessageId::Choke,
        }
    }
    /// Genera el mensaje unchoke
    fn generate_unchoke() -> Message {
        Message {
            len: 1,
            id: MessageId::Unchoke,
        }
    }

    /// Genera el mensaje interested
    fn generate_interested() -> Message {
        Message {
            len: 1,
            id: MessageId::Interested,
        }
    }

    /// Genera el mensaje not interested
    fn generate_not_interested() -> Message {
        Message {
            len: 1,
            id: MessageId::NotInterested,
        }
    }

    /// Convierte un array de 4 u8 a un u32
    pub fn convert_to_u32(bytes: &[u8]) -> u32 {
        let mut array: [u8; 4] = [0; 4];
        array[..4].clone_from_slice(&bytes[..4]);
        u32::from_be_bytes(array)
    }

    /// Genera el mensaje have.
    fn generate_have(bytes: Vec<u8>) -> Message {
        let index = Self::convert_to_u32(&bytes[1..5]);
        Message {
            len: 5,
            id: MessageId::Have(index),
        }
    }

    /// Genera el mensaje bitfield
    fn generate_bitfield(len: u32, bytes: Vec<u8>) -> Result<Message, PeerProtocolError> {
        Ok(Message {
            len,
            id: MessageId::Bitfield(bytes[1..].to_vec()),
        })
    }

    /// Genera el mensaje request
    fn generate_request(bytes: Vec<u8>) -> Message {
        let index = Self::convert_to_u32(&bytes[1..5]);
        let begin = Self::convert_to_u32(&bytes[5..9]);
        let length = Self::convert_to_u32(&bytes[9..]);
        Message {
            len: 13,
            id: MessageId::Request(index, begin, length),
        }
    }

    /// Genera el mensaje piece.
    fn generate_piece(len: u32, bytes: Vec<u8>) -> Result<Message, PeerProtocolError> {
        let index = Self::convert_to_u32(&bytes[1..5]);
        let begin = Self::convert_to_u32(&bytes[5..9]);
        let block = bytes[9..].to_vec();
        Ok(Message {
            len,
            id: MessageId::Piece(index, begin, block),
        })
    }

    /// Genera el mensaje cancel
    fn generate_cancel(bytes: Vec<u8>) -> Message {
        let index = Self::convert_to_u32(&bytes[1..5]);
        let begin = Self::convert_to_u32(&bytes[5..9]);
        let block = Self::convert_to_u32(&bytes[9..]);
        Message {
            len: 13,
            id: MessageId::Cancel(index, begin, block),
        }
    }

    fn equals(&self, message: Message) -> bool {
        self.len == message.len && self.id == message.id
    }

    pub fn send_keep_alive() -> Vec<u8> {
        vec![0, 0, 0, 0]
    }

    pub fn send_choke() -> Vec<u8> {
        vec![0, 0, 0, 1, 0]
    }

    pub fn send_unchoke() -> Vec<u8> {
        vec![0, 0, 0, 1, 1]
    }

    pub fn send_interested() -> Vec<u8> {
        vec![0, 0, 0, 1, 2]
    }

    pub fn send_not_interested() -> Vec<u8> {
        vec![0, 0, 0, 1, 3]
    }

    pub fn send_have(piece: u32) -> Vec<u8> {
        let mut byte_piece = u32::to_be_bytes(piece).to_vec();
        let mut vec: Vec<u8> = vec![0, 0, 0, 5, 4];
        vec.append(&mut byte_piece);
        vec
    }

    pub fn send_bitfield(bitfield: &mut Vec<u8>) -> Result<Vec<u8>, PeerProtocolError> {
        let len: u32 = (bitfield.len() + 1)
            .try_into()
            .or(Err(PeerProtocolError::FailToConvertError))?;
        let mut vec = u32::to_be_bytes(len).to_vec();
        vec.push(5);
        vec.append(bitfield);
        Ok(vec)
    }

    pub fn send_piece(
        index: u32,
        begin: u32,
        block: &mut Vec<u8>,
    ) -> Result<Vec<u8>, PeerProtocolError> {
        let len: u32 = (block.len() + 9)
            .try_into()
            .or(Err(PeerProtocolError::FailToConvertError))?;
        let mut vec = u32::to_be_bytes(len).to_vec();
        vec.push(7);
        let mut byte_index = u32::to_be_bytes(index).to_vec();
        let mut byte_begin = u32::to_be_bytes(begin).to_vec();
        vec.append(&mut byte_index);
        vec.append(&mut byte_begin);
        vec.append(block);
        Ok(vec)
    }

    pub fn send_request(index: u32, begin: u32, length: u32) -> Vec<u8> {
        let mut byte_index = u32::to_be_bytes(index).to_vec();
        let mut byte_begin = u32::to_be_bytes(begin).to_vec();
        let mut byte_length = u32::to_be_bytes(length).to_vec();
        let mut vec: Vec<u8> = vec![0, 0, 0, 13, 6];
        vec.append(&mut byte_index);
        vec.append(&mut byte_begin);
        vec.append(&mut byte_length);
        vec
    }

    pub fn send_cancel(index: u32, begin: u32, block: u32) -> Vec<u8> {
        let mut byte_index = u32::to_be_bytes(index).to_vec();
        let mut byte_begin = u32::to_be_bytes(begin).to_vec();
        let mut byte_block = u32::to_be_bytes(block).to_vec();
        let mut vec: Vec<u8> = vec![0, 0, 0, 13, 8];
        vec.append(&mut byte_index);
        vec.append(&mut byte_begin);
        vec.append(&mut byte_block);
        vec
    }
}

#[cfg(test)]
mod messages_should {
    use super::*;
    use std::vec;

    #[test]
    fn generate_keep_alive() {
        let keep_alive = Message::generate_keep_alive();
        assert_eq!(keep_alive.len, 0);
        assert_eq!(keep_alive.id, MessageId::KeepAlive);

        let expected = Message::new(0, vec![]).unwrap();
        assert_eq!(keep_alive.equals(expected), true);
    }

    #[test]
    fn generate_choke() {
        let choke = Message::generate_choke();
        assert_eq!(choke.len, 1);
        assert_eq!(choke.id, MessageId::Choke);

        let expected = Message::new(1, vec![0]).unwrap();
        assert_eq!(choke.equals(expected), true);
    }

    #[test]
    fn generate_unchoke() {
        let unchoke = Message::generate_unchoke();
        assert_eq!(unchoke.len, 1);
        assert_eq!(unchoke.id, MessageId::Unchoke);

        let expected = Message::new(1, vec![1]).unwrap();
        assert_eq!(unchoke.equals(expected), true);
    }

    #[test]
    fn generate_interested() {
        let interested = Message::generate_interested();
        assert_eq!(interested.len, 1);
        assert_eq!(interested.id, MessageId::Interested);
        let expected = Message::new(1, vec![2]).unwrap();
        assert_eq!(interested.equals(expected), true);
    }

    #[test]
    fn generate_not_interested() {
        let not_interested = Message::generate_not_interested();
        assert_eq!(not_interested.len, 1);
        assert_eq!(not_interested.id, MessageId::NotInterested);
        let expected = Message::new(1, vec![3]).unwrap();
        assert_eq!(not_interested.equals(expected), true);
    }

    #[test]
    fn fail_if_wrong_format() {
        assert_eq!(
            Message::new(1, [10].to_vec()).unwrap_err().to_string(),
            "El formato del mensaje no es valido"
        );
    }

    #[test]
    fn generate_have() {
        let message = Message::generate_have(vec![4, 0, 0, 1, 0]);
        assert_eq!(message.len, 5);
        assert_eq!(message.id, MessageId::Have(256));

        let expected = Message::new(5, vec![4, 0, 0, 1, 0]).unwrap();
        assert_eq!(message.equals(expected), true);
    }

    #[test]
    fn generate_request() {
        let message = Message::generate_request(vec![6, 0, 0, 1, 0, 1, 2, 3, 5, 0, 0, 0, 255]);
        assert_eq!(message.len, 13);
        assert_eq!(message.id, MessageId::Request(256, 16909061, 255));

        let expected = Message::new(13, vec![6, 0, 0, 1, 0, 1, 2, 3, 5, 0, 0, 0, 255]).unwrap();
        assert_eq!(message.equals(expected), true);
    }

    #[test]
    fn generate_cancel() {
        let message = Message::generate_cancel(vec![8, 0, 1, 0, 0, 1, 2, 3, 5, 6, 7, 8, 9]);
        assert_eq!(message.len, 13);
        assert_eq!(message.id, MessageId::Cancel(65536, 16909061, 101124105));

        let expected = Message::new(13, vec![8, 0, 1, 0, 0, 1, 2, 3, 5, 6, 7, 8, 9]).unwrap();
        assert_eq!(message.equals(expected), true);
    }

    #[test]
    fn generate_bitfield() {
        let message = Message::generate_bitfield(8, vec![5, 0, 0, 1, 0, 1, 2, 7]).unwrap();
        assert_eq!(message.len, 8);
        assert_eq!(message.id, MessageId::Bitfield(vec![0, 0, 1, 0, 1, 2, 7]));

        let expected = Message::new(8, vec![5, 0, 0, 1, 0, 1, 2, 7]).unwrap();
        assert_eq!(message.equals(expected), true);
    }

    #[test]
    fn generate_piece() {
        let message = Message::generate_piece(11, vec![7, 0, 0, 1, 0, 1, 2, 7, 4, 5, 6]).unwrap();
        assert_eq!(message.len, 11);
        assert_eq!(message.id, MessageId::Piece(256, 16910084, vec![5, 6]));

        let expected = Message::new(11, vec![7, 0, 0, 1, 0, 1, 2, 7, 4, 5, 6]).unwrap();
        assert_eq!(message.equals(expected), true);
    }

    #[test]
    fn send_fixed_len_messages() {
        assert_eq!(Message::send_keep_alive(), vec![0, 0, 0, 0]);
        assert_eq!(Message::send_choke(), vec![0, 0, 0, 1, 0]);
        assert_eq!(Message::send_unchoke(), vec![0, 0, 0, 1, 1]);
        assert_eq!(Message::send_interested(), vec![0, 0, 0, 1, 2]);
        assert_eq!(Message::send_not_interested(), vec![0, 0, 0, 1, 3]);
    }

    #[test]
    fn send_have() {
        let result = Message::send_have(5325234);
        let expected: Vec<u8> = vec![0, 0, 0, 5, 4, 0, 81, 65, 178];
        assert_eq!(result, expected);
    }

    #[test]
    fn send_request() {
        let result = Message::send_request(256, 512, 1024);
        let expected: Vec<u8> = vec![0, 0, 0, 13, 6, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 4, 0];
        assert_eq!(result, expected);
    }

    #[test]
    fn send_cancel() {
        let result = Message::send_cancel(1, 2, 3);
        let expected: Vec<u8> = vec![0, 0, 0, 13, 8, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3];
        assert_eq!(result, expected);
    }

    #[test]
    fn send_bitfield() {
        let mut vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        let result = Message::send_bitfield(&mut vec).unwrap();
        let expected: Vec<u8> = vec![
            0, 0, 0, 16, 5, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn send_piece() {
        let mut vec = vec![3, 2, 3, 4, 5, 6];
        let result = Message::send_piece(1, 2, &mut vec).unwrap();
        let expected: Vec<u8> = vec![0, 0, 0, 15, 7, 0, 0, 0, 1, 0, 0, 0, 2, 3, 2, 3, 4, 5, 6];
        assert_eq!(result, expected);
    }
}
