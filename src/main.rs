use qaid::services::excel_reader::ExcelReader;
use qaid::domain::data_stores::DBReader;

fn main() {
    // Get info from template
    let template_file_path = "tests/assets/Template.xlsx".to_owned();

    let mut template = ExcelReader::default();
    template.read_db(template_file_path, "Sheet1".to_owned()).unwrap();
    template.is_schema_ok().expect("Template: Error in schema");
    
    // dbg!(&template);

    // Test dataset against template
    let data_file_path = "tests/assets/Table01.xlsx".to_owned();
    let mut data = ExcelReader::default();
    let read_result = data.read_db(data_file_path, "Sheet1".to_owned());
    let check_schema_result = data.check_against_schema(&template.schema);
    if check_schema_result.is_err() {
        dbg!(&check_schema_result);
    }
}