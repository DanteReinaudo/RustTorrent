use super::errors::PiecesError;
use crate::bitclient::client::Event;
use crate::downloads::downloader::Downloader;
use crate::pieces::block::Block;
use crate::torrent_file::metainfo::MetaInfo;
use gtk4::glib::Sender as gtkSender;
use std::sync::mpsc::Sender;
use std::time::Instant;

/******************************************************************************************/
/*                                       PIECE                                          */
/******************************************************************************************/

/// Estructura que modela la pieza.
/// Tiene un largo, indice, vector de bloques, hash de la pieza y un bool que indica sis se completo.
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Piece {
    // largo de esta pieza
    pub length: u32,
    // largo normal de las piezas, la ultima puede tener menor o igual length que piece length
    pub piece_length: u32,
    pub index: u32,
    pub blocks: Vec<Block>,
    pub hash: Vec<u8>,
    pub is_complete: bool,
}

#[allow(dead_code)]
impl Piece {
    /// Inicializa la pieza
    pub fn new(
        length: u32,
        index: u32,
        piece_length: u32,
        hash: Vec<u8>,
        block_size: u32,
    ) -> Piece {
        let mut blocks: Vec<Block> = vec![];
        let num_blocks = ((length as f64) / (block_size as f64)).ceil() as u32;

        for i in 0..num_blocks {
            let block_length: u32 = {
                if i < num_blocks - 1 {
                    block_size
                } else {
                    length - (block_size * (num_blocks - 1))
                }
            };
            let block = Block::new(i, block_length);
            blocks.push(block);
        }

        Piece {
            length,
            piece_length,
            index,
            hash,
            blocks,
            is_complete: false,
        }
    }

    /// Almacena la data en el bloque correspondiente
    /// En caso de que se complete la pieza, se verificara si el hash es correcto.
    /// En caso de que sea correcto, se unifica la data de todos los bloques y se llamara a downloader para que la almacene en el archivo.
    /// Si no coinciden los hashes, se marca la pieza como corrupta, se borra toda la data almacenada en los bloques y no se descarga la pieza.
    pub fn store(
        &mut self,
        downloader: &mut Downloader,
        block_index: u32,
        data: Vec<u8>,
        log: Sender<String>,
        event_bus: gtkSender<Event>,
    ) -> Result<(), PiecesError> {
        let start = Instant::now();
        self.blocks[block_index as usize].data = data;

        if self.have_all_blocks() {
            // concatenate data from blocks together
            let mut data = vec![];
            for block in self.blocks.iter() {
                data.extend(block.data.clone());
            }

            let hashed_data = MetaInfo::hashing(&data);
            if self.hash == hashed_data {
                println!(
                    "[DESCARGA] Se completo la pieza {} y es correcta, la guardo en el archivo.",
                    self.index
                );
                let string = "- [INFO] La pieza ".to_owned()
                    + &self.index.to_string()
                    + " es correcta y se completo su descarga.";
                if let Err(err) = log.send(string) {
                    println!("[Error] Fallo al intentar escribir sobre el log: {:?}", err);
                }

                event_bus
                    .send(Event::DownloadedPiece())
                    .or(Err(PiecesError::DownloadingError))?;

                let offset = self.index as u64 * self.piece_length as u64;
                downloader
                    .download(data, offset)
                    .or(Err(PiecesError::DownloadingError))?;
                self.clear_block_data();

                let duration = start.elapsed().as_secs_f64();
                let speed = (self.length as f64 / 1048576_f64) / duration;
                event_bus
                    .send(Event::UpdateSpeed(speed))
                    .or(Err(PiecesError::DownloadingError))?;

                self.is_complete = true;
            } else {
                let string = "[INFO] La pieza ".to_owned()
                    + &self.index.to_string()
                    + " esta corrupta, borro la data descargada!";
                log.send(string).expect("panic message");

                println!("La Pieza esta corrupta, borro la data descargada!");
                println!("Esperaba {:?}", self.hash);
                println!("Obtuve {:?}", hashed_data);
                self.clear_block_data();
            }
        }
        Ok(())
    }

    /// Busca en su vector de bloques cual es el proximo bloque necesario a pedir.
    pub fn next_block_to_request(&self) -> Option<&Block> {
        if self.is_complete {
            return None;
        }
        for (_i, block) in self.blocks.iter().enumerate() {
            if block.data == vec![] && !block.requested {
                return Some(block);
            }
        }
        None
    }

    /// Verifica si tiene todos los bloques para saber si esta completa
    pub fn have_all_blocks(&self) -> bool {
        if self.is_complete {
            return true;
        }
        for block in self.blocks.iter() {
            if block.data == vec![] {
                return false;
            }
        }
        true
    }

    /// En caso de que la pieza este corrupta se borra toda la data almacenada en los bloques.
    pub fn clear_block_data(&mut self) {
        for block in self.blocks.iter_mut() {
            block.data = vec![];
            block.requested = false;
        }
    }

    /// Marca el bloque como que fue solicitado.
    pub fn mark_as_requested(&mut self, block_index: u32) {
        self.blocks[block_index as usize].requested = true;
    }
}

#[cfg(test)]
mod piece_should {
    use super::*;

    static BLOCK_SIZE: u32 = 16384; // 2^14

    #[test]
    fn initialize() {
        let p = Piece::new(256, 4, 4, vec![1, 2, 3], BLOCK_SIZE);
        assert_eq!(
            p,
            Piece {
                length: 256,
                piece_length: 4,
                index: 4,
                blocks: vec![Block::new(0, 256)],
                hash: vec![1, 2, 3],
                is_complete: false,
            }
        );
    }

    #[test]
    fn initialize_not_have_all_blocks() {
        let p = Piece::new(256, 4, 4, vec![1, 2, 3], BLOCK_SIZE);
        assert_eq!(p.have_all_blocks(), false);
    }

    #[test]
    fn find_next_block() {
        let mut p = Piece::new(256, 4, 4, vec![1, 2, 3], BLOCK_SIZE);
        assert_eq!(
            p.next_block_to_request(),
            Some(&Block {
                index: 0,
                length: 256,
                data: vec![],
                requested: false
            })
        );

        p.is_complete = true;
        assert_eq!(p.next_block_to_request(), None);
    }

    #[test]
    fn have_all_blocks() {
        let p = Piece {
            length: 256,
            piece_length: 4,
            index: 4,
            blocks: vec![Block {
                index: 12,
                length: 12,
                data: vec![1, 3, 4],
                requested: false,
            }],
            hash: vec![1, 2, 3],
            is_complete: true,
        };
        assert_eq!(p.have_all_blocks(), true);
    }

    #[test]
    fn not_have_all_blocks() {
        let p = Piece {
            length: 256,
            piece_length: 4,
            index: 4,
            blocks: vec![Block {
                index: 12,
                length: 12,
                data: vec![],
                requested: false,
            }],
            hash: vec![1, 2, 3],
            is_complete: false,
        };
        assert_eq!(p.have_all_blocks(), false);
    }

    #[test]
    fn clear_block_data() {
        let mut p = Piece {
            length: 256,
            piece_length: 4,
            index: 4,
            blocks: vec![Block {
                index: 12,
                length: 12,
                data: vec![1, 2],
                requested: false,
            }],
            hash: vec![1, 2, 3],
            is_complete: false,
        };

        p.clear_block_data();
        assert_eq!(
            p.blocks,
            vec![Block {
                index: 12,
                length: 12,
                data: vec![],
                requested: false
            }]
        );
    }
    /*
       #[test]
       fn download_piece() {
           let directory_path = String::from("./downloads");
           let file_name = String::from("download_piece.txt");
           let expected_path = String::from("./downloads/download_piece.txt");
           let mut downloader = Downloader::new(&directory_path, &file_name, 7).unwrap();
           let (tx, _rx) = mpsc::channel();
           let piece1: Vec<u8> = vec![1, 2, 3, 4];
           let piece2: Vec<u8> = vec![5, 6, 7];

           let hash_piece1 = MetaInfo::hashing(&piece1);
           let hash_piece2 = MetaInfo::hashing(&piece2);
           let mut p1 = Piece::new(4, 0, 4, hash_piece1, 2);
           let mut p2 = Piece::new(3, 1, 4, hash_piece2, 2);

           p1.store(&mut downloader, 0, [1, 2].to_vec(), tx.clone())
               .unwrap();
           p1.store(&mut downloader, 1, [3, 4].to_vec(), tx.clone())
               .unwrap();

           p2.store(&mut downloader, 0, [5, 6].to_vec(), tx.clone())
               .unwrap();
           p2.store(&mut downloader, 1, [7].to_vec(), tx.clone())
               .unwrap();

           let vec: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7];
           let mut file = File::open(&expected_path).unwrap();
           let mut data = Vec::new();
           file.read_to_end(&mut data).unwrap();
           assert_eq!(data, vec);
       }

       #[test]
       fn download_corrupt_piece() {
           let directory_path = String::from("./downloads");
           let file_name = String::from("download_corrupt_piece.txt");
           let expected_path = String::from("./downloads/download_corrupt_piece.txt");
           let mut downloader = Downloader::new(&directory_path, &file_name, 4).unwrap();
           let (tx, _rx) = mpsc::channel();
           let piece: Vec<u8> = vec![1, 2, 3, 4];
           let hash_piece1 = MetaInfo::hashing(&piece);
           let mut p1 = Piece::new(4, 0, 4, hash_piece1, 2);

           p1.store(&mut downloader, 0, [5, 6].to_vec(), tx.clone())
               .unwrap();
           p1.store(&mut downloader, 1, [7, 8].to_vec(), tx.clone())
               .unwrap();

           let vec: Vec<u8> = vec![0, 0, 0, 0];
           let mut file = File::open(&expected_path).unwrap();
           let mut data = Vec::new();
           file.read_to_end(&mut data).unwrap();
           assert_eq!(data, vec);
       }

    */
}
