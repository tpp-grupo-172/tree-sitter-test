#![allow(dead_code)]

use std::path::{Path, PathBuf};
use tree_sitter::{Parser, TreeCursor, Node};
use crate::models::{
    analysis_result::AnalysisResult, class_info::ClassInfo, function_call::FunctionCall, function_info::FunctionInfo, import_info::ImportInfo, local_variable::LocalVariable, parameter_info::ParameterInfo
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
            "import_statement" => {
                let imports = parse_import_statement(source, &node, path, root_path);
                result.imports.extend(imports);
            }
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
                    line: node.start_position().row + 1,
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
            "public_field_definition" => {
                if let Some(arrow) = node.named_children(&mut node.walk())
                    .find(|c| c.kind() == "arrow_function")
                {
                    let name = node.named_children(&mut node.walk())
                        .find(|c| c.kind() == "property_identifier")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("<unnamed>")
                        .to_string();

                    let parameters = parse_parameters(source, &arrow);

                    let return_type = arrow.child_by_field_name("return_type")
                        .and_then(|n| n.named_children(&mut n.walk()).next())
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());

                    let function_calls = arrow.named_children(&mut arrow.walk())
                        .find(|c| c.kind() == "statement_block")
                        .map(|body| find_calls(source, &body, &result.imports))
                        .unwrap_or_else(|| find_calls(source, &arrow, &result.imports));

                    let func = FunctionInfo {
                        name,
                        line: node.start_position().row + 1,
                        end_line: node.end_position().row + 1,
                        parameters,
                        return_type,
                        function_calls: Some(function_calls),
                        local_variables: vec![]
                    };

                    if let Some(class) = current_class.as_deref_mut() {
                        class.methods.push(func);
                    }
                }
            }
            "lexical_declaration" => {
                let mut decl_cursor = node.walk();
                for child in node.named_children(&mut decl_cursor) {
                    if child.kind() == "variable_declarator" {
                        let name = child.named_children(&mut child.walk())
                            .find(|c| c.kind() == "identifier")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .unwrap_or("<unnamed>")
                            .to_string();

                        if let Some(arrow) = child.named_children(&mut child.walk())
                            .find(|c| c.kind() == "arrow_function" || c.kind() == "function_expression")
                        {
                            let parameters = parse_parameters(source, &arrow);

                            let return_type = arrow.child_by_field_name("return_type")
                                .and_then(|n| n.named_children(&mut n.walk()).next())
                                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                .map(|s| s.to_string());

                            let function_calls = arrow.named_children(&mut arrow.walk())
                                .find(|c| c.kind() == "statement_block")
                                .map(|body| find_calls(source, &body, &result.imports))
                                .unwrap_or_else(|| find_calls(source, &arrow, &result.imports));

                            let func = FunctionInfo {
                                name,
                                line: child.start_position().row + 1,
                                end_line: child.end_position().row + 1,
                                parameters,
                                return_type,
                                function_calls: Some(function_calls),
                                local_variables: vec![]
                            };

                            if let Some(class) = current_class.as_deref_mut() {
                                class.methods.push(func);
                            } else {
                                result.functions.push(func);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if kind != "class_declaration" && kind != "method_definition" && kind != "public_field_definition" && cursor.goto_first_child() {
            analyze_node(path, root_path, source, cursor, result, current_class);
            cursor.goto_parent();
        }

        if !cursor.goto_next_sibling() {
            break;
        }
    }
}


fn parse_import_statement(
    source: &str,
    node: &Node,
    current_file: &Path,
    project_roots: &[PathBuf],
) -> Vec<ImportInfo> {
    let mut results = vec![];

    let module_name = node.children(&mut node.walk())
        .find(|c| c.kind() == "string")
        .and_then(|n| n.child_by_field_name("fragment")
            .or_else(|| n.named_children(&mut n.walk()).find(|c| c.kind() == "string_fragment")))
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("")
        .to_string();

    let import_path = resolve_ts_import(current_file, &module_name, project_roots);

    let Some(clause) = node.children(&mut node.walk()).find(|c| c.kind() == "import_clause") else {
        results.push(ImportInfo { name: module_name, line: node.start_position().row + 1, path: import_path, imported_names: vec![] });
        return results;
    };

    let mut cursor = clause.walk();
    for child in clause.named_children(&mut cursor) {
        match child.kind() {
            "named_imports" => {
                let mut imported_names = vec![];
                let mut inner = child.walk();
                for specifier in child.named_children(&mut inner) {
                    if specifier.kind() == "import_specifier" {
                        if let Some(id) = specifier.named_children(&mut specifier.walk())
                            .find(|c| c.kind() == "identifier")
                        {
                            if let Ok(name) = id.utf8_text(source.as_bytes()) {
                                imported_names.push(name.to_string());
                            }
                        }
                    }
                }
                let parsed_name = module_name
                    .trim_start_matches("./")
                    .trim_start_matches("../")
                    .split('/')
                    .last()
                    .unwrap_or(&module_name)
                    .to_string();
                results.push(ImportInfo {
                    name: parsed_name,
                    line: node.start_position().row + 1, 
                    path: import_path.clone(),
                    imported_names,
                });
            }
            "namespace_import" => {
                let alias = child.named_children(&mut child.walk())
                    .find(|c| c.kind() == "identifier")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("")
                    .to_string();
                results.push(ImportInfo {
                    name: alias,
                    line: node.start_position().row + 1, 
                    path: import_path.clone(),
                    imported_names: vec![],
                });
            }
            _ => {}
        }
    }

    results
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

    let local_variables = node.child_by_field_name("body")
        .map(|body| find_local_variables(source, &body))
        .unwrap_or_default();

    FunctionInfo { name, line: node.start_position().row + 1, end_line: node.end_position().row + 1, parameters, return_type, function_calls, local_variables }
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

                            // Verificar si object es un import real o una variable local
                            let is_real_import = imports.iter().any(|i| {
                                i.name == object || i.imported_names.contains(&object)
                            });

                            if is_real_import {
                                calls.push(FunctionCall { 
                                    name: property, 
                                    line: node.start_position().row + 1, 
                                    import_name: Some(object), 
                                    object_name: None 
                                });
                            } else {
                                calls.push(FunctionCall { 
                                    name: property, 
                                    line: node.start_position().row + 1, 
                                    import_name: None, 
                                    object_name: Some(object)
                                });
                            }
                        }
                        "identifier" => {
                            let name = func_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                            let import_name = imports.iter()
                                .find(|i| i.imported_names.contains(&name))
                                .map(|i| i.name.clone());
                            calls.push(FunctionCall { name, line: node.start_position().row + 1, import_name, object_name: None });
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

fn find_local_variables(source: &str, node: &Node) -> Vec<LocalVariable> {
    let mut variables = vec![];
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "lexical_declaration" | "variable_declaration" => {
                let mut decl_cursor = child.walk();
                for declarator in child.named_children(&mut decl_cursor) {
                    if declarator.kind() == "variable_declarator" {
                        let var_name = declarator.child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string());

                        let assigned_from = declarator.child_by_field_name("value")
                            .filter(|n| n.kind() == "call_expression" || n.kind() == "new_expression")
                            .and_then(|n| {
                                if n.kind() == "new_expression" {
                                    // new Product(...) → "Product"
                                    n.child_by_field_name("constructor")
                                        .and_then(|c| c.utf8_text(source.as_bytes()).ok())
                                        .map(|s| s.to_string())
                                } else {
                                    // createProduct(...) o obj.createProduct(...)
                                    n.child_by_field_name("function")
                                        .and_then(|f| {
                                            if f.kind() == "member_expression" {
                                                f.child_by_field_name("property")
                                            } else {
                                                Some(f)
                                            }
                                        })
                                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                        .map(|s| s.to_string())
                                }
                            });

                        if let Some(name) = var_name {
                            variables.push(LocalVariable {
                                name,
                                assigned_from,
                                line: declarator.start_position().row + 1,
                            });
                        }
                    }
                }
                variables.extend(find_local_variables(source, &child));
            }
            "statement_block" | "if_statement" | "for_statement" | 
            "while_statement" | "try_statement" | "block" => {
                variables.extend(find_local_variables(source, &child));
            }
            _ => {}
        }
    }

    variables
}

fn resolve_ts_import(current_file: &Path, module: &str, project_roots: &[PathBuf]) -> Option<PathBuf> {
    if module.starts_with('.') {
        let dir = current_file.parent()?;
        let base = dir.join(module);
        return find_ts_module(&base);
    }

    for root in project_roots {
        let base = root.join(module);
        if let Some(found) = find_ts_module(&base) {
            return Some(found);
        }
    }

    None
}


fn find_ts_module(base: &Path) -> Option<PathBuf> {
    for ext in &["ts", "tsx", "js", "jsx"] {
        let candidate = base.with_extension(ext);
        if candidate.exists() {
            return candidate.canonicalize().ok();
        }
    }

    let index = base.join("index.ts");
    if index.exists() {
        return index.canonicalize().ok();
    }

    None
}

#[allow(dead_code)]
fn print_tree(source: &str, node: Node, indent: usize) {
    let indent_str = " ".repeat(indent);
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");
    eprintln!("{}{}: '{}'", indent_str, node.kind(), text);
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        print_tree(source, child, indent + 2);
    }
}