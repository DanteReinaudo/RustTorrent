use super::errors::DownloaderError;
use std::fs::create_dir;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;

/******************************************************************************************/
/*                                 Downloader                                              */
/******************************************************************************************/

/// Estructura encargada de almacenar las piezas en el archivo, asi como tambien de uploadear.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Downloader {
    file: File,
    pub path: String,
    size: u64,
}

#[allow(dead_code)]
impl Downloader {
    /// Se inicializa con un directorio de descargas, un nombre del archivo y su tamaño.
    /// El archivo se crea con un tamaño fijo que es igual al tamaño del torrent a descargar.
    /// Si no existe el directorio o el archivo los crea.
    pub fn new(
        directory_path: &str,
        file_name: &str,
        size: u64,
    ) -> Result<Downloader, DownloaderError> {
        let folder = Path::new(directory_path);
        if !folder.is_dir() {
            let _f = create_dir(folder).or(Err(DownloaderError::FileCreationError))?;
        }

        let path_name = directory_path.to_string() + "/" + file_name;
        let path = Path::new(&path_name);
        if !path.exists() {
            let f = File::create(path).or(Err(DownloaderError::FileCreationError))?;
            let _ = f.set_len(size);
        }
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .or(Err(DownloaderError::FileCreationError))?;
        Ok(Downloader {
            file,
            path: path_name,
            size,
        })
    }

    /// Abre el archivo en un offset y almacena el vector de u8 data a partir de ahi.
    pub fn download(&mut self, data: Vec<u8>, offset: u64) -> Result<(), DownloaderError> {
        if (data.len() as u64) + offset > self.size {
            return Err(DownloaderError::DataSizeError);
        }
        self.file
            .seek(SeekFrom::Start(offset))
            .or(Err(DownloaderError::FileWritingError))?;
        self.file
            .write_all(&data)
            .or(Err(DownloaderError::FileWritingError))?;
        Ok(())
    }

    /// Abre el archivo a partir de un offset y lee la cantidad especificada en length
    pub fn upload(&mut self, offset: u64, length: u64) -> Result<Vec<u8>, DownloaderError> {
        if length + offset > self.size {
            return Err(DownloaderError::DataSizeError);
        }
        let mut buffer = vec![0; length as usize];
        //let mut buffer  = [0;4];
        self.file
            .seek(SeekFrom::Start(offset))
            .or(Err(DownloaderError::FileWritingError))?;
        self.file
            .read_exact(&mut buffer)
            .or(Err(DownloaderError::FileReadingError))?;
        Ok(buffer.to_vec())
    }
}

#[cfg(test)]
mod downloader_should {
    use super::*;
    use std::io::Read;

    #[ignore]
    #[test]
    fn initialize() {
        let directory_path = String::from("./downloads");
        let file_name = String::from("prueba.txt");
        let expected_path = String::from("./downloads/prueba.txt");
        let downloader = Downloader::new(&directory_path, &file_name, 8).unwrap();

        let vec: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let mut file = File::open(&expected_path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        assert_eq!(downloader.path, expected_path);
        assert_eq!(downloader.size, 8);
        assert_eq!(data, vec);
    }

    #[ignore]
    #[test]
    fn store_information() {
        let directory_path = String::from("./downloads");
        let file_name = String::from("otra_prueba.txt");
        let expected_path = String::from("./downloads/otra_prueba.txt");
        let mut downloader = Downloader::new(&directory_path, &file_name, 10).unwrap();

        downloader.download([2, 4].to_vec(), 3).unwrap();

        let vec: Vec<u8> = vec![0, 0, 0, 2, 4, 0, 0, 0, 0, 0];
        let mut file = File::open(&expected_path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        assert_eq!(downloader.path, expected_path);
        assert_eq!(downloader.size, 10);
        assert_eq!(data, vec);
    }

    #[test]
    fn upload_information() {
        let directory_path = String::from("./downloads");
        let file_name = String::from("otra_prueba.txt");
        let mut downloader = Downloader::new(&directory_path, &file_name, 10).unwrap();

        downloader.download([2, 4].to_vec(), 3).unwrap();

        let data = downloader.upload(3, 2).unwrap();
        assert_eq!(data, vec![2, 4]);
    }

    #[test]
    fn fail_if_wrong_data_size() {
        let directory_path = String::from("./downloads");
        let file_name = String::from("prueba.txt");
        let mut downloader = Downloader::new(&directory_path, &file_name, 8).unwrap();

        assert_eq!(
            downloader
                .download(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
                .unwrap_err()
                .to_string(),
            "El tamaño de los datos es invalido"
        );
    }

    #[test]
    fn fail_if_wrong_data_len_plus_offset_size() {
        let directory_path = String::from("./downloads");
        let file_name = String::from("prueba.txt");
        let mut downloader = Downloader::new(&directory_path, &file_name, 8).unwrap();

        assert_eq!(
            downloader
                .download(vec![2, 3, 4], 6)
                .unwrap_err()
                .to_string(),
            "El tamaño de los datos es invalido"
        );
    }
}
