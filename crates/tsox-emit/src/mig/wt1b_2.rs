#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::mig::m3b::{members, parameter_list};
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
use tsox_frontend::ast::mig::m3g_3::{
    node_is_decorated, node_or_child_is_decorated, skip_outer_expressions, OuterExpressionKinds,
};
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_class_expression, is_class_static_block_declaration, is_constructor_declaration,
    is_get_accessor_declaration, is_identifier, is_private_identifier, is_property_declaration,
    is_set_accessor_declaration, is_string_literal,
};
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::node_source_file::LanguageVariant;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{
    has_static_modifier, has_syntactic_modifier, is_class_like, is_method_or_accessor, is_static,
    modifiers_to_flags,
};
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::mig::m3i::is_identifier_text;

use crate::printer::{AutoGenerateOptions, GeneratedIdentifierFlags};

use super::m4g::r33k7_defs::has_decorators;
use crate::mig::m4q::r33k12_defs::is_private_identifier_class_element_declaration;
use super::m4h_2::is_class_named_evaluation_helper_block;
use super::m4h_5::{
    ClassInfo, EsDecoratorTransformer, LexicalEntry, LexicalEntryKind, MemberInfo,
};
use super::wt1b::r39k05_defs::{
    new_function_call_call_r39k05, new_get_accessor_declaration_full_r39k05,
    new_run_initializers_helper_r39k05, new_set_function_name_helper_r39k05,
    new_string_literal_from_node_r39k05, R39k05NodeVisitorExt,
};
use super::m4m_2::{is_generated_identifier, move_range_past_decorators};
use super::m4m_4::move_range_past_decorators as move_range_past_decorators_2;

fn class_or_constructor_parameter_is_decorated(
    use_legacy_decorators: bool,
    node: &Arc<Node>,
) -> bool {
    if node_is_decorated(use_legacy_decorators, node, None, None) {
        return true;
    }
    match get_first_constructor_with_body(node) {
        Some(constructor) => {
            node_or_child_is_decorated(use_legacy_decorators, &constructor, Some(node), None)
        }
        None => false,
    }
}

fn modifier_list_from_node_list(modifiers: Option<NodeList>) -> Option<Arc<ModifierList>> {
    modifiers.map(|m| {
        let flags = modifiers_to_flags(&m.nodes);
        Arc::new(ModifierList::new(m.nodes.clone(), flags))
    })
}

fn method_asterisk_token(member: &Arc<Node>) -> Option<Arc<Node>> {
    match &member.data {
        tsox_frontend::ast::node_data_generated::NodeData::MethodDeclaration(d) => {
            d.asterisk_token.clone()
        }
        _ => None,
    }
}

impl EsDecoratorTransformer {
    pub fn enter_class(&mut self, ci: Arc<ClassInfo>) {
        let top = self.top.take();
        self.top = Some(Box::new(LexicalEntry {
            kind: LexicalEntryKind::Class,
            next: top,
            class_info_data: Some(ci),
            saved_pending_expressions: std::mem::take(&mut self.pending_expressions),
            class_this_data: None,
            class_super_data: None,
            depth: 0,
        }));
        self.update_state();
    }

    pub fn exit_class(&mut self) {
        let top = self.top.as_mut().expect("top should be set");
        debug_assert_eq!(top.kind, LexicalEntryKind::Class);
        self.pending_expressions = top.saved_pending_expressions.clone();
        let next = top.next.take();
        self.top = next;
        self.update_state();
    }

    pub fn enter_class_element(&mut self, node: &Arc<Node>) {
        debug_assert!(self.top.as_ref().is_some_and(|t| t.kind == LexicalEntryKind::Class));
        let top = self.top.take();
        let mut entry = Box::new(LexicalEntry {
            kind: LexicalEntryKind::ClassElement,
            next: top,
            class_info_data: None,
            saved_pending_expressions: Vec::new(),
            class_this_data: None,
            class_super_data: None,
            depth: 0,
        });
        if is_class_static_block_declaration(node) || (is_property_declaration(node) && has_static_modifier(node)) {
            if let Some(class_info) = entry.next.as_ref().and_then(|t| t.class_info_data.clone()) {
                entry.class_this_data = class_info.class_this.clone();
                entry.class_super_data = class_info.class_super.clone();
            }
        }
        self.top = Some(entry);
        self.update_state();
    }

    pub fn exit_class_element(&mut self) {
        debug_assert!(self
            .top
            .as_ref()
            .is_some_and(|t| t.kind == LexicalEntryKind::ClassElement));
        let next = self.top.as_mut().unwrap().next.take();
        self.top = next;
        self.update_state();
    }

    pub fn enter_name(&mut self) {
        debug_assert!(self
            .top
            .as_ref()
            .is_some_and(|t| t.kind == LexicalEntryKind::ClassElement));
        let top = self.top.take();
        self.top = Some(Box::new(LexicalEntry {
            kind: LexicalEntryKind::Name,
            next: top,
            class_info_data: None,
            saved_pending_expressions: Vec::new(),
            class_this_data: None,
            class_super_data: None,
            depth: 0,
        }));
        self.update_state();
    }

    pub fn exit_name(&mut self) {
        debug_assert!(self.top.as_ref().is_some_and(|t| t.kind == LexicalEntryKind::Name));
        let next = self.top.as_mut().unwrap().next.take();
        self.top = next;
        self.update_state();
    }

    pub fn enter_other(&mut self) {
        if self.top.as_ref().is_some_and(|t| t.kind == LexicalEntryKind::Other) {
            debug_assert!(self.pending_expressions.is_empty());
            self.top.as_mut().unwrap().depth += 1;
        } else {
            let top = self.top.take();
            self.top = Some(Box::new(LexicalEntry {
                kind: LexicalEntryKind::Other,
                next: top,
                class_info_data: None,
                saved_pending_expressions: std::mem::take(&mut self.pending_expressions),
                class_this_data: None,
                class_super_data: None,
                depth: 0,
            }));
            self.update_state();
        }
    }

    pub fn exit_other(&mut self) {
        debug_assert!(self.top.as_ref().is_some_and(|t| t.kind == LexicalEntryKind::Other));
        let top = self.top.as_mut().unwrap();
        if top.depth > 0 {
            debug_assert!(self.pending_expressions.is_empty());
            top.depth -= 1;
        } else {
            self.pending_expressions = top.saved_pending_expressions.clone();
            let next = top.next.take();
            self.top = next;
            self.update_state();
        }
    }

    pub fn class_element_visitor_visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::Constructor => Some(self.visit_constructor_declaration(node)),
            SyntaxKind::MethodDeclaration => Some(self.visit_method_declaration(node)),
            SyntaxKind::GetAccessor => Some(self.visit_get_accessor_declaration(node)),
            SyntaxKind::SetAccessor => Some(self.visit_set_accessor_declaration(node)),
            SyntaxKind::PropertyDeclaration => self.visit_property_declaration(node),
            SyntaxKind::ClassStaticBlockDeclaration => self.visit_class_static_block_declaration(node),
            _ => Some(self.visit(node)),
        }
    }

    pub fn constructor_class_element_visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if is_constructor_declaration(node) {
            return self.class_element_visitor_visit(node);
        }
        Some(node.clone())
    }

    pub fn export_stripping_modifier_visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::ExportKeyword {
            return None;
        }
        self.modifier_visitor_visit(node)
    }

    pub fn create_helper_variable(&mut self, node: &Arc<Node>, suffix: &str) -> Arc<Node> {
        let name = format!("{}_{}", get_helper_variable_name(&self.transformer.emit_context(), node), suffix);
        self.transformer.factory().generated_name_node(&self.transformer.factory().new_unique_name_ex(
            &name,
            AutoGenerateOptions {
                flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
                ..Default::default()
            },
        ))
    }

    pub fn create_let(&mut self, name: &Arc<Node>, initializer: Option<Arc<Node>>) -> Arc<Node> {
        let decl = self.transformer.factory().new_variable_declaration(name, None, None, initializer.as_ref());
        let list = self.transformer.factory().new_node_list(vec![decl]);
        let decl_list = self.transformer.factory().new_variable_declaration_list(&list, NodeFlags::Let);
        self.transformer.factory().new_variable_statement(None, &decl_list)
    }

    pub fn create_class_info(&mut self, node: &Arc<Node>) -> Arc<ClassInfo> {
        let mut ci = ClassInfo {
            class: Arc::clone(node),
            class_decorators_name: None,
            class_descriptor_name: None,
            class_extra_initializers_name: None,
            class_this: None,
            class_super: None,
            metadata_reference: self.transformer.factory().generated_name_node(&self.transformer.factory().new_unique_name_ex(
                "_metadata",
                AutoGenerateOptions {
                    flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
                    ..Default::default()
                },
            )),
            member_infos: Default::default(),
            instance_method_extra_initializers_name: None,
            static_method_extra_initializers_name: None,
            static_non_field_decoration_statements: Vec::new(),
            non_static_non_field_decoration_statements: Vec::new(),
            static_field_decoration_statements: Vec::new(),
            non_static_field_decoration_statements: Vec::new(),
            has_static_initializers: false,
            has_non_ambient_instance_fields: false,
            has_static_private_class_elements: false,
            pending_static_initializers: Vec::new(),
            pending_instance_initializers: Vec::new(),
        };

        if node_is_decorated(false, node, None, None) {
            let needs_unique_class_this = members(node)
                .iter()
                .any(|member| {
                    (is_private_identifier_class_element_declaration(member)
                        || is_auto_accessor_property_declaration(member))
                        && has_static_modifier(member)
                });
            let flags = if needs_unique_class_this {
                GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES
            } else {
                GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL
            };
            ci.class_this = Some(self.transformer.factory().generated_name_node(&self.transformer.factory().new_unique_name_ex(
                "_classThis",
                AutoGenerateOptions {
                    flags,
                    ..Default::default()
                },
            )));
        }

        for member in members(node).iter() {
            if is_method_or_accessor(member) && node_or_child_is_decorated(false, member, Some(node), None) {
                if has_static_modifier(member) {
                    if ci.static_method_extra_initializers_name.is_none() {
                        ci.static_method_extra_initializers_name = Some(self.transformer.factory().generated_name_node(&self.transformer.factory().new_unique_name_ex(
                            "_staticExtraInitializers",
                            AutoGenerateOptions {
                                flags: GeneratedIdentifierFlags::OPTIMISTIC
                                    | GeneratedIdentifierFlags::FILE_LEVEL,
                                ..Default::default()
                            },
                        )));
                        let renamed_class_this = match ci.class_this.clone() {
                            Some(class_this) => class_this,
                            None => self.transformer.factory().new_this_expression(),
                        };
                        let initializer = new_run_initializers_helper_r39k05(
                            &self.transformer.factory(),
                            &renamed_class_this,
                            &ci.static_method_extra_initializers_name.clone().unwrap(),
                        );
                        self.set_initializer_source_map_range(&initializer, node);
                        ci.pending_static_initializers.push(initializer);
                    }
                } else if ci.instance_method_extra_initializers_name.is_none() {
                    ci.instance_method_extra_initializers_name = Some(self.transformer.factory().generated_name_node(&self.transformer.factory().new_unique_name_ex(
                        "_instanceExtraInitializers",
                        AutoGenerateOptions {
                            flags: GeneratedIdentifierFlags::OPTIMISTIC | GeneratedIdentifierFlags::FILE_LEVEL,
                            ..Default::default()
                        },
                    )));
                    let initializer = new_run_initializers_helper_r39k05(
                        &self.transformer.factory(),
                        &self.transformer.factory().new_this_expression(),
                        &ci.instance_method_extra_initializers_name.clone().unwrap(),
                    );
                    self.set_initializer_source_map_range(&initializer, node);
                    ci.pending_instance_initializers.push(initializer);
                }
            }

            if is_class_static_block_declaration(member) {
                if !is_class_named_evaluation_helper_block(&self.transformer.emit_context(), member) {
                    ci.has_static_initializers = true;
                }
            } else if is_property_declaration(member) {
                if has_static_modifier(member) {
                    ci.has_static_initializers =
                        ci.has_static_initializers || member.initializer().is_some() || has_decorators(member);
                } else {
                    ci.has_non_ambient_instance_fields =
                        ci.has_non_ambient_instance_fields || !has_syntactic_modifier(member, ModifierFlags::Ambient);
                }
            }

            if (is_private_identifier_class_element_declaration(member)
                || is_auto_accessor_property_declaration(member))
                && has_static_modifier(member)
            {
                ci.has_static_private_class_elements = true;
            }

            if ci.static_method_extra_initializers_name.is_some()
                && ci.instance_method_extra_initializers_name.is_some()
                && ci.has_static_initializers
                && ci.has_non_ambient_instance_fields
                && ci.has_static_private_class_elements
            {
                break;
            }
        }

        Arc::new(ci)
    }

    fn set_initializer_source_map_range(&self, initializer: &Arc<Node>, node: &Arc<Node>) {
        match node.name() {
            Some(name) => self
                .emit_context()
                .set_source_map_range(initializer, name.loc.clone()),
            None => self
                .emit_context()
                .set_source_map_range(initializer, move_range_past_decorators(node)),
        }
    }

    pub fn emit_member_info_declarations(&mut self, ci: &mut ClassInfo, is_static: bool) -> Vec<Arc<Node>> {
        let mut stmts = Vec::new();
        let entries: Vec<(Arc<Node>, MemberInfo)> = ci
            .member_infos
            .iter()
            .map(|(k, v)| (k.clone(), clone_member_info(v)))
            .collect();
        for (member, mi) in entries {
            if tsox_frontend::ast::utilities::is_static(&member) != is_static {
                continue;
            }
            stmts.push(self.create_let_of(&mi.member_decorators_name));
            if let Some(initializers_name) = &mi.member_initializers_name {
                let array = self
                    .factory()
                    .new_array_literal_expression(&self.transformer.factory().new_node_list(Vec::new()), false);
                stmts.push(self.create_let(initializers_name, Some(array)));
            }
            if let Some(extra_initializers_name) = &mi.member_extra_initializers_name {
                let array = self
                    .factory()
                    .new_array_literal_expression(&self.transformer.factory().new_node_list(Vec::new()), false);
                stmts.push(self.create_let(extra_initializers_name, Some(array)));
            }
            if let Some(descriptor_name) = &mi.member_descriptor_name {
                stmts.push(self.create_let(descriptor_name, None));
            }
        }
        stmts
    }

    fn create_let_of(&mut self, name: &Option<Arc<Node>>) -> Arc<Node> {
        match name {
            Some(name) => self.create_let(name, None),
            None => self.create_let(&self.transformer.factory().new_identifier(""), None),
        }
    }

    pub fn can_ignore_empty_string_literal_in_assigned_name(&self, node: Option<&Arc<Node>>) -> bool {
        let Some(node) = node else {
            return false;
        };
        let inner_expression = skip_outer_expressions(node, OuterExpressionKinds::all());
        is_class_expression(&inner_expression)
            && inner_expression.name().is_none()
            && !class_or_constructor_parameter_is_decorated(false, &inner_expression)
    }

    pub fn create_descriptor_method(
        &mut self,
        original: &Arc<Node>,
        name: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<Arc<Node>>,
        kind: &str,
        parameters: NodeList,
        body: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let body = body.unwrap_or_else(|| self.transformer.factory().new_block(&self.transformer.factory().new_node_list(Vec::new()), false));

        let func_expr = self.transformer.factory().new_function_expression(
            modifiers,
            asterisk_token.as_ref(),
            None,
            None,
            &parameters,
            None,
            None,
            &body,
        );
        self.transformer.emit_context().set_original(&func_expr, original);
        self.transformer.emit_context()
            .set_source_map_range(&func_expr, move_range_past_decorators(original));
        self.transformer.emit_context().set_emit_flags(&func_expr, EmitFlags::NO_COMMENTS);

        let prefix = if kind == "get" || kind == "set" {
            Some(kind.to_string())
        } else {
            None
        };
        let function_name = new_string_literal_from_node_r39k05(&self.transformer.factory(), name);
        let named_function = new_set_function_name_helper_r39k05(
            &self.transformer.factory(),
            &func_expr,
            &function_name,
            prefix.as_deref(),
        );

        let method = self.transformer.factory().new_property_assignment(
            None,
            &self.transformer.factory().new_identifier(kind),
            None,
            None,
            &named_function,
        );
        self.transformer.emit_context().set_original(&method, original);
        self.transformer.emit_context()
            .set_source_map_range(&method, move_range_past_decorators(original));
        self.transformer.emit_context().set_emit_flags(&method, EmitFlags::NO_COMMENTS);
        method
    }

    pub fn create_method_descriptor_object(
        &mut self,
        member: &Arc<Node>,
        modifiers: Option<NodeList>,
    ) -> Arc<Node> {
        let parameters = self
            .transformer
            .visitor()
            .visit_node_list(parameter_list(member).map(|p| &**p));
        let body = member.body().map(|b| self.transformer.visitor().visit_node(&b));
        let asterisk_token = method_asterisk_token(member);
        let value_method = self.create_descriptor_method(
            member,
            &member.name().unwrap(),
            modifier_list_from_node_list(modifiers),
            asterisk_token,
            "value",
            NodeList::new(parameters),
            body,
        );
        self.transformer.factory()
            .new_object_literal_expression(&self.transformer.factory().new_node_list(vec![value_method]), false)
    }

    pub fn create_get_accessor_descriptor_object(
        &mut self,
        member: &Arc<Node>,
        modifiers: Option<NodeList>,
    ) -> Arc<Node> {
        let body = member.body().map(|b| self.transformer.visitor().visit_node(&b));
        let get_method = self.create_descriptor_method(
            member,
            &member.name().unwrap(),
            modifier_list_from_node_list(modifiers),
            None,
            "get",
            NodeList::new(Vec::new()),
            body,
        );
        self.transformer.factory()
            .new_object_literal_expression(&self.transformer.factory().new_node_list(vec![get_method]), false)
    }

    pub fn create_set_accessor_descriptor_object(
        &mut self,
        member: &Arc<Node>,
        modifiers: Option<NodeList>,
    ) -> Arc<Node> {
        let parameters = self
            .transformer
            .visitor()
            .visit_node_list(parameter_list(member).map(|p| &**p));
        let body = member.body().map(|b| self.transformer.visitor().visit_node(&b));
        let set_method = self.create_descriptor_method(
            member,
            &member.name().unwrap(),
            modifier_list_from_node_list(modifiers),
            None,
            "set",
            NodeList::new(parameters),
            body,
        );
        self.transformer.factory()
            .new_object_literal_expression(&self.transformer.factory().new_node_list(vec![set_method]), false)
    }

    pub fn create_accessor_property_descriptor_object(
        &mut self,
        member: &Arc<Node>,
        _modifiers: Option<NodeList>,
    ) -> Arc<Node> {
        let backing_field_name = self.transformer.factory().generated_name_node(
            &self.transformer.factory().new_generated_private_name_for_node_ex(
            &member.name().unwrap(),
            AutoGenerateOptions {
                suffix: "_accessor_storage".to_string(),
                ..Default::default()
            },
        ));
        let getter_body = self.transformer.factory().new_block(
            &self.transformer.factory().new_node_list(vec![self.transformer.factory().new_return_statement(
                Some(&self.transformer.factory().new_property_access_expression(
                    &self.transformer.factory().new_this_expression(),
                    None,
                    &backing_field_name,
                    NodeFlags::empty(),
                )),
            )]),
            false,
        );
        let getter = self.create_descriptor_method(
            member,
            &member.name().unwrap(),
            None,
            None,
            "get",
            NodeList::new(Vec::new()),
            Some(getter_body),
        );
        let setter_body = self.transformer.factory().new_block(
            &self.transformer.factory().new_node_list(vec![self.transformer.factory().new_expression_statement(
                &self.transformer.factory().new_assignment_expression(
                    &self.transformer.factory().new_property_access_expression(
                        &self.transformer.factory().new_this_expression(),
                        None,
                        &backing_field_name,
                        NodeFlags::empty(),
                    ),
                    &self.transformer.factory().new_identifier("value"),
                ),
            )]),
            false,
        );
        let setter = self.create_descriptor_method(
            member,
            &member.name().unwrap(),
            None,
            None,
            "set",
            NodeList::new(vec![self.transformer.factory().new_parameter_declaration(
                None,
                None,
                &self.transformer.factory().new_identifier("value"),
                None,
                None,
                None,
            )]),
            Some(setter_body),
        );
        self.transformer.factory()
            .new_object_literal_expression(&self.transformer.factory().new_node_list(vec![getter, setter]), false)
    }

    pub fn create_method_descriptor_forwarder(
        &mut self,
        modifiers: Option<NodeList>,
        name: &Arc<Node>,
        descriptor_name: &Arc<Node>,
    ) -> Arc<Node> {
        let static_only = self.static_only_modifier_visitor.visit_modifiers(modifiers.as_ref());
        new_get_accessor_declaration_full_r39k05(
            &self.transformer.factory(),
            static_only,
            name,
            None,
            self.transformer.factory().new_node_list(Vec::new()),
            None,
            None,
            self.transformer.factory().new_block(
                &self.transformer.factory().new_node_list(vec![self.transformer.factory().new_return_statement(
                    Some(&self.transformer.factory().new_property_access_expression(
                        descriptor_name,
                        None,
                        &self.transformer.factory().new_identifier("value"),
                        NodeFlags::empty(),
                    )),
                )]),
                false,
            ),
        )
    }

    pub fn create_get_accessor_descriptor_forwarder(
        &mut self,
        modifiers: Option<NodeList>,
        name: &Arc<Node>,
        descriptor_name: &Arc<Node>,
    ) -> Arc<Node> {
        let static_only = self.static_only_modifier_visitor.visit_modifiers(modifiers.as_ref());
        new_get_accessor_declaration_full_r39k05(
            &self.transformer.factory(),
            static_only,
            name,
            None,
            self.transformer.factory().new_node_list(Vec::new()),
            None,
            None,
            self.transformer.factory().new_block(
                &self.transformer.factory().new_node_list(vec![self.transformer.factory().new_return_statement(
                Some(&new_function_call_call_r39k05(
                    &self.transformer.factory(),
                    &self.transformer.factory().new_property_access_expression(
                        descriptor_name,
                        None,
                        &self.transformer.factory().new_identifier("get"),
                        NodeFlags::empty(),
                    ),
                    &self.transformer.factory().new_this_expression(),
                    &[],
                )),
                )]),
                false,
            ),
        )
    }

    pub fn create_set_accessor_descriptor_forwarder(
        &mut self,
        modifiers: Option<NodeList>,
        name: &Arc<Node>,
        descriptor_name: &Arc<Node>,
    ) -> Arc<Node> {
        let static_only = self.static_only_modifier_visitor.visit_modifiers(modifiers.as_ref());
        self.transformer.factory().new_set_accessor_declaration(
            static_only,
            name,
            None,
            self.transformer.factory().new_node_list(vec![self.transformer.factory().new_parameter_declaration(
                None,
                None,
                &self.transformer.factory().new_identifier("value"),
                None,
                None,
                None,
            )]),
            None,
            None,
            self.transformer.factory().new_block(
                &self.transformer.factory().new_node_list(vec![self.transformer.factory().new_return_statement(
                Some(&new_function_call_call_r39k05(
                    &self.transformer.factory(),
                    &self.transformer.factory().new_property_access_expression(
                        descriptor_name,
                        None,
                        &self.transformer.factory().new_identifier("set"),
                        NodeFlags::empty(),
                    ),
                    &self.transformer.factory().new_this_expression(),
                    &[self.transformer.factory().new_identifier("value")],
                )),
                )]),
                false,
            ),
        )
    }

    pub fn create_metadata(&mut self, name: &Arc<Node>, class_super: Option<&Arc<Node>>) -> Arc<Node> {
        let super_metadata = match class_super {
            Some(class_super) => self.create_symbol_metadata_reference(class_super),
            None => self.transformer.factory().new_token(SyntaxKind::NullKeyword),
        };
        let object_create = self.transformer.factory().new_call_expression(
            &self.transformer.factory().new_property_access_expression(
                &self.transformer.factory().new_identifier("Object"),
                None,
                &self.transformer.factory().new_identifier("create"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.transformer.factory().new_node_list(vec![super_metadata]),
            NodeFlags::empty(),
        );

        let symbol_check = self.transformer.factory().new_logical_and_expression(
            &self.transformer.factory()
                .new_type_check(&self.transformer.factory().new_identifier("Symbol"), "function"),
            &self.transformer.factory().new_property_access_expression(
                &self.transformer.factory().new_identifier("Symbol"),
                None,
                &self.transformer.factory().new_identifier("metadata"),
                NodeFlags::empty(),
            ),
        );

        let conditional = self.transformer.factory().new_conditional_expression(
            &symbol_check,
            &self.transformer.factory().new_token(SyntaxKind::QuestionToken),
            &object_create,
            &self.transformer.factory().new_token(SyntaxKind::ColonToken),
            &self.transformer.factory().new_void_zero_expression(),
        );

        let var_decl = self.transformer.factory().new_variable_declaration(name, None, None, Some(&conditional));
        let var_decl_list = self
            .factory()
            .new_variable_declaration_list(&self.transformer.factory().new_node_list(vec![var_decl]), NodeFlags::Const);
        self.transformer.factory().new_variable_statement(None, &var_decl_list)
    }

    pub fn create_symbol_metadata(&mut self, target: &Arc<Node>, value: &Arc<Node>) -> Arc<Node> {
        let symbol_metadata = self.transformer.factory().new_property_access_expression(
            &self.transformer.factory().new_identifier("Symbol"),
            None,
            &self.transformer.factory().new_identifier("metadata"),
            NodeFlags::empty(),
        );

        let descriptor = self.transformer.factory().new_object_literal_expression(
            &self.transformer.factory().new_node_list(vec![
                self.transformer.factory().new_property_assignment(
                    None,
                    &self.transformer.factory().new_identifier("enumerable"),
                    None,
                    None,
                    &self.transformer.factory().new_true_expression(),
                ),
                self.transformer.factory().new_property_assignment(
                    None,
                    &self.transformer.factory().new_identifier("configurable"),
                    None,
                    None,
                    &self.transformer.factory().new_true_expression(),
                ),
                self.transformer.factory().new_property_assignment(
                    None,
                    &self.transformer.factory().new_identifier("writable"),
                    None,
                    None,
                    &self.transformer.factory().new_true_expression(),
                ),
                self.transformer.factory().new_property_assignment(
                    None,
                    &self.transformer.factory().new_identifier("value"),
                    None,
                    None,
                    value,
                ),
            ]),
            false,
        );

        let define_property = self.transformer.factory().new_call_expression(
            &self.transformer.factory().new_property_access_expression(
                &self.transformer.factory().new_identifier("Object"),
                None,
                &self.transformer.factory().new_identifier("defineProperty"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.transformer.factory()
                .new_node_list(vec![target.clone(), symbol_metadata, descriptor]),
            NodeFlags::empty(),
        );

        let if_statement = self.transformer.factory().new_if_statement(
            value,
            &self.transformer.factory().new_expression_statement(&define_property),
            None,
        );
        self.transformer.emit_context().set_emit_flags(&if_statement, EmitFlags::SINGLE_LINE);
        if_statement
    }

    pub fn create_symbol_metadata_reference(&mut self, class_super: &Arc<Node>) -> Arc<Node> {
        let symbol_metadata = self.transformer.factory().new_property_access_expression(
            &self.transformer.factory().new_identifier("Symbol"),
            None,
            &self.transformer.factory().new_identifier("metadata"),
            NodeFlags::empty(),
        );
        let element_access = self.transformer.factory().new_element_access_expression(
            class_super,
            None,
            &symbol_metadata,
            NodeFlags::empty(),
        );
        self.transformer.factory().new_binary_expression(
            None,
            &element_access,
            None,
            &self.transformer.factory().new_token(SyntaxKind::QuestionQuestionToken),
            &self.transformer.factory().new_token(SyntaxKind::NullKeyword),
        )
    }
}

pub fn get_helper_variable_name(ec: &crate::printer::EmitContext, node: &Arc<Node>) -> String {
    let name = node.name();
    let mut declaration_name = String::new();
    match &name {
        Some(name)
            if is_identifier(name)
                && !is_generated_identifier(ec, name) =>
        {
            declaration_name = name.text().to_string()
        }
        Some(name) if is_private_identifier(name) && !ec.has_auto_generate_info(name) => {
            let text = name.text();
            if text.len() > 1 {
                declaration_name = text[1..].to_string();
            }
        }
        Some(name) if is_string_literal(name) && is_identifier_text(&name.text(), LanguageVariant::Standard) => {
            declaration_name = name.text().to_string()
        }
        _ => {
            if is_class_like(node) {
                declaration_name = "class".to_string();
            } else {
                declaration_name = "member".to_string();
            }
        }
    }

    if is_get_accessor_declaration(node) {
        declaration_name = format!("get_{}", declaration_name);
    }
    if is_set_accessor_declaration(node) {
        declaration_name = format!("set_{}", declaration_name);
    }
    if name.as_ref().is_some_and(|n| is_private_identifier(n)) {
        declaration_name = format!("private_{}", declaration_name);
    }
    if is_static(node) {
        declaration_name = format!("static_{}", declaration_name);
    }
    format!("_{}", declaration_name)
}

fn clone_member_info(mi: &MemberInfo) -> MemberInfo {
    MemberInfo {
        member_decorators_name: mi.member_decorators_name.clone(),
        member_initializers_name: mi.member_initializers_name.clone(),
        member_extra_initializers_name: mi.member_extra_initializers_name.clone(),
        member_descriptor_name: mi.member_descriptor_name.clone(),
    }
}
