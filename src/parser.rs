use tree_sitter::{Parser, TreeCursor};
use crate::models::{analysis_result::AnalysisResult, class_info::ClassInfo, function_info::FunctionInfo};


pub fn parse_source(source: &str) -> AnalysisResult {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_python::language()).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let root_node = tree.root_node();

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
            
                let func_info = FunctionInfo {
                    name,
                    parameters,
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


fn get_function_parameters<'a>(source: &'a str, node: &tree_sitter::Node<'a>) -> Vec<String> {
    let mut params = Vec::new();
    if let Some(param_node) = node.child_by_field_name("parameters") {
        for child in param_node.named_children(&mut param_node.walk()) {
            match child.kind() {
                "identifier" => {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        params.push(name.to_string());
                    }
                }
                "default_parameter" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if let Ok(name) = name_node.utf8_text(source.as_bytes()) {
                            params.push(name.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    params
}