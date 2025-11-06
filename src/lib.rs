pub mod models;
pub mod parser;

use std::{fs, path::Path, path::PathBuf};

pub fn run_analysis(file_path: &Path) -> Result<String, String> {
    let source_code = fs::read_to_string(file_path).expect("Could not read input file");
    let result = parser::parse_file(&source_code);
    let json = serde_json::to_string_pretty(&result).unwrap();

    let output_dir = PathBuf::from("parsed-files");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }
    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
    let output_path = output_dir.join(format!("{}.json", file_stem));
    fs::write(&output_path, &json).expect("Failed to write output");
    
    Ok(json)
}