use std::fmt;

/******************************************************************************************/
/*                                 Downloader ERROR                                       */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum DownloaderError {
    FileCreationError,
    FileWritingError,
    DataSizeError,
    FileReadingError,
}

impl fmt::Display for DownloaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DownloaderError::FileCreationError => write!(f, "Hubo un error al crear el archivo"),
            DownloaderError::FileWritingError => {
                write!(f, "Hubo un error al escribir el bloque en el archivo")
            }
            DownloaderError::DataSizeError => {
                write!(f, "El tamaño de los datos es invalido")
            }
            DownloaderError::FileReadingError => {
                write!(f, "Error al leer el archivo")
            }
        }
    }
}
