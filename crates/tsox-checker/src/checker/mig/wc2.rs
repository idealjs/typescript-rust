#![allow(unused_imports)]

#[path = "r20k3_defs.rs"]
pub mod r20k3_defs;
#[path = "r21k3_defs.rs"]
pub mod r21k3_defs;
#[path = "r22k3_defs.rs"]
pub mod r22k3_defs;
#[path = "r23k3_defs.rs"]
pub mod r23k3_defs;
#[path = "r24k8_defs.rs"]
pub mod r24k8_defs;

use self::r22k3_defs::*;

use crate::checker::checker::*;
use crate::checker::checker_iteration::*;
use crate::checker::checker_this_container::get_this_parameter;
use crate::checker::mig::m1e::EnumLiteralValue;
use crate::checker::mig::m1d_2::get_base_type_node_of_class;
use crate::checker::mig::m3a_2::has_dot_dot_dot_token;
use crate::checker::jsx_impl_chunk::is_jsx_opening_like_element;
use crate::checker::mig::wc3_3::is_zero_big_int;
use crate::checker::types::*;
use std::sync::{Arc, OnceLock};
use tsox_frontend::ast::mig::m3b::{members, properties};
use tsox_frontend::ast::mig::m3e_4::{
    get_immediately_invoked_function_expression, get_this_container,
};
use tsox_frontend::ast::mig::m3f_2::has_dynamic_name;
use tsox_frontend::ast::mig::x1a::arguments;
use tsox_frontend::ast::mig::x4ast::get_class_like_declaration_of_symbol;
use tsox_frontend::ast::{self, Node, NodeData, Symbol, SyntaxKind};
use tsox_frontend::evaluator::EvalValue;

impl Checker {
    pub fn get_contextual_type_for_await_operand(
        &mut self,
        node: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextual_type_for_await_operand"); 
        if let Some(contextual_type) = self.get_contextual_type(node, context_flags) {
            if let Some(contextual_awaited_type) = self.get_awaited_type_no_alias(&contextual_type) {
                let promise_like = self.create_promise_like_type(&contextual_awaited_type);
                return Some(self.get_union_type(vec![
                    Arc::clone(&contextual_awaited_type),
                    promise_like,
                ]));
            }
        }
        None
    }

    pub fn get_contextual_type_for_decorator(&mut self, decorator: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextual_type_for_decorator"); 
        let signature = self.get_decorator_call_signature(decorator)?;
        Some(self.get_or_create_type_from_signature(&signature))
    }

    pub fn get_contextual_type_for_object_literal_method(
        &mut self,
        node: &Arc<Node>,
        context_flags: ContextFlags,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextual_type_for_object_literal_method"); 
        if node.flags.intersects(ast::NodeFlags::InWithStatement) {
            return None;
        }
        self.get_contextual_type_for_object_literal_element(node, context_flags)
    }

    pub fn get_contextually_typed_parameter_type(
        &mut self,
        parameter: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextually_typed_parameter_type"); 
        let Some(function) = parameter.parent() else {
            return None;
        };
        if !self.is_context_sensitive_function_or_object_literal_method(&function) {
            return None;
        }
        if let Some(iife) = get_immediately_invoked_function_expression(&function) {
            let args = self.get_effective_call_arguments(&iife);
            let index_of_parameter = params_of(&function)
                .iter()
                .position(|p| Arc::ptr_eq(p, parameter))
                .unwrap_or(0);
            if has_dot_dot_dot_token(parameter) {
                return Some(self.get_spread_argument_type(
                    &args,
                    index_of_parameter,
                    args.len(),
                ));
            }
            let any_sig = self.any_signature();
            let cached = {
                let links = self.signature_links.get_or_default(&iife);
                let cached = links.resolved_signature.clone();
                links.resolved_signature = Some(Arc::clone(&any_sig));
                cached
            };
            let t = if (index_of_parameter as usize) < args.len() {
                let arg_type = self
                    .check_expression_ex(&args[index_of_parameter as usize], CheckMode::Normal);
                Some(self.get_widened_literal_type(&arg_type))
            } else if parameter.initializer().is_some() {
                None
            } else {
                Some(self.undefined_widening_type())
            };
            self.signature_links
                .get_or_default(&iife)
                .resolved_signature = cached;
            return t;
        }
        let contextual_signature = self.get_contextual_signature(&function)?;
        let index = params_of(&function)
            .iter()
            .position(|p| Arc::ptr_eq(p, parameter))
            .unwrap_or(0)
            - if get_this_parameter(&function).is_some() {
                1
            } else {
                0
            };
        if has_dot_dot_dot_token(parameter)
            && params_of(&function)
                .last()
                .map(|p| Arc::ptr_eq(p, parameter))
                .unwrap_or(false)
        {
            return self.get_rest_type_at_position(&contextual_signature, index);
        }
        self.try_get_type_at_position(&contextual_signature, index)
    }

    pub fn get_declared_type_of_alias(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_alias"); 
        if let Some(links) = self.declared_type_links.get(symbol) {
            if let Some(t) = &links.declared_type {
                return Arc::clone(t);
            }
        }
        let resolved = self.resolve_alias(symbol);
        let t = self.get_declared_type_of_symbol(&resolved);
        self.declared_type_links.get_or_default(symbol).declared_type = Some(Arc::clone(&t));
        t
    }

    pub fn get_declared_type_of_enum(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_enum"); 
        if let Some(links) = self.declared_type_links.get(symbol) {
            if let Some(t) = &links.declared_type {
                return Arc::clone(t);
            }
        }
        let mut member_type_list: Vec<Arc<Type>> = vec![];
        for declaration in &symbol.declarations {
            if matches!(declaration.kind, SyntaxKind::EnumDeclaration) {
                for member in members(&declaration) {
                    if !has_dynamic_name(&member) {
                        let member_symbol = self.get_symbol_of_declaration(&member).unwrap_or_else(|| panic!("missing symbol"));
                        let value = self.get_enum_member_value(&member).value;
                        let member_type = match value {
                            Some(EvalValue::String(s)) => self.get_enum_literal_type(
                                EnumLiteralValue::Str(s),
                                symbol,
                                Some(Arc::clone(&member_symbol)),
                            ),
                            Some(EvalValue::Number(n)) => self.get_enum_literal_type(
                                EnumLiteralValue::Num(n.0),
                                symbol,
                                Some(Arc::clone(&member_symbol)),
                            ),
                            _ => self.create_computed_enum_type(&member_symbol),
                        };
                        self.declared_type_links
                            .get_or_default(&member_symbol)
                            .declared_type = Some(self.get_fresh_type_of_literal_type(&member_type));
                        member_type_list.push(member_type);
                    }
                }
            }
        }
        let enum_type = if !member_type_list.is_empty() {
            let alias = crate::checker::types::TypeAlias::new(
                Some(Arc::clone(symbol)),
                Vec::new(),
            );
            self.get_union_type_worker(member_type_list, UnionReduction::Literal, Some(&alias), None)
        } else {
            self.create_computed_enum_type(symbol)
        };
        self.declared_type_links.get_or_default(symbol).declared_type = Some(Arc::clone(&enum_type));
        enum_type
    }

    pub fn get_declared_type_of_enum_member(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_enum_member"); 
        if let Some(links) = self.declared_type_links.get(symbol) {
            if let Some(t) = &links.declared_type {
                return Arc::clone(t);
            }
        }
        let enum_type = self.get_declared_type_of_enum(&self.get_parent_of_symbol(symbol).unwrap_or_else(|| Arc::clone(symbol)));
        if self.declared_type_links.get(symbol).and_then(|l| l.declared_type.clone()).is_none() {
            self.declared_type_links.get_or_default(symbol).declared_type = Some(enum_type);
        }
        self.declared_type_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
            .unwrap_or_else(|| self.unknown_type())
    }

    pub fn get_declaring_constructor(&self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_declaring_constructor"); 
        for declaration in &symbol.declarations {
            let container = get_this_container(declaration, false, false);
            if ast::is_constructor_declaration(&container) {
                return Some(container);
            }
        }
        None
    }

    pub fn get_decorator_call_signature(&mut self, decorator: &Arc<Node>) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("get_decorator_call_signature"); 
        if self.legacy_decorators {
            return self.get_legacy_decorator_call_signature(decorator);
        }
        self.get_es_decorator_call_signature(decorator)
    }

    pub fn get_default_construct_signatures(&mut self, class_type: &Arc<Type>) -> Vec<Arc<Signature>> { ::tsox_core::fntrace::enter("get_default_construct_signatures"); 
        let base_constructor_type = self.get_base_constructor_type_of_class(class_type);
        let base_signatures = self.get_signatures_of_type(
            &base_constructor_type.unwrap_or_else(|| self.unknown_type()),
            SignatureKind::Construct,
        );
        let declaration = class_type
            .symbol()
            .and_then(|s| get_class_like_declaration_of_symbol(s));
        let is_abstract = declaration
            .as_ref()
            .map(|d| ast::has_syntactic_modifier(d, ast::ModifierFlags::Abstract))
            .unwrap_or(false);
        if base_signatures.is_empty() {
            let flags = if is_abstract {
                SignatureFlags::Construct | SignatureFlags::Abstract
            } else {
                SignatureFlags::Construct
            };
            let local_type_parameters = interface_local_type_parameters(class_type);
            return vec![self.new_signature(
                flags,
                None,
                &local_type_parameters,
                None,
                &[],
                class_type,
                None,
                0,
            )];
        }
        let base_type_node = get_base_type_node_of_class(class_type);
        let is_java_script = declaration
            .as_ref()
            .map(|d| ast::is_in_js_file(d))
            .unwrap_or(false);
        let type_arguments = base_type_node
            .as_ref()
            .map(|n| self.get_type_arguments_from_node(n))
            .unwrap_or_default();
        let type_arg_count = type_arguments.len() as i32;
        let mut result: Vec<Arc<Signature>> = vec![];
        for base_sig in &base_signatures {
            let min_type_argument_count = self.get_min_type_argument_count(&base_sig.type_parameters);
            let type_param_count = base_sig.type_parameters.len() as i32;
            if is_java_script || (type_arg_count >= min_type_argument_count as i32 && type_arg_count <= type_param_count) {
                let sig = if type_param_count != 0 {
                    let filled = self.fill_missing_type_arguments(
                        &type_arguments,
                        &base_sig.type_parameters,
                        min_type_argument_count,
                        is_java_script,
                    );
                    self.create_signature_instantiation(base_sig, &filled)
                } else {
                    self.clone_signature(base_sig)
                };
                let mut sig = Arc::clone(&sig);
                Arc::get_mut(&mut sig).map(|s| {
                    s.type_parameters = interface_local_type_parameters(class_type);
                    s.resolved_return_type = OnceLock::from(Arc::clone(class_type));
                    if is_abstract {
                        s.flags.insert(SignatureFlags::Abstract);
                    } else {
                        s.flags.remove(SignatureFlags::Abstract);
                    }
                });
                result.push(sig);
            }
        }
        result
    }

    pub fn get_definitely_falsy_part_of_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_definitely_falsy_part_of_type"); 
        if t.flags.intersects(TypeFlags::String) {
            return self.empty_string_type();
        }
        if t.flags.intersects(TypeFlags::Number) {
            return self.zero_type();
        }
        if t.flags.intersects(TypeFlags::BigInt) {
            return self.zero_big_int_type();
        }
        if Arc::ptr_eq(t, &self.regular_false_type())
            || Arc::ptr_eq(t, &self.false_type())
            || t.flags.intersects(
                TypeFlags::Void | TypeFlags::Undefined | TypeFlags::Null | TYPE_FLAGS_ANY_OR_UNKNOWN,
            )
            || (t.flags.intersects(TypeFlags::StringLiteral) && get_string_literal_value(t) == "")
            || (t.flags.intersects(TypeFlags::NumberLiteral) && get_number_literal_value(t) == 0.0)
            || (t.flags.intersects(TypeFlags::BigIntLiteral) && is_zero_big_int(t))
        {
            return Arc::clone(t);
        }
        self.never_type()
    }

    pub fn get_es_decorator_call_signature(&mut self, decorator: &Arc<Node>) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("get_es_decorator_call_signature"); 
        let Some(node) = decorator.parent() else {
            return None;
        };
        if self.signature_links.get(&node).map(|l| l.decorator_signature.is_some()).unwrap_or(false) {
            return self.es_decorator_signature_result(&node);
        }
        let any_sig = self.any_signature();
        self.signature_links.get_or_default(&node).decorator_signature = Some(any_sig);
        match node.kind {
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                let target_type = self.get_type_of_symbol(&self.get_symbol_of_declaration(&node).unwrap_or_else(|| panic!("missing symbol")));
                let context_type = self.new_class_decorator_context_type(&target_type);
                let sig = self.new_es_decorator_call_signature(&target_type, &context_type, &target_type);
                self.signature_links.get_or_default(&node).decorator_signature = Some(sig);
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                let Some(parent) = node.parent() else {
                    return self.es_decorator_signature_result(&node);
                };
                if !ast::is_class_like(&parent) {
                    return self.es_decorator_signature_result(&node);
                }
                let value_type = if ast::is_method_declaration(&node) {
                    self.get_signature_from_declaration(&node)
                        .map(|s| self.get_or_create_type_from_signature(&s))
                        .unwrap_or_else(|| self.unknown_type())
                } else {
                    self.get_type_of_node(&node)
                };
                let this_type = if ast::has_static_modifier(&node) {
                    let Some(parent) = node.parent() else {
                        return self.es_decorator_signature_result(&node);
                    };
                    self.get_type_of_symbol(&self.get_symbol_of_declaration(&parent).unwrap_or_else(|| panic!("missing symbol")))
                } else {
                    let Some(parent) = node.parent() else {
                        return self.es_decorator_signature_result(&node);
                    };
                    self.get_declared_type_of_class_or_interface(&self.get_symbol_of_declaration(&parent).unwrap_or_else(|| panic!("missing symbol")))
                };
                let target_type = if ast::is_get_accessor_declaration(&node) {
                    self.new_getter_function_type(&value_type)
                } else if ast::is_set_accessor_declaration(&node) {
                    self.new_setter_function_type(&value_type)
                } else {
                    Arc::clone(&value_type)
                };
                let context_type = self.new_class_member_decorator_context_type_for_node(&node, &this_type, &value_type);
                let sig = self.new_es_decorator_call_signature(&target_type, &context_type, &target_type);
                self.signature_links.get_or_default(&node).decorator_signature = Some(sig);
            }
            SyntaxKind::PropertyDeclaration => {
                let Some(parent) = node.parent() else {
                    return self.es_decorator_signature_result(&node);
                };
                if !ast::is_class_like(&parent) {
                    return self.es_decorator_signature_result(&node);
                }
                let value_type = self.get_type_of_node(&node);
                let this_type = if ast::has_static_modifier(&node) {
                    let Some(parent) = node.parent() else {
                        return self.es_decorator_signature_result(&node);
                    };
                    self.get_type_of_symbol(&self.get_symbol_of_declaration(&parent).unwrap_or_else(|| panic!("missing symbol")))
                } else {
                    let Some(parent) = node.parent() else {
                        return self.es_decorator_signature_result(&node);
                    };
                    self.get_declared_type_of_class_or_interface(&self.get_symbol_of_declaration(&parent).unwrap_or_else(|| panic!("missing symbol")))
                };
                let target_type = if ast::has_accessor_modifier(&node) {
                    self.new_class_accessor_decorator_target_type(&this_type, &value_type)
                } else {
                    self.undefined_type()
                };
                let return_type = if ast::has_accessor_modifier(&node) {
                    self.new_class_accessor_decorator_result_type(&this_type, &value_type)
                } else {
                    self.new_class_field_decorator_initializer_mutator_type(&this_type, &value_type)
                };
                let context_type = self.new_class_member_decorator_context_type_for_node(&node, &this_type, &value_type);
                let sig = self.new_es_decorator_call_signature(&target_type, &context_type, &return_type);
                self.signature_links.get_or_default(&node).decorator_signature = Some(sig);
            }
            _ => {}
        }
        self.es_decorator_signature_result(&node)
    }

    fn es_decorator_signature_result(&self, node: &Arc<Node>) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("es_decorator_signature_result"); 
        let links = self.signature_links.get(node)?;
        match &links.decorator_signature {
            Some(sig) => {
                if Arc::ptr_eq(sig, &self.any_signature()) {
                    None
                } else {
                    Some(Arc::clone(sig))
                }
            }
            None => None,
        }
    }

    pub fn get_effective_call_arguments(&mut self, node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_effective_call_arguments"); 
        if ast::is_jsx_opening_fragment(node) {
            let empty_fresh = self.empty_fresh_jsx_object_type();
            return vec![self.create_synthetic_expression(node, &empty_fresh, false, None)];
        }
        if ast::is_tagged_template_expression(node) {
            let template = tagged_template_expression_template(node);
            let template_strings_array = self.get_global_template_strings_array_type();
            let first_arg = self.create_synthetic_expression(&template, &template_strings_array, false, None);
            if !ast::is_template_expression(&template) {
                return vec![first_arg];
            }
            let spans = template_expression_template_spans(&template);
            let mut args: Vec<Arc<Node>> = Vec::with_capacity(spans.len() + 1);
            args.push(first_arg);
            for span in &spans {
                args.push(template_span_expression(span));
            }
            return args;
        }
        if ast::is_decorator(node) {
            return self.get_effective_decorator_arguments(node);
        }
        if ast::is_binary_expression(node) {
            return vec![binary_expression_left(node)];
        }
        if is_jsx_opening_like_element(node) {
            let attributes = attributes_arc(node);
            let has_properties = !properties(&attributes).is_empty();
            let parent_has_children = ast::is_jsx_opening_element(node)
                && node
                    .parent()
                    .map(|p| !members(&p).is_empty())
                    .unwrap_or(false);
            if has_properties || parent_has_children {
                return vec![attributes];
            }
            return vec![];
        }
        let args = arguments(node);
        let spread_index = self.get_spread_argument_index(args);
        if spread_index >= 0 {
            let mut effective_args: Vec<Arc<Node>> = args[..spread_index as usize].to_vec();
            for i in spread_index as usize..args.len() {
                let arg = &args[i];
                let mut spread_type: Option<Arc<Type>> = None;
                if ast::is_spread_element(arg) {
                    let arg_expr = arg.expression().expect("spread element expression");
                    if !self.flow_loop_stack.is_empty() {
                        spread_type = Some(self.check_expression_ex(arg_expr, CheckMode::Normal));
                    } else {
                        spread_type = Some(self.check_expression_cached(arg_expr));
                    }
                }
                if let Some(spread_type) = spread_type {
                    if is_tuple_type(&spread_type) {
                        if let Some(tuple_data) = spread_type.target_tuple_type() {
                            let element_infos = tuple_data.element_infos.clone();
                            for (j, t) in self.get_element_types(&spread_type).into_iter().enumerate() {
                                let flags = element_infos[j].flags;
                                let synthetic_type = if flags.intersects(ElementFlags::Rest) {
                                    self.create_array_type(Arc::clone(&t))
                                } else {
                                    Arc::clone(&t)
                                };
                                let synthetic_arg = self.create_synthetic_expression(
                                    arg,
                                    &synthetic_type,
                                    flags.intersects(ELEMENT_FLAGS_VARIABLE),
                                    element_infos[j].labeled_declaration.clone().as_ref(),
                                );
                                effective_args.push(synthetic_arg);
                            }
                            continue;
                        }
                    }
                }
                effective_args.push(Arc::clone(arg));
            }
            return effective_args;
        }
        args.to_vec()
    }

    pub fn get_element_type_of_slice_of_tuple_type(
        &mut self,
        t: &Arc<Type>,
        index: i32,
        end_skip_count: i32,
        writing: bool,
        no_reductions: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_element_type_of_slice_of_tuple_type"); 
        let length = self.get_type_reference_arity(t) - end_skip_count;
        let element_infos = t.target_tuple_type()?.element_infos.clone();
        if index < length {
            let type_arguments = self.get_type_arguments(t);
            let mut element_types: Vec<Arc<Type>> = vec![];
            for i in index..length {
                let mut e = Arc::clone(&type_arguments[i as usize]);
                if element_infos[i as usize].flags.intersects(ElementFlags::Variadic) {
                    e = self.get_indexed_access_type(&e, &self.number_type());
                }
                element_types.push(e);
            }
            if writing {
                return Some(self.get_intersection_type(element_types));
            }
            let reduction = if no_reductions {
                UnionReduction::None
            } else {
                UnionReduction::Literal
            };
            return Some(self.get_union_type_ex(element_types, reduction));
        }
        None
    }

    pub fn get_entity_name_for_decorator_metadata(&self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_entity_name_for_decorator_metadata"); 
        let node = node?;
        match node.kind {
            SyntaxKind::IntersectionType => {
                self.get_entity_name_for_decorator_metadata_from_type_list(&intersection_type_node_types(node))
            }
            SyntaxKind::UnionType => self.get_entity_name_for_decorator_metadata_from_type_list(&union_type_node_types(node)),
            SyntaxKind::ConditionalType => self.get_entity_name_for_decorator_metadata_from_type_list(&vec![
                conditional_type_node_true_type(node),
                conditional_type_node_false_type(node),
            ]),
            SyntaxKind::ParenthesizedType => {
                self.get_entity_name_for_decorator_metadata(parenthesized_type_node_type(node).as_ref())
            }
            SyntaxKind::NamedTupleMember => {
                self.get_entity_name_for_decorator_metadata(named_tuple_member_type(node).as_ref())
            }
            SyntaxKind::TypeReference => Some(Arc::clone(&type_reference_node_type_name(node))),
            _ => None,
        }
    }

    fn get_entity_name_for_decorator_metadata_from_type_list(&self, types: &[Arc<Node>]) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_entity_name_for_decorator_metadata_from_type_list"); 
        for t in types {
            if let Some(found) = self.get_entity_name_for_decorator_metadata(Some(t)) {
                return Some(found);
            }
        }
        None
    }
}

fn attributes_arc(node: &Node) -> Arc<Node> { ::tsox_core::fntrace::enter("attributes_arc"); 
    match &node.data {
        NodeData::JsxOpeningElement(d) => Arc::clone(&d.attributes),
        NodeData::JsxSelfClosingElement(d) => Arc::clone(&d.attributes),
        _ => panic!("AsJsxOpeningLikeElement on wrong node kind"),
    }
}

pub(crate) fn params_of(node: &Node) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("params_of"); 
    tsox_frontend::ast::mig::m3f_2::node_parameters(node)
        .map(|l| l.nodes.as_slice())
        .unwrap_or(&[])
}

pub(crate) fn is_tuple_type(t: &Type) -> bool { ::tsox_core::fntrace::enter("is_tuple_type"); 
    t.object_flags.contains(ObjectFlags::Tuple)
}

pub(crate) fn get_string_literal_value(t: &Type) -> String { ::tsox_core::fntrace::enter("get_string_literal_value"); 
    match &t.data {
        TypeData::Literal(l) => match &l.value {
            LiteralValue::String(s) => s.clone(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

pub(crate) fn get_number_literal_value(t: &Type) -> f64 { ::tsox_core::fntrace::enter("get_number_literal_value"); 
    match &t.data {
        TypeData::Literal(l) => match &l.value {
            LiteralValue::Number(n) => n.0,
            _ => 0.0,
        },
        _ => 0.0,
    }
}

pub(crate) fn interface_local_type_parameters(t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("interface_local_type_parameters"); 
    match &t.data {
        TypeData::Interface(i) => i.all_type_parameters[i
            .outer_type_parameter_count
            .min(i.all_type_parameters.len())..]
            .to_vec(),
        _ => Vec::new(),
    }
}
