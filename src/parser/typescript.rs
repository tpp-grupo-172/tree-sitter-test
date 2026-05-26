#![allow(dead_code)]

use std::path::{Path, PathBuf};
use tree_sitter::{Parser, TreeCursor, Node};
use crate::models::{
    analysis_result::AnalysisResult, class_info::{ClassInfo, FieldAnnotation}, function_call::FunctionCall, function_info::FunctionInfo, import_info::ImportInfo, local_variable::LocalVariable, parameter_info::ParameterInfo
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
                let name_node = node.child_by_field_name("name");
                let name = name_node
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("<unnamed>")
                    .to_string();
                let name_start_col = name_node.map(|n| n.start_position().column).unwrap_or(0);
                let name_end_col = name_node.map(|n| n.end_position().column).unwrap_or(0);

                let mut class_info = ClassInfo {
                    name,
                    line: node.start_position().row + 1,
                    name_start_col,
                    name_end_col,
                    methods: vec![],
                    fields: vec![],
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

                // Constructor shorthand: constructor(private searchService: SearchService)
                // Los parámetros con modificador de acceso crean implícitamente un campo de clase.
                let is_constructor = node.child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s == "constructor")
                    .unwrap_or(false);
                if is_constructor {
                    if let Some(params) = node.child_by_field_name("parameters") {
                        for param in params.named_children(&mut params.walk()) {
                            if matches!(param.kind(), "required_parameter" | "optional_parameter") {
                                let has_access_modifier = param.named_children(&mut param.walk())
                                    .any(|c| c.kind() == "accessibility_modifier");
                                if has_access_modifier {
                                    let pname = param.child_by_field_name("pattern")
                                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                        .map(|s| s.to_string());
                                    let ptype = param.child_by_field_name("type")
                                        .and_then(|n| n.named_children(&mut n.walk()).next())
                                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                        .map(|s| s.to_string());
                                    if let (Some(name), Some(annotation)) = (pname, ptype) {
                                        if let Some(class) = current_class.as_deref_mut() {
                                            class.fields.push(FieldAnnotation { name, annotation });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(class) = current_class.as_deref_mut() {
                    class.methods.push(func);
                }
            }
            "public_field_definition" => {
                // Captura campos con tipo: `private searchService: SearchService;`
                let fname = node.named_children(&mut node.walk())
                    .find(|c| c.kind() == "property_identifier")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());
                let ftype = node.named_children(&mut node.walk())
                    .find(|c| c.kind() == "type_annotation")
                    .and_then(|n| n.named_children(&mut n.walk()).next())
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());
                if let (Some(name), Some(annotation)) = (fname, ftype) {
                    if let Some(class) = current_class.as_deref_mut() {
                        class.fields.push(FieldAnnotation { name, annotation });
                    }
                }

                if let Some(arrow) = node.named_children(&mut node.walk())
                    .find(|c| c.kind() == "arrow_function")
                {
                    let name_node = node.named_children(&mut node.walk())
                        .find(|c| c.kind() == "property_identifier");
                    let name = name_node
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("<unnamed>")
                        .to_string();
                    let name_start_col = name_node.map(|n| n.start_position().column).unwrap_or(0);
                    let name_end_col = name_node.map(|n| n.end_position().column).unwrap_or(0);

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
                        name_start_col,
                        name_end_col,
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
            "lexical_declaration" | "variable_declaration" => {
                let mut decl_cursor = node.walk();
                for child in node.named_children(&mut decl_cursor) {
                    if child.kind() == "variable_declarator" {
                        if let Some(import) = parse_require_import(source, &child, path, root_path) {
                            result.imports.push(import);
                            continue;
                        }

                        let name_node = child.named_children(&mut child.walk())
                            .find(|c| c.kind() == "identifier");
                        let name = name_node
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .unwrap_or("<unnamed>")
                            .to_string();
                        let name_start_col = name_node.map(|n| n.start_position().column).unwrap_or(0);
                        let name_end_col = name_node.map(|n| n.end_position().column).unwrap_or(0);

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
                                name_start_col,
                                name_end_col,
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


fn parse_require_import(
    source: &str,
    node: &Node,
    current_file: &Path,
    project_roots: &[PathBuf],
) -> Option<ImportInfo> {
    let value = node.child_by_field_name("value")?;
    if value.kind() != "call_expression" {
        return None;
    }

    let func = value.child_by_field_name("function")?;
    if func.utf8_text(source.as_bytes()).ok()? != "require" {
        return None;
    }

    let args = value.child_by_field_name("arguments")?;
    let module_name = args.named_children(&mut args.walk())
        .find(|c| c.kind() == "string")
        .and_then(|s| s.child_by_field_name("fragment")
            .or_else(|| s.named_children(&mut s.walk()).find(|c| c.kind() == "string_fragment")))
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("")
        .to_string();

    let import_path = resolve_ts_import(current_file, &module_name, project_roots);

    let name_node = node.child_by_field_name("name")?;

    match name_node.kind() {
        "object_pattern" => {
            let imported_names: Vec<String> = name_node.named_children(&mut name_node.walk())
                .filter_map(|c| match c.kind() {
                    "shorthand_property_identifier_pattern" | "identifier" => {
                        c.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
                    }
                    "pair_pattern" => {
                        c.child_by_field_name("value")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string())
                    }
                    _ => None,
                })
                .collect();

            let parsed_name = module_name
                .trim_start_matches("./")
                .trim_start_matches("../")
                .split('/')
                .last()
                .unwrap_or(&module_name)
                .to_string();

            Some(ImportInfo {
                name: parsed_name,
                line: node.start_position().row + 1,
                path: import_path,
                imported_names,
            })
        }
        "identifier" => {
            let name = name_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
            Some(ImportInfo {
                name,
                line: node.start_position().row + 1,
                path: import_path,
                imported_names: vec![],
            })
        }
        _ => None,
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
    let name_node = node.child_by_field_name("name");
    let name = name_node
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("<unnamed>")
        .to_string();
    let name_start_col = name_node.map(|n| n.start_position().column).unwrap_or(0);
    let name_end_col = name_node.map(|n| n.end_position().column).unwrap_or(0);

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

    FunctionInfo { name, line: node.start_position().row + 1, end_line: node.end_position().row + 1, name_start_col, name_end_col, parameters, return_type, function_calls, local_variables }
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
                    let call_line = child.start_position().row + 1;

                    match func_node.kind() {
                        "member_expression" => {
                            let prop_node = func_node.child_by_field_name("property");
                            let property = prop_node
                                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                .unwrap_or("")
                                .to_string();
                            let start_col = prop_node.map(|n| n.start_position().column).unwrap_or(0);
                            let end_col   = prop_node.map(|n| n.end_position().column).unwrap_or(0);

                            let obj_node = func_node.child_by_field_name("object");
                            let obj_kind = obj_node.map(|n| n.kind()).unwrap_or("");

                            if obj_kind == "call_expression" {
                                // Llamada encadenada: hola().bar()
                                let chain_source_fn = obj_node
                                    .and_then(|n| n.child_by_field_name("function"))
                                    .and_then(|f| {
                                        if f.kind() == "member_expression" {
                                            f.child_by_field_name("property")
                                        } else {
                                            Some(f) // identifier
                                        }
                                    })
                                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                    .map(|s| s.to_string());

                                calls.push(FunctionCall {
                                    name: property, line: call_line, start_col, end_col,
                                    import_name: None, object_name: None, chain_source_fn,
                                });
                            } else {
                                // obj.method() o module.function()
                                let object = obj_node
                                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                    .unwrap_or("")
                                    .to_string();

                                let is_real_import = imports.iter().any(|i| {
                                    i.name == object || i.imported_names.contains(&object)
                                });

                                if is_real_import {
                                    calls.push(FunctionCall {
                                        name: property, line: call_line, start_col, end_col,
                                        import_name: Some(object), object_name: None, chain_source_fn: None,
                                    });
                                } else {
                                    calls.push(FunctionCall {
                                        name: property, line: call_line, start_col, end_col,
                                        import_name: None, object_name: Some(object), chain_source_fn: None,
                                    });
                                }
                            }
                        }
                        _ => {
                            // Llamada directa: foo() — identifier u otro
                            let name = func_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                            let import_name = imports.iter()
                                .find(|i| i.imported_names.contains(&name))
                                .map(|i| i.name.clone());
                            let start_col = func_node.start_position().column;
                            let end_col   = func_node.end_position().column;
                            calls.push(FunctionCall {
                                name, line: call_line, start_col, end_col,
                                import_name, object_name: None, chain_source_fn: None,
                            });
                        }
                    }
                }
                calls.extend(find_calls(source, &child, imports));
            }
            _ => calls.extend(find_calls(source, &child, imports)),
        }
    }

    calls
}

const ARRAY_ITER_METHODS: &[&str] = &[
    "forEach", "map", "filter", "find", "findLast", "findIndex",
    "some", "every", "reduce", "reduceRight",
];

/// Dado un `call_expression` del tipo `this.items.reduce((sum, item) => ...)`,
/// extrae el parámetro "elemento" del callback y lo devuelve como `LocalVariable`
/// con `iterated_from = "this.items"`, para que el backend pueda resolver su tipo.
fn extract_callback_params(source: &str, call_expr: &Node) -> Vec<LocalVariable> {
    let mut vars = vec![];

    let Some(func) = call_expr.child_by_field_name("function") else { return vars; };
    if func.kind() != "member_expression" { return vars; }

    let method = match func.child_by_field_name("property")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
    {
        Some(m) if ARRAY_ITER_METHODS.contains(&m) => m.to_string(),
        _ => return vars,
    };

    let obj_text = match func.child_by_field_name("object")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(|s| s.to_string())
    {
        Some(o) if o.starts_with("this.") || o.starts_with("self.") => o,
        _ => return vars,
    };

    let Some(args) = call_expr.child_by_field_name("arguments") else { return vars; };

    // reduce/reduceRight: callback(acc, item) → elemento en índice 1; el resto en índice 0
    let elem_idx = if method == "reduce" || method == "reduceRight" { 1 } else { 0 };

    for arrow in args.named_children(&mut args.walk())
        .filter(|c| c.kind() == "arrow_function")
    {
        // Arrow con paréntesis: (sum, item) => ...  →  field "parameters"
        // Arrow sin paréntesis: item => ...          →  field "parameter"
        let param_name = if let Some(params) = arrow.child_by_field_name("parameters") {
            params.named_children(&mut params.walk())
                .nth(elem_idx)
                .and_then(|p| {
                    if p.kind() == "identifier" {
                        p.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
                    } else {
                        p.child_by_field_name("pattern")
                            .or_else(|| p.named_children(&mut p.walk()).find(|c| c.kind() == "identifier"))
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string())
                    }
                })
        } else if elem_idx == 0 {
            arrow.child_by_field_name("parameter")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|s| s.to_string())
        } else {
            None
        };

        if let Some(name) = param_name {
            vars.push(LocalVariable {
                name,
                assigned_from: None,
                assigned_identifier: None,
                iterated_from: Some(obj_text.clone()),
                destructured_property: None,
                line: call_expr.start_position().row + 1,
            });
        }
    }

    vars
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
                        let name_node = declarator.child_by_field_name("name");
                        let rhs = declarator.child_by_field_name("value");

                        // Pelar non_null_expression (TypeScript's `expr!`) para llegar al call real
                        let rhs_inner = rhs.map(|n| {
                            if n.kind() == "non_null_expression" {
                                let mut w = n.walk();
                                n.named_children(&mut w).next().unwrap_or(n)
                            } else {
                                n
                            }
                        });

                        let assigned_from = rhs_inner
                            .filter(|n| n.kind() == "call_expression" || n.kind() == "new_expression")
                            .and_then(|n| {
                                if n.kind() == "new_expression" {
                                    n.child_by_field_name("constructor")
                                        .and_then(|c| c.utf8_text(source.as_bytes()).ok())
                                        .map(|s| s.to_string())
                                } else {
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

                        // Destructuring: const { userAPI, productAPI } = buildApp()
                        if name_node.map(|n| n.kind()) == Some("object_pattern") {
                            let pattern = name_node.unwrap();
                            let line = declarator.start_position().row + 1;
                            for prop in pattern.named_children(&mut pattern.walk()) {
                                match prop.kind() {
                                    "shorthand_property_identifier_pattern" => {
                                        if let Some(prop_name) = prop.utf8_text(source.as_bytes()).ok() {
                                            variables.push(LocalVariable {
                                                name: prop_name.to_string(),
                                                assigned_from: assigned_from.clone(),
                                                assigned_identifier: None,
                                                iterated_from: None,
                                                destructured_property: Some(prop_name.to_string()),
                                                line,
                                            });
                                        }
                                    }
                                    "pair_pattern" => {
                                        let key = prop.child_by_field_name("key")
                                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                            .map(|s| s.to_string());
                                        let value = prop.child_by_field_name("value")
                                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                            .map(|s| s.to_string());
                                        if let (Some(k), Some(v)) = (key, value) {
                                            variables.push(LocalVariable {
                                                name: v,
                                                assigned_from: assigned_from.clone(),
                                                assigned_identifier: None,
                                                iterated_from: None,
                                                destructured_property: Some(k),
                                                line,
                                            });
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            continue;
                        }

                        let var_name = name_node
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string());

                        let assigned_identifier = rhs
                            .filter(|n| n.kind() == "identifier")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string());

                        // Captura parámetros de callbacks de array:
                        // const x = this.items.reduce((sum, item) => ..., 0)
                        if let Some(call) = rhs_inner.filter(|n| n.kind() == "call_expression") {
                            variables.extend(extract_callback_params(source, &call));
                        }

                        if let Some(name) = var_name {
                            variables.push(LocalVariable {
                                name,
                                assigned_from,
                                assigned_identifier,
                                iterated_from: None,
                                destructured_property: None,
                                line: declarator.start_position().row + 1,
                            });
                        }
                    }
                }
                variables.extend(find_local_variables(source, &child));
            }
            // Captura asignaciones this.attr = value en cuerpos de constructor/método
            "expression_statement" => {
                let mut expr_cursor = child.walk();
                for expr in child.named_children(&mut expr_cursor) {
                    if expr.kind() == "assignment_expression" {
                        let lhs = expr.child_by_field_name("left");
                        let rhs = expr.child_by_field_name("right");

                        let var_name = lhs
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string());

                        let rhs_inner = rhs.map(|n| {
                            if n.kind() == "non_null_expression" {
                                let mut w = n.walk();
                                n.named_children(&mut w).next().unwrap_or(n)
                            } else {
                                n
                            }
                        });

                        let assigned_from = rhs_inner
                            .filter(|n| n.kind() == "call_expression" || n.kind() == "new_expression")
                            .and_then(|n| {
                                if n.kind() == "new_expression" {
                                    n.child_by_field_name("constructor")
                                        .and_then(|c| c.utf8_text(source.as_bytes()).ok())
                                        .map(|s| s.to_string())
                                } else {
                                    n.child_by_field_name("function")
                                        .and_then(|f| {
                                            if f.kind() == "member_expression" { f.child_by_field_name("property") } else { Some(f) }
                                        })
                                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                        .map(|s| s.to_string())
                                }
                            });

                        let assigned_identifier = rhs
                            .filter(|n| n.kind() == "identifier")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string());

                        if let Some(name) = var_name {
                            variables.push(LocalVariable {
                                name,
                                assigned_from,
                                assigned_identifier,
                                iterated_from: None,
                                destructured_property: None,
                                line: expr.start_position().row + 1,
                            });
                        }
                    }
                }
                variables.extend(find_local_variables(source, &child));
            }
            "expression_statement" if child.named_children(&mut child.walk())
                .any(|c| c.kind() == "call_expression") =>
            {
                // Captura arrow function callbacks sobre this/self arrays:
                // this.items.forEach((item) => item.method())
                for expr in child.named_children(&mut child.walk()) {
                    if expr.kind() == "call_expression" {
                        variables.extend(extract_callback_params(source, &expr));
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