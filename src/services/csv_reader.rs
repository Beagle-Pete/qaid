use std::fs::File;

use csv::{Reader, StringRecord};

use crate::{domain::data_stores::DBReader};
use crate::domain::{APIError, Cell, PrimType, PrimTypeData, SchemaInfo};

#[derive(Debug)]
pub struct CsvReaderBuilder {
    pub db: String,
}

#[derive(Debug, Default)]
pub struct CsvReader {
    db: String,
    headers: Vec<String>,
    data: Vec<Vec<Cell>>,
    data_size: (usize, usize),
    schema: Vec<SchemaInfo>,
    has_merged_cells: bool,
}

impl CsvReaderBuilder {
    pub fn parse(db: String) -> CsvReader {
        CsvReader {
            db,
            ..Default::default()
        }
    }
}

impl DBReader for CsvReader {
    fn read_db(&mut self) -> Result<(), APIError> {
        let file = File::open(&self.db)
            .map_err(|_| APIError::FailedToOpen)?;
        let mut rdr = Reader::from_reader(file);

        // Get headers
        let headers = rdr.headers()
            .map_err(|_| APIError::FailedToRead)?;

        let headers_vec = record_to_vec(headers);

        // Get data
        let mut data = vec![];

        // Iterate through rows
        for (ii, record) in rdr.records().enumerate() {
            if let Ok(row) = record {
                let row_data = record_to_vec(&row);

                let mut row_data2 = vec![];
                for (jj, col) in row_data.iter().enumerate() {
                    let cell = Cell::new(
                        Ok(PrimTypeData::String(col.trim().to_owned())), 
                        (ii, jj)
                    );
                    row_data2.push(cell);
                }
                data.push(row_data2);
            }
        }

        // Get schema. Schema for .csv file will be all Strings
        let schema = headers_vec.iter()
            .map(|header| SchemaInfo::new(header.to_owned(), PrimType::String))
            .collect();

        let row_count = data.len();
        let col_count = data[0].len();

        self.headers = headers_vec;
        self.data = data;
        self.data_size = (row_count, col_count);
        self.schema = schema;
        self.has_merged_cells = false;
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

fn record_to_vec(record: &StringRecord) -> Vec<String> {
    record.iter()
        .map(|el| el.to_owned())
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

        assert_eq!(csv_file.data[0][1].data, Ok(PrimTypeData::String("String1".to_owned())));
        assert_eq!(csv_file.data[0][1].cell_address, (0, 1));

        assert!(!csv_file.has_merged_cells);
    }

    #[test]
    fn file_does_not_exist() {
        let mut csv_file = CsvReaderBuilder::parse("tests/assets/Non_existant_file.csv".to_owned());
        
        assert_eq!(csv_file.read_db(), Err(APIError::FailedToOpen));
    }
}