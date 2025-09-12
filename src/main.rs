mod base64;

fn main() {
    // base64::decode_file("test.base64", None).unwrap()
    // base64::encode_file("test.base64.decoded", None).unwrap()
    // base64::decode_file("test.txt", None).unwrap()
    base64::cli::run();
}
