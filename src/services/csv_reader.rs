use std::{
    collections::HashMap,
    fs::File,
};

use csv::{Reader, StringRecord};

use crate::domain::{APIError, Cell, Data, Headers, ReportError, ReportInfo,data_stores::DBReader, schema::{FieldSchema, Schema}};

#[derive(Debug)]
pub struct CsvReaderBuilder {
    pub file_path: String,
}

#[derive(Debug, Default)]
pub struct CsvReader {
    file: String,
    schema_definition: Vec<FieldSchema>,
    headers: Headers,
    data: Data,
    data_size: (usize, usize),
    // schema: Vec<SchemaInfo>,
    schema: Schema,
    report: HashMap<ReportError, Vec<ReportInfo>>,
}

impl CsvReaderBuilder {
    pub fn parse(file: String, schema_definition: Vec<FieldSchema>) -> CsvReader {
        CsvReader {
            file,
            schema_definition,
            ..Default::default()
        }
    }
}

impl DBReader for CsvReader {
    fn read_db(&mut self) -> Result<(), APIError> {
        let file = File::open(&self.file)
            .map_err(|_| APIError::FailedToOpen)?;
        let mut rdr = Reader::from_reader(file);

        // Get headers
        let headers = {
            let r_headers = rdr.headers()
            .map_err(|_| APIError::FailedToRead)?;
        
            let vec_headers = record_to_vec(r_headers);

            Headers::parse(vec_headers)?
        };

        // Get schema. Schema for .csv file will be all Strings
        let schema = Schema::parse(self.schema_definition.clone(), &headers)?;

        // Get data
        let mut data = vec![];

        // Iterate through rows
        for record in rdr.records() {
            let row = record
                .map_err(|_| APIError::FailedToReadCsv)?;

            let row_data = record_to_vec(&row);

            let mut row_data3 = vec![];
            for col in row_data {
                row_data3.push(col.to_owned());
            }
            data.push(row_data3);
        }

        let (data, data_report) = Data::parse(data, headers.as_ref(), schema.as_ref())?;
        for (report_error, report_info) in data_report {
            self.add_issue(report_error, report_info);
        }

        let row_count = data.as_ref().len();
        let col_count = headers.as_ref().len();

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

fn record_to_vec(record: &StringRecord) -> Vec<String> {
    record.iter()
        .map(|el| el.trim().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    use super::*;

    use crate::domain::PrimTypeData;

    #[test]
    fn successful_csv_read() {
        let schema = vec![
            FieldSchema::new("col1".to_owned(), "String".to_owned()),
            FieldSchema::new("col2".to_owned(), "String".to_owned()),
            FieldSchema::new("col3".to_owned(), "Int".to_owned()),
        ];
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/csv_01.csv".to_owned(), schema);
        csv_file.read_db().unwrap();

        assert_eq!(csv_file.get_headers(), &["col1", "col2", "col3"]);

        assert_eq!(csv_file.data_size, (4, 3));

        assert_eq!(csv_file.get_data_at(0, 1).unwrap().data, PrimTypeData::String("String1".to_owned()));
        assert_eq!(csv_file.get_data_at(0, 1).unwrap().cell_address, (0, 1));
    }

    #[test]
    fn successful_csv_read_2() {
        let schema = vec![
            FieldSchema::new("PID".to_owned(), "String".to_owned()),
            FieldSchema::new("Impressions".to_owned(), "Float".to_owned()),
            FieldSchema::new("Placements".to_owned(), "String".to_owned()),
            FieldSchema::new("DateTime".to_owned(), "DateTime".to_owned()),
            FieldSchema::new("Boolean".to_owned(), "Bool".to_owned()),
        ];
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/csv_02.csv".to_owned(), schema);
        csv_file.read_db().unwrap();

        assert_eq!(csv_file.get_headers(), &["PID", "Impressions", "Placements", "DateTime", "Boolean"]);

        assert_eq!(csv_file.data_size, (10, 5));

        assert_eq!(csv_file.get_data_at(0, 1).unwrap().data, PrimTypeData::Float(2.0));
        assert_eq!(csv_file.get_data_at(0, 1).unwrap().cell_address, (0, 1));

        let (row, col) = (9, 2);
        assert_eq!(
            csv_file.get_data_at(row, col).unwrap().data, 
            PrimTypeData::String("As the rental car rolled to a stop on the dark road, her fear increased by the moment.".to_owned())
        );
        
        let (row, col) = (0, 3);
        assert_eq!(csv_file.get_data_at(row, col).unwrap().data, PrimTypeData::DateTime(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(), 
                NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap()
            )
        ));
        assert_eq!(csv_file.get_data_at(row, col).unwrap().cell_address, (row, col));
    }

    #[test]
    fn empty_csv_should_fail() {
        let schema = vec![
            FieldSchema::new("col1".to_owned(), "String".to_owned()),
            FieldSchema::new("col2".to_owned(), "String".to_owned()),
            FieldSchema::new("col3".to_owned(), "Int".to_owned()),
        ];
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/csv_empty.csv".to_owned(), schema);
        
        assert_eq!(csv_file.read_db(), Err(APIError::NoData));
    }

    #[test]
    fn file_does_not_exist() {
        let schema = vec![FieldSchema::default()];
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/Non_existant_file.csv".to_owned(), schema);
        
        assert_eq!(csv_file.read_db(), Err(APIError::FailedToOpen));
    }

    #[test]
    fn header_data_unequal_length_should_fail() {
        let schema = vec![
            FieldSchema::new("col1".to_owned(), "String".to_owned()),
            FieldSchema::new("col2".to_owned(), "String".to_owned()),
            FieldSchema::new("col3".to_owned(), "Int".to_owned()),
        ];
        let mut csv_file_01 = CsvReaderBuilder::parse("tests/assets/csv_unequal_length_01.csv".to_owned(), schema.clone());
        let mut csv_file_02 = CsvReaderBuilder::parse("tests/assets/csv_unequal_length_02.csv".to_owned(), schema.clone());
        let mut csv_file_03 = CsvReaderBuilder::parse("tests/assets/csv_unequal_length_03.csv".to_owned(), schema);

        assert_eq!(csv_file_01.read_db(), Err(APIError::FailedToReadCsv));
        assert_eq!(csv_file_02.read_db(), Err(APIError::FailedToReadCsv));
        assert_eq!(csv_file_03.read_db(), Err(APIError::FailedToReadCsv));
    }
}