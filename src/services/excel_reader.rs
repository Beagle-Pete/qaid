use crate::{domain::data_stores::DBReader};
use crate::domain::{APIError, PrimType, PrimTypeData};

use calamine::{Data, Reader, Xlsx, open_workbook};

#[derive(Debug, Default)]
pub struct ExcelReader {
    headers: Vec<String>,
    data: Vec<Vec<CellInfo>>,
    schema: Vec<PrimType>,
}

#[derive(Debug, Clone)]
struct CellInfo {
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

        // Get schema of data
        let mut schema = Vec::new();

        let first_row = rows[0].clone();

        for col in first_row.iter() {
            match col.data {
                Ok(PrimTypeData::Bool(_)) =>  schema.push(PrimType::Bool),
                Ok(PrimTypeData::DateTime(_)) =>  schema.push(PrimType::DateTime),
                Ok(PrimTypeData::Empty) =>  schema.push(PrimType::Empty),
                Ok(PrimTypeData::UnexpectedError) =>  schema.push(PrimType::UnexpectedError),
                Ok(PrimTypeData::Float(_)) =>  schema.push(PrimType::Float),
                Ok(PrimTypeData::Int(_)) =>  schema.push(PrimType::Int),
                Ok(PrimTypeData::String(_)) => schema.push(PrimType::String),
                _ => schema.push(PrimType::UnexpectedError),
            }
        }

        println!("{}, {}", range.get_size().0, range.get_size().1);
        
        self.headers = headers;
        self.data = rows;
        self.schema = schema;
        Ok(())
    }
}