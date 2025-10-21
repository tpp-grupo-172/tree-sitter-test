use serde::Serialize;
use crate::models::function_info::FunctionInfo;

#[derive(Debug, Serialize)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<FunctionInfo>,
}