use std::io::{Cursor, Read, Result, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};
use bios::{BiosCompressionType, bios_compression_type};

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

pub fn decompress_huffman(input: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(input);
    let header = cursor.read_u8()?;

    if bios_compression_type(header) != Some(BiosCompressionType::Huffman) {
        return Err(Error::new(ErrorKind::InvalidData, "compression header mismatch"));
    }

    let bit_length = header & 0xF;
    let decompressed_size: usize = cursor.read_u24::<LittleEndian>()? as usize;

    let huffman_tree = {
        let tree_size = cursor.read_u8()? as usize * 2 + 1;

        let mut tree_data: Vec<u8> = vec![0; tree_size];
        cursor.read_exact(&mut tree_data)?;

        fn read_node(tree_data: &[u8], offset: usize, is_leaf: bool) -> Result<HuffmanNode> {
            let node = *tree_data.get(offset).ok_or_else(|| Error::new(ErrorKind::InvalidData, "node offset out of bounds"))?;

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

    let mut output: Vec<u8> = Vec::with_capacity(decompressed_size);
    let mut bits = cursor.read_u32::<LittleEndian>()?;
    let mut remaining_bits = 32;
    let mut current_node = &huffman_tree;

    while output.len() < decompressed_size {
        if let &HuffmanNode::Branch { ref node0, ref node1 } = current_node {
            current_node = if bits & 0x80000000 == 0 { node0 } else { node1 };

            if let &HuffmanNode::Leaf { value } = current_node {
                output.push(value);
                current_node = &huffman_tree;
            }

            bits <<= 1;
            remaining_bits -= 1;

            if (remaining_bits == 0) && (output.len() < decompressed_size) {
                bits = cursor.read_u32::<LittleEndian>()?;
                remaining_bits = 32;
            }
        } else {
            unreachable!();
        };
    }

    assert_eq!(output.len(), decompressed_size);
    Ok(output)
}

pub fn compress_huffman(input: &[u8]) -> Result<Vec<u8>> {
    Err(Error::new(ErrorKind::Other, "unimplemented"))
}
