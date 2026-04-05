#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ReportError {
    FailedToParse,
    MergedCells,
    EmptyCell,
    CustomRule(String),
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportInfo {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub val: String,
    pub context: String,
}

impl ReportInfo {
    pub fn new(start: (usize, usize), end: (usize, usize), val: String, context: String) -> Self {
        Self {
            start,
            end,
            val,
            context,
        }
    }
}