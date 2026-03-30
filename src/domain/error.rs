#[derive(Debug, Clone)]
pub enum APIError {
    FailedToOpen,
    FailedToRead,
    FailedToParseDate,
    SchemaParseErr(Vec<String>),
    DataSchemaCheckErr(String),
    NoData,
    UnexpectedError,
}