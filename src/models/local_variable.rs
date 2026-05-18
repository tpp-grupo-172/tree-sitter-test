use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LocalVariable {
    pub name: String,
    /// Nombre de la función cuando el RHS es una llamada: `x = foo()` → `Some("foo")`
    pub assigned_from: Option<String>,
    /// Nombre del identificador cuando el RHS es una variable simple: `self.x = param` → `Some("param")`
    pub assigned_identifier: Option<String>,
    pub line: usize,
}