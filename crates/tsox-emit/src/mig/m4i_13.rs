use std::sync::Arc;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{Node, SourceFile, SyntaxKind};
use super::m4i_12::ConstEnumInliningTransformer;
use crate::printer::mig::m4m_2::SynthesizedComment;
use crate::printer::{EmitContext, NodeFactory};
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::jsnum::PseudoBigInt;
use tsox_core::jsnum::Number;
use tsox_core::collections::ordered_map::OrderedMap;
use crate::mig::m3m::TransformOptions;
use tsox_frontend::ast::node_is_synthesized;
use tsox_frontend::scanner::mig::m3i::get_text_of_node;
use tsox_frontend::ast::mig::m3f::get_semantic_jsx_children;
use tsox_frontend::format::mig::m4o::EmitFlags;
use crate::mig::m4j::decode_entities;
use crate::mig::m4j::r36k3_defs::{R36K3NodeAccessExt, R36K3NodeFactoryExt};
use super::m4i_8::r40k18_defs::{R40K18EmitContextExt, R40K18EmitResolverExt};

pub enum ConstantValue {
    String(String),
    Number(Number),
    PseudoBigInt(PseudoBigInt),
}

impl<'a> ConstEnumInliningTransformer<'a> {
    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(self.emit_context)
    }

    pub fn visit(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit"); 
        match node.kind {
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                let parse = self.emit_context.parse_node(&node);
                let value = match parse {
                    None => None,
                    Some(parse) => self.emit_resolver.get_constant_value(&parse),
                };
                if let Some(value) = value {
                    let replacement = match value {
                        ConstantValue::Number(v) => {
                            if v.is_inf() {
                                if v.abs() == v {
                                    self.factory().new_identifier("Infinity")
                                } else {
                                    self.factory().new_prefix_unary_expression(
                                        SyntaxKind::MinusToken,
                                        &self.factory().new_identifier("Infinity"),
                                    )
                                }
                            } else if v.is_nan() {
                                self.factory().new_identifier("NaN")
                            } else if v.abs() == v {
                                self.factory().new_numeric_literal(&v.to_string(), 0)
                            } else {
                                self.factory().new_prefix_unary_expression(
                                    SyntaxKind::MinusToken,
                                    &self.factory().new_numeric_literal(&v.abs().to_string(), 0),
                                )
                            }
                        }
                        ConstantValue::String(v) => {
                            self.factory().new_string_literal(&v, 0)
                        }
                        ConstantValue::PseudoBigInt(v) => {
                            if v.is_zero() {
                                self.factory().new_big_int_literal("0", 0)
                            } else if !v.negative {
                                self.factory().new_big_int_literal(&v.base10_value, 0)
                            } else {
                                self.factory().new_prefix_unary_expression(
                                    SyntaxKind::MinusToken,
                                    &self.factory().new_big_int_literal(&v.base10_value, 0),
                                )
                            }
                        }
                    };

                    if self.compiler_options.remove_comments.is_false_or_unknown() {
                        let original = self.emit_context.most_original(&node);
                        if !node_is_synthesized(&original) {
                            let original_text = get_text_of_node(&original);
                            let escaped_text = safe_multi_line_comment(&original_text);
                            let comment = SynthesizedComment {
                                kind: SyntaxKind::MultiLineCommentTrivia,
                                loc: TextRange::undefined(),
                                has_leading_new_line: false,
                                has_trailing_new_line: false,
                                text: escaped_text,
                            };
                            self.emit_context
                                .emit_nodes_get_mut(&replacement)
                                .trailing_comments
                                .push(comment);
                        }
                    }
                    return replacement;
                }
                self.visit_each_child(node)
            }
            _ => self.visit_each_child(node),
        }
    }

    pub fn visit_each_child(&mut self, node: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child"); 
        let mut changed = false;
        tsox_frontend::ast::node_data_generated::for_each_child(&node, |child| {
            let visited = self.visit(child.clone());
            changed |= !Arc::ptr_eq(&visited, child);
            true
        });
        if !changed {
            return node;
        }
        panic!(
            "ConstEnumInliningTransformer visit_each_child rebuild for kind {:?} pending factory update-function port (progress_notes_r39k14.md)",
            node.kind
        )
    }
}

pub fn safe_multi_line_comment(text: &str) -> String { ::tsox_core::fntrace::enter("safe_multi_line_comment"); 
    let mut b = String::with_capacity(text.len() + 2);
    b.push(' ');
    let mut text = text;
    loop {
        match text.find("*/") {
            None => break,
            Some(i) => {
                b.push_str(&text[..i]);
                b.push_str("*_/");
                text = &text[i + 2..];
            }
        }
    }
    b.push_str(text);
    b.push(' ');
    b
}

pub struct JSXTransformer<'a> {
    emit_context: &'a EmitContext,
    compiler_options: &'a CompilerOptions,
    emit_resolver: &'a EmitResolver,
    import_specifier: String,
    filename_declaration: Option<Arc<Node>>,
    utilized_implicit_runtime_imports: OrderedMap<String, std::collections::HashMap<String, Arc<Node>>>,
    in_jsx_child: bool,
    current_source_file: Option<Arc<SourceFile>>,
}

pub fn new_jsx_transformer<'a>(opts: &'a TransformOptions<'a>) -> JSXTransformer<'a> { ::tsox_core::fntrace::enter("new_jsx_transformer"); 
    JSXTransformer {
        emit_context: opts.context,
        compiler_options: opts.compiler_options,
        emit_resolver: &opts.emit_resolver,
        import_specifier: String::new(),
        filename_declaration: None,
        utilized_implicit_runtime_imports: OrderedMap::new(),
        in_jsx_child: false,
        current_source_file: None,
    }
}

impl<'a> JSXTransformer<'a> {
    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(self.emit_context)
    }

    pub fn combine_properties_into_new_expression(
        &mut self,
        mut expressions: Vec<Arc<Node>>,
        props: Vec<Arc<Node>>,
    ) -> (Vec<Arc<Node>>, Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("combine_properties_into_new_expression"); 
        if props.is_empty() {
            return (expressions, props);
        }
        let new_obj = self
            .factory()
            .new_object_literal_expression(&*self.factory().new_node_list(props), false);
        expressions.push(new_obj);
        (expressions, Vec::new())
    }

    pub fn convert_jsx_children_to_children_prop_object(&mut self, children: &[Arc<Node>]) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("convert_jsx_children_to_children_prop_object"); 
        let prop = self.convert_jsx_children_to_children_prop_assignment(children)?;
        Some(
            self.factory()
                .new_object_literal_expression(&*self.factory().new_node_list(vec![prop]), false),
        )
    }

    pub fn convert_jsx_children_to_children_prop_assignment(&mut self, children: &[Arc<Node>]) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("convert_jsx_children_to_children_prop_assignment"); 
        let non_whitespace_children = get_semantic_jsx_children(children);
        if non_whitespace_children.len() == 1
            && (non_whitespace_children[0].kind != SyntaxKind::JsxExpression
                || non_whitespace_children[0]
                    .as_jsx_expression()
                    .dot_dot_dot_token
                    .is_none())
        {
            let Some(result) = self.transform_jsx_child_to_expression(&non_whitespace_children[0])
            else {
                return None;
            };
            return Some(
                self.factory().new_property_assignment(
                    None,
                    &self.factory().new_identifier("children"),
                    None,
                    None,
                    &result,
                ),
            );
        }
        let mut results: Vec<Arc<Node>> = Vec::with_capacity(non_whitespace_children.len());
        for child in &non_whitespace_children {
            let Some(res) = self.transform_jsx_child_to_expression(child) else {
                continue;
            };
            self.emit_context.set_emit_flags_shared(
                &res,
                self.emit_context.emit_flags(&res) & !EmitFlags::START_ON_NEW_LINE,
            );
            results.push(res);
        }
        if results.is_empty() {
            return None;
        }
        Some(
            self.factory().new_property_assignment(
                None,
                &self.factory().new_identifier("children"),
                None,
                None,
                &self.factory()
                    .new_array_literal_expression(&*self.factory().new_node_list(results), false),
            ),
        )
    }

    pub fn transform_jsx_child_to_expression(&mut self, child: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_jsx_child_to_expression"); 
        match child.kind {
            SyntaxKind::JsxText => {
                let text = get_text_of_node(child);
                if text.chars().all(|c| c.is_whitespace()) {
                    None
                } else {
                    Some(self.factory().new_string_literal(&text, 0))
                }
            }
            SyntaxKind::JsxExpression => child.as_jsx_expression().expression.clone(),
            SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxElement => None,
            _ => None,
        }
    }
}

pub fn add_line_of_jsx_text(b: &mut String, trimmed_line: &str, is_initial: bool) { ::tsox_core::fntrace::enter("add_line_of_jsx_text"); 
    let decoded = decode_entities(trimmed_line);
    if !is_initial {
        b.push(' ');
    }
    b.push_str(&decoded);
}
