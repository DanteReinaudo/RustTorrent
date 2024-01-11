use crate::encoder::bencode_parser::Bencode;

pub struct EncodingParser;
#[allow(dead_code)]
impl EncodingParser {
    /// Esta funcion recibe un string de tipo String y la codifica en formato de Bencoding,
    /// devolviendo el string bencodeado como un Vec<u8>
    fn encode_string(&self, string: String) -> Vec<u8> {
        let len = string.len().to_string();
        (len + ":" + &string).as_bytes().to_vec()
    }

    /// Esta funcion recibe un string como Vec<u8> y la codifica en formato de Bencoding,
    /// devolviendo el string bencodeado como un Vec<u8>
    fn encode_byte_string(&self, bytestring: Vec<u8>) -> Vec<u8> {
        let mut vec: Vec<u8> = bytestring.len().to_string().as_bytes().to_vec();
        vec.push(b':');
        for element in bytestring {
            vec.push(element);
        }
        vec
    }

    /// Esta funcion recibe un i64 numero y lo codifica en formato de Bencoding,
    /// devolviendo el numero bencodeado como un Vec<u8>
    fn encode_number(&self, number: i64) -> Vec<u8> {
        let str = String::from("i");
        (str + &*number.to_string() + "e").as_bytes().to_vec()
    }

    /// Esta funcion recibe una lista como vector de Bencodes y la codifica en formato de Bencoding,
    /// devolviendo la Lista bencodeado como un Vec<u8>
    pub fn encode_list(&self, list: Vec<Bencode>) -> Vec<u8> {
        let mut vec: Vec<u8> = "l".as_bytes().to_vec();
        for bencode in list {
            let result = self.encode(bencode);
            for byte in result {
                vec.push(byte);
            }
        }
        vec.push(b'e');
        vec
    }

    /// Esta funcion recibe un Diccionario como vector de tuplas de Strings y Bencodes
    /// y lo codifica en formato de Bencoding.
    /// devolviendo el diccionario bencodeado como un Vec<u8>
    fn encode_dict(&self, dict: Vec<(String, Bencode)>) -> Vec<u8> {
        let mut vec: Vec<u8> = "d".as_bytes().to_vec();
        for (key, value) in &dict {
            let key_encode = self.encode_string(key.to_string());
            for key_byte in key_encode {
                vec.push(key_byte);
            }
            let value_encode = self.encode(value.clone());
            for value_byte in value_encode {
                vec.push(value_byte);
            }
        }
        vec.push(b'e');
        vec
    }

    /// Esta funcion recibe un enumerativo Bencode
    /// y llama a la funcion que corresponde en funcion a que campo del enumerativo se trata
    pub fn encode(&self, bencode: Bencode) -> Vec<u8> {
        match bencode {
            Bencode::String(string) => self.encode_string(string),
            Bencode::ByteString(bytestring) => self.encode_byte_string(bytestring),
            Bencode::Int(number) => self.encode_number(number),
            Bencode::Dictionary(dict) => self.encode_dict(dict),
            Bencode::List(list) => self.encode_list(list),
        }
    }
}

/******************************************************************************************/
/*                                        TESTS                                           */
/******************************************************************************************/

#[cfg(test)]
mod decoding_parser_should {
    use super::*;
    use crate::encoder::bencode_parser::DecodingParser;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn encode_string() {
        let parser = EncodingParser;
        assert_eq!(
            parser.encode_string(String::from("hola")),
            "4:hola".as_bytes()
        );
    }

    #[test]
    fn encode_byte_string() {
        let parser = EncodingParser;
        assert_eq!(
            parser.encode_byte_string("mundo".as_bytes().to_vec()),
            "5:mundo".as_bytes()
        );
    }

    #[test]
    fn encode_number() {
        let parser = EncodingParser;
        assert_eq!(parser.encode_number(32), "i32e".as_bytes());
    }

    #[test]
    fn encode_negative_number() {
        let parser = EncodingParser;
        assert_eq!(parser.encode_number(-3), "i-3e".as_bytes());
    }
    #[test]
    fn encode_zero_number() {
        let parser = EncodingParser;
        assert_eq!(parser.encode_number(0), "i0e".as_bytes());
    }

    #[test]
    fn encode_list() {
        let parser = EncodingParser;
        let mut vec: Vec<Bencode> = vec![];
        vec.push(Bencode::String(String::from("spam")));
        vec.push(Bencode::String(String::from("eggs")));
        assert_eq!(parser.encode_list(vec), "l4:spam4:eggse".as_bytes());
    }

    #[test]
    fn encode_dict() {
        let parser = EncodingParser;
        let mut dict = vec![];
        dict.push((String::from("cow"), Bencode::String(String::from("moo"))));
        dict.push((String::from("spam"), Bencode::String(String::from("eggs"))));
        assert_eq!(
            parser.encode_dict(dict),
            "d3:cow3:moo4:spam4:eggse".as_bytes()
        );
    }

    #[test]
    fn encode_file() {
        let torrent_path = "./torrents/example.torrent";
        let mut file = File::open(torrent_path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let decoded_data: Bencode = DecodingParser.decode_from_u8(data.clone()).unwrap();
        let encoded_data = EncodingParser.encode(decoded_data);
        assert_eq!(data, encoded_data);
    }
}
