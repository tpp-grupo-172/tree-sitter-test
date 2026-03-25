# tree-sitter-test

Analizador sintáctico basado en [Tree-sitter](https://tree-sitter.github.io/tree-sitter/), escrito en Rust. Actualmente soporta Python y TypeScript/JavaScript, y está diseñado para ser fácilmente extensible a nuevos lenguajes agregando un módulo de parsing y registrando la extensión de archivo correspondiente.

## Qué analiza

Por cada archivo analizado, el parser extrae:

- **Imports**: nombre del módulo, path absoluto resuelto en el proyecto, y nombres específicos importados (ej: `from math import add, subtract`)
- **Funciones**: nombre, línea de definición, parámetros (con tipo y valor por defecto si los tiene), tipo de retorno, y llamadas a funciones dentro del cuerpo
- **Clases**: nombre, línea de definición, y métodos (con la misma información que las funciones)
- **Llamadas a funciones**: nombre de la función llamada, línea donde ocurre la llamada, y el módulo del que proviene si es resolvible


## Instalación y uso

### Requisitos

- [Rust](https://rust-lang.org/tools/install/)
- Cargo (incluido con Rust)

### Compilar

```bash
cargo build
```

### Ejecutar sobre un archivo

El binario toma un nombre de archivo dentro de la carpeta `input-files/`:

```bash
cargo run -- <nombre_de_archivo>
```

Por ejemplo:

```bash
cargo run -- main.py
cargo run -- main.ts
```

El resultado se escribe como JSON en `parsed-files/<nombre>.json`.

### Usar como biblioteca

`run_analysis` es la función pública principal, exportada desde `lib.rs`. Recibe el path del archivo y una lista de directorios raíz del proyecto para la resolución de imports:

```rust
use tree_sitter_test::run_analysis;
use std::path::{Path, PathBuf};

let result = run_analysis(
    Path::new("ruta/al/archivo.py"),
    &[PathBuf::from("ruta/al/proyecto")]
);
```

Devuelve `Result<String, String>` donde el `Ok` contiene el JSON serializado del análisis.

## Tests

Para correr los tests, ejecutar:

```bash
cargo test
```

## Cómo agregar un lenguaje

**1. Agregar la gramática de Tree-sitter en `Cargo.toml`:**
```
tree-sitter-javascript = "0.20"
```

**2. Crear un nuevo módulo en `src/parser/`:**
```
src/parser/
└── javascript.rs  
```

El módulo debe exponer una función pública parse con la siguiente firma:
```
rustpub fn parse(source: &str, path: &Path, root_path: &[PathBuf]) -> AnalysisResult
```

**3. Registrar el lenguaje en parser/mod.rs:**
```
pub mod javascript;

pub fn parse_file(source: &str, path: &Path, root_path: &[PathBuf]) -> AnalysisResult {
    match path.extension().and_then(|e| e.to_str()) {
        Some("py")  => python::parse(source, path, root_path),
        Some("js")  => javascript::parse(source, path, root_path), // nuevo
        _ => panic!("Unsupported file type"),
    }
}
```