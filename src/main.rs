mod models;
mod parser;

use std::{fs, env};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Invalid inout parameters");
        std::process::exit(1);
    }

    let file_name = &args[1];
    let input_path = format!("input-files/{}", file_name);
    let source_code = fs::read_to_string(input_path).expect("Could not read input file");

    let result = parser::parse_source(&source_code);

    let json = serde_json::to_string_pretty(&result).unwrap();
    let split_name = file_name.split('.').next().unwrap();
    let output_path = format!("parsed-files/{}.json", split_name);
    fs::write(&output_path, &json).expect("Failed to write output");
}