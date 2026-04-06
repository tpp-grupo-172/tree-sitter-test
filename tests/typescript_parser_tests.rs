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

#[test]
fn test_arrow_function_top_level() {
    let source = "const add = (a: number, b: number): number => a + b;";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "add");
    assert_eq!(result.functions[0].return_type.as_deref(), Some("number"));
    assert_eq!(result.functions[0].parameters.len(), 2);
    assert_eq!(result.functions[0].parameters[0].name, "a");
}


#[test]
fn test_async_function() {
    let source = "\
async function fetchData(url: string): Promise<string> {
    return url;
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "fetchData");
    assert_eq!(result.functions[0].return_type.as_deref(), Some("Promise<string>"));
    assert_eq!(result.functions[0].parameters[0].name, "url");
}

#[test]
fn test_const_function_expression() {
    let source = "const add = function(a: number, b: number): number { return a + b; }";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "add");
    assert_eq!(result.functions[0].return_type.as_deref(), Some("number"));
    assert_eq!(result.functions[0].parameters.len(), 2);
    assert_eq!(result.functions[0].parameters[0].name, "a");
    assert_eq!(result.functions[0].parameters[1].name, "b");
}

// ---------------------------- Classes ----------------------------

#[test]
fn test_simple_class() {
    let source = "\
class MyClass {
    myMethod(): void {}
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.classes.len(), 1);
    assert_eq!(result.classes[0].name, "MyClass");
    assert_eq!(result.classes[0].methods.len(), 1);
    assert_eq!(result.classes[0].methods[0].name, "myMethod");
}

#[test]
fn test_class_method_with_typed_params() {
    let source = "\
class Calculator {
    multiply(a: number, b: number): number {
        return a * b;
    }
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let method = &result.classes[0].methods[0];
    assert_eq!(method.name, "multiply");
    assert_eq!(method.return_type.as_deref(), Some("number"));
    assert_eq!(method.parameters.len(), 2);
    assert_eq!(method.parameters[0].name, "a");
    assert_eq!(method.parameters[0].param_type.as_deref(), Some("number"));
    assert_eq!(method.parameters[1].name, "b");
    assert_eq!(method.parameters[1].param_type.as_deref(), Some("number"));
}

#[test]
fn test_class_method_call_resolved() {
    let source = "\
import { subtract } from './math_utils';
class MyClass {
    myMethod(x: number, y: number): number {
        return subtract(x, y);
    }
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.classes[0].methods[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "subtract");
    assert_eq!(calls[0].import_name.as_deref(), Some("math_utils"));
}

#[test]
fn test_class_not_added_to_functions() {
    let source = "\
class MyClass {
    myMethod(): void {}
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert!(result.functions.is_empty());
    assert_eq!(result.classes.len(), 1);
}

#[test]
fn test_class_with_constructor() {
    let source = "\
class Geometry {
    shapeName: string;
    constructor(shapeName: string) {
        this.shapeName = shapeName;
    }
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.classes[0].name, "Geometry");
    let constructor = result.classes[0].methods.iter().find(|m| m.name == "constructor");
    assert!(constructor.is_some());
    assert_eq!(constructor.unwrap().parameters[0].name, "shapeName");
    assert_eq!(constructor.unwrap().parameters[0].param_type.as_deref(), Some("string"));
}

#[test]
fn test_arrow_function_class_field() {
    let source = "\
class Calculator {
    add = (a: number, b: number): number => a + b;
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.classes[0].methods.len(), 1);
    assert_eq!(result.classes[0].methods[0].name, "add");
    assert_eq!(result.classes[0].methods[0].return_type.as_deref(), Some("number"));
}

// ---------------------------- Line Numbers ----------------------------

#[test]
fn test_ts_function_line_number() {
    let source = "function foo(): void {}\n\nfunction bar(): void {}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions[0].line, 1);
    assert_eq!(result.functions[1].line, 3);
}

#[test]
fn test_ts_class_and_method_line_number() {
    let source = "\
class MyClass {
    myMethod(): void {}
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.classes[0].line, 1);
    assert_eq!(result.classes[0].methods[0].line, 2);
}

#[test]
fn test_ts_function_call_line_number() {
    let source = "\
import { add } from './math_utils';
function compute(x: number, y: number): number {
    return add(x, y);
}";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls[0].line, 3);
}