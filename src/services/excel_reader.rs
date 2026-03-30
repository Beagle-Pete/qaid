use calamine::{Data, Reader, Xlsx, open_workbook};

use crate::{domain::data_stores::DBReader};
use crate::domain::{APIError, Cell, PrimType, PrimTypeData, SchemaInfo};

#[derive(Debug)]
pub struct ExcelReaderBuilder {
    pub db: String,
    pub sheet: String,
}

#[derive(Debug, Default)]
pub struct ExcelReader {
    db: String,
    sheet: String,
    headers: Vec<String>,
    data: Vec<Vec<Cell>>,
    data_size: (usize, usize),
    schema: Vec<SchemaInfo>,
    has_merged_cells: bool,
}

impl ExcelReaderBuilder {
    pub fn parse(db: String, sheet: String) -> ExcelReader {
        ExcelReader {
            db,
            sheet,
            ..Default::default()
        }
    }
}

impl DBReader for ExcelReader {
    fn read_db(&mut self) -> Result<(), APIError> {
        let mut workbook: Xlsx<_> = open_workbook(self.db.clone())
            .map_err(|_| APIError::FailedToOpen)?;
        
        let range = workbook.worksheet_range(&self.sheet)
            .map_err(|_| APIError::FailedToRead)?;

        let (row_count, col_count) = range.get_size();

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

                cells.push(Cell{
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

        if rows.is_empty() {
            return Err(APIError::NoData)
        }

        // Determine schema from the first row
        let schema = parse_shema(&headers, rows[0].clone())?;
        
        self.headers = headers;
        self.data = rows;
        self.data_size = (row_count, col_count);
        self.schema = schema;

        Ok(())
    }

    fn get_schema(&self) -> &Vec<SchemaInfo> {
        &self.schema
    }

    fn get_data(&self) -> &Vec<Vec<Cell>> {
        &self.data
    }

    fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

fn parse_shema(header: &[String], row: Vec<Cell>) -> Result<Vec<SchemaInfo>, APIError> {
    let mut schema = Vec::new();

    let mut schema_is_ok = true;
    let mut headers_with_err = vec![];

    for (index, col) in row.iter().enumerate() {
        let header_name = header[index].clone();
        match col.data {
            Ok(PrimTypeData::Bool(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::Bool)),
            Ok(PrimTypeData::DateTime(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::DateTime)),
            Ok(PrimTypeData::Empty) =>  schema.push(SchemaInfo::new(header_name, PrimType::Empty)),
            Ok(PrimTypeData::Float(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::Float)),
            Ok(PrimTypeData::Int(_)) =>  schema.push(SchemaInfo::new(header_name, PrimType::Int)),
            Ok(PrimTypeData::String(_)) => schema.push(SchemaInfo::new(header_name, PrimType::String)),
            Ok(PrimTypeData::UnexpectedError) =>  {
                schema_is_ok = false;
                headers_with_err.push(header_name.clone());
                schema.push(SchemaInfo::new(header_name, PrimType::UnexpectedError))
            },
            Err(_) => {
                schema_is_ok = false;
                headers_with_err.push(header_name.clone());
                schema.push(SchemaInfo::new(header_name, PrimType::UnexpectedError))
            },
        }
    }

    if !schema_is_ok {
        return Err(APIError::SchemaParseErr(headers_with_err))
    } 

    Ok(schema)
}