use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImportInfo {
    pub name: String,
    pub path: Option<PathBuf>,
}