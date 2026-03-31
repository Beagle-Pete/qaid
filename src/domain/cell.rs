use crate::domain::{APIError, PrimTypeData};

#[derive(Debug, Clone)]
pub struct Cell {
    pub data: Result<PrimTypeData, APIError>,
    pub cell_address: (usize, usize),
}

impl Cell {
    pub fn new(data: Result<PrimTypeData, APIError>, cell_address: (usize, usize)) -> Self {
        Self {
            data,
            cell_address,
        }
    }
}