use super::errors::DecodingError;
use std::fmt::Write;

#[allow(dead_code)]
pub struct URLEncoder;

#[allow(dead_code)]
impl URLEncoder {
    /// Esta funcion recibe un Vec<u8> y devuelve el mismo encodeada con urlencode.
    /// En caso de haber un error al codificar el vector devuelve decoding error.
    pub fn urlencode(&self, string: Vec<u8>) -> Result<String, DecodingError> {
        let mut encoded = String::from("");
        for byte in string {
            match byte {
                b'A'..=b'Z' => encoded.push(byte as char),
                b'a'..=b'z' => encoded.push(byte as char),
                b'0'..=b'9' => encoded.push(byte as char),
                b'-' | b'_' | b'.' | b',' | b'~' => encoded.push(byte as char),

                _ => write!(&mut encoded, "%{:02x}", byte)
                    .or(Err(DecodingError::URLEncodingError))?,
            }
        }
        Ok(encoded)
    }
}

/******************************************************************************************/
/*                                        TESTS                                           */
/******************************************************************************************/

#[cfg(test)]
mod client_should {
    use super::*;

    #[test]
    fn urlencode_sentences() {
        let encoder = URLEncoder;
        let expected = String::from("This%20string%20will%20be%20URL%20encoded.");
        let urlencoded_sentence = encoder
            .urlencode("This string will be URL encoded.".as_bytes().to_vec())
            .unwrap();
        assert_eq!(urlencoded_sentence, expected);
    }

    #[test]
    fn urlencode_abc_upper() {
        let encoder = URLEncoder;
        let expected = String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        let urlencoded_sentence = encoder
            .urlencode("ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes().to_vec())
            .unwrap();
        assert_eq!(urlencoded_sentence, expected);
    }

    #[test]
    fn urlencode_abc_lower() {
        let encoder = URLEncoder;
        let expected = String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_lowercase());
        let urlencoded_sentence = encoder
            .urlencode(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .to_lowercase()
                    .as_bytes()
                    .to_vec(),
            )
            .unwrap();
        assert_eq!(urlencoded_sentence, expected);
    }

    #[test]
    fn urlencode_numbers() {
        let encoder = URLEncoder;
        let expected = String::from("0123456789");
        let urlencoded_sentence = encoder.urlencode("0123456789".as_bytes().to_vec()).unwrap();
        assert_eq!(urlencoded_sentence, expected);
    }

    #[test]
    fn urlencode_special_chars() {
        let encoder = URLEncoder;
        let expected = String::from("-_.,~");
        let urlencoded_sentence = encoder.urlencode("-_.,~".as_bytes().to_vec()).unwrap();
        assert_eq!(urlencoded_sentence, expected);
    }

    #[test]
    fn urlencode_space() {
        let encoder = URLEncoder;
        let expected = String::from("%20");
        let urlencoded_sentence = encoder.urlencode(" ".as_bytes().to_vec()).unwrap();
        assert_eq!(urlencoded_sentence, expected);
    }

    #[test]
    fn urlencode_id() {
        let encoder = URLEncoder;
        let expected = String::from("-4R0001-D23T25F26S27");
        let urlencoded_sentence = encoder
            .urlencode("-4R0001-D23T25F26S27".as_bytes().to_vec())
            .unwrap();
        assert_eq!(urlencoded_sentence, expected);
    }
}
