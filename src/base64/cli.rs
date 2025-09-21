use std::fmt::Display;
use std::io::{stdin, stdout, Write, BufRead, BufReader, BufWriter};
use crate::base64::{
    algorithms,
    error::Base64Error
};
use std::fs::{File, OpenOptions};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use text_io::read;

pub const BASE64_EXTENSION: &str = "base64";
pub const DEBASE64_EXTENSION: &str = "txt";

pub fn encode_file(input_file: &str, output_file: &str) -> Result<(), Base64Error> {
    let mut writer = File::create(output_file)?;

    let mut reader = BufReader::new(File::open(input_file)?);

    let mut line = Vec::new();
    let mut line_number = 1;
    let mut first_line = true;
    while reader.read_until(b'\n', &mut line)? != 0 {
        if let Some(b'\n') = line.last() {
            line.pop(); // remove \n
        }

        if line.len() > 76 {
            return Err(Base64Error::IncorrectStringLength{line: line_number, len: line.len() as u64});
        }
        
        let encoded_line = algorithms::encode(&line);

        if first_line {
            write!(writer, "{}", encoded_line)?;
            first_line = false;
        } else {
            write!(writer, "\n{}", encoded_line)?;
        }

        line.clear();
        line_number += 1;
    }

    Ok(())
}

pub fn decode_file(input_file: &str, output_file: &str) -> Result<(), Base64Error> {
    File::create(output_file)?;

    let mut writer = OpenOptions::new()
        .create(true)
        .append(true)
        // .truncate(true)
        .open(output_file)?;

    let reader = BufReader::new(File::open(input_file)?);

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.len() > 76 {
            return Err(Base64Error::IncorrectStringLength{line: i as u64, len: line.len() as u64});
        }

        if line.starts_with('-') {
            continue;
        }

        if i != 0 {
            writer.write_all(b"\n")?;
        }

        match algorithms::decode(&line) {
            Ok(decoded_line) => writer.write_all(&*decoded_line)?,
            Err(mut error) => {
                error.set_line(i as u64);
                return Err(error)
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum Options {
    Encode,
    Decode,
}

impl Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Encode => write!(f, "encode"),
            Options::Decode => write!(f, "decode"),
        }
    }
}

impl TryFrom<&str> for Options {
    type Error = Base64Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "encode" => Ok(Self::Encode),
            "decode" => Ok(Self::Decode),
            _ => Err(Base64Error::UnknownOption)
        }
    }
}

pub fn run_encode_branch () {
    let mut input_file = String::new();
    while input_file.is_empty() {
        print!("Name a file you want to encode: ");
        input_file = read!("{}\n");

        if input_file.is_empty() {
            print!("You must provide a name. ");
        }
    }
    print!("Name a file, where to store the result. You may leave it blank, to store data in {input_file}.{BASE64_EXTENSION}: ");
    let output_file: String = read!("{}\n");

    let output_file = match output_file.is_empty() {
        true => format!("{input_file}.{BASE64_EXTENSION}"),
        false => output_file,
    };

    if let Err(e) = encode_file(&input_file, &output_file) {
        println!("Error occurred: {}", e);
    }
}

pub fn run_decode_branch () {
    let mut input_file = String::new();
    while input_file.is_empty() {
        print!("Name a file you want to decode: ");
        input_file = read!("{}\n");

        if input_file.is_empty() {
            print!("You must provide a name. ");
        }
    }
    print!("Name a file, where you want to store the result. You may leave it blank, to store data in {input_file}.{DEBASE64_EXTENSION}: ");
    let output_file: String = read!("{}\n");

    let output_file = match output_file.is_empty() {
        true => format!("{input_file}.{DEBASE64_EXTENSION}"),
        false => output_file,
    };

    if let Err(e) = decode_file(&input_file, &output_file) {
        println!("Error occurred: {}", e);
    }
}

pub fn run() {
    loop {
        println!("Options:");
        println!("  > {}", Options::Encode);
        println!("  > {}", Options::Decode);
        print!("Please chose what to do: ");
        let line: String = read!("{}\n");

        match Options::try_from(line.as_str()) {
            Ok(Options::Encode) => run_encode_branch(),
            Ok(Options::Decode) => run_decode_branch(),
            Err(_) => print!("Invalid option. Try again."),
        }

        println!("\n\n\n");
    }
}