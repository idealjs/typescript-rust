#![allow(unused_imports, dead_code)]

use std::sync::Arc;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::*;
use tsox_frontend::ast::mig::m3b::member_list;
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
use tsox_frontend::ast::mig::m3g_3::{
    node_is_decorated, node_or_child_is_decorated, skip_outer_expressions, OuterExpressionKinds,
};
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::utilities::get_heritage_clause;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::mig::m4h_2::{
    class_has_declared_or_explicitly_assigned_name,
    inject_class_named_evaluation_helper_block_if_missing, is_class_named_evaluation_helper_block,
};
use crate::mig::m4h_4::r36k9_defs::cloned_node_list;
use crate::mig::m4h_5::{ClassInfo, EsDecoratorTransformer, LexicalEntryKind, MemberInfo, MemberInfoMap};
use crate::mig::m4h_6::r36k28_defs::R36K28NodeVisitorExt;
use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::mig::m4l_3::r39k10_defs::R39K10NodeFactoryExt;
use crate::mig::m4m_4::move_range_past_decorators;
use crate::mig::m4n_4::AssignedNameOptions;
use crate::mig::wt1b::r39k05_defs::new_run_initializers_helper_r39k05;
use crate::mig::wt1b_4::class_has_class_this_assignment;
use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags, NodeFactory};

fn new_file_level_generated_name(f: &NodeFactory, text: &str) -> Arc<Node> {
    f.generated_name_node(&f.new_unique_name_ex(
        text,
        AutoGenerateOptions {
            flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
            prefix: String::new(),
            suffix: String::new(),
        },
    ))
}

pub(crate) fn clone_class_info(ci: &ClassInfo) -> ClassInfo {
    let mut member_infos = MemberInfoMap::default();
    for (key, info) in ci.member_infos.iter() {
        member_infos.set(
            key.clone(),
            MemberInfo {
                member_decorators_name: info.member_decorators_name.clone(),
                member_initializers_name: info.member_initializers_name.clone(),
                member_extra_initializers_name: info.member_extra_initializers_name.clone(),
                member_descriptor_name: info.member_descriptor_name.clone(),
            },
        );
    }
    ClassInfo {
        class: ci.class.clone(),
        class_decorators_name: ci.class_decorators_name.clone(),
        class_descriptor_name: ci.class_descriptor_name.clone(),
        class_extra_initializers_name: ci.class_extra_initializers_name.clone(),
        class_this: ci.class_this.clone(),
        class_super: ci.class_super.clone(),
        metadata_reference: ci.metadata_reference.clone(),
        member_infos,
        instance_method_extra_initializers_name: ci.instance_method_extra_initializers_name.clone(),
        static_method_extra_initializers_name: ci.static_method_extra_initializers_name.clone(),
        static_non_field_decoration_statements: ci.static_non_field_decoration_statements.clone(),
        non_static_non_field_decoration_statements: ci
            .non_static_non_field_decoration_statements
            .clone(),
        static_field_decoration_statements: ci.static_field_decoration_statements.clone(),
        non_static_field_decoration_statements: ci.non_static_field_decoration_statements.clone(),
        has_static_initializers: ci.has_static_initializers,
        has_non_ambient_instance_fields: ci.has_non_ambient_instance_fields,
        has_static_private_class_elements: ci.has_static_private_class_elements,
        pending_static_initializers: ci.pending_static_initializers.clone(),
        pending_instance_initializers: ci.pending_instance_initializers.clone(),
    }
}

fn decorators_of(node: &Arc<Node>) -> Vec<Arc<Node>> {
    node.modifier_nodes()
        .iter()
        .filter(|m| is_decorator(m))
        .cloned()
        .collect()
}

fn is_private_identifier_class_element_declaration(member: &Arc<Node>) -> bool {
    (is_property_declaration(member) || is_method_or_accessor(member))
        && member.name().map(|n| is_private_identifier(&n)).unwrap_or(false)
}

fn is_method_or_accessor(member: &Arc<Node>) -> bool {
    is_method_declaration(member)
        || matches!(member.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
}

fn class_or_constructor_parameter_is_decorated(node: &Arc<Node>) -> bool {
    if node_is_decorated(false, node, None, None) {
        return true;
    }
    match get_first_constructor_with_body(node) {
        Some(constructor) => node_or_child_is_decorated(false, &constructor, Some(node), None),
        None => false,
    }
}

fn new_constructor_declaration(f: &NodeFactory, body: &Arc<Node>) -> Arc<Node> {
    let parameters = f.new_node_list(Vec::new());
    Arc::new(Node::new(
        SyntaxKind::Constructor,
        NodeData::ConstructorDeclaration(ndg::ConstructorDeclarationData {
            modifiers: None,
            type_parameters: None,
            parameters: Arc::new(cloned_node_list(&parameters)),
            type_node: None,
            full_signature: None,
            body: Some(body.clone()),
        }),
    ))
}

fn new_immediately_invoked_arrow_function(f: &NodeFactory, statements: Vec<Arc<Node>>) -> Arc<Node> {
    let statements_list = f.new_node_list(statements);
    let body = f.new_block(&statements_list, true);
    let arrow = f.new_arrow_function(
        None,
        None,
        &f.new_node_list(Vec::new()),
        None,
        None,
        &f.new_token(SyntaxKind::EqualsGreaterThanToken),
        &body,
    );
    let parenthesized = f.new_parenthesized_expression(&arrow);
    f.new_call_expression(
        &parenthesized,
        None,
        None,
        f.new_node_list(Vec::new()),
        NodeFlags::empty(),
    )
}

fn inject_class_this_assignment_if_missing(
    ec: &mut crate::printer::EmitContext,
    f: &NodeFactory,
    node: &Arc<Node>,
    class_this: &Arc<Node>,
) -> Arc<Node> {
    if class_has_class_this_assignment(ec, node) {
        return node.clone();
    }

    let expression = f.new_assignment_expression(class_this, &f.new_this_expression());
    let statement = f.new_expression_statement(&expression);
    let statements = f.new_node_list(vec![statement.clone()]);
    let body = f.new_block(&statements, false);
    let static_block = f.new_class_static_block_declaration(None, body);
    ec.set_class_this(&static_block, class_this);

    if let Some(name) = node.name() {
        ec.set_source_map_range(&statement, name.loc);
    }

    let members = member_list(node).expect("expected class members");
    let mut new_members: Vec<Arc<Node>> = Vec::with_capacity(1 + members.nodes.len());
    new_members.push(static_block);
    new_members.extend(members.nodes.iter().cloned());
    let mut members_list = f.new_node_list(new_members);
    if let Some(l) = Arc::get_mut(&mut members_list) {
        l.loc = members.loc;
    }

    let updated_node = match &node.data {
        NodeData::ClassDeclaration(d) => f.update_class_declaration(
            node,
            d.modifiers.clone(),
            d.name.as_ref(),
            None,
            d.heritage_clauses.as_deref(),
            &members_list,
        ),
        NodeData::ClassExpression(d) => f.update_class_expression(
            node,
            d.modifiers.clone(),
            node.name(),
            None,
            d.heritage_clauses.as_deref(),
            &members_list,
        ),
        _ => unreachable!(),
    };
    ec.set_class_this(&updated_node, class_this);
    updated_node
}

impl EsDecoratorTransformer {
    pub fn transform_class_like(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let mut ec = self.transformer.emit_context();
        ec.start_variable_environment();

        let mut node = node.clone();
        if !class_has_declared_or_explicitly_assigned_name(&ec, &node)
            && class_or_constructor_parameter_is_decorated(&node)
        {
            let f = self.transformer.factory();
            let empty_name = f.new_string_literal("", TOKEN_FLAGS_NONE);
            node = inject_class_named_evaluation_helper_block_if_missing(&ec, &node, &empty_name, None);
        }

        let class_reference = {
            let f = self.transformer.factory();
            f.get_local_name_ex(&node, AssignedNameOptions::default())
        };

        let mut ci = self.create_class_info(&node);
        let mut class_definition_statements: Vec<Arc<Node>> = Vec::new();
        let mut leading_block_statements: Vec<Arc<Node>> = Vec::new();
        let mut trailing_block_statements: Vec<Arc<Node>> = Vec::new();
        let mut synthetic_constructor: Option<Arc<Node>> = None;
        let mut heritage_clauses: Option<Arc<NodeList>> = None;
        let mut should_transform_private_static_elements_in_class = false;

        // 1. Class decorators are evaluated outside of the private name scope of the class.
        let class_decorators = self.transform_all_decorators_of_declaration(&decorators_of(&node));
        if !class_decorators.is_empty() {
            let (decorators_array, empty_array, class_this) = {
                let f = self.transformer.factory();
                {
                    let ci_mut =
                        Arc::get_mut(&mut ci).expect("class info uniquely owned before enter_class");
                    ci_mut.class_decorators_name =
                        Some(new_file_level_generated_name(&f, "_classDecorators"));
                    ci_mut.class_descriptor_name =
                        Some(new_file_level_generated_name(&f, "_classDescriptor"));
                    ci_mut.class_extra_initializers_name =
                        Some(new_file_level_generated_name(&f, "_classExtraInitializers"));
                }
                let decorators_nodes = f.new_node_list(class_decorators.clone());
                let decorators_array = f.new_array_literal_expression(&decorators_nodes, false);
                let empty_array = f.new_array_literal_expression(&f.new_node_list(Vec::new()), false);
                let class_this = ci
                    .class_this
                    .clone()
                    .expect("decorated class should have _classThis");
                (decorators_array, empty_array, class_this)
            };
            class_definition_statements.push(
                self.create_let(ci.class_decorators_name.as_ref().unwrap(), Some(decorators_array)),
            );
            class_definition_statements
                .push(self.create_let(ci.class_descriptor_name.as_ref().unwrap(), None));
            class_definition_statements.push(
                self.create_let(ci.class_extra_initializers_name.as_ref().unwrap(), Some(empty_array)),
            );
            class_definition_statements.push(self.create_let(&class_this, None));

            if ci.has_static_private_class_elements {
                should_transform_private_static_elements_in_class = true;
                self.should_transform_private_static_elements_in_file = true;
            }
        }

        // 2. ClassHeritage clause is evaluated outside of the private name scope of the class.
        let extends_clause = get_heritage_clause(&node, SyntaxKind::ExtendsKeyword);
        let mut extends_element: Option<Arc<Node>> = None;
        if let Some(hc) = &extends_clause {
            if let NodeData::HeritageClause(d) = &hc.data {
                if let Some(first) = d.types.nodes.first() {
                    extends_element = Some(first.clone());
                }
            }
        }
        let mut extends_expression: Option<Arc<Node>> = None;
        if let Some(element) = &extends_element {
            let expression = match &element.data {
                NodeData::ExpressionWithTypeArguments(d) => d.expression.clone(),
                _ => unreachable!(),
            };
            extends_expression = Some(self.transformer.visitor().visit_node(&expression));
        }

        if let (Some(extends_element), Some(extends_expression)) = (&extends_element, &extends_expression) {
            let (class_super, safe_extends_expression, updated_extends_clause) = {
                let f = self.transformer.factory();
                let class_super = new_file_level_generated_name(&f, "_classSuper");
                Arc::get_mut(&mut ci)
                    .expect("class info uniquely owned before enter_class")
                    .class_super = Some(class_super.clone());

                let unwrapped = skip_outer_expressions(extends_expression, OuterExpressionKinds::ALL);
                let mut safe_extends_expression = extends_expression.clone();
                if ((unwrapped.kind == SyntaxKind::ClassExpression
                    || unwrapped.kind == SyntaxKind::FunctionExpression)
                    && unwrapped.name().is_none())
                    || unwrapped.kind == SyntaxKind::ArrowFunction
                {
                    let zero = f.new_numeric_literal("0", TokenFlags::default());
                    safe_extends_expression = f.new_comma_expression(&zero, extends_expression);
                }

                let updated_extends_element =
                    f.update_expression_with_type_arguments(extends_element, class_super.clone(), None);
                let hc_token = match &extends_clause.as_ref().unwrap().data {
                    NodeData::HeritageClause(d) => d.token,
                    _ => unreachable!(),
                };
                let updated_types = f.new_node_list(vec![updated_extends_element]);
                let updated_extends_clause =
                    f.update_heritage_clause(extends_clause.as_ref().unwrap(), hc_token, &updated_types);
                (class_super, safe_extends_expression, updated_extends_clause)
            };
            class_definition_statements.push(self.create_let(&class_super, Some(safe_extends_expression)));
            heritage_clauses = Some(self.transformer.factory().new_node_list(vec![updated_extends_clause]));
        }

        let renamed_class_this = match &ci.class_this {
            Some(class_this) => class_this.clone(),
            None => self.transformer.factory().new_this_expression(),
        };

        // 4. For each member: member decorators and computed property names are evaluated in two passes.
        self.enter_class(ci.clone());
        leading_block_statements
            .push(self.create_metadata(&ci.metadata_reference, ci.class_super.as_ref()));

        let members = self
            .non_constructor_class_element_visitor
            .visit_nodes(member_list(&node).unwrap());
        let mut members = self.constructor_class_element_visitor.visit_nodes(&members);

        let mut ci = clone_class_info(self.class_info_stack.as_ref().expect("class info on stack"));

        if !self.pending_expressions.is_empty() {
            self.outer_this = None;
            let pending = std::mem::take(&mut self.pending_expressions);
            let mut outer_this_let: Option<Arc<Node>> = None;
            {
                let f = self.transformer.factory();
                for expr in pending {
                    let mut expr = expr;
                    if expr.subtree_facts().intersects(SubtreeFacts::LexicalThis) {
                        expr = self.outer_this_visitor.visit_node(&expr);
                    }
                    let statement = f.new_expression_statement(&expr);
                    leading_block_statements.push(statement);
                }
                if let Some(outer_this) = self.outer_this.clone() {
                    outer_this_let = Some(f.new_this_expression());
                }
            }
            if let (Some(outer_this), Some(this_expression)) =
                (self.outer_this.clone(), outer_this_let)
            {
                let let_statement = self.create_let(&outer_this, Some(this_expression));
                class_definition_statements.insert(0, let_statement);
            }
        }
        self.exit_class();

        // If there are instance initializers but no constructor, synthesize one.
        if !ci.pending_instance_initializers.is_empty() && get_first_constructor_with_body(&node).is_none()
        {
            let initializer_statements = self.prepare_constructor(&mut ci);
            if !initializer_statements.is_empty() {
                let is_derived_class = extends_element
                    .as_ref()
                    .map(|element| {
                        let expression = match &element.data {
                            NodeData::ExpressionWithTypeArguments(d) => d.expression.clone(),
                            _ => unreachable!(),
                        };
                        skip_outer_expressions(&expression, OuterExpressionKinds::ALL).kind
                            != SyntaxKind::NullKeyword
                    })
                    .unwrap_or(false);
                let constructor_body = {
                    let f = self.transformer.factory();
                    let mut constructor_statements: Vec<Arc<Node>> = Vec::new();
                    if is_derived_class {
                        let spread_arguments = f.new_spread_element(f.new_identifier("arguments"));
                        let super_call = f.new_call_expression(
                            &f.new_keyword_expression(SyntaxKind::SuperKeyword),
                            None,
                            None,
                            f.new_node_list(vec![spread_arguments]),
                            NodeFlags::empty(),
                        );
                        constructor_statements.push(f.new_expression_statement(&super_call));
                    }
                    constructor_statements.extend_from_slice(&initializer_statements);
                    let statements = f.new_node_list(constructor_statements);
                    f.new_block(&statements, true)
                };
                synthetic_constructor = Some({
                    let f = self.transformer.factory();
                    new_constructor_declaration(&f, &constructor_body)
                });
            }
        }

        // Used in class definition steps 5,7,11 and 6,8.
        let (static_extra_empty_array, instance_extra_empty_array) = {
            let f = self.transformer.factory();
            let build = || f.new_array_literal_expression(&f.new_node_list(Vec::new()), false);
            (
                ci.static_method_extra_initializers_name
                    .clone()
                    .map(|_| build()),
                ci.instance_method_extra_initializers_name
                    .clone()
                    .map(|_| build()),
            )
        };
        if let (Some(name), Some(empty_array)) = (
            ci.static_method_extra_initializers_name.clone(),
            static_extra_empty_array,
        ) {
            class_definition_statements.push(self.create_let(&name, Some(empty_array)));
        }
        if let (Some(name), Some(empty_array)) = (
            ci.instance_method_extra_initializers_name.clone(),
            instance_extra_empty_array,
        ) {
            class_definition_statements.push(self.create_let(&name, Some(empty_array)));
        }

        // Emit member info variable declarations; static member vars first, then non-static.
        if ci.member_infos.iter().count() > 0 {
            class_definition_statements.extend(self.emit_member_info_declarations(&mut ci, true));
            class_definition_statements.extend(self.emit_member_info_declarations(&mut ci, false));
        }

        // 5-8. Element decorators are applied.
        leading_block_statements.extend(ci.static_non_field_decoration_statements.iter().cloned());
        leading_block_statements.extend(ci.non_static_non_field_decoration_statements.iter().cloned());
        leading_block_statements.extend(ci.static_field_decoration_statements.iter().cloned());
        leading_block_statements.extend(ci.non_static_field_decoration_statements.iter().cloned());

        // 9-10. Class decorators are applied and the class binding is initialized.
        if ci.class_descriptor_name.is_some()
            && ci.class_decorators_name.is_some()
            && ci.class_extra_initializers_name.is_some()
            && ci.class_this.is_some()
        {
            let es_decorate_statement = {
                let f = self.transformer.factory();
                let value_property = f.new_property_assignment(
                    None,
                    &f.new_identifier("value"),
                    None,
                    None,
                    &renamed_class_this,
                );
                let properties = f.new_node_list(vec![value_property]);
                let class_descriptor = f.new_object_literal_expression(&properties, false);
                let class_descriptor_assignment = f
                    .new_assignment_expression(ci.class_descriptor_name.as_ref().unwrap(), &class_descriptor);
                let class_name_reference = f.new_property_access_expression(
                    &renamed_class_this,
                    None,
                    &f.new_identifier("name"),
                    NodeFlags::empty(),
                );
                let context_obj = f
                    .new_es_decorate_class_context_object(&class_name_reference, &ci.metadata_reference);
                let null_token = f.new_token(SyntaxKind::NullKeyword);
                let es_decorate_helper = f.new_es_decorate_helper(
                    &null_token,
                    &class_descriptor_assignment,
                    ci.class_decorators_name.as_ref().unwrap(),
                    &context_obj,
                    &null_token,
                    ci.class_extra_initializers_name.as_ref().unwrap(),
                );
                f.new_expression_statement(&es_decorate_helper)
            };
            ec.set_source_map_range(&es_decorate_statement, move_range_past_decorators(&node));
            leading_block_statements.push(es_decorate_statement);

            let class_reference_assignment = {
                let f = self.transformer.factory();
                let class_descriptor_value_ref = f.new_property_access_expression(
                    ci.class_descriptor_name.as_ref().unwrap(),
                    None,
                    &f.new_identifier("value"),
                    NodeFlags::empty(),
                );
                let class_this_assignment = f
                    .new_assignment_expression(ci.class_this.as_ref().unwrap(), &class_descriptor_value_ref);
                f.new_assignment_expression(&class_reference, &class_this_assignment)
            };
            let statement = self.transformer.factory().new_expression_statement(&class_reference_assignment);
            leading_block_statements.push(statement);
        }

        leading_block_statements.push(self.create_symbol_metadata(&renamed_class_this, &ci.metadata_reference));

        // 11-12. Static extra initializers and static fields are initialized.
        if !ci.pending_static_initializers.is_empty() {
            let pending_static = std::mem::take(&mut ci.pending_static_initializers);
            let statements = {
                let f = self.transformer.factory();
                let mut statements: Vec<Arc<Node>> = Vec::new();
                for initializer in pending_static {
                    let initializer_statement = f.new_expression_statement(&initializer);
                    ec.set_source_map_range(&initializer_statement, ec.source_map_range(&initializer));
                    statements.push(initializer_statement);
                }
                statements
            };
            trailing_block_statements.extend(statements);
        }

        // 13. Class extra initializers.
        if let Some(class_extra_initializers_name) = ci.class_extra_initializers_name.clone() {
            let run_class_initializers_statement = {
                let f = self.transformer.factory();
                let helper = new_run_initializers_helper_r39k05(&f, &renamed_class_this, &class_extra_initializers_name);
                f.new_expression_statement(&helper)
            };
            match node.name() {
                Some(name) => ec.set_source_map_range(&run_class_initializers_statement, name.loc),
                None => ec.set_source_map_range(
                    &run_class_initializers_statement,
                    move_range_past_decorators(&node),
                ),
            }
            trailing_block_statements.push(run_class_initializers_statement);
        }

        if !leading_block_statements.is_empty()
            && !trailing_block_statements.is_empty()
            && !ci.has_static_initializers
        {
            leading_block_statements.extend(trailing_block_statements.drain(..));
        }

        let mut leading_static_block: Option<Arc<Node>> = None;
        if !leading_block_statements.is_empty() {
            leading_static_block = Some({
                let f = self.transformer.factory();
                let statements = f.new_node_list(leading_block_statements);
                let body = f.new_block(&statements, true);
                f.new_class_static_block_declaration(None, body)
            });
        }
        if let Some(leading_static_block) = &leading_static_block {
            if should_transform_private_static_elements_in_class {
                ec.add_emit_flags(leading_static_block, EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS);
            }
        }

        let mut trailing_static_block: Option<Arc<Node>> = None;
        if !trailing_block_statements.is_empty() {
            trailing_static_block = Some({
                let f = self.transformer.factory();
                let statements = f.new_node_list(trailing_block_statements);
                let body = f.new_block(&statements, true);
                f.new_class_static_block_declaration(None, body)
            });
        }

        if leading_static_block.is_some() || synthetic_constructor.is_some() || trailing_static_block.is_some()
        {
            let members_list = {
                let f = self.transformer.factory();
                let mut new_members: Vec<Arc<Node>> = Vec::with_capacity(members.nodes.len() + 3);

                let mut existing_named_evaluation_helper_block_index = -1_i64;
                for (i, m) in members.nodes.iter().enumerate() {
                    if is_class_named_evaluation_helper_block(&ec, m) {
                        existing_named_evaluation_helper_block_index = i as i64;
                        break;
                    }
                }

                match &leading_static_block {
                    Some(leading_static_block) => {
                        let split = (existing_named_evaluation_helper_block_index + 1) as usize;
                        new_members.extend(members.nodes[..split].iter().cloned());
                        new_members.push(leading_static_block.clone());
                        new_members.extend(members.nodes[split..].iter().cloned());
                    }
                    None => new_members.extend(members.nodes.iter().cloned()),
                }

                if let Some(synthetic_constructor) = &synthetic_constructor {
                    new_members.push(synthetic_constructor.clone());
                }

                if let Some(trailing_static_block) = &trailing_static_block {
                    new_members.push(trailing_static_block.clone());
                }

                let mut members_list = f.new_node_list(new_members);
                if let Some(l) = Arc::get_mut(&mut members_list) {
                    l.loc = members.loc;
                }
                members_list
            };
            members = members_list;
        }

        let lexical_environment = ec.end_variable_environment();

        let class_expression = if !class_decorators.is_empty() {
            let mut class_expression = {
                let f = self.transformer.factory();
                f.new_class_expression(None, None, None, heritage_clauses.clone(), members.clone())
            };
            ec.set_original(&class_expression, &node);
            if let Some(class_this) = ci.class_this.clone() {
                class_expression = {
                    let f = self.transformer.factory();
                    inject_class_this_assignment_if_missing(&mut ec, &f, &class_expression, &class_this)
                };
            }

            // We use `var` instead of `let` so we can leverage NamedEvaluation to define the class name.
            let (reference_declaration, return_expr) = {
                let f = self.transformer.factory();
                let reference_declaration =
                    f.new_variable_declaration(&class_reference, None, None, Some(&class_expression));
                let return_expr = match &ci.class_this {
                    Some(class_this) => f.new_assignment_expression(&class_reference, class_this),
                    None => class_reference.clone(),
                };
                (reference_declaration, return_expr)
            };
            let reference_var_decl_list = {
                let f = self.transformer.factory();
                f.new_variable_declaration_list(
                    &f.new_node_list(vec![reference_declaration]),
                    NodeFlags::empty(),
                )
            };
            let return_statement = {
                let f = self.transformer.factory();
                f.new_return_statement(Some(&return_expr))
            };
            class_definition_statements.push({
                let f = self.transformer.factory();
                f.new_variable_statement(None, &reference_var_decl_list)
            });
            class_definition_statements.push(return_statement);
            class_expression
        } else {
            let class_expression = {
                let f = self.transformer.factory();
                f.new_class_expression(None, node.name().cloned(), None, heritage_clauses.clone(), members.clone())
            };
            ec.set_original(&class_expression, &node);
            let return_statement = {
                let f = self.transformer.factory();
                f.new_return_statement(Some(&class_expression))
            };
            class_definition_statements.push(return_statement);
            class_expression
        };

        if should_transform_private_static_elements_in_class {
            ec.add_emit_flags(&class_expression, EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS);
            if let Some(class_member_list) = member_list(&class_expression) {
                for member in &class_member_list.nodes {
                    if (is_private_identifier_class_element_declaration(member)
                        || is_auto_accessor_property_declaration(member))
                        && has_static_modifier(member)
                    {
                        ec.add_emit_flags(member, EmitFlags::TRANSFORM_PRIVATE_STATIC_ELEMENTS);
                    }
                }
            }
        }

        let merged_statements = ec.merge_environment(class_definition_statements, lexical_environment);
        let f = self.transformer.factory();
        new_immediately_invoked_arrow_function(&f, merged_statements)
    }

    pub(crate) fn store_class_info(&mut self, updated: ClassInfo) {
        let arc = Arc::new(updated);
        let mut cursor = self.top.as_deref_mut();
        while let Some(entry) = cursor {
            if entry.kind == LexicalEntryKind::Class {
                entry.class_info_data = Some(arc.clone());
                break;
            }
            cursor = entry.next.as_deref_mut();
        }
        self.class_info_stack = Some(arc);
    }
}
