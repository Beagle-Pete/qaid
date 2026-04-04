use crate::domain::Cell;

pub struct Table(Vec<Vec<Cell>>);

impl Table {
    pub fn new(data: Vec<Vec<Cell>>) -> Self {
        Self(data)
    }

    pub fn print_data(self) {
        let data = self.0;

        for row in data {
            for col in row {
                let t = col.data.to_string();
                print!("  {}  |", t);
            }
            println!()
        }
    }
}