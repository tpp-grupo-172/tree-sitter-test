use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LocalVariable {
    pub name: String,
    pub assigned_from: Option<String>,
    pub line: usize,
}