use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub line: usize,
    pub import_name: Option<String>,
    pub object_name: Option<String>
}