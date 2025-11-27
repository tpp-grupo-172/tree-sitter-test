use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub import_name: Option<String>
}