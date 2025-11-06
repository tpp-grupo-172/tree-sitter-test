use serde::Serialize;
use crate::models::parameter_info::ParameterInfo;

#[derive(Debug, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
}