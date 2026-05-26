use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LocalVariable {
    pub name: String,
    /// Nombre de la función cuando el RHS es una llamada: `x = foo()` → `Some("foo")`
    pub assigned_from: Option<String>,
    /// Nombre del identificador cuando el RHS es una variable simple: `self.x = param` → `Some("param")`
    pub assigned_identifier: Option<String>,
    /// Texto del iterable cuando la variable viene de un loop: `for item in self.items` → `Some("self.items")`
    pub iterated_from: Option<String>,
    /// Nombre de la propiedad original cuando la variable viene de destructuring: `const { userAPI } = buildApp()` → `Some("userAPI")`
    pub destructured_property: Option<String>,
    pub line: usize,
}