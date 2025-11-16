use lzv_compression::error::LzvError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Fail to read or write data in file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown command: \"{0}\"")]
    UnknownCommand(String),
    #[error("Unknown algorithm: \"{0}\"")]
    UnknownAlgorithm(String),
    #[error("Unknown extension: \"{0}\"")]
    UnknownExtension(String),
    #[error("LzvError: \"{0}\"")]
    Lzv(#[from] LzvError),
}
