/******************************************************************************************/
/*                                       BLOCK                                           */
/******************************************************************************************/

/// Estructura que modela a un bloque.
/// Tiene un indice, un tamaño, la data corespondiente y un bool que indica si ya fue pedido o no.
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub struct Block {
    pub requested: bool,
    pub index: u32,
    pub length: u32,
    pub data: Vec<u8>,
}

#[allow(dead_code)]
impl Block {
    pub fn new(index: u32, length: u32) -> Self {
        Block {
            index,
            length,
            data: vec![],
            requested: false,
        }
    }
}

#[cfg(test)]
mod block_should {
    use super::Block;

    #[test]
    fn initialize() {
        let block: Block = Block::new(0, 16);
        assert_eq!(
            block,
            Block {
                index: 0,
                length: 16,
                data: vec![],
                requested: false,
            }
        );
    }
}
