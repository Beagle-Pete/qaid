use crate::domain::schema::FieldSchema;

#[derive(Debug, Clone, PartialEq)]
pub enum APIError {
    FailedToOpen,
    FailedToRead,
    FailedToReadCsv,
    FailedToParseDate,
    BadSchema(BadSchemaInfo),
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BadSchemaInfo {
    pub incorrect_type: Vec<FieldSchema>,
    pub duplicate_names: Vec<String>,
    pub missing_fields: Vec<String>,
}