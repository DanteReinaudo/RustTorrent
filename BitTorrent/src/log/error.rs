use std::fmt;

/******************************************************************************************/
/*                                  Logger ERROR                                        */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LogError {
    FileCreationError,
    OpenFileError,
    WriteFileError,
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogError::OpenFileError => write!(f, "No se pudo abrir el archivo"),
            LogError::WriteFileError => write!(f, "No se pudo escribir en el archivo"),
            LogError::FileCreationError => write!(f, "Error al crear el archivo"),
        }
    }
}
