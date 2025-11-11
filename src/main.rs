mod models;
mod parser;
use std::{env, path::Path};
use tree_sitter_test::run_analysis;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Invalid input parameters");
        std::process::exit(1);
    }

    let file_name = &args[1];
    let file_path = format!("input-files/{}", file_name);

    match run_analysis(Path::new(&file_path)) {
        Ok(_) => println!("Analysis complete"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

