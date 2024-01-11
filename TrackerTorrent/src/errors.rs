use std::fmt;
/******************************************************************************************/
/*                                  BitTracker ERROR                                        */
/******************************************************************************************/

/// Enumeracion de los posibles errores que puede tener nuestro tracker.
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum BitTrackerError {
    OpenFileError,
    WriteConnectionError,
    ReadConnectionError,
    FailToConnectError,
    URLEncodingError,
    InvalidSyntaxError,
    RequestError,
    ReadFileError,
    DecodingError,
    IntegerConvertionError,
    FileCreationError,
    WriteFileError,
    WriteLogError,
    InvalidPeerState,
    MutexLockError,
    DataSizeError,
    FileReadingError,
    FileWritingError,
    InvalidMessageError,
    InvalidUTF8HandshakeError,
    BadPeerResponseError,
    UploadError,
}
#[allow(dead_code)]
impl fmt::Display for BitTrackerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BitTrackerError::WriteConnectionError => {
                write!(f, "No se pudo escribir en la conexion")
            }
            BitTrackerError::ReadConnectionError => write!(f, "No se pudo leer desde la conexion"),
            BitTrackerError::FailToConnectError => write!(f, "No se pudo establecer la conexion"),
            BitTrackerError::URLEncodingError => {
                write!(f, "No se pudo encodear en formato URL el valor recibido")
            }
            BitTrackerError::InvalidSyntaxError => {
                write!(f, "Sintaxis invalida")
            }
            BitTrackerError::RequestError => write!(f, "No se pudo realizar la request"),
            BitTrackerError::OpenFileError => write!(f, "No se pudo abrir el archivo"),
            BitTrackerError::ReadFileError => write!(f, "No se pudo leer el archivo"),
            BitTrackerError::DecodingError => write!(f, "No se pudo parsear el archivo"),
            BitTrackerError::IntegerConvertionError => {
                write!(f, "Fallo al intentar castear un entero")
            }
            BitTrackerError::WriteFileError => write!(f, "No se pudo escribir en el archivo"),
            BitTrackerError::FileCreationError => write!(f, "Error al crear el archivo"),
            BitTrackerError::WriteLogError => write!(f, "No se pudo escribir en el archivo de log"),
            BitTrackerError::InvalidPeerState => {
                write!(f, "El estado de la request del peer es invalido")
            }
            BitTrackerError::MutexLockError => write!(f, "Error al tomar el lock"),
            BitTrackerError::FileReadingError => write!(f, "Error al leer el archivo"),
            BitTrackerError::FileWritingError => write!(f, "Error al escribir el archivo"),
            BitTrackerError::DataSizeError => write!(f, "Tamaño invalido"),
            BitTrackerError::InvalidMessageError => write!(f, "Mesaje invalido"),
            BitTrackerError::InvalidUTF8HandshakeError => write!(f, "Handshake invalido"),
            BitTrackerError::UploadError => write!(f, "Error al subir el archivo"),
            BitTrackerError::BadPeerResponseError => {
                write!(f, "Error al recibir respuesta del peer")
            }
        }
    }
}

/******************************************************************************************/
/*                                  Request ERROR                                        */
/******************************************************************************************/
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum RequestError {
    InvalidSyntaxError,
    InvalidParameterError,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RequestError::InvalidSyntaxError => {
                write!(f, "No se pudo parsear el diccionario")
            }
            RequestError::InvalidParameterError => {
                write!(f, "No se pudo parsear el diccionario")
            }
        }
    }
}
