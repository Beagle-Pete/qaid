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
}

impl AsRef<Vec<Vec<Cell>>> for Data {
    fn as_ref(&self) -> &Vec<Vec<Cell>> {
        &self.0
    }
}