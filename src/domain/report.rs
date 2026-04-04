#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ReportError {
    FailedToParse,
    MergedCells,
    CustomRule(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportInfo {
    start: (usize, usize),
    end: (usize, usize),
    val: String,
    context: String,
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