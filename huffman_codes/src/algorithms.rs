use std::io::ErrorKind;

use bit_stream::{BitStreamReader, BitStreamWriter, BitReader};
use crate::tree::{Direction, Node};

pub fn encode(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let file_bytes = BitStreamReader::read_all(input_path)?;

    let mut node_weights= vec![0; 256];
    for byte in file_bytes {
        node_weights[byte as usize] += 1;
    }

    println!("node_weights: {:?}", node_weights);

    let mut non_zero_nodes = Vec::new();
    for (byte, weight) in node_weights.iter().enumerate() {
        if *weight != 0 {   
            non_zero_nodes.push(Node::Leaf {
                value: byte as u8,
                weight: *weight,
            });
        }
    }

    let huffman_root = Node::huffman_tree(non_zero_nodes).unwrap();
    
    // write dict
    let mut writer = BitStreamWriter::create(output_path)?;
    writer.skip_bytes(8);
    for weight in node_weights {
        writer.write_bit_sequence(weight.to_le_bytes().as_slice(), 32)?
    }

    let dictionary = huffman_root.generate_dict();
    
    // encode file
    let bytes = BitStreamReader::read_all(input_path)?;
    for byte in bytes {
        let (numeric_repr, bit_len) = dictionary[&byte];
        let bytes = numeric_repr.to_le_bytes();
        writer.write_bit_sequence(&bytes, bit_len as usize)?
    }
        
    let data_length = (writer.bytes_written - 8 - 1024) * 8 + writer.bit_count;
    println!("data_length: {}", data_length);
    writer.write_byte_sequence_unchecked_at(&data_length.to_le_bytes(), 0)?;

    Ok(())
}

pub fn decode(input_path: &str, output_path: &str) -> std::io::Result<()> {
    let mut reader: BitStreamReader = BitStreamReader::open(input_path)?;

    let length_bytes = reader.read_bit_sequence(64)?;
    let length_bytes_slice = <&[u8; 8]>::try_from(&*length_bytes)
        .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "slice length is not 8"))?;
    
    let data_length = u64::from_le_bytes(*length_bytes_slice);
    println!("data_length: {}", data_length);
    
    let dict_files_bytes: Vec<u8> = reader.read_bit_sequence(1024 * 8)?;
    let mut byte_weights: Vec<u32> = Vec::from([0u32; 256]);
    for (i, ctr) in (0..dict_files_bytes.len()).step_by(4).enumerate() {
        if let Ok(arr) = <&[u8; 4]>::try_from(&dict_files_bytes[ctr..ctr + 4]) {
            byte_weights[i] = u32::from_le_bytes(*arr);
        }
    }

    println!("byte_weights: {:?}", byte_weights);
    
    let mut nodes: Vec<Node> = Vec::new();
    for (byte, weight) in byte_weights.iter().enumerate() {
        if *weight != 0 {   
            nodes.push(Node::Leaf {
                value: byte as u8,
                weight: *weight,
            });
        }
    }
    
    let huffman_root = Node::huffman_tree(nodes).unwrap();
    
    // decode file
    let mut writer: BitStreamWriter = BitStreamWriter::create(output_path)?;
    let mut current_node = &huffman_root;
    let mut processed_bits = 0;
    let read_window = 8 * 2048;
    
    let mut bytes = reader.read_bit_sequence(read_window)?;
    while !bytes.is_empty() {
        for bit in BitReader::new(&bytes) {
            let dir = match bit {
                true => Direction::Right,
                false => Direction::Left,
            };
            if let Some(potential_leaf) = current_node.get_child(dir) {
                if potential_leaf.is_leaf() {
                    writer.write_bit_sequence(&[potential_leaf.value().unwrap()], 8)?;
                    current_node = &huffman_root;
                } else {
                    current_node = potential_leaf;
                }
            }

            processed_bits += 1;
            if processed_bits == data_length {
                break;
            }
        }

        bytes = reader.read_bit_sequence(read_window)?;
    }

    Ok(())
}
