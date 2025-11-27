use serde::Serialize;
use crate::models::function_info::FunctionInfo;
use crate::models::class_info::ClassInfo;
use crate::models::import_info::ImportInfo;

#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub imports: Vec<ImportInfo>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
}
