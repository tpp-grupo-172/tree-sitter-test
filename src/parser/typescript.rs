#![allow(dead_code)]

use std::path::{Path, PathBuf};
use tree_sitter::{Parser, TreeCursor, Node};
use crate::models::{
    analysis_result::AnalysisResult,
    class_info::ClassInfo,
    function_info::FunctionInfo,
    function_call::FunctionCall,
    import_info::ImportInfo,
    parameter_info::ParameterInfo,
};


pub fn parse(source: &str, path: &Path, root_path: &[PathBuf], is_jsx: bool) -> AnalysisResult {
    let mut parser = Parser::new();
    let language = if is_jsx {
        tree_sitter_typescript::language_tsx()
    } else {
        tree_sitter_typescript::language_typescript()
    };
    parser.set_language(language).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root_node = tree.root_node();

    let mut result = AnalysisResult {
        imports: vec![],
        functions: vec![],
        classes: vec![],
    };

    let mut none_class: Option<&mut ClassInfo> = None;
    analyze_node(path, root_path, source, &mut root_node.walk(), &mut result, &mut none_class);

    result
}


fn analyze_node(
    path: &Path,
    root_path: &[PathBuf],
    source: &str,
    cursor: &mut TreeCursor,
    result: &mut AnalysisResult,
    current_class: &mut Option<&mut ClassInfo>,
) {
    loop {
        let node = cursor.node();
        let kind = node.kind();

        match kind {
            "function_declaration" => {
                let func = parse_function(source, &node, &result.imports);
                if let Some(class) = current_class.as_deref_mut() {
                    class.methods.push(func);
                } else {
                    result.functions.push(func);
                }
            }
            "class_declaration" => {
                let name = node
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<unnamed>")
                    .to_string();

                let mut class_info = ClassInfo {
                    name,
                    methods: vec![],
                };

                if let Some(body) = node.child_by_field_name("body") {
                    let mut inner_cursor = body.walk();
                    let mut class_ref = Some(&mut class_info);
                    analyze_node(path, root_path, source, &mut inner_cursor, result, &mut class_ref);
                }

                result.classes.push(class_info);
            }
            "method_definition" => {
                let func = parse_function(source, &node, &result.imports);
                if let Some(class) = current_class.as_deref_mut() {
                    class.methods.push(func);
                }
            }
            _ => {}
        }

        if kind != "class_declaration" && kind != "method_definition" && cursor.goto_first_child() {
            analyze_node(path, root_path, source, cursor, result, current_class);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}


fn parse_function(source: &str, node: &Node, imports: &[ImportInfo]) -> FunctionInfo {
    let name = node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("<unnamed>")
        .to_string();

    let parameters = parse_parameters(source, node);

    let return_type = node.child_by_field_name("return_type")
        .and_then(|n| n.named_children(&mut n.walk()).next())
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(|s| s.to_string());

    let function_calls = node.child_by_field_name("body")
        .map(|body| find_calls(source, &body, imports));

    FunctionInfo { name, parameters, return_type, function_calls }
}


fn parse_parameters(source: &str, node: &Node) -> Vec<ParameterInfo> {
    let mut params = vec![];

    let Some(formal_params) = node.child_by_field_name("parameters") else {
        return params;
    };

    let mut cursor = formal_params.walk();
    for child in formal_params.named_children(&mut cursor) {
        match child.kind() {
            "required_parameter" | "optional_parameter" => {
                let name = child.child_by_field_name("pattern")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<?>")
                    .to_string();

                let param_type = child.child_by_field_name("type")
                    .and_then(|n| n.named_children(&mut n.walk()).next())
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());

                let default_value = child.child_by_field_name("value")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());

                params.push(ParameterInfo { name, param_type, default_value });
            }
            _ => {}
        }
    }

    params
}


fn find_calls(source: &str, node: &Node, imports: &[ImportInfo]) -> Vec<FunctionCall> {
    let mut calls = vec![];
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "call_expression" => {
                if let Some(func_node) = child.child_by_field_name("function") {
                    match func_node.kind() {
                        "member_expression" => {
                            let object = func_node.child_by_field_name("object")
                                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                .unwrap_or("")
                                .to_string();
                            let property = func_node.child_by_field_name("property")
                                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                .unwrap_or("")
                                .to_string();
                            calls.push(FunctionCall { name: property, import_name: Some(object) });
                        }
                        "identifier" => {
                            let name = func_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                            let import_name = imports.iter()
                                .find(|i| i.imported_names.contains(&name))
                                .map(|i| i.name.clone());
                            calls.push(FunctionCall { name, import_name });
                        }
                        _ => {}
                    }
                }
                calls.extend(find_calls(source, &child, imports));
            }
            _ => calls.extend(find_calls(source, &child, imports)),
        }
    }

    calls
}