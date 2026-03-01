#[derive(Debug, Clone)]
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