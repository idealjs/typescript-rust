use std::sync::Arc;

use tsox_core::core::compiler_options::{CompilerOptions, JsxEmit};
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{for_each_child, is_jsx_opening_element, is_jsx_self_closing_element, NodeData};
use tsox_frontend::ast::mig::m3f::{get_pragma_argument, get_pragma_from_source_file};

pub fn set_parent_in_children(node: &Arc<Node>) {
    let mut state: Option<Arc<Node>> = None;
    set_parent_in_children_visit(node, &mut state);
}

fn set_parent_in_children_visit(node: &Arc<Node>, parent: &mut Option<Arc<Node>>) {
    if let Some(p) = parent.as_ref() {
        node.set_parent(p);
    }
    let save_parent = parent.take();
    *parent = Some(Arc::clone(node));
    for_each_child(node, |n: &Arc<Node>| {
        set_parent_in_children_visit(n, parent);
        false
    });
    *parent = save_parent;
}

pub fn get_jsx_implicit_import_base(
    compiler_options: &CompilerOptions,
    file: &tsox_frontend::ast::node::SourceFile,
) -> String {
    let jsx_import_source_pragma = get_pragma_from_source_file(Some(file), "jsximportsource");
    let jsx_runtime_pragma = get_pragma_from_source_file(Some(file), "jsxruntime");
    if get_pragma_argument(jsx_runtime_pragma, "factory") == "classic" {
        return String::new();
    }
    if compiler_options.jsx == JsxEmit::ReactJSX
        || compiler_options.jsx == JsxEmit::ReactJSXDev
        || !compiler_options.jsx_import_source.is_empty()
        || jsx_import_source_pragma.is_some()
        || get_pragma_argument(jsx_runtime_pragma, "factory") == "automatic"
    {
        let mut result = get_pragma_argument(jsx_import_source_pragma, "factory");
        if result.is_empty() {
            result = compiler_options.jsx_import_source.clone();
        }
        if result.is_empty() {
            result = "react".to_string();
        }
        return result;
    }
    String::new()
}

pub fn get_jsx_runtime_import(base: &str, options: &CompilerOptions) -> String {
    if base.is_empty() {
        return base.to_string();
    }
    let runtime = if options.jsx == JsxEmit::ReactJSXDev {
        "jsx-dev-runtime"
    } else {
        "jsx-runtime"
    };
    format!("{}/{}", base, runtime)
}

pub fn is_jsx_opening_like_element(node: &Node) -> bool {
    is_jsx_opening_element(node) || is_jsx_self_closing_element(node)
}

pub fn get_semantic_jsx_children(children: &[Arc<Node>]) -> Vec<Arc<Node>> {
    children
        .iter()
        .filter(|child| match &child.data {
            NodeData::JsxExpression(d) => d.expression.is_some(),
            NodeData::JsxText(d) => !d.contains_only_trivia_white_spaces,
            _ => true,
        })
        .cloned()
        .collect()
}

pub fn create_expression_from_entity_name(
    factory: &crate::printer::NodeFactory,
    node: &Arc<Node>,
) -> Arc<Node> {
    if let NodeData::QualifiedName(d) = &node.data {
        let left = create_expression_from_entity_name(factory, &d.left);
        let right = factory.new_identifier(d.right.text());
        return factory.new_property_access_expression(
            &left,
            None,
            &right,
            tsox_frontend::ast::node_flags::NodeFlags::empty(),
        );
    }
    factory.new_identifier(node.text())
}

use tsox_frontend::format::mig::m4o_2::EmitHelper;

pub fn async_super_helper() -> Arc<EmitHelper> {
    Arc::new(EmitHelper {
        name: "typescript:async-super".to_string(),
        scoped: true,
        text: String::new(),
        text_callback: Some(Box::new(|make_unique_name: &dyn Fn(&str) -> String| {
            format!("\nconst {} = name => super[name];", make_unique_name("_superIndex"))
        })),
        priority: None,
        dependencies: Vec::new(),
        import_name: String::new(),
    })
}

pub fn advanced_async_super_helper() -> Arc<EmitHelper> {
    Arc::new(EmitHelper {
        name: "typescript:advanced-async-super".to_string(),
        scoped: true,
        text: String::new(),
        text_callback: Some(Box::new(|make_unique_name: &dyn Fn(&str) -> String| {
            format!(
                "\nconst {super_index} = (function (geti, seti) {{\n    const cache = Object.create(null);\n    return name => cache[name] || (cache[name] = {{ get value() {{ return geti(name); }}, set value(v) {{ seti(name, v); }} }});\n}})(name => super[name], (name, value) => super[name] = value);",
                super_index = make_unique_name("_superIndex")
            )
        })),
        priority: None,
        dependencies: Vec::new(),
        import_name: String::new(),
    })
}
