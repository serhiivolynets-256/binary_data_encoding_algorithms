use thiserror::Error;

#[derive(Error, Debug)]
pub enum Base64Error {
    #[error("Invalid input character on line: {line}, pos: {pos}")]
    InvalidInputCharacter { line: u64, pos: u64 },

    #[error("Incorrect string length {len} on line {line}")]
    IncorrectStringLength { line: u64, len: u64 },

    #[error("Incorrect use of padding on line {line} on pos {pos}")]
    IncorrectUseOfPadding { line: u64, pos: u64 },

    #[error("Available data after the end of the message")]
    AvailableDataAfterTheEndOfTheMessage,

    #[error("IO error")]
    IOError(#[from] std::io::Error),

    #[error("Unknown option")]
    UnknownOption,
}

impl Base64Error {
    pub fn set_line(&mut self, new_line: u64) {
        match self {
            Base64Error::InvalidInputCharacter { line, .. } => *line = new_line,
            Base64Error::IncorrectStringLength { line, .. } => *line = new_line,
            Base64Error::IncorrectUseOfPadding { line, .. } => *line = new_line,
            Base64Error::AvailableDataAfterTheEndOfTheMessage => {}
            Base64Error::IOError(_) => {}
            Base64Error::UnknownOption => {}
        }
    }
}
