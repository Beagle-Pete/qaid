#[derive(Debug, Clone)]
pub enum APIError {
    FailedToOpen,
    FailedToRead,
    FailedToParseDate,
    UnexpectedError,
}