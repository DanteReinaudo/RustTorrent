use std::fmt;

/******************************************************************************************/
/*                                  PeerProtocol ERROR                                    */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum PeerProtocolError {
    HandshakeInvalidUTF8CharError,
    InvalidMessageFormatError,
    FailToConvertError,
}

impl fmt::Display for PeerProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PeerProtocolError::HandshakeInvalidUTF8CharError => write!(
                f,
                "El byte no puede ser convertido a un caracter UTF-8 valido"
            ),
            PeerProtocolError::InvalidMessageFormatError => {
                write!(f, "El formato del mensaje no es valido")
            }
            PeerProtocolError::FailToConvertError => {
                write!(f, "Fallo al intentar convertir un numero desde un string")
            }
        }
    }
}
