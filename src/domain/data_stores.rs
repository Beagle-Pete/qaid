use crate::domain::{APIError, SchemaInfo};

pub trait DBReader {
    fn read_db(&mut self, path: String, table: String) -> Result<(), APIError>;

    fn is_schema_ok(&self) -> Result<bool, APIError>;

    fn check_against_schema(&self, schema: &Vec<SchemaInfo>) -> Result<(), APIError>;
}