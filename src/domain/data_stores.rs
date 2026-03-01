use crate::domain::APIError;

pub trait DBReader {
    fn read_db(&mut self, path: String, table: String) -> Result<(), APIError>;
}