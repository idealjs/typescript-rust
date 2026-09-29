use std::sync::Arc;
use tsox_frontend::ast::*;

use crate::mig::m4h_5::{ClassInfo, EsDecoratorTransformer, MemberInfo};
use super::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::mig::m4h::r40k17_defs::clone_class_info;
use crate::mig::m4h_6::r36k28_defs::R36K28NodeVisitorExt;
use crate::mig::m4h_4::r36k9_defs::cloned_node_list;
use crate::mig::m4m_4::move_range_past_decorators;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;

fn is_private_identifier_class_element_declaration(member: &Arc<Node>) -> bool {
    (is_property_declaration(member) || is_method_or_accessor(member))
        && member.name().map(|n| is_private_identifier(&n)).unwrap_or(false)
}

pub struct PartialResult {
    pub modifiers: Option<Arc<ModifierList>>,
    pub referenced_name: Option<Arc<Node>>,
    pub name: Option<Arc<Node>>,
    pub initializers_name: Option<Arc<Node>>,
    pub extra_initializers_name: Option<Arc<Node>>,
    pub descriptor_name: Option<Arc<Node>>,
    pub this_arg: Option<Arc<Node>>,
}

pub type CreateDescriptorFn<'a> =
    &'a dyn Fn(&mut EsDecoratorTransformer, &Arc<Node>, Option<&Arc<ModifierList>>) -> Option<Arc<Node>>;

impl EsDecoratorTransformer {
    pub fn partial_transform_class_element(
        &mut self,
        member: &Arc<Node>,
        ci: Option<&mut ClassInfo>,
        create_descriptor: Option<CreateDescriptorFn<'_>>,
    ) -> PartialResult {
        let mut ec = self.transformer.emit_context();
        let mut result = PartialResult {
            modifiers: None,
            referenced_name: None,
            name: None,
            initializers_name: None,
            extra_initializers_name: None,
            descriptor_name: None,
            this_arg: None,
        };

        let ci = match ci {
            None => {
                let modifiers = self.modifier_visitor.visit_modifiers(member.modifiers());
                self.enter_name();
                let name = self.visit_property_name(member.name().unwrap());
                self.exit_name();
                result.modifiers = modifiers;
                result.name = Some(name);
                return result;
            }
            Some(ci) => ci,
        };

        let saved_class_this = self.class_this.take();
        let member_decorators =
            self.transform_all_decorators_of_declaration(&decorators(member));
        self.class_this = saved_class_this;
        let modifiers = self.modifier_visitor.visit_modifiers(member.modifiers());
        result.modifiers = modifiers.clone();

        if !member_decorators.is_empty() {
            let member_decorators_name = self.create_helper_variable(member, "decorators");
            let member_decorators_assignment = {
                let f = self.transformer.factory();
                let decorators_nodes = f.new_node_list(member_decorators.clone());
                let member_decorators_array = f.new_array_literal_expression(&decorators_nodes, false);
                f.new_assignment_expression(&member_decorators_name, &member_decorators_array)
            };
            let mut mi = MemberInfo {
                member_decorators_name: Some(member_decorators_name.clone()),
                member_initializers_name: None,
                member_extra_initializers_name: None,
                member_descriptor_name: None,
            };
            ci.member_infos.set(member.clone(), mi);
            self.pending_expressions.push(member_decorators_assignment);

            let kind = if is_get_accessor_declaration(member) {
                "getter"
            } else if is_set_accessor_declaration(member) {
                "setter"
            } else if is_method_declaration(member) {
                "method"
            } else if is_auto_accessor_property_declaration(member) {
                "accessor"
            } else if is_property_declaration(member) {
                "field"
            } else {
                panic!("Unexpected class element kind.")
            };

            let member_name = member.name();
            let mut property_name_computed = false;
            let mut property_name_expr: Option<Arc<Node>> = None;
            if let Some(name) = member_name {
                if is_identifier(name) || is_private_identifier(name) {
                    property_name_computed = false;
                    property_name_expr = Some(name.clone());
                } else if is_property_name_literal(name) {
                    property_name_computed = true;
                    property_name_expr = Some(self.transformer.factory().new_string_literal_from_node(name));
                } else if is_computed_property_name(name) {
                    let cpn_expression = match &name.data {
                        NodeData::ComputedPropertyName(d) => d.expression.clone(),
                        _ => unreachable!(),
                    };
                    if is_property_name_literal(&cpn_expression) && !is_identifier(&cpn_expression) {
                        property_name_computed = true;
                        property_name_expr =
                            Some(self.transformer.factory().new_string_literal_from_node(&cpn_expression));
                    } else {
                        self.enter_name();
                        let (referenced_name, updated_name) = self.visit_referenced_property_name(name);
                        self.exit_name();
                        property_name_computed = true;
                        property_name_expr = Some(referenced_name.clone());
                        result.referenced_name = Some(referenced_name);
                        result.name = Some(updated_name);
                    }
                }
            }

            let member_is_private = member_name.is_some_and(|n| is_private_identifier(n));
            let context_obj = {
                let f = self.transformer.factory();
                f.new_es_decorate_class_element_context_object(
                    kind,
                    property_name_computed,
                    property_name_expr
                        .as_ref()
                        .unwrap_or_else(|| member.name().unwrap_or(member)),
                    is_static(member),
                    member_is_private,
                    is_property_declaration(member) || is_get_accessor_declaration(member) || is_method_declaration(member),
                    is_property_declaration(member) || is_set_accessor_declaration(member),
                    &ci.metadata_reference,
                )
            };

            if is_method_or_accessor(member) {
                let method_extra_initializers_name = if is_static(member) {
                    ci.static_method_extra_initializers_name.clone()
                } else {
                    ci.instance_method_extra_initializers_name.clone()
                }
                .unwrap();

                let descriptor_arg =
                    if is_private_identifier_class_element_declaration(member) && create_descriptor.is_some() {
                        let create_descriptor = create_descriptor.unwrap();
                        let async_mods =
                            self.async_only_modifier_visitor.visit_modifiers(modifiers.as_ref());
                        let descriptor = create_descriptor(self, member, async_mods.as_ref()).unwrap();
                        let descriptor_name = self.create_helper_variable(member, "descriptor");
                        result.descriptor_name = Some(descriptor_name.clone());
                        self.transformer
                            .factory()
                            .new_assignment_expression(&descriptor_name, &descriptor)
                    } else {
                        self.transformer.factory().new_token(SyntaxKind::NullKeyword)
                    };

                let es_decorate_statement = {
                    let f = self.transformer.factory();
                    let this_expression = f.new_this_expression();
                    let null_token = f.new_token(SyntaxKind::NullKeyword);
                    let es_decorate_expr = f.new_es_decorate_helper(
                        &this_expression,
                        &descriptor_arg,
                        &member_decorators_name,
                        &context_obj,
                        &null_token,
                        &method_extra_initializers_name,
                    );
                    f.new_expression_statement(&es_decorate_expr)
                };
                ec.set_source_map_range(&es_decorate_statement, move_range_past_decorators(member));
                self.append_decoration_statement(ci, member, es_decorate_statement);
            } else if is_property_declaration(member) {
                let initializers_name = self.create_helper_variable(member, "initializers");
                let extra_initializers_name = self.create_helper_variable(member, "extraInitializers");
                result.initializers_name = Some(initializers_name.clone());
                result.extra_initializers_name = Some(extra_initializers_name.clone());
                if is_static(member) {
                    result.this_arg = ci.class_this.clone();
                }
                if let Some(mi) = ci.member_infos.get_mut(member) {
                    mi.member_initializers_name = Some(initializers_name);
                    mi.member_extra_initializers_name = Some(extra_initializers_name);
                }

                let ctor_arg = if is_auto_accessor_property_declaration(member) {
                    self.transformer.factory().new_this_expression()
                } else {
                    self.transformer.factory().new_token(SyntaxKind::NullKeyword)
                };

                let descriptor_arg = if is_private_identifier_class_element_declaration(member)
                    && has_accessor_modifier(member)
                    && create_descriptor.is_some()
                {
                    let create_descriptor = create_descriptor.unwrap();
                    let descriptor = create_descriptor(self, member, None).unwrap();
                    let descriptor_name = self.create_helper_variable(member, "descriptor");
                    result.descriptor_name = Some(descriptor_name.clone());
                    if let Some(mi) = ci.member_infos.get_mut(member) {
                        mi.member_descriptor_name = Some(descriptor_name.clone());
                    }
                    self.transformer
                        .factory()
                        .new_assignment_expression(&descriptor_name, &descriptor)
                } else {
                    self.transformer.factory().new_token(SyntaxKind::NullKeyword)
                };

                let es_decorate_statement = {
                    let f = self.transformer.factory();
                    let es_decorate_expr = f.new_es_decorate_helper(
                        &ctor_arg,
                        &descriptor_arg,
                        &member_decorators_name,
                        &context_obj,
                        result.initializers_name.as_ref().unwrap(),
                        result.extra_initializers_name.as_ref().unwrap(),
                    );
                    f.new_expression_statement(&es_decorate_expr)
                };
                ec.set_source_map_range(&es_decorate_statement, move_range_past_decorators(member));
                self.append_decoration_statement(ci, member, es_decorate_statement);
            }
        }

        if result.name.is_none() {
            self.enter_name();
            result.name = Some(self.visit_property_name(member.name().unwrap()));
            self.exit_name();
        }

        let modifiers_empty = result
            .modifiers
            .as_ref()
            .is_none_or(|m| m.nodes.is_empty());
        if modifiers_empty && (is_method_declaration(member) || is_property_declaration(member)) {
            ec.add_emit_flags(result.name.as_ref().unwrap(), EmitFlags::NO_LEADING_COMMENTS);
        }

        result
    }

    pub fn append_decoration_statement(&mut self, ci: &mut ClassInfo, member: &Arc<Node>, stmt: Arc<Node>) {
        if is_method_or_accessor(member) || is_auto_accessor_property_declaration(member) {
            if is_static(member) {
                ci.static_non_field_decoration_statements.push(stmt);
            } else {
                ci.non_static_non_field_decoration_statements.push(stmt);
            }
        } else if is_property_declaration(member) && !is_auto_accessor_property_declaration(member) {
            if is_static(member) {
                ci.static_field_decoration_statements.push(stmt);
            } else {
                ci.non_static_field_decoration_statements.push(stmt);
            }
        } else {
            panic!("Unexpected class element kind.")
        }
    }

    pub fn visit_method_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> {
        self.enter_class_element(node);
        let taken = self.class_info_stack.take();
        let mut ci = taken.as_ref().map(|c| clone_class_info(c));
        let result = self.partial_transform_class_element(
            node,
            ci.as_mut(),
            Some(&|tx, member, modifiers| {
                Some(tx.create_method_descriptor_object(member, modifiers.map(|m| cloned_node_list(&m.list))))
            }),
        );
        if let Some(updated) = ci {
            self.store_class_info(updated);
        } else {
            self.class_info_stack = taken;
        }
        if let Some(descriptor_name) = &result.descriptor_name {
            self.exit_class_element();
            let forwarder = self.create_method_descriptor_forwarder(
                result.modifiers.clone().map(|m| cloned_node_list(&m.list)),
                result.name.as_ref().unwrap(),
                descriptor_name,
            );
            return self.finish_class_element(forwarder, node);
        }
        let parameters = tsox_frontend::ast::mig::m3b::parameter_list(node)
            .map(|pl| self.transformer.visitor().visit_nodes(pl))
            .unwrap_or_else(|| Arc::new(tsox_frontend::ast::NodeList::new(Vec::new())));
        let body = self
            .transformer
            .visitor()
            .visit_node(node.body().as_ref().unwrap());
        self.exit_class_element();
        let asterisk_token = match &node.data {
            NodeData::MethodDeclaration(d) => d.asterisk_token.clone(),
            _ => unreachable!(),
        };
        let method = self.transformer.factory().update_method_declaration(
            node,
            result.modifiers.clone(),
            asterisk_token.as_ref(),
            result.name.as_ref().unwrap(),
            None,
            None,
            &parameters,
            None,
            None,
            Some(&body),
        );
        self.finish_class_element(method, node)
    }

    pub fn visit_get_accessor_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> {
        self.enter_class_element(node);
        let taken = self.class_info_stack.take();
        let mut ci = taken.as_ref().map(|c| clone_class_info(c));
        let result = self.partial_transform_class_element(
            node,
            ci.as_mut(),
            Some(&|tx, member, modifiers| {
                Some(tx.create_get_accessor_descriptor_object(member, modifiers.map(|m| cloned_node_list(&m.list))))
            }),
        );
        if let Some(updated) = ci {
            self.store_class_info(updated);
        } else {
            self.class_info_stack = taken;
        }
        if let Some(descriptor_name) = &result.descriptor_name {
            self.exit_class_element();
            let forwarder = self.create_get_accessor_descriptor_forwarder(
                result.modifiers.clone().map(|m| cloned_node_list(&m.list)),
                result.name.as_ref().unwrap(),
                descriptor_name,
            );
            return self.finish_class_element(forwarder, node);
        }
        let parameters = tsox_frontend::ast::mig::m3b::parameter_list(node)
            .map(|pl| self.transformer.visitor().visit_nodes(pl))
            .unwrap_or_else(|| Arc::new(tsox_frontend::ast::NodeList::new(Vec::new())));
        let body = self
            .transformer
            .visitor()
            .visit_node(node.body().as_ref().unwrap());
        self.exit_class_element();
        let accessor = self.transformer.factory().update_get_accessor_declaration(
            node,
            result.modifiers.clone(),
            result.name.as_ref().unwrap(),
            None,
            &parameters,
            None,
            None,
            Some(&body),
        );
        self.finish_class_element(accessor, node)
    }

    pub fn visit_set_accessor_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> {
        self.enter_class_element(node);
        let taken = self.class_info_stack.take();
        let mut ci = taken.as_ref().map(|c| clone_class_info(c));
        let result = self.partial_transform_class_element(
            node,
            ci.as_mut(),
            Some(&|tx, member, modifiers| {
                Some(tx.create_set_accessor_descriptor_object(member, modifiers.map(|m| cloned_node_list(&m.list))))
            }),
        );
        if let Some(updated) = ci {
            self.store_class_info(updated);
        } else {
            self.class_info_stack = taken;
        }
        if let Some(descriptor_name) = &result.descriptor_name {
            self.exit_class_element();
            let forwarder = self.create_set_accessor_descriptor_forwarder(
                result.modifiers.clone().map(|m| cloned_node_list(&m.list)),
                result.name.as_ref().unwrap(),
                descriptor_name,
            );
            return self.finish_class_element(forwarder, node);
        }
        let parameters = tsox_frontend::ast::mig::m3b::parameter_list(node)
            .map(|pl| self.transformer.visitor().visit_nodes(pl))
            .unwrap_or_else(|| Arc::new(tsox_frontend::ast::NodeList::new(Vec::new())));
        let body = self
            .transformer
            .visitor()
            .visit_node(node.body().as_ref().unwrap());
        self.exit_class_element();
        let accessor = self.transformer.factory().update_set_accessor_declaration(
            node,
            result.modifiers.clone(),
            result.name.as_ref().unwrap(),
            None,
            &parameters,
            None,
            None,
            Some(&body),
        );
        self.finish_class_element(accessor, node)
    }

    pub fn finish_class_element(&mut self, updated: Arc<Node>, original: &Arc<Node>) -> Arc<Node> {
        if updated.id() != original.id() {
            self.transformer.emit_context().assign_comment_range(&updated, original);
            self.transformer
                .emit_context()
                .set_source_map_range(&updated, move_range_past_decorators(original));
        }
        updated
    }
}

fn decorators(node: &Arc<Node>) -> Vec<Arc<Node>> {
    node.modifier_nodes()
        .iter()
        .filter(|m| is_decorator(m))
        .cloned()
        .collect()
}
