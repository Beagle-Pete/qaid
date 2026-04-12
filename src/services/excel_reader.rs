use std::collections::HashMap;

use calamine::{Cell as CalCell, CellType, Data as CalData, HeaderRow, Range, Reader, Xlsx, open_workbook};
use chrono::{Duration, NaiveDateTime, NaiveDate, NaiveTime};

use crate::domain::{APIError, Cell, Data, Headers, ReportError, ReportInfo, data_stores::DBReader, schema::{FieldSchema, Schema}};

#[derive(Debug)]
pub struct ExcelReaderBuilder {
    pub db: String,
    pub sheet: String,
}

#[derive(Debug, Default)]
pub struct ExcelReader {
    db: String,
    sheet: String,
    schema_definition: Vec<FieldSchema>,
    headers: Headers,
    data: Data,
    data_size: (usize, usize),
    schema: Schema,
    report: HashMap<ReportError, Vec<ReportInfo>>,
}

impl ExcelReaderBuilder {
    pub fn parse(db: String, sheet: String, schema_definition: Vec<FieldSchema>) -> ExcelReader {
        ExcelReader {
            db,
            sheet,
            schema_definition,
            ..Default::default()
        }
    }
}

impl DBReader for ExcelReader {
    fn read_db(&mut self) -> Result<(), APIError> {
        let mut workbook: Xlsx<_> = open_workbook(self.db.clone())
            .map_err(|_| APIError::FailedToOpen)?;
        
        workbook.load_merged_regions()
            .map_err(|_| APIError::FailedToRead)?;
        
        let range = workbook
            .with_header_row(HeaderRow::FirstNonEmptyRow)
            .worksheet_range(&self.sheet)
            .map_err(|_| APIError::FailedToRead)?;

        if range.height() == 0_usize {
            return Err(APIError::EmptyFile)
        }

        // Parse headers
        let headers = {
            let headers_tmp = range.headers()
            .ok_or(APIError::UnexpectedError)?;

            Headers::parse(headers_tmp)?
        };

        // Parse schema
        let schema = Schema::parse(self.schema_definition.clone(), &headers)?;

        // Remove header from Range
        let range = remove_row(&range, 0);
    
        let (row_count, col_count) = range.get_size();
        let mut data = Vec::with_capacity(row_count);
        
        
        // Iterate through rows
        for (ii, row) in range.rows().enumerate() {
            // Iterate through columns
            let mut cells = Vec::with_capacity(col_count);
            for (jj, cell) in row.iter().enumerate() {
                // dbg!(&ii, &jj, &cell);
                let cell_data = match cell {
                    CalData::Bool(val) => val.to_string(),
                    CalData::DateTime(val) => {
                        let (y, m, d, hr, min, sec, milli) = val.to_ymd_hms_milli();
                        let date = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
                        // dbg!(&date);

                        // Add up time as milliseconds and add to midnight to avoid rounding errors
                        let total_ms = hr as i64 * 3_600_000 + min as i64 * 60_000 + sec as i64 * 1_000 + milli as i64;
                        let midnight = NaiveTime::from_num_seconds_from_midnight_opt(0, 0)
                            .unwrap_or_default();
                        let time = midnight + Duration::milliseconds(total_ms);

                        let date_time = NaiveDateTime::new(date, time);
                        date_time.format("%Y-%m-%dT%H:%M:%S").to_string()
                    },
                    // TODO: Add more parse rules. ISO 8601 has many valid formats.
                    CalData::DateTimeIso(val) => val.to_string(),
                    // TODO: Implement this correctly. This should be chrono::TimeDelta
                    CalData::DurationIso(val) => val.to_string(),
                    CalData::Empty => "".to_owned(),
                    CalData::Float(val) => val.to_string(),
                    CalData::Int(val) => val.to_string(),
                    CalData::String(val) => val.to_string(),
                    // TODO: Pass error to PrimTypeData::UnexpectedError 
                    CalData::Error(_) => {
                        let context = format!("Unexpected error at ({},{}). Could not determine data type.", ii, jj);
                        self.add_issue(ReportError::UnexpectedError, ReportInfo::new((ii, jj), (ii, jj), "".to_owned(), context));
                        return Err(APIError::UnexpectedError);
                    },
                };

                cells.push(cell_data);
            }
            data.push(cells);
        }

        let (data, report_tmp) = Data::parse(data, headers.as_ref(), schema.as_ref())?;
        for (report_error, report_info) in report_tmp {
            self.add_issue(report_error, report_info);
        }

        // Get merged cells
        workbook.merged_regions_by_sheet(&self.sheet).iter()
            .for_each(|(_, _, dimensions)| {
                let (row_start, col_start) = dimensions.start;
                let row_start = row_start - 1;
                let start = (row_start as usize, col_start as usize);

                let (row_end, col_end) = dimensions.end;
                let row_end = row_end - 1;
                let end = (row_end as usize, col_end as usize);

                let val = data.as_ref()[start.0][start.1].data.to_string();
                let context = format!("Merged cell at ({},{}) to ({},{})", start.0, start.1, end.0, end.1);
                self.add_issue(ReportError::MergedCells, ReportInfo::new(start, end, val, context))
            });
        
        self.headers = headers;
        self.data = data;
        self.data_size = (row_count, col_count);
        self.schema = schema;

        Ok(())
    }

    fn get_schema(&self) -> &HashMap<String, String> {
        self.schema.as_ref()
    }

    fn get_data(&self) -> &Vec<Vec<Cell>> {
        self.data.as_ref()
    }
    
    fn get_data_at(&self, row: usize, col: usize) -> Option<&Cell> {
        self.data
            .as_ref()
            .get(row)?
            .get(col)
    }

    fn get_headers(&self) -> &Vec<String> {
        self.headers.as_ref()
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

    fn print_data(&self) {
        self.data.print_data();
    }
}

fn remove_row<T: CellType + Clone>(range: &Range<T>, row_to_remove: usize) -> Range<T> {
    let Some((start_row, start_col)) = range.start() else {
        return range.clone();
    };

    // `used_cells()` returns relative row/col coordinates.
    let cells = range
        .used_cells()
        .filter_map(|(row, col, value)| {
            if row == row_to_remove {
                return None;
            }

            let shifted_row = if row > row_to_remove { row - 1 } else { row };

            Some(CalCell::new(
                (start_row + shifted_row as u32, start_col + col as u32),
                value.clone(),
            ))
        })
        .collect();

    Range::from_sparse(cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::PrimTypeData;

    #[test]
    fn successful_excel_read() {
        let schema = vec![
            FieldSchema::new("PID".to_owned(), "String".to_owned()),
            FieldSchema::new("Impressions".to_owned(), "Int".to_owned()),
            FieldSchema::new("Placements".to_owned(), "String".to_owned()),
            FieldSchema::new("DateTime".to_owned(), "DateTime".to_owned()),
            FieldSchema::new("Boolean".to_owned(), "Bool".to_owned()),
        ];
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Excel_Normal_01.xlsx".to_owned(), "Sheet1".to_owned(), schema);
        excel_file.read_db().unwrap();

        assert_eq!(excel_file.get_headers(), &["PID", "Impressions", "Placements", "DateTime", "Boolean"]);
        assert_eq!(excel_file.data_size, (10, 5));

        let (row, col) = (0, 0);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::String("56b605f1-ddb7-4ff7-8180-1a5c8e11147a".to_owned()));
        assert_eq!(excel_file.get_data_at(row, col).unwrap().cell_address, (row, col));

        let (row, col) = (2, 1);
        assert_eq!(excel_file.get_data_at(row, col).unwrap().data, PrimTypeData::Int(34));
        
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

        let issues = excel_file.get_issues();
        let empty_cell_issues = issues.get(&ReportError::EmptyCell).unwrap();
        dbg!(&empty_cell_issues);

        assert_eq!(issues.len(), 1);
        assert_eq!(empty_cell_issues.len(), 2);

        assert_eq!(empty_cell_issues[0].start, (2, 2));
        assert_eq!(empty_cell_issues[0].end, (2, 2));

        assert_eq!(empty_cell_issues[1].start, (5, 1));
        assert_eq!(empty_cell_issues[1].end, (5, 1));
    }

    #[test]
    fn datatimeiso_test() {
        let schema = vec![
            FieldSchema::new("DateTimeIso".to_owned(), "DateTime".to_owned()),
        ];
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Excel_DateTimeIso.xlsx".to_owned(), "Sheet1".to_owned(), schema);
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

    #[test]
    fn merged_cells_test() {
        let schema = vec![
            FieldSchema::new("PID".to_owned(), "String".to_owned()),
            FieldSchema::new("Impressions".to_owned(), "Int".to_owned()),
            FieldSchema::new("Placements".to_owned(), "String".to_owned()),
            FieldSchema::new("DateTime".to_owned(), "DateTime".to_owned()),
            FieldSchema::new("Boolean".to_owned(), "Bool".to_owned()),
        ];
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Excel_Merged_Cells.xlsx".to_owned(), "Sheet1".to_owned(), schema);
        excel_file.read_db().unwrap();

        let issues = excel_file.get_issues();
        let merged_cell_issues = excel_file.get_issues().get(&ReportError::MergedCells).unwrap();
        let parse_issues = excel_file.get_issues().get(&ReportError::FailedToParse).unwrap();

        assert_eq!(issues.len(), 2);
        assert_eq!(merged_cell_issues.len(), 2);
        assert_eq!(parse_issues.len(), 2);

        assert_eq!(merged_cell_issues[0].start, (1, 0));
        assert_eq!(merged_cell_issues[0].end, (2, 0));

        assert_eq!(merged_cell_issues[1].start, (1, 2));
        assert_eq!(merged_cell_issues[1].end, (2, 3));

        // Date times that are in serial numbers are not covered yet
        assert_eq!(parse_issues[0].start, (1, 3));
        assert_eq!(parse_issues[0].end, (1, 3));
        assert_eq!(parse_issues[0].val, "45661.2916666667".to_owned());

        assert_eq!(parse_issues[1].start, (2, 3));
        assert_eq!(parse_issues[1].end, (2, 3));
        assert_eq!(parse_issues[1].val, "45878".to_owned());
    }

    #[test]
    fn file_without_data_should_return_err() {
        let schema = vec![
            FieldSchema::new("ID".to_owned(), "String".to_owned()),
            FieldSchema::new("City".to_owned(), "String".to_owned()),
            FieldSchema::new("State".to_owned(), "String".to_owned()),
            FieldSchema::new("Zip Code".to_owned(), "String".to_owned()),
        ];
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Excel_No_Data.xlsx".to_owned(), "Sheet1".to_owned(), schema);
        
        assert_eq!(excel_file.read_db(), Err(APIError::NoData));
    }

    #[test]
    fn empty_table_should_return_err() {
        let schema = vec![
            FieldSchema::new("col1".to_owned(), "String".to_owned()),
            FieldSchema::new("col2".to_owned(), "String".to_owned()),
            FieldSchema::new("col3".to_owned(), "Int".to_owned()),
        ];
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Excel_Empty.xlsx".to_owned(), "Sheet1".to_owned(), schema);
        
        assert_eq!(excel_file.read_db(), Err(APIError::EmptyFile));
    }

    #[test]
    fn test_for_duplicate_headers() {
        let schema = vec![
            FieldSchema::new("col1".to_owned(), "String".to_owned()),
            FieldSchema::new("col2".to_owned(), "String".to_owned()),
            FieldSchema::new("col3".to_owned(), "Int".to_owned()),
        ];
        let mut excel_file = ExcelReaderBuilder::parse("tests/assets/Excel_Bad_Header.xlsx".to_owned(), "Sheet1".to_owned(), schema);
        
        let expected_err = crate::domain::BadHeaderInfo {
            empty: vec![1, 7],
            duplicate: vec!["city".to_owned(), "zipcode".to_owned()]
        };
        
        assert_eq!(excel_file.read_db(), Err(APIError::BadHeaders(expected_err)));
    }
}
