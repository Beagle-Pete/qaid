use std::collections::HashMap;

use crate::domain::{APIError, Cell, ReportError, ReportInfo};

pub trait DBReader {
    fn read_db(&mut self) -> Result<(), APIError>;

    fn get_schema(&self) -> &HashMap<String, String>;

    fn get_data(&self) -> &Vec<Vec<Cell>>;

    fn get_data_at(&self, row: usize, col: usize) -> Option<&Cell>;

    fn print_data(&self);
    
    fn get_headers(&self) -> &Vec<String>;

    fn add_issue(&mut self, error_type: ReportError, error_info: ReportInfo);

    fn get_issues(&self) -> &HashMap<ReportError, Vec<ReportInfo>>;
    
}