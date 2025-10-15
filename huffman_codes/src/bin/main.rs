use huffman_codes::algorithms::{encode, decode};
use std::time::Instant;

fn run() {
    // let input_path = "sample3.xls";
    let input_path = "test.txt";
    let encoded_file_path = "encoded_file";
    let decoded_file_path = "decoded_file";
    
    println!("Encoding file ...");
    let start = Instant::now();
    encode(&input_path, &encoded_file_path).unwrap();
    println!("Encoding time: {:?}", start.elapsed());
    
    println!("Decode file ...");
    let start = Instant::now();
    decode(&encoded_file_path, &decoded_file_path).unwrap();
    println!("Decoding time: {:?}", start.elapsed());
}

fn main() {
    run();
}
