use crate::domain::{APIError, Cell, SchemaInfo};

pub fn check_against_schema(data: &Vec<Vec<Cell>>, schema: &[SchemaInfo]) -> Result<(), APIError> {
    // TODO: This check should be able to perform a partial check if length of schema and data aren't equal
    // If schema header and data header are in different order this should get corresponding index of both arrays
    // If data has more or less headers it should report out the discrepancy, but compare what is available
    let col_count = data[0].len();

    if schema.len() != col_count {
        return Err(APIError::DataSchemaCheckErr("Schema length and data lenth are not equal".to_owned()))
    }

    let mut mismatch = "".to_owned();

    for row in data {
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