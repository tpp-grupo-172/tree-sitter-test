use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImportInfo {
    pub name: String,
    pub line: usize,
    pub path: Option<PathBuf>,
    pub imported_names: Vec<String>,
}