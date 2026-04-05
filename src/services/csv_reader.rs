use std::fs::File;
use std::collections::HashMap;

use csv::{Reader, StringRecord};

use crate::domain::{
    APIError,
    Cell,
    data_stores::DBReader,
    PrimType,
    PrimTypeData,
    ReportError,
    ReportInfo,
    SchemaInfo,
};

#[derive(Debug)]
pub struct CsvReaderBuilder {
    pub file_path: String,
}

#[derive(Debug, Default)]
pub struct CsvReader {
    file: String,
    headers: Vec<String>,
    data: Vec<Vec<Cell>>,
    data_size: (usize, usize),
    schema: Vec<SchemaInfo>,
    report: HashMap<ReportError, Vec<ReportInfo>>,
}

impl CsvReaderBuilder {
    pub fn parse(file: String) -> CsvReader {
        CsvReader {
            file,
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
        let headers = rdr.headers()
            .map_err(|_| APIError::FailedToRead)?;

        let headers = record_to_vec(headers);

        // Get data
        let mut data = vec![];

        // Iterate through rows
        for (ii, record) in rdr.records().enumerate() {
            let row = record
                .map_err(|_| APIError::FailedToReadCsv)?;

            let row_data = record_to_vec(&row);

            let mut row_data2 = vec![];
            for (jj, col) in row_data.iter().enumerate() {
                let cell = Cell::new(
                    PrimTypeData::String(col.to_owned()), 
                    (ii, jj)
                );
                row_data2.push(cell);
            }
            data.push(row_data2);
        }

        if data.is_empty() {
            return Err(APIError::NoData);
        }

        // Get schema. Schema for .csv file will be all Strings
        let schema: Vec<SchemaInfo> = headers.iter()
            .map(|header| SchemaInfo::new(header.to_owned(), PrimType::String))
            .collect();

        let row_count = data.len();
        let col_count = headers.len();


        self.headers = headers;
        self.data = data;
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

fn record_to_vec(record: &StringRecord) -> Vec<String> {
    record.iter()
        .map(|el| el.trim().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_csv_read() {
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/csv_01.csv".to_owned());
        csv_file.read_db().unwrap();

        assert_eq!(csv_file.headers, ["col1", "col2", "col3"]);

        assert_eq!(csv_file.data_size, (4, 3));

        assert_eq!(csv_file.data[0][1].data, PrimTypeData::String("String1".to_owned()));
        assert_eq!(csv_file.data[0][1].cell_address, (0, 1));
    }

    #[test]
    fn empty_csv_should_fail() {
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/csv_empty.csv".to_owned());
        
        assert_eq!(csv_file.read_db(), Err(APIError::NoData));
    }

    #[test]
    fn file_does_not_exist() {
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/Non_existant_file.csv".to_owned());
        
        assert_eq!(csv_file.read_db(), Err(APIError::FailedToOpen));
    }

    #[test]
    fn header_data_unequal_length_should_fail() {
        let mut csv_file_01 = CsvReaderBuilder::parse("tests/assets/csv_unequal_length_01.csv".to_owned());
        let mut csv_file_02 = CsvReaderBuilder::parse("tests/assets/csv_unequal_length_02.csv".to_owned());
        let mut csv_file_03 = CsvReaderBuilder::parse("tests/assets/csv_unequal_length_03.csv".to_owned());

        assert_eq!(csv_file_01.read_db(), Err(APIError::FailedToReadCsv));
        assert_eq!(csv_file_02.read_db(), Err(APIError::FailedToReadCsv));
        assert_eq!(csv_file_03.read_db(), Err(APIError::FailedToReadCsv));
    }
}