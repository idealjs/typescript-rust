#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::{Node, ModifierList};
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::is_assignment_expression;
use tsox_frontend::format::mig::m4o::NodeFactory;

use crate::mig::m4g::r33k7_defs::{
    binary_left, computed_property_name_expression, parenthesized_expression, PrivateIdentifierKind,
};
use crate::mig::m4g::{ClassFieldsTransformer, ClassFacts, PrivateIdentifierInfo};
use crate::mig::m4g_2::r37k13_defs::{ClassFieldsTransformerR37k13, K13Visitor, NodeFactoryR37k13};
use crate::mig::w11b::should_be_captured_in_temp_variable;
use crate::mig::m4j_2::extract_modifiers;

pub fn k15_m3c_visitor() -> tsox_frontend::ast::mig::m3c::NodeVisitor { ::tsox_core::fntrace::enter("k15_m3c_visitor"); 
    tsox_frontend::ast::mig::m3c::NodeVisitor {
        factory: tsox_frontend::ast::mig::m3c::NodeFactory {
            hooks: tsox_frontend::ast::mig::m3c::NodeFactoryHooks::default(),
            text_count: 0,
            node_count: 0,
        },
    }
}

fn kind_to_str(kind: &PrivateIdentifierKind) -> &'static str { ::tsox_core::fntrace::enter("kind_to_str"); 
    match kind {
        PrivateIdentifierKind::Field => "f",
        PrivateIdentifierKind::Method => "m",
        PrivateIdentifierKind::Accessor => "a",
        PrivateIdentifierKind::Untransformed => "u",
    }
}

pub fn new_class_private_field_get_helper_k15(
    f: &NodeFactory,
    receiver: &Arc<Node>,
    state: &Arc<Node>,
    kind: &PrivateIdentifierKind,
    fn_: Option<&Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_class_private_field_get_helper_k15"); 
    let mut args = vec![
        receiver.clone(),
        state.clone(),
        f.new_string_literal(kind_to_str(kind), 0),
    ];
    if let Some(fn_) = fn_ {
        args.push(fn_.clone());
    }
    f.new_call_expression(
        &f.new_unscoped_helper_name("__classPrivateFieldGet"),
        None,
        None,
        f.new_node_list(&args),
        NodeFlags::empty(),
    )
}

pub fn new_class_private_field_set_helper_k15(
    f: &NodeFactory,
    receiver: &Arc<Node>,
    state: &Arc<Node>,
    value: &Arc<Node>,
    kind: &PrivateIdentifierKind,
    fn_: Option<&Arc<Node>>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_class_private_field_set_helper_k15"); 
    let mut args = vec![
        receiver.clone(),
        state.clone(),
        value.clone(),
        f.new_string_literal(kind_to_str(kind), 0),
    ];
    if let Some(fn_) = fn_ {
        args.push(fn_.clone());
    }
    f.new_call_expression(
        &f.new_unscoped_helper_name("__classPrivateFieldSet"),
        None,
        None,
        f.new_node_list(&args),
        NodeFlags::empty(),
    )
}

pub trait NodeFactoryR39k15 {
    fn new_assignment_target_wrapper(&self, name: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node>;
    fn new_class_private_field_in_helper(&self, name: &Arc<Node>, receiver: &Arc<Node>) -> Arc<Node>;
    fn new_syntax_list(&self, nodes: Vec<Arc<Node>>) -> Arc<Node>;
    fn new_expression_statement(&self, expression: &Arc<Node>) -> Arc<Node>;
    fn new_function_expression(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Option<Arc<NodeList>>,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_assignment_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node>;
    fn new_class_static_block_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        body: &Arc<Node>,
    ) -> Arc<Node>;
    fn new_comma_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node>;
    fn inline_expressions(&self, expressions: Vec<Arc<Node>>) -> Arc<Node>;
    fn new_function_bind_call(
        &self,
        target: &Arc<Node>,
        this_arg: &Arc<Node>,
        arguments: Option<Vec<Arc<Node>>>,
    ) -> Arc<Node>;
    fn update_property_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn update_property_access_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        flags: NodeFlags,
    ) -> Arc<Node>;
    fn update_call_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
        arguments: Arc<NodeList>,
        flags: NodeFlags,
    ) -> Arc<Node>;
    fn update_tagged_template_expression(
        &self,
        node: &Arc<Node>,
        tag: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
        template: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node>;
    fn update_binary_expression(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        left: Arc<Node>,
        type_node: Option<Arc<Node>>,
        operator_token: Arc<Node>,
        right: Arc<Node>,
    ) -> Arc<Node>;
    fn update_for_statement(
        &self,
        node: &Arc<Node>,
        initializer: Option<Arc<Node>>,
        condition: Option<Arc<Node>>,
        incrementor: Option<Arc<Node>>,
        statement: &Arc<Node>,
    ) -> Arc<Node>;
    fn update_expression_statement(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node>;
    fn update_computed_property_name(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node>;
}

fn property_declaration_data(
    node: &Arc<Node>,
    modifiers: Option<Arc<ModifierList>>,
    name: Option<Arc<Node>>,
    postfix_token: Option<Arc<Node>>,
    type_node: Option<Arc<Node>>,
    initializer: Option<Arc<Node>>,
) -> ndg::PropertyDeclarationData { ::tsox_core::fntrace::enter("property_declaration_data"); 
    let old = match &node.data {
        NodeData::PropertyDeclaration(d) => d,
        _ => panic!("expected PropertyDeclaration"),
    };
    ndg::PropertyDeclarationData {
        modifiers,
        name: name.unwrap_or_else(|| old.name.clone()),
        postfix_token,
        type_node,
        initializer,
    }
}

impl NodeFactoryR39k15 for NodeFactory {
    fn new_assignment_target_wrapper(&self, name: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_assignment_target_wrapper"); 
        let assignment = self.new_binary_expression(
            None,
            name,
            None,
            self.new_token(SyntaxKind::EqualsToken),
            expression,
        );
        self.new_parenthesized_expression(&assignment)
    }

    fn new_class_private_field_in_helper(&self, name: &Arc<Node>, receiver: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_class_private_field_in_helper"); 
        self.new_call_expression(
            &self.new_unscoped_helper_name("__classPrivateFieldIn"),
            None,
            None,
            self.new_node_list(&[name.clone(), receiver.clone()]),
            NodeFlags::empty(),
        )
    }

    fn new_syntax_list(&self, nodes: Vec<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_syntax_list"); 
        Arc::new(Node::new(
            SyntaxKind::SyntaxList,
            NodeData::SyntaxList(ndg::SyntaxListData { children: nodes }),
        ))
    }

    fn new_expression_statement(&self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_expression_statement"); 
        Arc::new(Node::new(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ndg::ExpressionStatementData {
                expression: expression.clone(),
            }),
        ))
    }

    fn new_function_expression(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Option<Arc<NodeList>>,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_function_expression"); 
        Arc::new(Node::new(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(ndg::FunctionExpressionData {
                modifiers,
                asterisk_token,
                name,
                type_parameters,
                parameters: parameters.expect("expected ParameterList"),
                type_node,
                full_signature,
                body: body.expect("expected function body"),
            }),
        ))
    }

    fn new_assignment_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_assignment_expression"); 
        self.new_binary_expression(None, left, None, self.new_token(SyntaxKind::EqualsToken), right)
    }

    fn new_class_static_block_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        body: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_class_static_block_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::ClassStaticBlockDeclaration,
            NodeData::ClassStaticBlockDeclaration(ndg::ClassStaticBlockDeclarationData {
                modifiers,
                body: body.clone(),
            }),
        ))
    }

    fn new_comma_expression(&self, left: &Arc<Node>, right: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_comma_expression"); 
        self.new_binary_expression(None, left, None, self.new_token(SyntaxKind::CommaToken), right)
    }

    fn inline_expressions(&self, expressions: Vec<Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("inline_expressions"); 
        let mut iter = expressions.into_iter();
        let mut expression = iter.next().expect("expected at least one expression");
        for next in iter {
            expression = self.new_comma_expression(&expression, &next);
        }
        expression
    }

    fn new_function_bind_call(
        &self,
        target: &Arc<Node>,
        this_arg: &Arc<Node>,
        arguments: Option<Vec<Arc<Node>>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_function_bind_call"); 
        let mut args = vec![this_arg.clone()];
        args.extend(arguments.unwrap_or_default());
        self.new_method_call(target, &self.new_identifier("bind"), &args)
    }

    fn update_property_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        postfix_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_property_declaration"); 
        let mut updated = Node::new(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(property_declaration_data(
                node, modifiers, name, postfix_token, type_node, initializer,
            )),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_property_access_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        name: Option<Arc<Node>>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_property_access_expression"); 
        let old = match &node.data {
            NodeData::PropertyAccessExpression(d) => d,
            _ => panic!("expected PropertyAccessExpression"),
        };
        let mut updated = Node::new(
            SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAccessExpression(ndg::PropertyAccessExpressionData {
                expression: expression.clone(),
                question_dot_token,
                name: name.unwrap_or_else(|| old.name.clone()),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }

    fn update_call_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
        arguments: Arc<NodeList>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_call_expression"); 
        let mut updated = Node::new(
            SyntaxKind::CallExpression,
            NodeData::CallExpression(ndg::CallExpressionData {
                expression: expression.clone(),
                question_dot_token,
                type_arguments,
                arguments,
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }

    fn update_tagged_template_expression(
        &self,
        node: &Arc<Node>,
        tag: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
        template: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_tagged_template_expression"); 
        let mut updated = Node::new(
            SyntaxKind::TaggedTemplateExpression,
            NodeData::TaggedTemplateExpression(ndg::TaggedTemplateExpressionData {
                tag: tag.clone(),
                question_dot_token,
                type_arguments,
                template: template.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }

    fn update_binary_expression(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        left: Arc<Node>,
        type_node: Option<Arc<Node>>,
        operator_token: Arc<Node>,
        right: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_binary_expression"); 
        let mut updated = Node::new(
            SyntaxKind::BinaryExpression,
            NodeData::BinaryExpression(ndg::BinaryExpressionData {
                modifiers,
                left,
                type_node,
                operator_token,
                right,
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_for_statement(
        &self,
        node: &Arc<Node>,
        initializer: Option<Arc<Node>>,
        condition: Option<Arc<Node>>,
        incrementor: Option<Arc<Node>>,
        statement: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_for_statement"); 
        let mut updated = Node::new(
            SyntaxKind::ForStatement,
            NodeData::ForStatement(ndg::ForStatementData {
                initializer,
                condition,
                incrementor,
                statement: statement.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_expression_statement(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("update_expression_statement"); 
        let mut updated = Node::new(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ndg::ExpressionStatementData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_computed_property_name(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("update_computed_property_name"); 
        let mut updated = Node::new(
            SyntaxKind::ComputedPropertyName,
            NodeData::ComputedPropertyName(ndg::ComputedPropertyNameData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }
}

pub trait K13VisitorR39k15 {
    fn visit_nodes_list(&mut self, nodes: &Arc<NodeList>) -> Option<Arc<NodeList>>;
    fn visit_modifiers_list(
        &mut self,
        modifiers: Option<&Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>>;
}

impl K13VisitorR39k15 for K13Visitor {
    fn visit_nodes_list(&mut self, nodes: &Arc<NodeList>) -> Option<Arc<NodeList>> { ::tsox_core::fntrace::enter("visit_nodes_list"); 
        let visited: Vec<Arc<Node>> = nodes.nodes.iter().map(|n| self.visit_each_child(n)).collect();
        Some(Arc::new(NodeList::new(visited)))
    }

    fn visit_modifiers_list(
        &mut self,
        modifiers: Option<&Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("visit_modifiers_list"); 
        let modifiers = modifiers?;
        let visited: Vec<Arc<Node>> =
            modifiers.list.nodes.iter().map(|m| self.visit_each_child(m)).collect();
        Some(Arc::new(ModifierList::new(visited, modifiers.modifier_flags)))
    }
}

pub trait ClassFieldsTransformerR39k15 {
    fn get_hoisted_function_name(&mut self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn extract_non_static_non_accessor_modifiers(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<ModifierList>>;
    fn create_call_binding(&mut self, node: &Arc<Node>) -> (Arc<Node>, Arc<Node>);
    fn create_private_identifier_access(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
    ) -> Arc<Node>;
    fn create_private_identifier_access_helper(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
    ) -> Arc<Node>;
    fn create_private_identifier_assignment(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
        right: &Arc<Node>,
        operator: SyntaxKind,
    ) -> Arc<Node>;
    fn inject_pending_expressions(&mut self, expression: &Arc<Node>) -> Arc<Node>;
    fn visit_this_expression_k15(&mut self, node: &Arc<Node>) -> Arc<Node>;
    fn get_property_name_expression_if_needed(
        &mut self,
        name: &Arc<Node>,
        should_hoist: bool,
    ) -> Option<Arc<Node>>;
    fn transform_property_or_class_static_block(
        &mut self,
        property: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Option<Arc<Node>>;
}

impl ClassFieldsTransformerR39k15 for ClassFieldsTransformer {
    fn get_hoisted_function_name(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_hoisted_function_name"); 
        let name = node.name();
        debug_assert!(name.is_some() && tsox_frontend::ast::is_private_identifier(name.unwrap()));
        let info = self.access_private_identifier(name?)?;
        match info.kind {
            PrivateIdentifierKind::Method => info.method_name,
            PrivateIdentifierKind::Accessor => {
                if ndg::is_get_accessor_declaration(node) {
                    info.getter_name
                } else if ndg::is_set_accessor_declaration(node) {
                    info.setter_name
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn extract_non_static_non_accessor_modifiers(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("extract_non_static_non_accessor_modifiers"); 
        let ec = Arc::new(crate::printer::EmitContext::new());
        extract_modifiers(
            &ec,
            node.modifiers().map(|m| m.as_ref()),
            !(tsox_frontend::ast::ModifierFlags::Static
                | tsox_frontend::ast::ModifierFlags::Accessor),
        )
        .map(Arc::new)
    }

    fn create_call_binding(&mut self, node: &Arc<Node>) -> (Arc<Node>, Arc<Node>) { ::tsox_core::fntrace::enter("create_call_binding"); 
        if tsox_frontend::ast::mig::m3g_2::is_super_property(node) {
            return (self.factory().new_this_expression(), node.clone());
        }
        if tsox_frontend::ast::node_data_generated::is_property_access_expression(node) {
            let expression = node.expression().expect("property access requires expression");
            if should_be_captured_in_temp_variable(&expression) {
                let this_arg = self.factory().new_temp_variable_r37k13();
                self.emit_context().add_variable_declaration(&this_arg);
                let target = self.factory().new_property_access_expression(
                    &self.factory().new_parenthesized_expression(&self.factory().new_assignment_expression(&this_arg, &expression)),
                    None,
                    node.name().expect("property access requires name"),
                    node.flags,
                );
                return (this_arg, target);
            }
            return (
                expression.clone(),
                self.factory().new_property_access_expression(
                    &expression,
                    None,
                    node.name().expect("property access requires name"),
                    node.flags,
                ),
            );
        }
        (self.factory().new_this_expression(), node.clone())
    }

    fn create_private_identifier_access(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_identifier_access"); 
        let receiver = self.visitor().visit_node(receiver);
        self.create_private_identifier_access_helper(info, &receiver)
    }

    fn create_private_identifier_access_helper(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_identifier_access_helper"); 
        self.emit_context().set_comment_range(
            receiver,
            tsox_core::core::text::TextRange::new(0, receiver.end()),
        );
        let name = match info.kind {
            PrivateIdentifierKind::Accessor => info.getter_name.clone(),
            PrivateIdentifierKind::Method => info.method_name.clone(),
            PrivateIdentifierKind::Field => {
                if info.is_static {
                    info.variable_name.clone()
                } else {
                    None
                }
            }
            PrivateIdentifierKind::Untransformed => None,
        };
        new_class_private_field_get_helper_k15(
            &self.factory(),
            receiver,
            &info.brand_check_identifier.clone().unwrap(),
            &info.kind,
            name.as_ref(),
        )
    }

    fn create_private_identifier_assignment(
        &mut self,
        info: &PrivateIdentifierInfo,
        receiver: &Arc<Node>,
        right: &Arc<Node>,
        operator: SyntaxKind,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_private_identifier_assignment"); 
        let mut receiver = self.visitor().visit_node(receiver);
        let mut right = self.visitor().visit_node(right);

        if tsox_frontend::ast::utilities::is_compound_assignment(operator) {
            let f = self.factory();
            let read_expression = f.new_temp_variable_r37k13();
            self.emit_context().add_variable_declaration(&read_expression);
            let initialize_expression = f.new_assignment_expression(&read_expression, &receiver);
            receiver = read_expression.clone();
            right = f.new_binary_expression(
                None,
                &self.create_private_identifier_access_helper(info, &read_expression),
                None,
                f.new_token(crate::mig::m4m_4::get_non_assignment_operator_for_compound_assignment(operator)),
                &right,
            );
        }

        self.emit_context().set_comment_range(
            &receiver,
            tsox_core::core::text::TextRange::new(0, receiver.end()),
        );

        new_class_private_field_set_helper_k15(
            &self.factory(),
            &receiver,
            &info.brand_check_identifier.clone().unwrap(),
            &right,
            &info.kind,
            info.setter_name.as_ref(),
        )
    }

    fn inject_pending_expressions(&mut self, expression: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("inject_pending_expressions"); 
        if self.pending_expressions.is_empty() {
            return expression.clone();
        }
        let f = self.factory();
        if tsox_frontend::ast::node_data_generated::is_parenthesized_expression(expression) {
            let inner = parenthesized_expression(expression).clone();
            let mut exprs = std::mem::take(&mut self.pending_expressions);
            exprs.push(inner);
            let inlined = f.inline_expressions(exprs);
            f.update_parenthesized_expression(expression, &inlined)
        } else {
            let mut exprs = std::mem::take(&mut self.pending_expressions);
            exprs.push(expression.clone());
            f.inline_expressions(exprs)
        }
    }

    fn visit_this_expression_k15(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_this_expression_k15"); 
        let data = self.lexical_environment.as_ref().and_then(|env| env.data.as_ref());
        if let Some(data) = data {
            if self.inside_computed_property_name
                && self.should_transform_this_in_static_initializers
                && (!data.facts.contains(ClassFacts::ClassWasDecorated) || self.legacy_decorators)
            {
                if let Some(class_this) = self.try_get_class_this_no_container() {
                    return class_this;
                }
            }
        }
        if self.should_transform_this_in_static_initializers && self.current_class_element.is_some()
        {
            let element = self.current_class_element.as_ref().unwrap();
            let is_static_block = element.kind == SyntaxKind::ClassStaticBlockDeclaration;
            let is_static_property = element.kind == SyntaxKind::PropertyDeclaration
                && tsox_frontend::ast::has_static_modifier(element);
            if (is_static_block || is_static_property) && data.is_some() {
                if let Some(class_this) = self.try_get_class_this_no_container() {
                    return class_this;
                }
                if let Some(data) = data {
                    if data.facts.contains(ClassFacts::ClassWasDecorated)
                        && self.legacy_decorators
                    {
                        return self.factory().new_parenthesized_expression(
                            &self.factory().new_void_zero_expression(),
                        );
                    }
                }
            }
        }
        node.clone()
    }

    fn get_property_name_expression_if_needed(
        &mut self,
        name: &Arc<Node>,
        should_hoist: bool,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_property_name_expression_if_needed"); 
        if !tsox_frontend::ast::is_computed_property_name(name) {
            return None;
        }
        let cache_assignment = find_computed_property_name_cache_assignment_k15(name);
        let saved_lexical_environment = self.lexical_environment.take();
        let saved_inside_computed_property_name = self.inside_computed_property_name;
        self.inside_computed_property_name = true;
        if let Some(env) = &saved_lexical_environment {
            if env.previous.is_some() {
                self.lexical_environment = env.previous.clone().map(|b| *b);
            }
        }
        let expression =
            self.visitor().visit_node(computed_property_name_expression(name));
        self.lexical_environment = saved_lexical_environment;
        self.inside_computed_property_name = saved_inside_computed_property_name;
        let inner_expression = tsox_frontend::ast::mig::m3g_3::skip_outer_expressions(
            &expression,
            tsox_frontend::ast::mig::m3g_3::OuterExpressionKinds::PARTIALLY_EMITTED_EXPRESSIONS,
        );
        let inlinable = crate::mig::m4m_2::is_simple_inlineable_expression(&inner_expression);
        let already_transformed = cache_assignment.is_some()
            || (is_assignment_expression(&inner_expression, true)
                && tsox_frontend::ast::node_data_generated::is_identifier(binary_left(&inner_expression)));
        if !already_transformed && !inlinable && should_hoist {
            let generated_name = self.factory().new_generated_name_for_node(name);
            if self.requires_block_scoped_var() {
                self.emit_context().add_lexical_declaration(&generated_name);
            } else {
                self.emit_context().add_variable_declaration(&generated_name);
            }
            return Some(self.factory().new_assignment_expression(&generated_name, &expression));
        }
        if inlinable || tsox_frontend::ast::node_data_generated::is_identifier(&inner_expression) {
            return None;
        }
        Some(expression)
    }

    fn transform_property_or_class_static_block(
        &mut self,
        property: &Arc<Node>,
        receiver: &Arc<Node>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_property_or_class_static_block"); 
        let expression = if property.kind == SyntaxKind::ClassStaticBlockDeclaration {
            let saved = self.current_class_element.replace(property.clone());
            let visited = self.class_element_visitor().visit_each_child(property);
            self.current_class_element = saved;
            visited
        } else {
            self.transform_property(property, receiver)?
        };
        let statement = self.factory().new_expression_statement(&expression);
        let property_flags = self.emit_context().emit_flags(property);
        self.emit_context().set_original(&statement, property);
        self.emit_context()
            .add_emit_flags(&statement, property_flags & tsox_frontend::format::mig::m4o::EmitFlags::NO_COMMENTS);
        self.emit_context().set_comment_range(&statement, property.loc);
        let property_original_node = self.emit_context().most_original(property);
        if property_original_node.kind == SyntaxKind::Parameter {
            self.emit_context()
                .set_source_map_range(&statement, property_original_node.loc);
            self.emit_context().add_emit_flags(&statement, tsox_frontend::format::mig::m4o::EmitFlags::NO_COMMENTS);
        } else {
            self.emit_context()
                .set_source_map_range(&statement, crate::mig::m4m_4::move_range_past_modifiers(property));
        }
        Some(statement)
    }
}

fn find_computed_property_name_cache_assignment_k15(name: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_computed_property_name_cache_assignment_k15"); 
    let mut node = Arc::clone(name.expression().expect("computed property name requires expression"));
    loop {
        node = tsox_frontend::ast::mig::m3g_3::skip_outer_expressions(
            &node,
            tsox_frontend::ast::mig::m3g_3::OuterExpressionKinds::empty(),
        );
        if node.kind == SyntaxKind::BinaryExpression
            && binary_operator_token_kind(&node) == SyntaxKind::CommaToken
        {
            node = binary_left(&node).clone();
            continue;
        }
        if is_assignment_expression(&node, true)
            && tsox_frontend::ast::node_data_generated::is_identifier(binary_left(&node))
        {
            return Some(node);
        }
        break;
    }
    None
}

fn binary_operator_token_kind(node: &Node) -> SyntaxKind { ::tsox_core::fntrace::enter("binary_operator_token_kind"); 
    match &node.data {
        NodeData::BinaryExpression(d) => d.operator_token.kind,
        _ => panic!("expected BinaryExpression"),
    }
}
