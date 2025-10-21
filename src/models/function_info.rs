use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<String>,
}