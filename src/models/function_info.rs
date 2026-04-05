use serde::Serialize;
use crate::models::{function_call::FunctionCall, parameter_info::ParameterInfo};

#[derive(Debug, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub end_line: usize,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub function_calls: Option<Vec<FunctionCall>>
}