use std::collections::HashMap;

use calamine::{Data, HeaderRow, Reader, Xlsx, open_workbook};
use chrono::{Duration, NaiveDateTime, NaiveDate, NaiveTime};

use crate::{domain::data_stores::DBReader};
use crate::domain::{APIError, Cell, PrimType, PrimTypeData, ReportError, ReportInfo, SchemaInfo};

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
    report: HashMap<ReportError, Vec<ReportInfo>>,
}

impl ExcelReaderBuilder {
    pub fn parse(db: String, sheet: String) -> ExcelReader {
        ExcelReader {
            db,
            sheet,
            // report: HashMap::new()
            ..Default::default()
        }
    }
}

impl DBReader for ExcelReader {
    fn read_db(&mut self) -> Result<(), APIError> {
        let mut workbook: Xlsx<_> = open_workbook(self.db.clone())
            .map_err(|_| APIError::FailedToOpen)?;
        
        let range = workbook
            .with_header_row(HeaderRow::FirstNonEmptyRow)
            .worksheet_range(&self.sheet)
            .map_err(|_| APIError::FailedToRead)?;

        let (row_count_with_header, col_count) = range.get_size();

        let headers = range.headers()
            .ok_or(APIError::UnexpectedError)?;

        let mut rows= Vec::with_capacity(row_count_with_header);
        
        // Iterate through rows. Skips first row with headers
        for (ii, row) in range.rows().enumerate().skip(1) {

            // Iterate through columns
            let mut cells = Vec::with_capacity(col_count);
            for (jj, cell) in row.iter().enumerate() {
                let cell_data = match cell {
                    Data::Bool(val) =>  PrimTypeData::Bool(val.to_owned()),
                    Data::DateTime(val) => {
                        let (y, m, d, hr, min, sec, milli) = val.to_ymd_hms_milli();
                        let date = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();

                        // Add up time as milliseconds and add to midnight to avoid rounding errors
                        let total_ms = hr as i64 * 3_600_000 + min as i64 * 60_000 + sec as i64 * 1_000 + milli as i64;
                        let midnight = NaiveTime::from_num_seconds_from_midnight_opt(0, 0)
                            .unwrap_or_default();
                        let time = midnight + Duration::milliseconds(total_ms);

                        let date_time = NaiveDateTime::new(date, time);
                        PrimTypeData::DateTime(date_time)
                    },
                    // TODO: Add more parse ruls. ISO 8601 has many valid formats.
                    Data::DateTimeIso(val) => {
                        // dbg!(&tt);
                        match NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S") {
                            Ok(date_time) => PrimTypeData::DateTime(date_time),
                            Err(_) => {
                                dbg!("Error Here {} {}", ii-1, jj);
                                let error_info = ReportInfo::new((ii-1, jj), (ii-1, jj), val.to_string(), "Failed to parse DateTimeIso".to_owned());
                                self.add_issue(ReportError::FailedToParse, error_info);
                                PrimTypeData::UnexpectedError
                            },
                        }
                    },
                    // TODO: Implement this correctly. This should be chrono::TimeDelta
                    Data::DurationIso(_) => {
                        PrimTypeData::DateTime(NaiveDateTime::default())
                    },
                    Data::Empty =>  PrimTypeData::Empty,
                    Data::Float(val) =>  PrimTypeData::Float(val.to_owned()),
                    Data::Int(val) =>  PrimTypeData::Int(val.to_owned()),
                    Data::String(val) => {
                        let val = val.trim().to_owned();
                        if val.is_empty() {
                            PrimTypeData::Empty
                        } else {
                            PrimTypeData::String(val)
                        }
                    },
                    // TODO: Pass error to PrimTypeData::UnexpectedError 
                    Data::Error(_) =>  PrimTypeData::UnexpectedError,
                };

                cells.push(Cell{
                    data: cell_data,
                    cell_address: (ii-1, jj),
                });

            }
            rows.push(cells);
        }

        if rows.is_empty() {
            return Err(APIError::NoData)
        }

        let row_count = row_count_with_header - 1;

        // Determine schema from the first row
        let schema = parse_shema(&headers, rows[0].clone())?;

        // TODO: check for merged cells
        
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
    
    fn get_data_at(&self, row: usize, col: usize) -> Option<&Cell> {
        self.data
            .get(row)?
            .get(col)
    }

    fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }

    fn add_issue(&mut self, error_type: ReportError, error_info: ReportInfo) {
        self.report
            .entry(error_type)
            .or_default()
            .push(error_info);
    }

    fn get_issues(&self) -> &HashMap<ReportError, Vec<ReportInfo>> {
        &self.report
    }
}

fn parse_shema(header: &[String], row: Vec<Cell>) -> Result<Vec<SchemaInfo>, APIError> {
    let mut schema = Vec::new();

    let mut schema_is_ok = true;
    let mut headers_with_err = vec![];

    for (index, col) in row.iter().enumerate() {
        let header_name = header[index].clone();
        match col.data {
            PrimTypeData::Bool(_) =>  schema.push(SchemaInfo::new(header_name, PrimType::Bool)),
            PrimTypeData::DateTime(_) =>  schema.push(SchemaInfo::new(header_name, PrimType::DateTime)),
            PrimTypeData::Empty =>  schema.push(SchemaInfo::new(header_name, PrimType::Empty)),
            PrimTypeData::Float(_) =>  schema.push(SchemaInfo::new(header_name, PrimType::Float)),
            PrimTypeData::Int(_) =>  schema.push(SchemaInfo::new(header_name, PrimType::Int)),
            PrimTypeData::String(_) => schema.push(SchemaInfo::new(header_name, PrimType::String)),
            PrimTypeData::UnexpectedError =>  {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_excel_read() {
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Table01.xlsx".to_owned(), "Sheet1".to_owned());
        excel_file.read_db().unwrap();

        assert_eq!(excel_file.headers, ["PID", "Impressions", "Placements", "DateTime", "Boolean"]);
        assert_eq!(excel_file.data_size, (10, 5));

        let (row, col) = (0, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::String("56b605f1-ddb7-4ff7-8180-1a5c8e11147a".to_owned()));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (2, 1);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::Float(34_f64));
        
        let (row, col) = (2, 4);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::Bool(false));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));
        
        let (row, col) = (2, 2);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::Empty);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));
        
        let (row, col) = (5, 1);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::Empty);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (4, 3);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(), 
                NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));
        
        let (row, col) = (0, 3);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), 
                NaiveTime::from_hms_micro_opt(1, 12, 0, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (3, 3);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2025, 12, 9).unwrap(), 
                NaiveTime::from_hms_micro_opt(14, 9, 0, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        dbg!(excel_file.get_issues());
    }

    #[test]
    fn datatimeiso_test() {
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Excel_DateTimeIso.xlsx".to_owned(), "Sheet1".to_owned());
        excel_file.read_db().unwrap();

        let (row, col) = (0, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 7, 22).unwrap(), 
                NaiveTime::from_hms_micro_opt(8, 45, 30, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (1, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2019, 11, 3).unwrap(), 
                NaiveTime::from_hms_micro_opt(23, 17, 5, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (2, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2025, 1, 14).unwrap(), 
                NaiveTime::from_hms_micro_opt(11, 32, 48, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (3, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2022, 4, 29).unwrap(), 
                NaiveTime::from_hms_micro_opt(16, 54, 11, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (4, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2020, 9, 8).unwrap(), 
                NaiveTime::from_hms_micro_opt(3, 6, 27, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (5, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 6, 17).unwrap(), 
                NaiveTime::from_hms_micro_opt(19, 21, 53, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (6, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2018, 12, 31).unwrap(), 
                NaiveTime::from_hms_micro_opt(7, 59, 44, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (7, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2021, 3, 5).unwrap(), 
                NaiveTime::from_hms_micro_opt(14, 38, 2, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (8, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(3089, 8, 20).unwrap(), 
                NaiveTime::from_hms_micro_opt(22, 10, 36, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (9, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(1912, 5, 11).unwrap(), 
                NaiveTime::from_hms_micro_opt(5, 47, 19, 0).unwrap()
            )
        ));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));
    }
}