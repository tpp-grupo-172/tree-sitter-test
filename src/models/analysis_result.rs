use serde::Serialize;
use crate::models::function_info::FunctionInfo;
use crate::models::class_info::ClassInfo;

#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub imports: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
}
