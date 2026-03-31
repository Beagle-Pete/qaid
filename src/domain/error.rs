#[derive(Debug, Clone, PartialEq)]
pub enum APIError {
    FailedToOpen,
    FailedToRead,
    FailedToParseDate,
    SchemaParseErr(Vec<String>),
    DataSchemaCheckErr(String),
    NoData,
    UnexpectedError,
}