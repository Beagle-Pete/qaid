#[derive(Debug, Clone, PartialEq)]
pub enum APIError {
    FailedToOpen,
    FailedToRead,
    FailedToReadCsv,
    FailedToParseDate,
    SchemaParseErr(Vec<String>),
    DataSchemaCheckErr(String),
    NoData,
    UnexpectedError,
}