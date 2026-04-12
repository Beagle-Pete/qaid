use std::collections::HashMap;

use chrono::NaiveDateTime;

use crate::{APIError, domain::{Cell, PrimTypeData, ReportError, ReportInfo}};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Data(Vec<Vec<Cell>>);

impl Data{
    pub fn parse(
        data: Vec<Vec<String>>, 
        headers: &[String], 
        schema: &HashMap<String, String>
    ) -> Result<(Self, Vec<(ReportError, ReportInfo)>), APIError> {

        if data.is_empty() {
            return Err(APIError::NoData)
        }

        let mut value_vec = vec![];
        let mut report = vec![];
        // Iterate through rows
        for (ii, row) in data.iter().enumerate() {
            
            let mut value_row = vec![];
            // Iterate through columns
            for (jj, val) in row.iter().enumerate() {
                let field_type = {
                    match schema.get(&headers[jj]) {
                        Some(val) => val,
                        None => continue
                    }
                };
                let val = val.trim().to_owned();

                let dyn_val = {
                    if val.is_empty() {
                        let context = format!("Empty cell at ({},{})", ii, jj);
                        report.push((
                            ReportError::EmptyCell, 
                            ReportInfo::new((ii, jj), (ii, jj), "".to_owned(), context)
                        ));
                        PrimTypeData::Empty
                    } else {
                        match field_type.to_lowercase().as_str() {
                            "string" => PrimTypeData::String(val.to_owned()),
                            "int" => PrimTypeData::Int({
                                // dbg!(&headers, &field_type, &schema);
                                val.parse::<i64>()
                                    .unwrap_or_else(|_| panic!("Failed at ({}, {}). Header: {}, Header_type: {}, val: {}", ii, jj, &headers[jj], field_type, val))
                            }),
                            "float" => PrimTypeData::Float({
                                val.parse::<f64>()
                                    .unwrap_or_else(|_| panic!("Failed at ({}, {}). Header: {}, Header_type: {}, val: {}", ii, jj, &headers[jj], field_type, val))
                            }),
                            "bool" => {
                                // Parse to boolean
                                let val_bool = match val.to_lowercase().as_str() {
                                    "true" => Some(true),
                                    "false" => Some(false),
                                    _ => {
                                        if let Ok(num) = val.parse::<f64>() {
                                            match num {
                                                1.0 => Some(true),
                                                0.0 => Some(false),
                                                _ => None
                                            }
                                        } else if let Ok(num) = val.parse::<i64>() {
                                            match num {
                                                1 => Some(true),
                                                0 => Some(false),
                                                _ => None
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                };

                                match val_bool {
                                    Some(val) => PrimTypeData::Bool(val),
                                    None => PrimTypeData::UnexpectedError("Failed to parse boolean".to_owned()),
                                }
                            },
                            "datetime" => {
                                // TODO: Check if val can be parsed into a float. If so then convert serial number to date time
                                let datetime = NaiveDateTime::parse_from_str(&val, "%Y-%m-%dT%H:%M:%S");
                                dbg!(&val, &datetime);

                                match datetime {
                                    Ok(dt) => PrimTypeData::DateTime(dt),
                                    Err(_) => {
                                        let context = format!("Failed to parse DateTime at ({},{})", ii, jj);
                                        report.push((
                                            ReportError::FailedToParse, 
                                            ReportInfo::new((ii, jj), (ii, jj), val, context)
                                        ));
                                        PrimTypeData::UnexpectedError("Failed to parse datetime".to_owned())
                                    },
                                }
                                
                            },
                            undefined => PrimTypeData::UnexpectedError(undefined.to_owned()),
                        }
                    }
                };

                let cell = Cell::new(
                    dyn_val, 
                    (ii, jj)
                );

                value_row.push(cell)
            }
            value_vec.push(value_row);
        }
        
        Ok((Self(value_vec), report))
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

#[derive(Debug, Clone, PartialEq)]
pub enum DynValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(chrono::NaiveDateTime),
    Undefined(String),
}