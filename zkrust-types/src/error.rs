pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
}