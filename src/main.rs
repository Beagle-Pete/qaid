use qaid::{
    domain::{APIError, data_stores::DBReader},
    qa::db_comparison::check_against_schema,
    services::excel_reader::ExcelReaderBuilder,
};

fn main() -> Result<(), APIError>  {
    // Get info from template
    let template_file_path = "tests/assets/Template.xlsx".to_owned();
    let sheet_name = "Sheet1".to_owned();

    let mut template = ExcelReaderBuilder::parse(template_file_path, sheet_name);
    template.read_db()?;
    
    dbg!(&template);

    // Test dataset against template
    let data_file_path = "tests/assets/Table01.xlsx".to_owned();
    let sheet_name = "Sheet1".to_owned();
    let mut data = ExcelReaderBuilder::parse(data_file_path, sheet_name);
    data.read_db()?;

    // Compare template and dataset
    let check_schema_result = check_against_schema(data.get_data(), template.get_schema());
    if check_schema_result.is_err() {
        dbg!(&check_schema_result);
    }

    Ok(())
}