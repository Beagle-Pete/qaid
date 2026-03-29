use crate::{domain::data_stores::DBReader};
use crate::domain::{APIError, PrimType, PrimTypeData, SchemaInfo};

use calamine::{Data, Reader, Xlsx, open_workbook};

#[derive(Debug, Default)]
pub struct ExcelReader {
    pub headers: Vec<String>,
    pub data: Vec<Vec<CellInfo>>,
    pub data_size: (usize, usize),
    pub schema: Vec<SchemaInfo>,
}

#[derive(Debug, Clone)]
pub struct CellInfo {
    data: Result<PrimTypeData, APIError>,
    cell_address: (usize, usize),

}

impl DBReader for ExcelReader {
    fn read_db(&mut self, path: String, table: String) -> Result<(), APIError> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|_| APIError::FailedToOpen)?;
        
        let range = workbook.worksheet_range(&table)
            .map_err(|_| APIError::FailedToRead)?;

        let row_count = range.get_size().0;
        let col_count = range.get_size().1;

        let headers = range.headers()
            .ok_or(APIError::UnexpectedError)?;

        let mut rows= Vec::with_capacity(row_count);
        
        // Iterate through rows
        for (ii, row) in range.rows().enumerate() {

            // Iterate through columns
            let mut cells = Vec::with_capacity(col_count);
            for (jj, cell) in row.iter().enumerate() {
                let cell_data = match cell {
                    Data::Bool(val) =>  Ok(PrimTypeData::Bool(val.to_owned())),
                    Data::DateTime(val) => {
                        let date_time_opt = val.as_datetime();

                        if let Some(date_time) = date_time_opt {
                            Ok(PrimTypeData::DateTime(date_time))
                        } else {
                            Err(APIError::FailedToParseDate)
                        }
                    },
                    // TODO: Verify what the iso format from calamine is
                    Data::DateTimeIso(val) => {
                        let date_time_res = chrono::NaiveDateTime::parse_from_str(val, "%Y-%m-%d %H:%M:%S");

                        if let Ok(date_time) = date_time_res {
                            Ok(PrimTypeData::DateTime(date_time))
                        } else {
                            Err(APIError::FailedToParseDate)
                        }                        
                    },
                    // TODO: Implement this correctly. This should be chrono::TimeDelta
                    Data::DurationIso(_) =>  Ok(PrimTypeData::DateTime(chrono::NaiveDateTime::default())),
                    Data::Empty =>  Ok(PrimTypeData::Empty),
                    // TODO: Pass error to PrimTypeData::UnexpectedError 
                    Data::Error(_) =>  Ok(PrimTypeData::UnexpectedError),
                    Data::Float(val) =>  Ok(PrimTypeData::Float(val.to_owned())),
                    Data::Int(val) =>  Ok(PrimTypeData::Int(val.to_owned())),
                    Data::String(val) => Ok(PrimTypeData::String(val.to_owned())),
                };

                cells.push(CellInfo{
                    data: cell_data,
                    cell_address: (ii, jj),
                });

            }
            rows.push(cells);
        }

        // Remove header
        if !rows.is_empty() {
            rows.remove(0);
        }

        // Get schema of data from first row
        let mut schema = Vec::new();

        let first_row = rows[0].clone();

        for (index, col) in first_row.iter().enumerate() {
            let header_name = headers[index].clone();
            match col.data {
                Ok(PrimTypeData::Bool(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::Bool)),
                Ok(PrimTypeData::DateTime(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::DateTime)),
                Ok(PrimTypeData::Empty) =>  schema.push(SchemaInfo::new(header_name, PrimType::Empty)),
                Ok(PrimTypeData::UnexpectedError) =>  schema.push(SchemaInfo::new(header_name, PrimType::UnexpectedError)),
                Ok(PrimTypeData::Float(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::Float)),
                Ok(PrimTypeData::Int(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::Int)),
                Ok(PrimTypeData::String(_)) => schema.push(SchemaInfo::new(header_name, PrimType::String)),
                _ => schema.push(SchemaInfo::new(header_name, PrimType::UnexpectedError)),
            }
        }
        
        self.headers = headers;
        self.data = rows;
        self.data_size = (row_count, col_count);
        self.schema = schema;
        Ok(())
    }

    fn is_schema_ok(&self) -> Result<bool, APIError> {
        let mut is_ok = true;
        let mut headers_with_err = vec![];

        for schema_info in &self.schema {
            if schema_info.data_type == PrimType::UnexpectedError {
                is_ok = false;
                headers_with_err.push(schema_info.header_name.clone());
            }
        }

        if headers_with_err.is_empty() {
            Ok(is_ok)
        } else {
            Err(APIError::SchemaParseErr(headers_with_err))
        }
    }

    fn check_against_schema(&self, schema: &Vec<SchemaInfo>) -> Result<(), APIError> {
        // TODO: This check should be able to perform a partial check if length of schema and data aren't equal
        // If schema header and data header are in different order this should get corresponding index of both arrays
        // If data has more or less headers it should report out the discrepancy, but compare what is available
        if schema.len() != self.data_size.1 {
            return Err(APIError::DataSchemaCheckErr("Schema length and data lenth are not equal".to_owned()))
        }

        let mut mismatch = "".to_owned();

        for row in &self.data {
            for (index, cell) in row.iter().enumerate() {                                
                if let Ok(cell_data) = &cell.data && cell_data.kind() != schema[index].data_type {
                    mismatch.push_str(&format!("Cell: ({}, {}) - Data: {:?} - Schema: {:?}\n", cell.cell_address.0, cell.cell_address.0, cell_data, schema[index]));
                }
            }
        }

        if !mismatch.is_empty() {
            println!("{mismatch}");
            return Err(APIError::DataSchemaCheckErr(mismatch))
        }

        Ok(())
    }
}