use crate::{APIError, domain::Cell};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data(Vec<Vec<Cell>>);

impl Data{
    pub fn parse(data: Vec<Vec<Cell>>) -> Result<Self, APIError> {

        if data.is_empty() {
            return Err(APIError::NoData)
        }

        Ok(Self(data))
    }

    pub fn print_data(&self) {
        let data = &self.0;

        for row in data {
            for col in row {
                let t = col.data.to_string();
                print!("  {}  |", t);
            }
            println!()
        }
    }
}

impl AsRef<Vec<Vec<Cell>>> for Data {
    fn as_ref(&self) -> &Vec<Vec<Cell>> {
        &self.0
    }
}