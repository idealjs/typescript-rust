#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::mig::m3g_2::{is_parameter_property_declaration, is_this_parameter};
use tsox_frontend::ast::mig::m3g_3::{skip_outer_expressions, OuterExpressionKinds};
use tsox_frontend::ast::mig::w7a::is_jsdoc_type_assertion;
use tsox_frontend::ast::node::{ModifierList, Node};
use tsox_frontend::ast::node_data_generated::{is_satisfies_expression, NodeData};
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{is_assertion_expression, node_is_missing};

use super::m4j_2::extract_modifiers;
use super::m4l::r33k9_defs::{
    OUTER_EXPRESSION_KINDS_ALL_EXCEPT_ASSERTIONS_OR_EXPRESSIONS_WITH_TYPE_ARGUMENTS,
};
use super::m4l::r39k20_defs::{R39K20NodeFactoryExt, R39K20NodeVisitorExt};
use super::m4f_2::r38k5_defs::R38K5NodeVisitorExt;
use super::m4l_6::TypeEraserTransformer;
use crate::mig::m4g::r33k7_defs::has_decorators;
use crate::mig::m4n::r37k19_defs::R37K19ArcNodeExt;

#[path = "r38k3_defs.rs"]
pub mod r38k3_defs;
#[path = "r38k3b_defs.rs"]
pub mod r38k3b_defs;
use r38k3_defs::{K3NodeAccessExt, TypeEraserK3Ext};
use r38k3b_defs::visit_accessor_declaration_k3;

impl TypeEraserTransformer {
    pub(crate) fn visit_inner_tail(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::PropertyDeclaration => {
                if self.compiler_options.experimental_decorators.is_true()
                    && node.has_syntactic_modifier(
                        ModifierFlags::Ambient.union(ModifierFlags::Abstract),
                    )
                    && has_decorators(node)
                {
                    let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                    let name = self.visitor().visit_node(&node.name().unwrap());
                    let initializer = self.visitor().visit_node_opt(node.initializer());
                    return Some(self.factory().r39k20_update_property_declaration(
                        node,
                        modifiers,
                        &name,
                        None,
                        None,
                        initializer,
                    ));
                }
                if node.has_syntactic_modifier(
                    ModifierFlags::Ambient.union(ModifierFlags::Abstract),
                ) {
                    return None;
                }
                let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                let name = self.visitor().visit_node(&node.name().unwrap());
                let initializer = self.visitor().visit_node_opt(node.initializer());
                Some(self.factory().r39k20_update_property_declaration(
                    node,
                    modifiers,
                    &name,
                    None,
                    None,
                    initializer,
                ))
            }

            SyntaxKind::Constructor => {
                if node_is_missing(node.body()) {
                    return None;
                }
                let parameters = self.visitor().visit_nodes(node.parameters()).unwrap_or_default();
                let body = self.visitor().visit_node_opt(node.body());
                Some(self.factory().update_constructor_declaration(
                    node,
                    None,
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                ))
            }

            SyntaxKind::MethodDeclaration => {
                if node_is_missing(node.body()) {
                    return None;
                }
                let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                let name = self.visitor().visit_node(&node.name().unwrap());
                let parameters = self.visitor().visit_nodes(node.parameters()).unwrap_or_default();
                let body = self.visitor().visit_node_opt(node.body());
                Some(self.factory().update_method_declaration(
                    node,
                    modifiers,
                    node.asterisk_token(),
                    &name,
                    None,
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                ))
            }

            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                visit_accessor_declaration_k3(self, node)
            }

            SyntaxKind::VariableDeclaration => {
                let name = self.visitor().visit_node(&node.name().unwrap());
                let initializer = self.visitor().visit_node_opt(node.initializer());
                let updated = self.factory().update_variable_declaration_node(
                    node,
                    &name,
                    None,
                    None,
                    initializer,
                );
                if let Some(type_node) = node.type_() {
                    self.emit_context
                        .set_type_node(&updated.name().unwrap(), type_node);
                }
                Some(updated)
            }

            SyntaxKind::HeritageClause => {
                if node.token() == SyntaxKind::ImplementsKeyword {
                    return None;
                }
                let types = self.visitor().visit_nodes(node.types()).unwrap_or_default();
                Some(
                    self.factory()
                        .update_heritage_clause(node, node.token(), &types),
                )
            }

            SyntaxKind::ClassDeclaration => {
                let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                let name = self.visitor().visit_node_opt(node.name());
                let heritage_clauses = self.visitor().visit_nodes(node.heritage_clauses());
                let members = self.visitor().visit_nodes(node.members()).unwrap_or_default();
                Some(self.factory().update_class_declaration(
                    node,
                    modifiers,
                    name.as_ref(),
                    None,
                    heritage_clauses.as_ref(),
                    &members,
                ))
            }

            SyntaxKind::ClassExpression => {
                let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                let name = self.visitor().visit_node_opt(node.name());
                let heritage_clauses = self.visitor().visit_nodes(node.heritage_clauses());
                let members = self.visitor().visit_nodes(node.members()).unwrap_or_default();
                Some(self.factory().update_class_expression(
                    node,
                    modifiers,
                    name.as_ref(),
                    None,
                    heritage_clauses.as_ref(),
                    &members,
                ))
            }

            SyntaxKind::FunctionDeclaration => {
                if node_is_missing(node.body()) {
                    return self.elide(node);
                }
                let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                let name = self.visitor().visit_node_opt(node.name());
                let parameters = self.visitor().visit_nodes(node.parameters()).unwrap_or_default();
                let body = self.visitor().visit_node_opt(node.body());
                Some(self.factory().update_function_declaration(
                    node,
                    modifiers,
                    node.asterisk_token(),
                    name.as_ref(),
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                ))
            }

            SyntaxKind::FunctionExpression => {
                let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                let name = self.visitor().visit_node_opt(node.name());
                let parameters = self.visitor().visit_nodes(node.parameters()).unwrap_or_default();
                let body = self.visitor().visit_node_opt(node.body());
                Some(self.factory().update_function_expression(
                    node,
                    modifiers,
                    node.asterisk_token(),
                    name.as_ref(),
                    None,
                    &parameters,
                    None,
                    None,
                    body.as_ref(),
                ))
            }

            SyntaxKind::ArrowFunction => {
                let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
                let parameters = self.visitor().visit_nodes(node.parameters()).unwrap_or_default();
                let body = self
                    .visitor()
                    .visit_node(node.body().expect("ArrowFunction body"));
                Some(self.factory().update_arrow_function(
                    node,
                    modifiers,
                    None,
                    &parameters,
                    None,
                    None,
                    node.equals_greater_than_token(),
                    &body,
                ))
            }

            SyntaxKind::Parameter => {
                if is_this_parameter(node) {
                    return None;
                }
                let mut modifiers: Option<Arc<ModifierList>> = None;
                if self
                    .parent_node
                    .as_ref()
                    .is_some_and(|parent| is_parameter_property_declaration(node, parent))
                {
                    modifiers = extract_modifiers(
                        &TypeEraserK3Ext::emit_context(self),
                        node.modifiers().map(|m| &**m),
                        ModifierFlags::ParameterPropertyModifier,
                    )
                    .map(Arc::new);
                }
                if has_decorators(node) {
                    let visited = self.visitor().visit_slice(&node.decorators());
                    modifiers = Some(match modifiers {
                        None => self.factory().new_modifier_list(visited),
                        Some(list) => self
                            .factory()
                            .new_modifier_list([list.list.nodes.clone(), visited].concat()),
                    });
                }
                let name = self.visitor().visit_node(&node.name().unwrap());
                let initializer = self.visitor().visit_node_opt(node.initializer());
                Some(self.factory().r39k20_update_parameter_declaration(
                    node,
                    modifiers,
                    node.dot_dot_dot_token(),
                    &name,
                    None,
                    None,
                    initializer.as_ref(),
                ))
            }

            SyntaxKind::CallExpression => {
                let (expression, question_dot_token, arguments) = match &node.data {
                    NodeData::CallExpression(d) => (
                        Arc::clone(&d.expression),
                        d.question_dot_token.clone(),
                        Arc::clone(&d.arguments),
                    ),
                    _ => unreachable!(),
                };
                let visited_expression = self.visitor().visit_node(&expression);
                let visited_arguments = self
                    .visitor()
                    .visit_nodes(Some(&arguments))
                    .unwrap_or_default();
                Some(self.factory().update_call_expression(
                    node,
                    Some(visited_expression),
                    question_dot_token,
                    None,
                    Arc::new(visited_arguments),
                    node.flags,
                ))
            }

            SyntaxKind::NewExpression => {
                let (expression, arguments) = match &node.data {
                    NodeData::NewExpression(d) => (Arc::clone(&d.expression), d.arguments.clone()),
                    _ => unreachable!(),
                };
                let visited_expression = self.visitor().visit_node(&expression);
                let visited_arguments = self
                    .visitor()
                    .visit_nodes(arguments.as_ref())
                    .unwrap_or_default();
                Some(self.factory().update_new_expression(
                    node,
                    &visited_expression,
                    None,
                    &visited_arguments,
                ))
            }

            SyntaxKind::TaggedTemplateExpression => {
                let (tag, question_dot_token, template) = match &node.data {
                    NodeData::TaggedTemplateExpression(d) => (
                        Arc::clone(&d.tag),
                        d.question_dot_token.clone(),
                        Arc::clone(&d.template),
                    ),
                    _ => unreachable!(),
                };
                let visited_tag = self.visitor().visit_node(&tag);
                let visited_template = self.visitor().visit_node(&template);
                Some(self.factory().r39k20_update_tagged_template_expression(
                    node,
                    &visited_tag,
                    None,
                    question_dot_token,
                    &visited_template,
                    node.flags,
                ))
            }

            SyntaxKind::NonNullExpression
            | SyntaxKind::TypeAssertionExpression
            | SyntaxKind::AsExpression
            | SyntaxKind::SatisfiesExpression => {
                let visited_expression = self
                    .visitor()
                    .visit_node(node.expression().expect("expression"));
                let mut partial = self
                    .factory()
                    .new_partially_emitted_expression(&visited_expression);
                self.emit_context().set_original(&partial, node);
                partial.set_loc(node.loc);
                Some(partial)
            }

            SyntaxKind::ParenthesizedExpression => {
                if !is_jsdoc_type_assertion(Some(node.as_ref())) {
                    let expression = skip_outer_expressions(
                        node.expression().expect("ParenthesizedExpression expression"),
                        OUTER_EXPRESSION_KINDS_ALL_EXCEPT_ASSERTIONS_OR_EXPRESSIONS_WITH_TYPE_ARGUMENTS,
                    );
                    if is_assertion_expression(&expression) || is_satisfies_expression(&expression) {
                        let visited_expression = self.visitor().visit_node(
                            node.expression().expect("ParenthesizedExpression expression"),
                        );
                        let mut partial = self
                            .factory()
                            .new_partially_emitted_expression(&visited_expression);
                        self.emit_context().set_original(&partial, node);
                        partial.set_loc(node.loc);
                        return Some(partial);
                    }
                }
                self.visitor().visit_each_child(node)
            }

            SyntaxKind::JsxSelfClosingElement => {
                let tag_name = self.visitor().visit_node(node.tag_name());
                let attributes = self.visitor().visit_node(node.attributes());
                Some(
                    self.factory()
                        .update_jsx_self_closing_element(node, &tag_name, None, &attributes),
                )
            }

            SyntaxKind::JsxOpeningElement => {
                let tag_name = self.visitor().visit_node(node.tag_name());
                let attributes = self.visitor().visit_node(node.attributes());
                Some(
                    self.factory()
                        .update_jsx_opening_element(node, &tag_name, None, &attributes),
                )
            }

            _ => self.visit_import_export_tail(node),
        }
    }}
