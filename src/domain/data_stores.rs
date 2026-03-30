use crate::domain::{APIError, Cell, SchemaInfo};

pub trait DBReader {
    fn read_db(&mut self) -> Result<(), APIError>;

    fn get_schema(&self) -> &Vec<SchemaInfo>;

    fn get_data(&self) -> &Vec<Vec<Cell>>;
    
    fn get_headers(&self) -> &Vec<String>;
    
}