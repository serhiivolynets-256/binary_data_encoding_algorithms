use bit_stream::{BitStreamReader, BitStreamWriter};

fn main() -> std::io::Result<()> {
    let file_name = "bitstream.bin";
    let mut writer = BitStreamWriter::create(file_name)?;

    let a1 = [0xE1, 0x01];
    let a2 = [0xEE, 0x00];
    writer.write_bit_sequence(&a1, 9)?;
    writer.write_bit_sequence(&a2, 9)?;

    let mut reader = BitStreamReader::open(file_name)?;
    let b1 = reader.read_bit_sequence(11)?;
    let b2 = reader.read_bit_sequence(7)?;

    println!("b1 = {:02X?}", b1);
    println!("b2 = {:02X?}", b2);

    Ok(())
}