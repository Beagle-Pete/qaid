use crate::domain::{APIError, data_stores::DBReader};

#[derive(Debug)]
pub struct QA<T: DBReader> {
    template: T,
    data_set: T,
}

impl<T: DBReader> QA<T> {
    pub fn new(template: T, data_set: T) -> Self{
        Self {
            template,
            data_set,
        }
    }

    pub fn check_against_schema(self) -> Result<(), APIError> {
        // TODO: This check should be able to perform a partial check if length of schema and data aren't equal
        // If schema header and data header are in different order this should get corresponding index of both arrays
        // If data has more or less headers it should report out the discrepancy, but compare what is available
        let data = self.data_set.get_data();
        let schema = self.template.get_schema();
        let col_count = data[0].len();
    
        if schema.len() != col_count {
            return Err(APIError::DataSchemaCheckErr("Schema length and data lenth are not equal".to_owned()))
        }
    
        let mut mismatch = "".to_owned();
    
        for row in data {
            for (index, cell) in row.iter().enumerate() {                                
                if cell.data.kind() != schema[index].data_type {
                    mismatch.push_str(&format!("Cell: ({}, {}) - Data: {:?} - Schema: {:?}\n", cell.cell_address.0, cell.cell_address.0, cell.data, schema[index]));
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ExcelReaderBuilder;

    #[test]
    fn test () {
        let mut template = ExcelReaderBuilder::parse("tests/assets/Template.xlsx".to_owned(), "Sheet1".to_owned());
        template.read_db().unwrap();

        let mut data_set = ExcelReaderBuilder::parse("tests/assets/Excel_Normal_01.xlsx".to_owned(), "Sheet1".to_owned());
        data_set.read_db().unwrap();

        let qa = QA::new(template, data_set);
        qa.check_against_schema();
    }
}