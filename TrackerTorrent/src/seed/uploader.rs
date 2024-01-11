use crate::errors::BitTrackerError;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

/******************************************************************************************/
/*                                 UPLOADER                                               */
/******************************************************************************************/

/// Estructura encargada de almacenar las piezas en el archivo, asi como tambien de uploadear.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Uploader {
    file: File,
    pub path: String,
}

#[allow(dead_code)]
impl Uploader {
    /// Se inicializa con un directorio de descargas, un nombre del archivo y su tamaño.
    /// El archivo se crea con un tamaño fijo que es igual al tamaño del torrent a descargar.
    /// Si no existe el directorio o el archivo los crea.
    pub fn new(path_name: &str) -> Result<Uploader, BitTrackerError> {
        let path = Path::new(&path_name);

        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .or(Err(BitTrackerError::FileCreationError))?;
        Ok(Uploader {
            file,
            path: path_name.to_string(),
        })
    }

    /// Abre el archivo a partir de un offset y lee la cantidad especificada en length
    pub fn upload(&mut self, offset: u64, length: u64) -> Result<Vec<u8>, BitTrackerError> {
        let mut buffer = vec![0; length as usize];
        self.file
            .seek(SeekFrom::Start(offset))
            .or(Err(BitTrackerError::FileWritingError))?;
        self.file
            .read_exact(&mut buffer)
            .or(Err(BitTrackerError::FileReadingError))?;
        Ok(buffer.to_vec())
    }
}

#[cfg(test)]
mod downloader_should {
    use super::*;
    use std::{io::Read, vec};

    #[test]
    fn upload_information() {
        let path = String::from("./downloads/INFORME - BITTORRENT.pdf");
        let mut downloader = Uploader::new(&path).unwrap();

        let bytes = downloader.upload(0, 10).unwrap();
        println!("Bytes uploadeados: {:?}", bytes);
        let path = Path::new(&path);
        let mut file = OpenOptions::new().read(true).open(path).unwrap();

        let mut buf = vec![];
        let mut vec = vec![];
        file.read_to_end(&mut buf).unwrap();
        for (i, byte) in buf.into_iter().enumerate() {
            if i < 10 {
                vec.push(byte)
            } else {
                break;
            }
        }
        println!("Todo el file: {:?}", vec);
    }
}
