use std::path::PathBuf;
use tree_sitter_test::parser::parse_file;

fn dummy_path() -> std::path::PathBuf {
    PathBuf::from("test_file.py")
}

fn dummy_roots() -> Vec<PathBuf> {
    vec![]
}

// ---------------------------- Imports ----------------------------

#[test]
fn test_plain_import() {
    let source = "import math";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].name, "math");
    assert!(result.imports[0].imported_names.is_empty());
}

#[test]
fn test_from_import_single() {
    let source = "from math import sqrt";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].name, "math");
    assert_eq!(result.imports[0].imported_names, vec!["sqrt"]);
}

#[test]
fn test_from_import_multiple() {
    let source = "from math import add, subtract";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].name, "math");
    assert_eq!(result.imports[0].imported_names, vec!["add", "subtract"]);
}

#[test]
fn test_multiple_imports() {
    let source = "import os\nimport sys";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.imports.len(), 2);
    assert_eq!(result.imports[0].name, "os");
    assert_eq!(result.imports[1].name, "sys");
}

// ---------------------------- Functions ----------------------------

#[test]
fn test_simple_function() {
    let source = "def greet():\n    pass";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "greet");
    assert!(result.functions[0].parameters.is_empty());
    assert!(result.functions[0].return_type.is_none());
}

#[test]
fn test_function_with_typed_params() {
    let source = "def add(a: int, b: int) -> int:\n    pass";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.functions.len(), 1);
    let func = &result.functions[0];
    assert_eq!(func.name, "add");
    assert_eq!(func.return_type.as_deref(), Some("int"));
    assert_eq!(func.parameters.len(), 2);
    assert_eq!(func.parameters[0].name, "a");
    assert_eq!(func.parameters[0].param_type.as_deref(), Some("int"));
    assert_eq!(func.parameters[1].name, "b");
    assert_eq!(func.parameters[1].param_type.as_deref(), Some("int"));
}

#[test]
fn test_function_with_default_param() {
    let source = "def greet(name, greeting=\"hello\"):\n    pass";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let func = &result.functions[0];
    assert_eq!(func.parameters.len(), 2);
    assert_eq!(func.parameters[1].name, "greeting");
    assert_eq!(func.parameters[1].default_value.as_deref(), Some("\"hello\""));
}

#[test]
fn test_function_with_typed_default_param() {
    let source = "def greet(name: str, greeting: str = \"hello\") -> str:\n    pass";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let func = &result.functions[0];
    assert_eq!(func.parameters.len(), 2);
    let p = &func.parameters[1];
    assert_eq!(p.name, "greeting");
    assert_eq!(p.param_type.as_deref(), Some("str"));
    assert_eq!(p.default_value.as_deref(), Some("\"hello\""));
}

// ---------------------------- Function Calls ----------------------------

#[test]
fn test_bare_call_resolved_via_from_import() {
    let source = "\
from math_utils import add
def compute(x, y):
    return add(x, y)";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "add");
    assert_eq!(calls[0].import_name.as_deref(), Some("math_utils"));
}

#[test]
fn test_dotted_call_resolved_via_plain_import() {
    let source = "\
import math_utils
def compute(x, y):
    return math_utils.subtract(x, y)";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "subtract");
    assert_eq!(calls[0].import_name.as_deref(), Some("math_utils"));
}

#[test]
fn test_unresolved_call_has_no_import() {
    let source = "def compute():\n    print('hello')";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "print");
    assert!(calls[0].import_name.is_none());
}

#[test]
fn test_multiple_calls_in_function() {
    let source = "\
from math_utils import add, subtract
def compute(x, y):
    a = add(x, y)
    b = subtract(x, y)
    return a + b";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.functions[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls.iter().any(|c| c.name == "add" && c.import_name.as_deref() == Some("math_utils")));
    assert!(calls.iter().any(|c| c.name == "subtract" && c.import_name.as_deref() == Some("math_utils")));
}

// ---------------------------- Classes ----------------------------

#[test]
fn test_simple_class() {
    let source = "\
class MyClass:
    def my_method(self):
        pass";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert_eq!(result.classes.len(), 1);
    assert_eq!(result.classes[0].name, "MyClass");
    assert_eq!(result.classes[0].methods.len(), 1);
    assert_eq!(result.classes[0].methods[0].name, "my_method");
}

#[test]
fn test_class_method_with_typed_params() {
    let source = "\
class Calculator:
    def multiply(self, a: int, b: int) -> int:
        pass";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let method = &result.classes[0].methods[0];
    assert_eq!(method.name, "multiply");
    assert_eq!(method.return_type.as_deref(), Some("int"));
    assert_eq!(method.parameters.len(), 3); // self, a, b
    assert_eq!(method.parameters[1].name, "a");
    assert_eq!(method.parameters[1].param_type.as_deref(), Some("int"));
}

#[test]
fn test_class_method_call_resolved() {
    let source = "\
from math_utils import subtract
class MyClass:
    def my_method(self, x: int, y: int):
        return subtract(x, y)";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    let calls = result.classes[0].methods[0].function_calls.as_ref().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "subtract");
    assert_eq!(calls[0].import_name.as_deref(), Some("math_utils"));
}

#[test]
fn test_class_not_added_to_functions() {
    let source = "\
class MyClass:
    def my_method(self):
        pass";
    let result = parse_file(source, &dummy_path(), &dummy_roots());

    assert!(result.functions.is_empty());
    assert_eq!(result.classes.len(), 1);
}