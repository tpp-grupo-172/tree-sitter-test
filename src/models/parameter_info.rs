use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: Option<String>, 
    pub default_value: Option<String>,
}