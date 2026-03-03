pub mod python;
pub mod typescript;

use std::path::{Path, PathBuf};
use crate::models::analysis_result::AnalysisResult;

#[allow(dead_code)]
pub fn parse_file(source: &str, path: &Path, root_path: &[PathBuf]) -> AnalysisResult {
    match path.extension().and_then(|e| e.to_str()) {
        Some("py") => python::parse(source, path, root_path),
        Some("ts") | Some("js") => typescript::parse(source, path, root_path, false),
        Some("tsx") | Some("jsx") => typescript::parse(source, path, root_path, true),
        _ => panic!("Unsupported file type"),
    }
}