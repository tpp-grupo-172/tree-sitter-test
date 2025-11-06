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

    let file_path = Path::new(&args[1]);
    match run_analysis(file_path) {
        Ok(json_output) => println!("{}", json_output),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}


