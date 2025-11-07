use tree_sitter::{Parser, TreeCursor, Node};
use crate::models::{analysis_result::AnalysisResult, class_info::ClassInfo, function_info::FunctionInfo, parameter_info::ParameterInfo};


pub fn parse_file(source: &str) -> AnalysisResult {
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
    analyze_node(source, &mut root_node.walk(), &mut result, &mut none_class);

    result
}


fn analyze_node(source: &str, cursor: &mut TreeCursor, result: &mut AnalysisResult, current_class: &mut Option<&mut ClassInfo>) {
    loop {
        let node = cursor.node();
        let kind = node.kind();

        match kind {
            "import_statement" => {
                let text = node.utf8_text(source.as_bytes()).unwrap_or("").trim().to_string();
                result.imports.push(text);
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

                let mut function_calls: Option<Vec<String>> = None;
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
                    analyze_node(source, &mut inner_cursor, result, &mut class_ref);
                }
            
                result.classes.push(class_info);
            }
            _ => {}
        }

        if kind != "class_definition" && cursor.goto_first_child() {
            analyze_node(source, cursor, result, current_class);
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

fn find_calls<'a>(source: &'a str, node: &tree_sitter::Node<'a>) -> Vec<String> {
    let mut cursor = node.walk();
    let mut calls = vec![];

    for child in node.children(&mut cursor) {
        match child.kind() {
            // Nodo de llamada de función en Python
            "call" => {
                if let Some(func_node) = child.child_by_field_name("function") {
                    let name = func_node.utf8_text(source.as_bytes()).unwrap().to_string();
                    calls.push(name);
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