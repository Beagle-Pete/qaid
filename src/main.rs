use std::collections::HashMap;


use qaid::{
    domain::{APIError, data_stores::DBReader, schema::{self, FieldSchema, Schema}},
    // qa::db_comparison::check_against_schema,
    services::{csv_reader::CsvReaderBuilder, excel_reader::ExcelReaderBuilder},
};

fn main() -> Result<(), APIError>  {
    let schema = vec![
        FieldSchema::new("PID".to_owned(), "String".to_owned()),
        FieldSchema::new("Impressions".to_owned(), "Float".to_owned()),
        FieldSchema::new("Placements".to_owned(), "String".to_owned()),
        FieldSchema::new("DateTime".to_owned(), "DateTime".to_owned()),
        FieldSchema::new("Boolean".to_owned(), "Bool".to_owned()),
    ];

    let data_csv_file_path =  "tests/assets/csv_02.csv".to_owned();
    let mut data_csv = CsvReaderBuilder::parse(data_csv_file_path, schema);

    // Get info from template
    let template_file_path = "tests/assets/Template.xlsx".to_owned();
    let sheet_name = "Sheet1".to_owned();

    // let mut template = ExcelReaderBuilder::parse(template_file_path, sheet_name);
    // template.read_db()?;
    
    // dbg!(&template);

    // Test dataset against template
    let data_file_path = "tests/assets/Table01.xlsx".to_owned();
    let sheet_name = "Sheet1".to_owned();
    // let mut data = ExcelReaderBuilder::parse(data_file_path, sheet_name);
    // data.read_db()?;

    // Compare template and dataset
    // let check_schema_result = check_against_schema(data.get_data(), template.get_schema());
    // if check_schema_result.is_err() {
    //     dbg!(&check_schema_result);
    // }

    let tt: HashMap<String, String> = HashMap::new();
    dbg!(&tt);

    Ok(())
}