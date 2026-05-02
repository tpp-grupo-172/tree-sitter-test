use serde::Serialize;
use crate::models::function_info::FunctionInfo;

#[derive(Debug, Serialize)]
pub struct ClassInfo {
    pub name: String,
    pub line: usize,
    pub name_start_col: usize,
    pub name_end_col: usize,
    pub methods: Vec<FunctionInfo>,
}