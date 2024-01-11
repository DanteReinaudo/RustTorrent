use std::fmt;
/******************************************************************************************/
/*                                  Tracker ERROR                                        */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum TrackerError {
    WriteConnectionError,
    ReadConnectionError,
    FailToConnectError,
    URLEncodingError,
    InvalidSyntaxError,
    RequestError,
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TrackerError::WriteConnectionError => write!(f, "No se pudo escribir en la conexion"),
            TrackerError::ReadConnectionError => write!(f, "No se pudo leer desde la conexion"),
            TrackerError::FailToConnectError => write!(f, "No se pudo establecer la conexion"),
            TrackerError::URLEncodingError => {
                write!(f, "No se pudo encodear en formato URL el valor recibido")
            }
            TrackerError::InvalidSyntaxError => {
                write!(f, "No se pudo parsear el diccionario")
            }
            TrackerError::RequestError => write!(f, "No se pudo realizar la request"),
        }
    }
}
