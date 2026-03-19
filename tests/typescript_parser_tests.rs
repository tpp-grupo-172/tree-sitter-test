use std::path::PathBuf;
use tree_sitter_test::parser::parse_file;

fn dummy_path() -> PathBuf {
    PathBuf::from("test_file.ts")
}

fn dummy_roots() -> Vec<PathBuf> {
    vec![]
}

// ---------------------------- Imports ----------------------------

#[test]
fn test_named_import() {
    let source = "import { add } from './math_utils';";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].name, "math_utils");
    assert_eq!(result.imports[0].imported_names, vec!["add"]);
}

#[test]
fn test_named_import_multiple() {
    let source = "import { add, subtract } from './math_utils';";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].name, "math_utils");
    assert_eq!(result.imports[0].imported_names, vec!["add", "subtract"]);
}

#[test]
fn test_namespace_import() {
    let source = "import * as math from './math_utils';";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].name, "math");
    assert!(result.imports[0].imported_names.is_empty());
}

#[test]
fn test_multiple_imports() {
    let source = "\
import * as math from './math_utils';
import { add, subtract } from './math_utils';";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 2);
    assert_eq!(result.imports[0].name, "math");
    assert_eq!(result.imports[1].name, "math_utils");
    assert_eq!(result.imports[1].imported_names, vec!["add", "subtract"]);
}

// ---------------------------- Functions ----------------------------

#[test]
fn test_simple_function() {
    let source = "function hola(): void {}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "hola");
    assert!(result.functions[0].parameters.is_empty());
    assert_eq!(result.functions[0].return_type.as_deref(), Some("void"));
}

#[test]
fn test_function_with_typed_params() {
    let source = "function add(a: number, b: number): number { return a + b; }";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    assert_eq!(func.name, "add");
    assert_eq!(func.return_type.as_deref(), Some("number"));
    assert_eq!(func.parameters.len(), 2);
    assert_eq!(func.parameters[0].name, "a");
    assert_eq!(func.parameters[0].param_type.as_deref(), Some("number"));
    assert_eq!(func.parameters[1].name, "b");
    assert_eq!(func.parameters[1].param_type.as_deref(), Some("number"));
}

#[test]
fn test_function_with_default_param() {
    let source = "function greet(name: string, greeting: string = \"hello\"): string { return greeting; }";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let func = &result.functions[0];
    assert_eq!(func.parameters.len(), 2);
    let p = &func.parameters[1];
    assert_eq!(p.name, "greeting");
    assert_eq!(p.param_type.as_deref(), Some("string"));
    assert_eq!(p.default_value.as_deref(), Some("\"hello\""));
}

#[test]
fn test_function_no_return_type() {
    let source = "function compute(x: number, y: number) { return x + y; }";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    assert!(result.functions[0].return_type.is_none());
}

// ---------------------------- Function Calls ----------------------------

#[test]
fn test_bare_call_resolved_via_named_import() {
    let source = "\
import { add } from './math_utils';
function compute(x: number, y: number): number {
    return add(x, y);
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "add");
    assert_eq!(calls[0].import_name.as_deref(), Some("math_utils"));
}

#[test]
fn test_dotted_call_resolved_via_namespace_import() {
    let source = "\
import * as math from './math_utils';
function compute(x: number, y: number): number {
    return math.subtract(x, y);
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "subtract");
    assert_eq!(calls[0].import_name.as_deref(), Some("math"));
}

#[test]
fn test_unresolved_call_has_no_import() {
    let source = "function compute(): void { console.log(\"hello\"); }";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "log");
    assert_eq!(calls[0].import_name.as_deref(), Some("console"));
}

#[test]
fn test_multiple_calls_in_function() {
    let source = "\
import { add, subtract } from './math_utils';
function compute(x: number, y: number): number {
    const a = add(x, y);
    const b = subtract(x, y);
    return a + b;
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls.iter().any(|c| c.name == "add" && c.import_name.as_deref() == Some("math_utils")));
    assert!(calls.iter().any(|c| c.name == "subtract" && c.import_name.as_deref() == Some("math_utils")));
}

