use thiserror::Error;

#[derive(Error, Debug)]
pub enum LzvError {
    #[error("Invalid input")]
    InvalidInput,
    #[error("There is no record in the dictionary: ({s:?}, {c})")]
    NoRecord { s: Option<u32>, c: u8 },
    #[error("There is no index in the dictionary: ({index})")]
    NoIndex { index: u32 },
    #[error("Invalid first index {index}")]
    InvalidFirstIndex { index: u32 },
    #[error("BitReader error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unprocessable index")]
    UnprocessableIndex,
}
