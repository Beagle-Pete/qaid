#[derive(Debug, Clone, PartialEq)]
pub enum APIError {
    FailedToOpen,
    FailedToRead,
    FailedToReadCsv,
    FailedToParseDate,
    SchemaParseErr(Vec<String>),
    DataSchemaCheckErr(String),
    BadHeaders(BadHeaderInfo),
    NoData,
    EmptyFile,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BadHeaderInfo {
    pub empty: Vec<usize>,
    pub duplicate: Vec<String>,
}