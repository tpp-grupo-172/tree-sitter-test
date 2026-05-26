use serde::Serialize;
use crate::models::function_info::FunctionInfo;

/// Anotación de tipo declarada a nivel de clase (campos de dataclass, class variables tipadas).
/// Ej: `items: List[CartItem]` → `FieldAnnotation { name: "items", annotation: "List[CartItem]" }`
#[derive(Debug, Serialize)]
pub struct FieldAnnotation {
    pub name: String,
    pub annotation: String,
}

#[derive(Debug, Serialize)]
pub struct ClassInfo {
    pub name: String,
    pub line: usize,
    pub name_start_col: usize,
    pub name_end_col: usize,
    pub methods: Vec<FunctionInfo>,
    /// Campos con anotación de tipo declarados en el cuerpo de la clase (dataclass fields, etc.)
    pub fields: Vec<FieldAnnotation>,
}