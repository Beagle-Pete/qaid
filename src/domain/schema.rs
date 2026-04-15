use std::collections::HashMap;

use crate::{APIError, domain::{Headers, error::BadSchemaInfo}, utils::get_duplicates};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FieldSchema {
    name: String,
    field_type: String,
}

impl FieldSchema {
    pub fn new(name: String, field_type: String) -> Self {
        Self {
            name,
            field_type,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Schema(HashMap<String, String>);

impl Schema {
    pub fn parse(schema: Vec<FieldSchema>, headers: &Headers) -> Result<Self, APIError> {

        // Test if user defined schema uses correct types
        let mut incorrect_type = vec![];

        schema.iter()
            .for_each(|field| {
                if !matches!(
                    field.field_type.to_lowercase().as_str(),
                    "string" | "int" | "float" | "bool" | "datetime"
                ) {
                    incorrect_type.push(field.clone());
                }
            });

        // Test if user defined schema has duplicate names
        let names: Vec<String> = schema.iter()
            .map(|field| field.name.clone())
            .collect();

        let duplicate_names = get_duplicates(&names);
        
        // Check that all headers are defined in schema
        let field_names: Vec<String> = schema.iter()
            .map(|field| field.name.to_lowercase().clone())
            .collect();
    
        let missing_fields: Vec<String> = headers.as_ref().iter()
            .filter(|header| !field_names.contains(&header.to_lowercase())).cloned()
            .collect();

        // if incorrect type or duplicate name is found return Err
        if !incorrect_type.is_empty() || !duplicate_names.is_empty() || !missing_fields.is_empty(){
            return Err(APIError::BadSchema(BadSchemaInfo {
                incorrect_type,
                duplicate_names,
                missing_fields
            }))
        }

        let scheme_hash_map: HashMap<String, String> = schema.into_iter()
            .map(|field| (field.name, field.field_type))
            .collect();

        Ok(Self(scheme_hash_map))
    }
}

impl AsRef<HashMap<String, String>> for Schema {
    fn as_ref(&self) -> &HashMap<String, String> {
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
    Undefined(String,)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn allowable_types_should_return_ok() {
        let schema = vec![
            FieldSchema::new("name1".to_owned(), "String".to_owned()),
            FieldSchema::new("name2".to_owned(), "STRING".to_owned()),
            FieldSchema::new("name3".to_owned(), "int".to_owned()),
            FieldSchema::new("name4".to_owned(), "flOAT".to_owned()),
            FieldSchema::new("name5".to_owned(), "dateTIME".to_owned()),
            FieldSchema::new("name6".to_owned(), "bool".to_owned()),
        ];

        let headers = vec![
            "name1".to_owned(),
            "name2".to_owned(),
            "name3".to_owned(),
            "name4".to_owned(),
            "name5".to_owned(),
            "name6".to_owned(),
        ];

        let headers = Headers::parse(headers).unwrap();

        let schema = Schema::parse(schema, &headers);

        assert!(schema.is_ok());
    }

    #[test]
    fn duplicate_names_should_return_err() {
        let schema = vec![
            FieldSchema::new("name1".to_owned(), "String".to_owned()),
            FieldSchema::new("name1".to_owned(), "STRING".to_owned()),
            FieldSchema::new("name3".to_owned(), "int".to_owned()),
            FieldSchema::new("name4".to_owned(), "flOAT".to_owned()),
            FieldSchema::new("name5".to_owned(), "dateTIME".to_owned()),
        ];

        let headers = vec![
            "name1".to_owned(),
            "name2".to_owned(),
            "name3".to_owned(),
            "name4".to_owned(),
            "name5".to_owned(),
        ];

        let headers = Headers::parse(headers).unwrap();

        let schema = Schema::parse(schema, &headers);

        assert!(schema.is_err());
    }

    #[test]
    fn invalid_type_should_return_err() {
        let schema = vec![
            FieldSchema::new("name1".to_owned(), "i32".to_owned()),
            FieldSchema::new("name2".to_owned(), "str".to_owned()),
            FieldSchema::new("name3".to_owned(), "int".to_owned()),
            FieldSchema::new("name4".to_owned(), "flOAT".to_owned()),
            FieldSchema::new("name5".to_owned(), "dateTIME".to_owned()),
        ];

        let headers = vec![
            "name1".to_owned(),
            "name2".to_owned(),
            "name3".to_owned(),
            "name4".to_owned(),
            "name5".to_owned(),
        ];

        let headers = Headers::parse(headers).unwrap();

        let schema = Schema::parse(schema, &headers);

        assert!(schema.is_err());
    }
}