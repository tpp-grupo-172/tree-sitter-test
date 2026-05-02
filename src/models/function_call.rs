use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub import_name: Option<String>,
    pub object_name: Option<String>
}