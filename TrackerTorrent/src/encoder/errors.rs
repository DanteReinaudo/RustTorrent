use std::fmt;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

/******************************************************************************************/
/*                                  DECODING ERROR                                        */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum DecodingError {
    CannotParseError(ParseIntError),
    InvalidUTF8CharError(FromUtf8Error),
    InvalidSyntaxError,
    URLEncodingError,
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodingError::CannotParseError(..) => {
                write!(f, "El string no puede ser parseado a numero")
            }
            DecodingError::InvalidSyntaxError => write!(f, "Sintaxis invalida"),
            DecodingError::InvalidUTF8CharError(..) => {
                write!(
                    f,
                    "El byte no puede ser convertido a un caracter UTF-8 valido"
                )
            }
            DecodingError::URLEncodingError => {
                write!(f, "No se pudo encodear en formato URL el valor recibido")
            }
        }
    }
}

impl From<ParseIntError> for DecodingError {
    fn from(err: ParseIntError) -> DecodingError {
        DecodingError::CannotParseError(err)
    }
}

impl From<FromUtf8Error> for DecodingError {
    fn from(err: FromUtf8Error) -> DecodingError {
        DecodingError::InvalidUTF8CharError(err)
    }
}
