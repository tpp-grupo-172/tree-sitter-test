#![allow(dead_code)]

use std::path::{Path,PathBuf};


use std::fs;

use tree_sitter::{Parser, TreeCursor, Node};
use crate::models::function_call::FunctionCall;
use crate::models::import_info::ImportInfo;
use crate::models::{analysis_result::AnalysisResult, class_info::ClassInfo, function_info::FunctionInfo, parameter_info::ParameterInfo};

pub fn parse_file(source: &str, path: &Path, root_path: &[PathBuf]) -> AnalysisResult {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_python::language()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root_node = tree.root_node();

    // print_tree(source, root_node, 0);

    let mut result = AnalysisResult {
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };
    let mut none_class: Option<&mut ClassInfo> = None;
    analyze_node(path, root_path, source, &mut root_node.walk(), &mut result, &mut none_class);

    result
}


fn analyze_node(path: &Path, root_path: &[PathBuf], source: &str, cursor: &mut TreeCursor, result: &mut AnalysisResult, current_class: &mut Option<&mut ClassInfo>) {
    loop {
        let node = cursor.node();
        let kind = node.kind();

        match kind {
            "import_statement" => {
                let text = node.utf8_text(source.as_bytes()).unwrap_or("").trim().to_string();
                let parts: Vec<&str> = text.split(' ').collect();
                let field_name: String = parts.get(1).unwrap().to_string();

                let import_path = resolve_python_import(path, &field_name, root_path);

                result.imports.push(ImportInfo { name: field_name, path: import_path });
            }
            "function_definition" => {
                let name = node
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<unnamed>")
                    .to_string();
            
                let parameters = get_function_parameters(source, &node);

                let mut return_type: Option<String> = None;

                if let Some(node_return_type) = node.child_by_field_name("type") {
                    return_type = Some(node_return_type.utf8_text(source.as_bytes()).unwrap().to_string());
                    println!("{:?}", return_type);
                }

                let mut function_calls: Option<Vec<FunctionCall>> = None;
                if let Some(node_return_body) = node.child_by_field_name("body") {
                    let calls = find_calls(source, &node_return_body);
                    function_calls = Some(calls);
                }     
            
                let func_info = FunctionInfo {
                    name,
                    parameters,
                    return_type: return_type,
                    function_calls
                };
            
                if let Some(class) = current_class.as_deref_mut() {
                    class.methods.push(func_info);
                } else {
                    result.functions.push(func_info);
                }
            }
            "class_definition" => {
                let name = node
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<unnamed>")
                    .to_string();
            
                let mut class_info = ClassInfo {
                    name,
                    methods: Vec::new(),
                };
            
                if let Some(body) = node.child_by_field_name("body") {
                    let mut inner_cursor = body.walk();
                    let mut class_ref = Some(&mut class_info);
                    analyze_node(path, root_path, source, &mut inner_cursor, result, &mut class_ref);
                }
            
                result.classes.push(class_info);
            }
            _ => {}
        }

        if kind != "class_definition" && cursor.goto_first_child() {
            analyze_node(path, root_path, source, cursor, result, current_class);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}


fn get_function_parameters<'a>(source: &'a str, node: &tree_sitter::Node<'a>) -> Vec<ParameterInfo> {
    let mut params: Vec<ParameterInfo> = Vec::new();
    if let Some(param_node) = node.child_by_field_name("parameters") {
        for child in param_node.named_children(&mut param_node.walk()) {
            match child.kind() {
                "identifier" => {
                    let name = child.utf8_text(source.as_bytes()).unwrap().to_string();
                    params.push(ParameterInfo {name, default_value: None, param_type: None});
                }
                "default_parameter" => {
                    if let Some(node_name) = child.child_by_field_name("name") {
                        if let Ok(name) = node_name.utf8_text(source.as_bytes()) {
                            let node_default_value = child.child_by_field_name("value").unwrap();
                            let default_value = node_default_value.utf8_text(source.as_bytes()).unwrap().to_string();
                            params.push(ParameterInfo {name: name.to_string(), default_value: Some(default_value), param_type: None});
                        }
                    }
                }
                "typed_parameter" => {
                    let mut name = "<?>".to_string();
                    let mut param_type: Option<String> = None;

                    let mut sub_cursor = child.walk();
                    for sub in child.named_children(&mut sub_cursor) {
                        match sub.kind() {
                            "identifier" => {
                                name = sub.utf8_text(source.as_bytes()).unwrap_or("<?>").to_string();
                            }
                            "type" => {
                                let parsed_param_type = sub.utf8_text(source.as_bytes()).unwrap_or("").trim().to_string();
                                if !parsed_param_type.is_empty() {
                                    param_type = Some(parsed_param_type);
                                }
                            }
                            _ => {}
                        }
                    }
                    params.push(ParameterInfo {name, default_value: None, param_type});
                }
                "typed_default_parameter" => {
                    if let Some(node_name) = child.child_by_field_name("name") {
                        if let Ok(name) = node_name.utf8_text(source.as_bytes()) {
                            let node_default_value = child.child_by_field_name("value").unwrap();
                            let default_value = node_default_value.utf8_text(source.as_bytes()).unwrap().to_string();
                            let node_param_type = child.child_by_field_name("type").unwrap();
                            let param_type = node_param_type.utf8_text(source.as_bytes()).unwrap().to_string();
                            params.push(ParameterInfo {name: name.to_string(), default_value: Some(default_value), param_type: Some(param_type)});
                        }
                    }
                }
                _ => {}
            }
        }
    }
    params
}

fn find_calls<'a>(source: &'a str, node: &tree_sitter::Node<'a>) -> Vec<FunctionCall> {
    let mut cursor = node.walk();
    let mut calls: Vec<FunctionCall> = vec![];

    for child in node.children(&mut cursor) {
        match child.kind() {
            // Nodo de llamada de función en Python
            "call" => {
                if let Some(func_node) = child.child_by_field_name("function") {
                    let name = func_node.utf8_text(source.as_bytes()).unwrap().to_string();
                    let mut function_name = String::new();
                    let mut import_name = None;
                    if name.clone().contains('.') {
                      let import_fuction_name: Vec<&str> = name.split('.').collect();
                      function_name = import_fuction_name.get(1).unwrap().to_string();
                      import_name = Some(import_fuction_name.get(0).unwrap().to_string());
                    } else {
                      function_name = name.clone();
                    }

                    calls.push(FunctionCall { name: function_name, import_name });
                }
            }
            // Recorrer recursivamente el resto del cuerpo
            _ => calls.extend(find_calls(source, &child)),
        }
    }

    calls
}
 
#[allow(dead_code)]
fn print_tree(source: &str, node: Node, indent: usize) {
    let indent_str = " ".repeat(indent);
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");
    println!("{}{}: '{}'", indent_str, node.kind(), text);
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        print_tree(source, child, indent + 2);
    }
}

pub fn resolve_python_import(
    current_file: &Path,
    import_path: &str,
    project_roots: &[PathBuf],
) -> Option<PathBuf> {
    if import_path.starts_with('.') {
        return resolve_relative_python_import(current_file, import_path);
    }

    resolve_absolute_python_import(import_path, project_roots)
}


fn resolve_relative_python_import(current_file: &Path, import_path: &str) -> Option<PathBuf> {
    let mut current_dir = current_file.parent()?.to_path_buf();

    let mut chars = import_path.chars();
    let mut levels = 0;

    while chars.clone().next() == Some('.') {
        chars.next();
        levels += 1;
    }

    for _ in 0..(levels - 1) {
        current_dir = current_dir.parent()?.to_path_buf();
    }

    let remaining = import_path.trim_start_matches('.');

    if remaining.is_empty() {
        return find_python_module(&current_dir);
    }

    let full = if remaining.starts_with('.') {
        current_dir.join(remaining.trim_start_matches('.').replace('.', "/"))
    } else {
        current_dir.join(remaining.replace('.', "/"))
    };

    find_python_module(&full)
}



fn resolve_absolute_python_import(
    import_path: &str,
    project_roots: &[PathBuf],
) -> Option<PathBuf> {
    let rel_path = import_path.replace('.', "/");

    for root in project_roots {
        let full = root.join(&rel_path);
        if let Some(found) = find_python_module(&full) {
            return Some(found);
        }
    }

    None
}

fn find_python_module(base: &Path) -> Option<PathBuf> {
    let file = base.with_extension("py");

    if file.exists() {
        return file.canonicalize().ok();
    }

    let init = base.join("__init__.py");
    if init.exists() {
        return init.canonicalize().ok();
    }

    if base.is_dir() {
        return base.canonicalize().ok();
    }

    None
}
