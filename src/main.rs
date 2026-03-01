use qaid::services::excel_reader::ExcelReader;
use qaid::domain::data_stores::DBReader;

fn main() {
    // let path = "tests/assets/Table01.xlsx".to_owned();
    let template_file_path = "tests/assets/Template.xlsx".to_owned();

    let mut test = ExcelReader::default();
    test.read_db(template_file_path, "Sheet1".to_owned()).unwrap();
    
    dbg!(test);
}