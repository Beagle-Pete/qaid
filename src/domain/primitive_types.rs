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

#[derive(Debug, Clone, PartialEq)]
pub enum PrimTypeData {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(chrono::NaiveDateTime),
    Empty,
    UnexpectedError(String),
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
            PrimTypeData::UnexpectedError(_) => PrimType::UnexpectedError,
        }
    }
}

impl std::fmt::Display for PrimTypeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimTypeData::String(val) => write!(f, "{}", val),
            PrimTypeData::Int(val) => write!(f, "{}", val),
            PrimTypeData::Float(val) => write!(f, "{}", val),
            PrimTypeData::Bool(val) => write!(f, "{}", val),
            PrimTypeData::DateTime(val) => write!(f, "{}", val),
            PrimTypeData::Empty => write!(f, ""),
            PrimTypeData::UnexpectedError(val) => write!(f, "{val}"),
        }        
    }
}