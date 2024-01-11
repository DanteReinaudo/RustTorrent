use std::fmt;

/******************************************************************************************/
/*                                CONNECTION ERROR                                        */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum ConnectionError {
    FailToConnectError,
    BadPeerResponseError,
    WriteConnectionError,
    ReadConnectionError,
    InvalidUTF8HandshakeError,
    UsizeFromU32Error,
    InvalidMessageError,
    MutexLockError,
    StorageError,
    UploadError,
}

#[allow(dead_code)]
impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConnectionError::FailToConnectError => write!(f, "No se pudo establecer la conexion"),
            ConnectionError::BadPeerResponseError => {
                write!(f, "La respuesta del peer no fue la esperada")
            }
            ConnectionError::WriteConnectionError => {
                write!(f, "No se pudo escribir en la conexion")
            }
            ConnectionError::ReadConnectionError => {
                write!(f, "No se pudo leer en la conexion")
            }
            ConnectionError::InvalidUTF8HandshakeError => {
                write!(f, "Fallo al parsear el Handshake response")
            }
            ConnectionError::UsizeFromU32Error => {
                write!(f, "Fallo al intentar convertir un u32 en un usize")
            }
            ConnectionError::InvalidMessageError => {
                write!(f, "Se recibio un mensaje invalido")
            }
            ConnectionError::MutexLockError => {
                write!(f, "Fallo al tratar de obtener el lock")
            }
            ConnectionError::StorageError => {
                write!(f, "Fallo de almacenar un bloque")
            }
            ConnectionError::UploadError => {
                write!(f, "Fallo al querer uploadear una pieza")
            }
        }
    }
}
