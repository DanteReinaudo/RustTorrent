use std::fmt;

/******************************************************************************************/
/*                                 PIECES ERROR                                       */
/******************************************************************************************/

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum PiecesError {
    DownloadingError,
}

impl fmt::Display for PiecesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PiecesError::DownloadingError => write!(f, "Hubo un error al descargar la pieza"),
        }
    }
}
