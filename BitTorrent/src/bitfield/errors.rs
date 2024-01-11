use std::fmt;

/******************************************************************************************/
/*                                  Bitfield ERROR                                        */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum BitfieldError {
    InvalidPositionError,
}

impl fmt::Display for BitfieldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BitfieldError::InvalidPositionError => {
                write!(f, "Se intento acceder a una posicion invalida")
            }
        }
    }
}
