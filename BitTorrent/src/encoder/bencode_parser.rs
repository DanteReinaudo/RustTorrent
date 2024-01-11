use crate::encoder::errors::DecodingError;
use std::fmt::Debug;

/******************************************************************************************/
/*                                 DECODING PARSER                                        */
/******************************************************************************************/

#[allow(dead_code)]
pub struct DecodingParser;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Bencode {
    Int(i64),
    String(String),
    ByteString(Vec<u8>),
    List(Vec<Bencode>),
    Dictionary(Vec<(String, Bencode)>),
}

#[allow(dead_code)]
impl Bencode {
    /// Esta funcion imprime el Bencode.
    pub fn print(&self) {
        self.println("");
    }

    /// Esta funcion recibe un una indentacion como &str e imprime el Bencode en formato legible por pantalla.
    fn println(&self, identation: &str) {
        if let Bencode::String(string) = self {
            println!("{}{}", identation, string);
        }
        if let Bencode::ByteString(bytestring) = self {
            println!("{}{:?}", identation.to_owned() + "  ", bytestring);
        }
        if let Bencode::Int(int) = self {
            println!("{}{}", identation, int);
        }
        if let Bencode::Dictionary(dict) = self {
            println!("{}d", identation);
            let new_identation = identation.to_owned() + "    ";
            let value_identation = new_identation.to_owned() + "    ";
            for (key, value) in dict {
                println!("{}Key: {}", new_identation, key);
                println!("{}Values: ", new_identation);
                value.println(&value_identation);
            }
            println!("{}e", identation);
        }
        if let Bencode::List(list) = self {
            println!("{}l", identation);
            let new_identation = identation.to_owned() + "    ";
            for element in list {
                element.println(&new_identation);
            }
            println!("{}e", identation);
        }
    }
}

type Result<T> = std::result::Result<T, DecodingError>;

#[allow(dead_code)]
impl DecodingParser {
    /// Esta funcion recibe un &[u8] numero codificado en formato de Bencoding y lo decodifica,
    /// devolviendo el i64 numero.
    fn decode_number(&self, byte_string: &[u8]) -> Result<i64> {
        let string = String::from_utf8(byte_string.to_vec())?;
        let string_number = string.replace(&['i', 'e'], "");
        let number = (string_number.parse::<i64>())?;
        Ok(number)
    }

    /// Esta funcion recibe un &[u8] string codificado en formato de Bencoding y lo decodifica,
    /// devolviendo el string como un campo del enumerativo Bencode.
    fn decode_string(&self, byte_string: &[u8]) -> Result<Bencode> {
        let mut init = 0;
        while (byte_string[init] as char) != ':' {
            init += 1;
        }
        let vec = byte_string[init + 1..].to_vec();
        let string_result = String::from_utf8(vec.clone());
        match string_result {
            Ok(s) => Ok(Bencode::String(s)),
            Err(_e) => Ok(Bencode::ByteString(vec)),
        }
    }
    /// Esta funcion recibe una &[u8] Lista codificada en formato de Bencoding y la decodifica,
    /// devolviendo la Lista como vector de campos del enumerativo Bencode.
    fn decode_list(&self, byte_string: &[u8]) -> Result<Vec<Bencode>> {
        let mut init: usize = 1;
        let bencoded_list = self
            .string_splitter(byte_string, &mut init)
            .or(Err(DecodingError::InvalidSyntaxError))?;
        Ok(bencoded_list)
    }

    /// Esta funcion recibe un &[u8] Diccionario codificada en formato de Bencoding y la decodifica,
    /// devolviendo el Diccionario como vector de tuplas de strings y campos del enumerativo Bencode.
    fn decode_dict(&self, byte_string: &[u8]) -> Result<Vec<(String, Bencode)>> {
        let mut init: usize = 1;
        let bencoded_list = self
            .string_splitter(byte_string, &mut init)
            .or(Err(DecodingError::InvalidSyntaxError))?;
        let bencoded_dict =
            Self::make_dict(bencoded_list).or(Err(DecodingError::InvalidSyntaxError))?;
        Ok(bencoded_dict)
    }

    /// Esta funcion recibe un &[u8] caracter y encuentra si se
    /// trata del caracter final de la codificacion del Bencode de Numero.
    /// Devuelve un usize con la cantidad de dígitos del número.
    fn find_end_of_int(&self, byte_string: &[u8], init: usize) -> usize {
        let mut end_int = init;
        while (byte_string[end_int] as char) != 'e' {
            end_int += 1;
        }
        end_int
    }

    /// Esta funcion recibe un &[u8] numero y encuentra si se
    /// trata del numero que da final a la codificacion del Bencode de un String.
    /// Devuelve un usize con el largo de la palabra.
    fn find_end_of_string(&self, byte_string: &[u8], init: usize) -> Result<usize> {
        let mut end_number = init;
        while (byte_string[end_number] as char).is_digit(10) {
            end_number += 1
        }
        let string = String::from_utf8(byte_string[init..end_number].to_vec())?;
        let word_length = string.parse::<usize>()?;
        let end_string = end_number + word_length;
        Ok(end_string)
    }

    /// Esta funcion recibe un &[u8] vector codificada en formato de Bencoding y la divide segun cooresponda
    /// en los distintos tipos de campos del enumerativo Bencode.
    /// Devuelve un vector con todos estos campos Bencode adjuntos.
    fn string_splitter(&self, byte_string: &[u8], end: &mut usize) -> Result<Vec<Bencode>> {
        let mut stack: Vec<Bencode> = vec![];
        let mut i = 1;
        while i < byte_string.len() {
            match byte_string[i] as char {
                'd' => {
                    let sub_list =
                        self.string_splitter(&byte_string[i..byte_string.len() - 1], end)?;
                    let sub_dict = Self::make_dict(sub_list)?;
                    stack.push(Bencode::Dictionary(sub_dict));
                    i += *end + 1;
                }
                'l' => {
                    let sub_list =
                        self.string_splitter(&byte_string[i..byte_string.len() - 1], end)?;
                    stack.push(Bencode::List(sub_list));
                    i += *end + 1;
                }
                'i' => {
                    let end_int = self.find_end_of_int(byte_string, i);
                    stack.push(Bencode::Int(self.decode_number(&byte_string[i..end_int])?));
                    i = end_int + 1;
                }
                '0'..='9' => {
                    let end_string = self.find_end_of_string(byte_string, i)?;
                    let decode_byte_string = self.decode_string(&byte_string[i..end_string + 1])?;
                    stack.push(decode_byte_string);
                    i = end_string + 1;
                }
                'e' => {
                    *end = i;
                    break;
                }
                _ => return Err(DecodingError::InvalidSyntaxError),
            }
        }
        Ok(stack)
    }

    /// Esta funcion recibe un string de tipo String y lo devuelve codificado en formato de Bencoding.
    pub fn decode_from_string(&self, string: String) -> Result<Bencode> {
        return self.decode_from_u8(string.as_bytes().to_vec());
    }

    /// Esta funcion recibe un Vec<u8> y lo devuelve codificado en formato de Bencode.
    pub fn decode_from_u8(&self, byte_string: Vec<u8>) -> Result<Bencode> {
        match byte_string[0] as char {
            'd' => {
                let bencoded_dict = self
                    .decode_dict(&byte_string)
                    .or(Err(DecodingError::InvalidSyntaxError))?;
                Ok(Bencode::Dictionary(bencoded_dict))
            }
            'l' => {
                let bencoded_list = self
                    .decode_list(&byte_string)
                    .or(Err(DecodingError::InvalidSyntaxError))?;
                Ok(Bencode::List(bencoded_list))
            }
            'i' => {
                let bencoded_int = self
                    .decode_number(&byte_string)
                    .or(Err(DecodingError::InvalidSyntaxError))?;
                Ok(Bencode::Int(bencoded_int))
            }
            '0'..='9' => {
                let bencoded_string = self
                    .decode_string(&byte_string)
                    .or(Err(DecodingError::InvalidSyntaxError))?;
                Ok(bencoded_string)
            }
            _ => Err(DecodingError::InvalidSyntaxError),
        }
    }

    /// Esta funcion recibe una lista de vectores de campos Bencode Vec<Bencode>
    /// y arma un Diccionario a partir de la misma.
    /// Devuelve el Diccionario como vector de tuplas de strings y campos del enumerativo Bencode.
    fn make_dict(list: Vec<Bencode>) -> Result<Vec<(String, Bencode)>> {
        let mut dict: Vec<(String, Bencode)> = vec![];
        let mut i = 0;
        while i < (list.len() - 1) {
            if let Bencode::String(s) = &list[i] {
                dict.push((s.clone(), list[i + 1].clone()));
            }
            i += 2;
        }
        Ok(dict)
    }
}

/******************************************************************************************/
/*                                        TESTS                                           */
/******************************************************************************************/

#[cfg(test)]
mod decoding_parser_should {
    use super::Bencode;
    use super::DecodingParser;

    #[test]
    fn decode_number() {
        let parser = DecodingParser;
        assert_eq!(
            parser.decode_from_string("i27e".to_string()).unwrap(),
            Bencode::Int(27)
        );
    }

    #[test]
    fn decode_string() {
        let parser = DecodingParser;
        assert_eq!(
            parser
                .decode_from_string("33:Debian CD from cdimage.debian.org".to_string())
                .unwrap(),
            Bencode::String("Debian CD from cdimage.debian.org".to_string())
        );
    }

    #[test]
    fn decode_simple_string() {
        let parser = DecodingParser;
        let string = String::from("4:spam");
        let string1 = String::from("spam");
        assert_eq!(
            parser.decode_from_string(string).unwrap(),
            Bencode::String(string1)
        )
    }

    #[test]
    fn decode_simple_number() {
        let parser = DecodingParser;
        let string = String::from("i4e");
        assert_eq!(parser.decode_from_string(string).unwrap(), Bencode::Int(4))
    }

    #[test]
    fn decode_list() {
        let parser = DecodingParser;
        let string = String::from("l3:aaai2ee");
        let string1 = String::from("aaa");
        let mut array: Vec<Bencode> = vec![];
        array.push(Bencode::String(string1));
        array.push(Bencode::Int(2));
        assert_eq!(
            parser.decode_from_string(string).unwrap(),
            Bencode::List(array)
        );
    }

    #[test]
    fn decode_list_of_dicts() {
        let parser = DecodingParser;
        let string = String::from("ld3:cow3:moo4:spam4:eggsed4:cayo5:nocheee");
        let mut dict1: Vec<(String, Bencode)> = vec![];
        dict1.push((String::from("cow"), Bencode::String(String::from("moo"))));
        dict1.push((String::from("spam"), Bencode::String(String::from("eggs"))));
        let mut dict2 = vec![];
        dict2.push((String::from("cayo"), Bencode::String(String::from("noche"))));
        let mut array: Vec<Bencode> = vec![];
        array.push(Bencode::Dictionary(dict1));
        array.push(Bencode::Dictionary(dict2));
        assert_eq!(
            parser.decode_from_string(string).unwrap(),
            Bencode::List(array)
        );
    }

    #[test]
    fn decode_dict() {
        let parser = DecodingParser;
        let string = String::from("d3:cow3:moo4:spam4:eggse");
        let mut dict: Vec<(String, Bencode)> = vec![];
        dict.push((String::from("cow"), Bencode::String(String::from("moo"))));
        dict.push((String::from("spam"), Bencode::String(String::from("eggs"))));
        assert_eq!(
            parser.decode_from_string(string).unwrap(),
            Bencode::Dictionary(dict)
        );
    }

    #[test]
    fn decode_dict_of_ints() {
        let parser = DecodingParser;
        let string = String::from("d3:cowi10e4:listi0ee");
        let mut dict: Vec<(String, Bencode)> = vec![];
        dict.push((String::from("cow"), Bencode::Int(10)));
        dict.push((String::from("list"), Bencode::Int(0)));
        assert_eq!(
            parser.decode_from_string(string).unwrap(),
            Bencode::Dictionary(dict)
        );
    }

    #[test]
    fn decode_dict_of_lists() {
        let parser = DecodingParser;
        let string = String::from("d3:cowle4:listl4:spam4:eggsee");
        let array1: Vec<Bencode> = vec![];
        let array2: Vec<Bencode> = vec![
            Bencode::String(String::from("spam")),
            Bencode::String(String::from("eggs")),
        ];
        let mut dict: Vec<(String, Bencode)> = vec![];
        dict.push((String::from("cow"), Bencode::List(array1)));
        dict.push((String::from("list"), Bencode::List(array2)));
        assert_eq!(
            parser.decode_from_string(string).unwrap(),
            Bencode::Dictionary(dict)
        );
    }

    #[test]
    fn decode_dict_of_dicts() {
        let parser = DecodingParser;
        let string = String::from("d4:cayod2:la5:nocheee");
        let mut dict1: Vec<(String, Bencode)> = vec![];
        dict1.push((String::from("la"), Bencode::String(String::from("noche"))));
        let mut dict2: Vec<(String, Bencode)> = vec![];
        dict2.push((String::from("cayo"), Bencode::Dictionary(dict1)));
        assert_eq!(
            parser.decode_from_string(string).unwrap(),
            Bencode::Dictionary(dict2)
        );
    }

    #[test]
    fn decode_number_from_u8() {
        let parser = DecodingParser;
        assert_eq!(parser.decode_number("i27e".as_bytes()).unwrap(), 27);
    }

    #[test]
    fn decode_negative_number_u8() {
        let parser = DecodingParser;
        assert_eq!(parser.decode_number("i-3e".as_bytes()).unwrap(), -3);
    }

    #[test]
    fn decode_string_from_u8() {
        let parser = DecodingParser;
        assert_eq!(
            parser
                .decode_string("33:Debian CD from cdimage.debian.org".as_bytes())
                .unwrap(),
            Bencode::String(String::from("Debian CD from cdimage.debian.org"))
        );
    }

    #[test]
    fn split_numbers_list_u8() {
        let parser = DecodingParser;
        let string = String::from("li3ei43ee");
        let mut init = 1;
        assert_eq!(
            parser
                .string_splitter(string.as_bytes(), &mut init)
                .unwrap(),
            [Bencode::Int(3), Bencode::Int(43)]
        );
    }

    #[test]
    fn split_strings_list_u8() {
        let parser = DecodingParser;
        let string = String::from("l3:aaa2:aae");
        let string1 = String::from("aaa");
        let string2 = String::from("aa");
        let mut init = 1;
        assert_eq!(
            parser
                .string_splitter(string.as_bytes(), &mut init)
                .unwrap(),
            [Bencode::String(string1), Bencode::String(string2)]
        );
    }

    #[test]
    fn fail_if_invalid_list_u8() {
        let parser = DecodingParser;
        let string = String::from("l3:aa2:aae");
        let mut init = 1;
        assert_eq!(
            parser
                .string_splitter(string.as_bytes(), &mut init)
                .unwrap_err()
                .to_string(),
            "Sintaxis invalida"
        );
    }

    #[test]
    fn split_strings_dict_u8() {
        let parser = DecodingParser;
        let string = String::from("d3:aaa2:aae");
        let string1 = String::from("aaa");
        let string2 = String::from("aa");
        let mut init = 1;
        assert_eq!(
            parser
                .string_splitter(string.as_bytes(), &mut init)
                .unwrap(),
            [Bencode::String(string1), Bencode::String(string2)]
        );
    }

    #[test]
    fn fail_if_invalid_dict_u8() {
        let parser = DecodingParser;
        let string = String::from("d3:a2:aae");
        let mut init = 1;
        assert_eq!(
            parser
                .string_splitter(string.as_bytes(), &mut init)
                .unwrap_err()
                .to_string(),
            "Sintaxis invalida"
        );
    }

    #[test]
    fn split_number_and_strings_list_u8() {
        let parser = DecodingParser;
        let string = String::from("l3:aaai2ee");
        let string1 = String::from("aaa");
        let mut init = 1;
        assert_eq!(
            parser
                .string_splitter(string.as_bytes(), &mut init)
                .unwrap(),
            [Bencode::String(string1), Bencode::Int(2)]
        );
    }

    #[test]
    fn split_list_of_lists_u8() {
        let parser = DecodingParser;
        let string = String::from("l1:ali2e2:aaee");
        let string1 = String::from("aa");
        let string2 = String::from("a");
        let mut array: Vec<Bencode> = vec![];
        array.push(Bencode::Int(2));
        array.push(Bencode::String(string1));
        let mut init = 1;
        assert_eq!(
            parser
                .string_splitter(string.as_bytes(), &mut init)
                .unwrap(),
            [Bencode::String(string2), Bencode::List(array)]
        );
    }

    #[test]
    fn decode_simple_string_u8() {
        let parser = DecodingParser;
        let string = String::from("4:spam");
        let string1 = String::from("spam");
        assert_eq!(
            parser.decode_from_u8(string.as_bytes().to_vec()).unwrap(),
            Bencode::String(string1)
        )
    }

    #[test]
    fn decode_simple_number_u8() {
        let parser = DecodingParser;
        let vec = String::from("i4e").as_bytes().to_vec();
        assert_eq!(parser.decode_from_u8(vec).unwrap(), Bencode::Int(4))
    }

    #[test]
    fn decode_list_from_u8() {
        let parser = DecodingParser;
        let vec = String::from("l3:aaai2ee").as_bytes().to_vec();
        let string1 = String::from("aaa");
        let mut array: Vec<Bencode> = vec![];
        array.push(Bencode::String(string1));
        array.push(Bencode::Int(2));
        assert_eq!(parser.decode_from_u8(vec).unwrap(), Bencode::List(array));
    }

    #[test]
    fn decode_list_of_dicts_u8() {
        let parser = DecodingParser;
        let vec = String::from("ld3:cow3:moo4:spam4:eggsed4:cayo5:nocheee")
            .as_bytes()
            .to_vec();
        let mut dict1 = vec![];
        dict1.push((String::from("cow"), Bencode::String(String::from("moo"))));
        dict1.push((String::from("spam"), Bencode::String(String::from("eggs"))));
        let mut dict2 = vec![];
        dict2.push((String::from("cayo"), Bencode::String(String::from("noche"))));
        let mut array: Vec<Bencode> = vec![];
        array.push(Bencode::Dictionary(dict1));
        array.push(Bencode::Dictionary(dict2));
        assert_eq!(parser.decode_from_u8(vec).unwrap(), Bencode::List(array));
    }

    #[test]
    fn decode_dict_from_u8() {
        let parser = DecodingParser;
        let vec = String::from("d3:cow3:moo4:spam4:eggse").as_bytes().to_vec();
        let mut dict = vec![];
        dict.push((String::from("cow"), Bencode::String(String::from("moo"))));
        dict.push((String::from("spam"), Bencode::String(String::from("eggs"))));
        assert_eq!(
            parser.decode_from_u8(vec).unwrap(),
            Bencode::Dictionary(dict)
        );
    }

    #[test]
    fn decode_dict_of_ints_u8() {
        let parser = DecodingParser;
        let vec = String::from("d3:cowi10e4:listi0ee").as_bytes().to_vec();
        let mut dict = vec![];
        dict.push((String::from("cow"), Bencode::Int(10)));
        dict.push((String::from("list"), Bencode::Int(0)));
        assert_eq!(
            parser.decode_from_u8(vec).unwrap(),
            Bencode::Dictionary(dict)
        );
    }

    #[test]
    fn decode_dict_of_lists_u8() {
        let parser = DecodingParser;
        let vec = String::from("d3:cowle4:listl4:spam4:eggsee")
            .as_bytes()
            .to_vec();
        let array1: Vec<Bencode> = vec![];
        let array2: Vec<Bencode> = vec![
            Bencode::String(String::from("spam")),
            Bencode::String(String::from("eggs")),
        ];
        let mut dict = vec![];
        dict.push((String::from("cow"), Bencode::List(array1)));
        dict.push((String::from("list"), Bencode::List(array2)));
        assert_eq!(
            parser.decode_from_u8(vec).unwrap(),
            Bencode::Dictionary(dict)
        );
    }

    #[test]
    fn decode_dict_of_dicts_u8() {
        let parser = DecodingParser;
        let vec = String::from("d4:cayod2:la5:nocheee").as_bytes().to_vec();
        let mut dict1 = vec![];
        dict1.push((String::from("la"), Bencode::String(String::from("noche"))));
        let mut dict2 = vec![];
        dict2.push((String::from("cayo"), Bencode::Dictionary(dict1)));
        assert_eq!(
            parser.decode_from_u8(vec).unwrap(),
            Bencode::Dictionary(dict2)
        );
    }
}
