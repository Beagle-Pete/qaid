#[derive(Debug, Clone, PartialEq)]
pub struct SchemaInfo {
    pub header_name: String,
    pub data_type: PrimType,
}

impl SchemaInfo {
    pub fn new(header_name: String, data_type: PrimType) -> Self {
        Self {
            header_name,
            data_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimType {
    String,
    Int,
    Float,
    Bool,
    DateTime,
    Empty,
    UnexpectedError,
}

#[derive(Debug, Clone)]
pub enum PrimTypeData {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(chrono::NaiveDateTime),
    Empty,
    UnexpectedError,
}

impl PrimTypeData {
    pub fn kind(&self) -> PrimType {
        match self {
            PrimTypeData::String(_) => PrimType::String,
            PrimTypeData::Int(_) => PrimType::Int,
            PrimTypeData::Float(_) => PrimType::Float,
            PrimTypeData::Bool(_) => PrimType::Bool,
            PrimTypeData::DateTime(_) => PrimType::DateTime,
            PrimTypeData::Empty => PrimType::Empty,
            PrimTypeData::UnexpectedError => PrimType::UnexpectedError,
        }
    }
}