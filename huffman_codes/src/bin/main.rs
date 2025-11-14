use huffman_codes::algorithms::{decode, encode};
use std::time::Instant;
use tracing::info;

fn run() {
    // let input_path = "sample3.xls";
    let input_path = "test.txt";
    let encoded_file_path = "encoded_file";
    let decoded_file_path = "decoded_file";

    info!("Encoding file ...");
    let start = Instant::now();
    encode(&input_path, &encoded_file_path).unwrap();
    info!("Encoding time: {:?}", start.elapsed());

    info!("Decode file ...");
    let start = Instant::now();
    decode(&encoded_file_path, &decoded_file_path).unwrap();
    info!("Decoding time: {:?}", start.elapsed());
}

fn main() {
    run();
}
