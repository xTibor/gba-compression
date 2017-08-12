#![allow(unused_imports, unused_variables)]

use std::io::{Cursor, Read, Write, Result, Error, ErrorKind};
use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt};
use compression::Compressor;
use compression::bios::{BiosCompressionType, bios_compression_type};

#[derive(Default)]
pub struct HuffmanCompressor;

#[derive(Debug)]
enum HuffmanNode {
    Branch {
        node0: Box<HuffmanNode>,
        node1: Box<HuffmanNode>,
    },
    Leaf {
        value: u8,
    },
}

impl Compressor for HuffmanCompressor {
    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(input);
        let header = cursor.read_u8()?;

        if bios_compression_type(header) != Some(BiosCompressionType::Huffman) {
            return Err(Error::new(ErrorKind::InvalidData, "Not a Huffman encoded stream"));
        }

        let bit_length = header & 0xF;
        let decompressed_size: usize = cursor.read_u24::<LittleEndian>()? as usize;

        let huffman_tree = {
            let tree_size = cursor.read_u8()? as usize * 2 + 1;

            let mut tree_data: Vec<u8> = vec![0; tree_size];
            cursor.read_exact(&mut tree_data)?;

            fn read_node(tree_data: &[u8], offset: usize, is_leaf: bool) -> Result<HuffmanNode> {
                let node = *tree_data.get(offset).ok_or_else(|| Error::new(ErrorKind::InvalidData, "Node offset out of bounds"))?;

                if is_leaf {
                    Ok(HuffmanNode::Leaf {
                        value: node,
                    })
                } else {
                    let node0_leaf = ((node >> 7) & 1) == 1;
                    let node1_leaf = ((node >> 6) & 1) == 1;
                    let node0_offset = (((offset + 3) & !1) - 1) as usize + (node & 0x3F) as usize * 2;
                    let node1_offset = node0_offset + 1;

                    Ok(HuffmanNode::Branch {
                        node0: Box::new(read_node(tree_data, node0_offset, node0_leaf)?),
                        node1: Box::new(read_node(tree_data, node1_offset, node1_leaf)?),
                    })
                }
            }

            read_node(&tree_data, 0, false)?
        };

        println!("cursor.position: {:08X}", cursor.position());
        println!("huffman_tree: {:#?}", huffman_tree);

        let buffer: Vec<u8> = Vec::with_capacity(decompressed_size);
        /*
        while buffer.len() < decompressed_size {
            let x = cursor.read_u32::<LittleEndian>()?;

        }*/

        Ok(())
    }

    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Unimplemented compression routine"))
    }
}

#[cfg(test)]
mod tests {
    use compression::bios::HuffmanCompressor;
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_decompress_1() {
    }

    #[test]
    fn test_compress_1() {
    }
}
