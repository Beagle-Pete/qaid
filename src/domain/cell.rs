use crate::domain::{APIError, PrimTypeData};

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub data: PrimTypeData,
    pub cell_address: (usize, usize),
}

impl Cell {
    pub fn new(data: PrimTypeData, cell_address: (usize, usize)) -> Self {
        Self {
            data,
            cell_address,
        }
    }
}