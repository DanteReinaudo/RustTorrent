use std::fmt;

/******************************************************************************************/
/*                                  Metainfo ERROR                                        */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum MetaInfoError {
    ReadFileError,
    OpenFileError,
    DecodingError,
    IntegerConvertionError,
}

#[allow(dead_code)]
impl fmt::Display for MetaInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetaInfoError::ReadFileError => write!(f, "No se pudo leer el archivo"),
            MetaInfoError::OpenFileError => write!(f, "No se pudo abrir el archivo"),
            MetaInfoError::DecodingError => write!(f, "No se pudo parsear el archivo"),
            MetaInfoError::IntegerConvertionError => {
                write!(f, "Fallo al intentar castear un entero")
            }
        }
    }
}
