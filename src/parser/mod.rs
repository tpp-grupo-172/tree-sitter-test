pub mod python;
pub mod typescript;

use std::path::{Path, PathBuf};
use crate::models::analysis_result::AnalysisResult;

#[allow(dead_code)]
pub fn parse_file(source: &str, path: &Path, root_path: &[PathBuf], old_tree: Option<&tree_sitter::Tree>) -> (AnalysisResult, tree_sitter::Tree) {
    match path.extension().and_then(|e| e.to_str()) {
        Some("py") => python::parse(source, path, root_path, old_tree),
        Some("ts") | Some("js") => typescript::parse(source, path, root_path, false, old_tree),
        Some("tsx") | Some("jsx") => typescript::parse(source, path, root_path, true, old_tree),
        _ => panic!("Unsupported file type"),
    }
}