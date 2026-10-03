use std::sync::Arc;

use crate::checker::types_impl_chunk::{LiteralValue, TypeData};
use crate::checker::types::*;
use tsox_frontend::ast::node_data_generated::{
    BinaryExpressionData, ConditionalTypeNodeData, ImportEqualsDeclarationData,
    IntersectionTypeNodeData, LiteralTypeNodeData, MappedTypeNodeData, NamedTupleMemberData,
    ParameterDeclarationData, ParenthesizedTypeNodeData, TaggedTemplateExpressionData,
    TemplateExpressionData, TemplateSpanData, TypeReferenceNodeData, UnionTypeNodeData,
    YieldExpressionData,
};
use tsox_frontend::ast::{Node, NodeData};

pub(crate) fn internal_symbol_name_computed() -> &'static str { ::tsox_core::fntrace::enter("internal_symbol_name_computed"); 
    "\u{FE}computed"
}

pub(crate) fn internal_symbol_name_type() -> &'static str { ::tsox_core::fntrace::enter("internal_symbol_name_type"); 
    "\u{FE}type"
}

macro_rules! node_data_accessor {
    ($fn_name:ident, $variant:ident, $ty:ty) => {
        pub(crate) fn $fn_name(node: &Node) -> &$ty {
            match &node.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!(stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

node_data_accessor!(
    tagged_template_expression_data,
    TaggedTemplateExpression,
    TaggedTemplateExpressionData
);
node_data_accessor!(template_expression_data, TemplateExpression, TemplateExpressionData);
node_data_accessor!(template_span_data, TemplateSpan, TemplateSpanData);
node_data_accessor!(binary_expression_data, BinaryExpression, BinaryExpressionData);
node_data_accessor!(
    intersection_type_node_data,
    IntersectionTypeNode,
    IntersectionTypeNodeData
);
node_data_accessor!(union_type_node_data, UnionTypeNode, UnionTypeNodeData);
node_data_accessor!(
    conditional_type_node_data,
    ConditionalTypeNode,
    ConditionalTypeNodeData
);
node_data_accessor!(
    parenthesized_type_node_data,
    ParenthesizedTypeNode,
    ParenthesizedTypeNodeData
);
node_data_accessor!(named_tuple_member_data, NamedTupleMember, NamedTupleMemberData);
node_data_accessor!(type_reference_node_data, TypeReferenceNode, TypeReferenceNodeData);
node_data_accessor!(mapped_type_node_data, MappedTypeNode, MappedTypeNodeData);
node_data_accessor!(
    import_equals_declaration_data,
    ImportEqualsDeclaration,
    ImportEqualsDeclarationData
);
node_data_accessor!(literal_type_node_data, LiteralTypeNode, LiteralTypeNodeData);
node_data_accessor!(yield_expression_data, YieldExpression, YieldExpressionData);
node_data_accessor!(
    parameter_declaration_data,
    ParameterDeclaration,
    ParameterDeclarationData
);

pub(crate) fn tagged_template_expression_template(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("tagged_template_expression_template"); 
    Arc::clone(&tagged_template_expression_data(node).template)
}

pub(crate) fn template_expression_template_spans(node: &Node) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("template_expression_template_spans"); 
    template_expression_data(node).template_spans.nodes.clone()
}

pub(crate) fn template_span_expression(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("template_span_expression"); 
    Arc::clone(&template_span_data(node).expression)
}

pub(crate) fn binary_expression_left(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("binary_expression_left"); 
    Arc::clone(&binary_expression_data(node).left)
}

pub(crate) fn binary_expression_right(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binary_expression_right"); 
    Some(Arc::clone(&binary_expression_data(node).right))
}

pub(crate) fn intersection_type_node_types(node: &Node) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("intersection_type_node_types"); 
    intersection_type_node_data(node).types.nodes.clone()
}

pub(crate) fn union_type_node_types(node: &Node) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("union_type_node_types"); 
    union_type_node_data(node).types.nodes.clone()
}

pub(crate) fn conditional_type_node_true_type(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("conditional_type_node_true_type"); 
    Arc::clone(&conditional_type_node_data(node).true_type)
}

pub(crate) fn conditional_type_node_false_type(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("conditional_type_node_false_type"); 
    Arc::clone(&conditional_type_node_data(node).false_type)
}

pub(crate) fn parenthesized_type_node_type(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parenthesized_type_node_type"); 
    Some(Arc::clone(&parenthesized_type_node_data(node).type_node))
}

pub(crate) fn named_tuple_member_type(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("named_tuple_member_type"); 
    Some(Arc::clone(&named_tuple_member_data(node).type_node))
}

pub(crate) fn type_reference_node_type_name(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("type_reference_node_type_name"); 
    Arc::clone(&type_reference_node_data(node).type_name)
}

pub(crate) fn mapped_type_node_type_parameter(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("mapped_type_node_type_parameter"); 
    Arc::clone(&mapped_type_node_data(node).type_parameter)
}

pub(crate) fn import_equals_declaration_module_reference(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("import_equals_declaration_module_reference"); 
    Arc::clone(&import_equals_declaration_data(node).module_reference)
}

pub(crate) fn yield_expression_asterisk_token(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("yield_expression_asterisk_token"); 
    yield_expression_data(node).asterisk_token.clone()
}

pub(crate) fn parameter_declaration_dot_dot_dot_token(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parameter_declaration_dot_dot_dot_token"); 
    parameter_declaration_data(node).dot_dot_dot_token.clone()
}

pub(crate) fn mapped_type_declaration(t: &Arc<Type>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("mapped_type_declaration"); 
    match &t.data {
        TypeData::Mapped(m) => m.declaration.clone(),
        _ => None,
    }
}

pub(crate) fn mapped_declaration_readonly_token(
    declaration: &Option<Arc<Node>>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("mapped_declaration_readonly_token"); 
    declaration.as_ref().and_then(|d| {
        match &d.data {
            NodeData::MappedTypeNode(m) => m.readonly_token.clone(),
            _ => None,
        }
    })
}

pub(crate) fn mapped_declaration_question_token(
    declaration: &Option<Arc<Node>>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("mapped_declaration_question_token"); 
    declaration.as_ref().and_then(|d| match &d.data {
        NodeData::MappedTypeNode(m) => m.question_token.clone(),
        _ => None,
    })
}

pub(crate) fn mapped_type_target(t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("mapped_type_target"); 
    match &t.data {
        TypeData::Mapped(m) => m.object.target.clone(),
        _ => None,
    }
}

pub(crate) fn interface_this_type(t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("interface_this_type"); 
    t.as_interface().and_then(|i| i.this_type.clone())
}

pub(crate) fn target_interface_type_parameters(t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("target_interface_type_parameters"); 
    match &t.data {
        TypeData::Interface(i) => i.all_type_parameters.clone(),
        TypeData::Object(o) => match &o.target {
            Some(target) => target_interface_type_parameters(target),
            None => Vec::new(),
        },
        _ => Vec::new(),
    }
}

pub(crate) fn index_type_target(t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("index_type_target"); 
    match &t.data {
        TypeData::Index(i) => i.target.clone().unwrap_or_else(|| Arc::clone(t)),
        _ => Arc::clone(t),
    }
}

pub(crate) fn conditional_root(t: &Arc<Type>) -> &crate::checker::types_impl_chunk_2::ConditionalRoot { ::tsox_core::fntrace::enter("conditional_root"); 
    match &t.data {
        TypeData::Conditional(c) => match c.root.as_deref() {
            Some(root) => root,
            None => panic!("conditional root missing"),
        },
        _ => panic!("conditional root on non-conditional type"),
    }
}

pub(crate) fn conditional_check_type(t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("conditional_check_type"); 
    match &t.data {
        TypeData::Conditional(c) => c.check_type.clone().unwrap_or_else(|| Arc::clone(t)),
        _ => Arc::clone(t),
    }
}

pub(crate) fn literal_type_regular_type(t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("literal_type_regular_type"); 
    match &t.data {
        TypeData::Literal(l) => l
            .regular_type
            .get()
            .cloned()
            .unwrap_or_else(|| Arc::clone(t)),
        _ => Arc::clone(t),
    }
}

pub(crate) fn literal_type_number_value(t: &Arc<Type>) -> f64 { ::tsox_core::fntrace::enter("literal_type_number_value"); 
    match &t.data {
        TypeData::Literal(l) => match &l.value {
            LiteralValue::Number(n) => n.0,
            _ => 0.0,
        },
        _ => 0.0,
    }
}

pub(crate) fn same_types(a: &[Arc<Type>], b: &[Arc<Type>]) -> bool { ::tsox_core::fntrace::enter("same_types"); 
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Arc::ptr_eq(x, y))
}

pub(crate) fn scanner_declaration_name_to_string(name: &Node) -> String { ::tsox_core::fntrace::enter("scanner_declaration_name_to_string"); 
    match name.kind {
        tsox_frontend::ast::SyntaxKind::ComputedPropertyName => match name.expression() {
            Some(e) => scanner_declaration_name_to_string(e),
            None => "x".to_string(),
        },
        _ => name.text().to_string(),
    }
}

impl crate::checker::checker_checker_checker::Checker {
    pub fn source_file_common_js_module_indicator(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("source_file_common_js_module_indicator"); 
        let file = self.get_source_file_of_node(node)?;
        file.common_js_module_indicator.clone()
    }
}
