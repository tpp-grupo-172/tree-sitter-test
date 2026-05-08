use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub import_name: Option<String>,
    pub object_name: Option<String>,
    /// Para llamadas encadenadas como `hola().chau()`: nombre de la función inmediatamente anterior
    /// en la cadena (aquí sería `"hola"`). Permite inferencia de tipos en el backend.
    pub chain_source_fn: Option<String>,
}