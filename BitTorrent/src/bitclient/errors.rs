use crate::downloads::errors::DownloaderError;
use crate::peer_connection::errors::ConnectionError;
use crate::pieces::errors::PiecesError;
use crate::torrent_file::errors::MetaInfoError;
use crate::tracker::errors::TrackerError;
use std::fmt;

/******************************************************************************************/
/*                                  CLIENT ERROR                                        */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum ClientError {
    ReadFileError,
    OpenFileError,
    DecodingError(MetaInfoError),
    FailToConnectError(ConnectionError),
    BadPeerResponseError,
    WriteConnectionError(ConnectionError),
    ReadConnectionError(ConnectionError),
    TrackerError(TrackerError),
    InvalidUTF8HandshakeError,
    InvalidBitfieldPositionError,
    InvalidMessageError,
    StorageError(PiecesError),
    CreateDownloaderError(DownloaderError),
    WriteLogError,
    FailToJoinThreadError,
    MutexLockError,
    TpcBindAsServerError,
    PeerConectionAsServerError,
    WriteConnectionAsServerError,
    UploadError,
    InvalidServerMessageError,
}

#[allow(dead_code)]
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::ReadFileError => write!(f, "No se pudo leer el archivo"),
            ClientError::OpenFileError => write!(f, "No se pudo abrir el archivo"),
            ClientError::BadPeerResponseError => {
                write!(f, "La respuesta del peer no fue la esperada")
            }
            ClientError::FailToConnectError(connection_error) => {
                write!(f, "{}", connection_error)
            }
            ClientError::DecodingError(meta_info_error) => {
                write!(f, "{}", meta_info_error)
            }
            ClientError::WriteConnectionError(connection_error) => {
                write!(f, "{}", connection_error)
            }
            ClientError::ReadConnectionError(connection_error) => {
                write!(f, "{}", connection_error)
            }
            ClientError::TrackerError(tracker_error) => {
                write!(f, "{}", tracker_error)
            }
            ClientError::InvalidUTF8HandshakeError => {
                write!(f, "Fallo al parsear el Handshake response")
            }
            ClientError::InvalidBitfieldPositionError => {
                write!(
                    f,
                    "Se intento accede a una posicion incorrecta del bitfield"
                )
            }
            ClientError::InvalidMessageError => {
                write!(f, "No pudo identificarse el mensaje")
            }
            ClientError::StorageError(piece_error) => {
                write!(f, "{}", piece_error)
            }
            ClientError::CreateDownloaderError(downloader_error) => {
                write!(f, "{}", downloader_error)
            }
            ClientError::WriteLogError => {
                write!(f, "Fallo al escribir en el archivo de log")
            }
            ClientError::FailToJoinThreadError => {
                write!(f, "Fallo al realizar el join de un thread")
            }
            ClientError::MutexLockError => {
                write!(f, "Fallo al realizar el Lock a un client")
            }
            ClientError::TpcBindAsServerError => {
                write!(f, "Fallo al realizar la conexion")
            }
            ClientError::PeerConectionAsServerError => {
                write!(f, "Fallo al realizar la conexion con el peer")
            }
            ClientError::WriteConnectionAsServerError => {
                write!(f, "Fallo al realizar la escritura en la conexion")
            }
            ClientError::UploadError => {
                write!(f, "Fallo al intentar uploadear una pieza")
            }
            ClientError::InvalidServerMessageError => {
                write!(f, "El servidor recibio un mensaje que no corresponde")
            }
        }
    }
}
