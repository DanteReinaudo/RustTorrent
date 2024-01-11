use crate::bitfield::errors::BitfieldError;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Bitfield {
    bitmap: Vec<u8>,
    pub len: usize,
    pub last_byte: usize,
}

#[allow(dead_code)]
impl Bitfield {
    pub fn new(len: usize) -> Bitfield {
        let bitmap_len: usize = ((len as f32) / 8.0).ceil() as usize;
        let bitmap: Vec<u8> = vec![0; bitmap_len];
        let last_byte = len % 8; //bytes que usa el ultimo bit
        Bitfield {
            bitmap,
            len,
            last_byte,
        }
    }

    pub fn from(bitmap: Vec<u8>, last_byte: usize) -> Bitfield {
        Bitfield {
            bitmap: bitmap.clone(),
            len: bitmap.len() * 8,
            last_byte,
        }
    }

    fn byte_position(pos: usize) -> usize {
        ((pos as f32) / 8.0).floor() as usize
    }

    fn offset_position(pos: usize) -> u8 {
        (pos % 8) as u8
    }

    fn offset_to_num(offset: u8) -> u8 {
        match offset {
            7 => 1,
            6 => 2,
            5 => 4,
            4 => 8,
            3 => 16,
            2 => 32,
            1 => 64,
            0 => 128,
            _ => 0,
        }
    }

    pub fn add_piece(&mut self, pos: usize) -> Result<bool, BitfieldError> {
        if pos >= self.len {
            return Err(BitfieldError::InvalidPositionError);
        };
        let have = self.have_piece(pos)?;
        if have {
            return Ok(false);
        };
        let position = Self::byte_position(pos);
        let offset = Self::offset_position(pos);
        self.bitmap[position] += Self::offset_to_num(offset);
        Ok(true)
    }

    pub fn remove_piece(&mut self, pos: usize) -> Result<bool, BitfieldError> {
        if pos > self.len {
            return Err(BitfieldError::InvalidPositionError);
        };
        let have = self.have_piece(pos)?;
        if !have {
            return Ok(false);
        };
        let position = Self::byte_position(pos);
        let offset = Self::offset_position(pos);
        self.bitmap[position] -= Self::offset_to_num(offset);
        Ok(true)
    }

    fn u8_to_binary(byte: u8) -> String {
        let binary_string = format!("{:b}", byte);
        let len = binary_string.len();
        let range = 8 - len;
        let mut string = String::from("");
        for _i in 0..range {
            string.push('0')
        }
        string.push_str(&binary_string);
        string
    }

    fn binary_have_offset(binary: String, offset: u8) -> bool {
        let binary_vec = binary.as_bytes();
        binary_vec[offset as usize] == 49
    }

    pub fn have_piece(&self, pos: usize) -> Result<bool, BitfieldError> {
        if pos >= self.len {
            return Err(BitfieldError::InvalidPositionError);
        }
        let position = Self::byte_position(pos);
        let offset = Self::offset_position(pos);
        let byte = self.bitmap[position];
        let binary = Self::u8_to_binary(byte);
        Ok(Self::binary_have_offset(binary, offset))
    }

    pub fn compare_bitmap(&self, bitfield: Bitfield) -> bool {
        bitfield.len != self.len
            && bitfield.bitmap != self.bitmap
            && self.last_byte == bitfield.last_byte
    }

    pub fn get_bitmap(&self) -> Vec<u8> {
        self.bitmap.clone()
    }
}

#[cfg(test)]
mod bitfield_should {
    use super::*;

    #[test]
    fn initialize_from_len() {
        let len: usize = 10;
        let bitfield = Bitfield::new(len);

        assert_eq!(bitfield.len, 10);
        assert_eq!(bitfield.bitmap, [0; 2]);
        assert_eq!(bitfield.last_byte, 2);
    }

    #[test]
    fn initialize_from_bitmap() {
        let bitmap = vec![255; 1];
        let bitfield = Bitfield::from(bitmap, 0);
        assert_eq!(bitfield.len, 8);
        assert_eq!(bitfield.bitmap, [255; 1]);
        assert_eq!(bitfield.last_byte, 0);
    }

    #[test]
    fn have_some_piece() {
        let len: usize = 64;
        let mut bitfield = Bitfield::new(len);
        bitfield.add_piece(15).unwrap();
        assert_eq!(bitfield.have_piece(15).unwrap(), true);
        assert_eq!(bitfield.have_piece(16).unwrap(), false);
    }

    #[test]
    fn add_piece_out_of_index() {
        let len: usize = 32;
        let mut bitfield = Bitfield::new(len);
        assert_eq!(
            bitfield.add_piece(33).unwrap_err().to_string(),
            "Se intento acceder a una posicion invalida"
        );
    }

    #[test]
    fn add_piece_with_offset() {
        let len: usize = 32;
        let mut bitfield = Bitfield::new(len);
        assert_eq!(bitfield.add_piece(19).unwrap(), true);
        assert_eq!(bitfield.bitmap, vec![0, 0, 16, 0]);
    }

    #[test]
    fn add_piece_with_zero_offset() {
        let len: usize = 16;
        let mut bitfield = Bitfield::new(len);
        assert_eq!(bitfield.add_piece(13).unwrap(), true);
        assert_eq!(bitfield.bitmap, vec![0, 4]);
    }

    #[test]
    fn add_piece_twice() {
        let len: usize = 16;
        let mut bitfield = Bitfield::new(len);
        assert_eq!(bitfield.add_piece(10).unwrap(), true);
        assert_eq!(bitfield.add_piece(10).unwrap(), false);
        assert_eq!(bitfield.bitmap, vec![0, 32]);
    }

    #[test]
    fn have_piece() {
        let len: usize = 64;
        let mut bitfield = Bitfield::new(len);
        bitfield.add_piece(40).unwrap();
        assert_eq!(bitfield.have_piece(40).unwrap(), true);
    }

    #[test]
    fn have_piece_is_false() {
        let len: usize = 64;
        let mut bitfield = Bitfield::new(len);
        bitfield.add_piece(40).unwrap();
        assert_eq!(bitfield.have_piece(35).unwrap(), false);
    }

    #[test]
    fn have_piece_out_of_index() {
        let len: usize = 8;
        let bitfield = Bitfield::new(len);
        assert_eq!(
            bitfield.have_piece(9).unwrap_err().to_string(),
            "Se intento acceder a una posicion invalida"
        );
    }

    #[test]
    fn remove_piece() {
        let len: usize = 32;
        let mut bitfield = Bitfield::new(len);
        bitfield.add_piece(30).unwrap();
        bitfield.add_piece(31).unwrap();
        assert_eq!(bitfield.remove_piece(30).unwrap(), true);
        assert_eq!(bitfield.bitmap, vec![0, 0, 0, 1]);
        assert_eq!(bitfield.remove_piece(31).unwrap(), true);
        assert_eq!(bitfield.bitmap, vec![0, 0, 0, 0]);
    }

    #[test]
    fn remove_piece_out_of_index() {
        let len: usize = 32;
        let mut bitfield = Bitfield::new(len);
        assert_eq!(
            bitfield.remove_piece(33).unwrap_err().to_string(),
            "Se intento acceder a una posicion invalida"
        );
    }

    #[test]
    fn remove_non_existent_piece() {
        let len: usize = 64;
        let mut bitfield = Bitfield::new(len);
        bitfield.add_piece(40).unwrap();
        assert_eq!(bitfield.remove_piece(23).unwrap(), false);
    }
}
