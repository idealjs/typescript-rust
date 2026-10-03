#![allow(unused_imports)]

#[path = "r29k1_defs.rs"]
pub(crate) mod r29k1_defs;

use crate::checker::checker::*;
use crate::checker::checker_heritage_retry_limit::TypeResolutionProperty;
use crate::checker::mapper::{append_type_mapping, prepend_type_mapping};
use crate::checker::mig::m1f::WideningContext;
use crate::checker::mig::m1g_3::{get_unique_type_parameter_name, has_type_parameter_by_name};
use crate::checker::mig::m1g_5::get_union_key;
use crate::checker::mig::m2c_3::some_type;
use crate::checker::mig::m2d_3::get_flow_node_of_node;
use crate::checker::mig::m2h::r22k10_defs::set_flow_node_of;
use crate::checker::mig::m1a::r19k2_defs::*;
use crate::checker::mig::w9a::new_type_mapper;
use crate::checker::mig::wc2::r21k3_defs::*;
use crate::checker::mig::wc2::r22k3_defs::*;
use crate::checker::mig::wc2::r23k3_defs::*;
use crate::checker::mig::wc2::r24k8_defs::*;
use crate::checker::mig::wc2::interface_local_type_parameters;
use crate::checker::mig::wc2_2::get_type_instantiation_key;
use crate::checker::types::*;
use crate::checker::utilities_get_assignment_target::is_shorthand_ambient_module_symbol;
use crate::checker::utilities_has_only_expression_initialization::is_optional_declaration;
use crate::checker::utilities_is_private_within_ambient::is_private_within_ambient;
use crate::checker::utilities_token_is_identifier_or_keyword::{is_object_literal_type, is_unit_type};
use crate::checker::mig::wc1c::r24k17_defs::R24K17FactoryExt;
use crate::checker::mig::wc3::r26k2_defs::R26K2FactoryExt;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::mig::m3e_4::{for_each_return_statement, get_declaration_of_kind};
use tsox_frontend::ast::mig::m3f::get_rest_parameter_element_type;
use tsox_frontend::ast::mig::m3f_2::node_type_parameters;
use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
use tsox_frontend::ast::mig::w7a::is_expression_of_optional_chain_root;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn get_object_type_instantiation(
        &mut self,
        t: &Arc<Type>,
        m: Option<&TypeMapper>,
        alias: Option<&TypeAlias>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_object_type_instantiation"); 
        let declaration = if t.object_flags.intersects(OBJECT_FLAGS_REFERENCE) {
            t.as_type_reference().and_then(|d| d.node.clone())
        } else if t.object_flags.intersects(OBJECT_FLAGS_INSTANTIATION_EXPRESSION_TYPE) {
            t.as_instantiation_expression_type().and_then(|d| d.node.clone())
        } else {
            t.symbol().and_then(|s| s.declarations.first().cloned())
        };
        let Some(declaration) = declaration else { return Arc::clone(t) };
        let target = if t.object_flags.intersects(OBJECT_FLAGS_REFERENCE) {
            self.type_node_links.get(&declaration).and_then(|l| l.resolved_type.clone())
        } else if t.object_flags.intersects(OBJECT_FLAGS_INSTANTIATED) {
            t.target().cloned()
        } else {
            Some(Arc::clone(t))
        };
        let Some(target) = target else { return Arc::clone(t) };
        let mut type_parameters = self
            .type_node_links
            .get(&declaration)
            .map(|l| l.outer_type_parameters.clone())
            .unwrap_or_default();
        if type_parameters.is_empty() {
            let mut outer = self.get_outer_type_parameters(&declaration, true);
            if target.alias.as_ref().map_or(true, |a| a.type_arguments.is_empty()) {
                if t.object_flags.intersects(OBJECT_FLAGS_REFERENCE | OBJECT_FLAGS_INSTANTIATION_EXPRESSION_TYPE) {
                    outer.retain(|tp| self.is_type_parameter_possibly_referenced(tp, &declaration));
                } else if t
                    .symbol()
                    .is_some_and(|s| s.flags.intersects(SymbolFlags::Method | SymbolFlags::TypeLiteral))
                {
                    let decls = t.symbol().map(|s| s.declarations.clone()).unwrap_or_default();
                    outer.retain(|tp| decls.iter().any(|d| self.is_type_parameter_possibly_referenced(tp, d)));
                }
            }
            self.type_node_links.get_or_default(&declaration).outer_type_parameters = outer.clone();
            type_parameters = outer;
        }
        if type_parameters.is_empty() {
            return Arc::clone(t);
        }
        let m = m.map(|mapper| Arc::new(mapper.clone()));
        let type_arguments: Vec<Arc<Type>> = type_parameters
            .iter()
            .map(|tp| match (&m, t.mapper()) {
                (Some(m2), m1) => self.map_type_with_composite_mapper(tp, m1, m2),
                (None, Some(m1)) => {
                    let t1 = m1.map(tp);
                    if !Arc::ptr_eq(&t1, tp) {
                        self.instantiate_type(&t1, None)
                    } else {
                        Arc::clone(tp)
                    }
                }
                (None, None) => Arc::clone(tp),
            })
            .collect();
        let new_alias = match alias {
            Some(a) => Some(a.clone()),
            None => self.instantiate_type_alias(t.alias.as_deref(), m.as_ref()),
        };
        let key = get_type_instantiation_key(
            &type_arguments,
            new_alias.as_ref(),
            t.object_flags.intersects(OBJECT_FLAGS_SINGLE_SIGNATURE_TYPE),
        );
        let instantiations_empty = r29k1_defs::object_type_instantiations_is_empty(&target);
        if instantiations_empty {
            let bootstrap_key = get_type_instantiation_key(&type_parameters, target.alias.as_deref(), false);
            r29k1_defs::object_type_instantiations_insert(&target, bootstrap_key, Arc::clone(&target));
        }
        if let Some(cached) = r29k1_defs::object_type_instantiations_get(&target, &key) {
            return cached;
        }
        let mut new_mapper = Some(Arc::new(new_type_mapper(type_parameters, type_arguments)));
        if target.object_flags.intersects(OBJECT_FLAGS_SINGLE_SIGNATURE_TYPE) && m.is_some() {
            new_mapper = self.combine_type_mappers(new_mapper.as_ref(), m.as_ref());
        }
        let result = if target.object_flags.intersects(OBJECT_FLAGS_REFERENCE) {
            match target.target() {
                Some(target_target) => self.create_deferred_type_reference(
                    target_target,
                    target.as_type_reference().and_then(|d| d.node.as_ref()),
                    new_mapper.as_ref(),
                    new_alias,
                ),
                None => panic!("reference type requires a target"),
            }
        } else if target.object_flags.intersects(OBJECT_FLAGS_MAPPED) {
            self.instantiate_mapped_type(&target, new_mapper.as_ref(), new_alias.as_ref())
        } else {
            self.instantiate_anonymous_type(&target, new_mapper.as_ref(), new_alias.as_ref())
        };
        r29k1_defs::object_type_instantiations_insert(&target, key, Arc::clone(&result));
        result
    }

    pub fn get_optional_call_signature(
        &mut self,
        signature: &Arc<Signature>,
        call_chain_flags: SignatureFlags,
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("get_optional_call_signature"); 
        if signature.flags & SignatureFlags::CallChainFlags == call_chain_flags {
            return Arc::clone(signature);
        }
        let key = CachedSignatureKey {
            sig: Arc::clone(signature),
            key: if call_chain_flags == SignatureFlags::IsInnerCallChain {
                SignatureKey::Inner
            } else {
                SignatureKey::Outer
            },
        };
        if let Some(cached) = self.cached_signatures.get(&key) {
            return Arc::clone(cached);
        }
        let mut result = self.clone_signature(signature);
        Arc::get_mut(&mut result).map(|s| s.flags.insert(call_chain_flags));
        self.cached_signatures.insert(key, Arc::clone(&result));
        result
    }

    pub fn get_optional_expression_type(&mut self, expr_type: &Arc<Type>, expression: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_optional_expression_type"); 
        if is_expression_of_optional_chain_root(expression) {
            return self.get_non_nullable_type(expr_type);
        }
        if ast::is_optional_chain(expression) {
            return self.remove_optional_type_marker(expr_type);
        }
        Arc::clone(expr_type)
    }

    pub fn get_or_create_substitution_type(&mut self, base_type: &Arc<Type>, constraint: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_or_create_substitution_type"); 
        let key = SubstitutionTypeKey {
            base_id: base_type.id(),
            constraint_id: constraint.id(),
        };
        if let Some(cached) = self.substitution_types.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.new_substitution_type(base_type, constraint);
        self.substitution_types.insert(key, Arc::clone(&result));
        result
    }

    pub fn get_or_create_type_from_signature(&mut self, sig: &Arc<Signature>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_or_create_type_from_signature"); 
        if let Some(existing) = sig.isolated_signature_type.get().cloned() {
            return existing;
        }
        let kind = sig.declaration.as_ref().map(|d| d.kind);
        let is_constructor = matches!(
            kind,
            None | Some(SyntaxKind::Unknown) | Some(SyntaxKind::Constructor)
                | Some(SyntaxKind::ConstructSignature) | Some(SyntaxKind::ConstructorType)
        );
        let symbol = sig
            .declaration
            .as_ref()
            .and_then(|d| self.get_symbol_of_declaration(d));
        let t = self.new_object_type(ObjectFlags::Anonymous | ObjectFlags::SingleSignatureType, symbol);
        if is_constructor {
            self.set_structured_type_members(&t, None, vec![], vec![Arc::clone(sig)], vec![]);
        } else {
            self.set_structured_type_members(&t, None, vec![Arc::clone(sig)], vec![], vec![]);
        }
        Arc::get_mut(&mut Arc::clone(&sig)).map(|_| ());
        let _ = sig.isolated_signature_type.set(Arc::clone(&t));
        t
    }

    pub fn get_outer_inference_type_parameters(&self) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_outer_inference_type_parameters"); 
        let mut result = vec![];
        for info in &self.inference_context_infos {
            if let Some(context) = &info.context {
                for inference in &context.inferences {
                    result.push(Arc::clone(&inference.type_parameter));
                }
            }
        }
        result
    }

    pub fn get_outer_type_parameters(&mut self, node: &Arc<Node>, include_this_types: bool) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_outer_type_parameters"); 
        let Some(parent) = node.parent() else {
            return vec![];
        };
        let kind = parent.kind;
        match kind {
            SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::MethodSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::MappedType
            | SyntaxKind::ConditionalType => {}
            _ => return vec![],
        }
        let mut outer_type_parameters = self.get_outer_type_parameters(&parent, include_this_types);
        if (matches!(kind, SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction) || ast::is_object_literal_method(&parent))
            && self.is_context_sensitive(&parent)
        {
            let symbol_type = self.get_type_of_symbol(&self.get_symbol_of_declaration(&parent).unwrap_or_else(|| panic!("missing symbol")));
            let signatures = self.get_signatures_of_type(&symbol_type, SignatureKind::Call);
            if let Some(signature) = signatures.first() {
                if !signature.type_parameters.is_empty() {
                    outer_type_parameters.extend(signature.type_parameters.iter().cloned());
                }
            }
        }
        if kind == SyntaxKind::MappedType {
            let type_parameter = mapped_type_node_type_parameter(&parent);
            let declared = self.get_declared_type_of_type_parameter(&self.get_symbol_of_declaration(&type_parameter).unwrap_or_else(|| panic!("missing symbol")));
            outer_type_parameters.push(declared);
            return outer_type_parameters;
        }
        if kind == SyntaxKind::ConditionalType {
            outer_type_parameters.extend(self.get_infer_type_parameters(&parent));
            return outer_type_parameters;
        }
        let own_type_parameters = node_type_parameters(&parent)
            .map(|list| list.nodes.clone())
            .unwrap_or_default();
        let mut outer_and_own = self.append_type_parameters(outer_type_parameters, own_type_parameters);
        if include_this_types
            && matches!(
                kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression | SyntaxKind::InterfaceDeclaration
            )
        {
            let declared = self.get_declared_type_of_class_or_interface(&self.get_symbol_of_declaration(&parent).unwrap_or_else(|| panic!("missing symbol")));
            if let Some(this_type) = interface_this_type(&declared) {
                outer_and_own.push(this_type);
            }
        }
        outer_and_own
    }

    pub fn get_outer_type_parameters_of_class_or_interface(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_outer_type_parameters_of_class_or_interface"); 
        let Some(declaration) = self.get_class_or_interface_like_declaration(symbol) else {
            return vec![];
        };
        self.get_outer_type_parameters(&declaration, false)
    }

    pub fn get_parameter_type_node_for_decorator_check(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_parameter_type_node_for_decorator_check"); 
        let type_node = node.type_();
        if parameter_declaration_dot_dot_dot_token(node).is_some() {
            return get_rest_parameter_element_type(type_node);
        }
        type_node.cloned()
    }

    pub fn get_parameter_type_of_full_signature(
        &mut self,
        node: &Arc<Node>,
        parameter: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_parameter_type_of_full_signature"); 
        let signature = self.get_signature_of_full_signature_type(node)?;
        let pos = node
            .parameters()
            .and_then(|list| list.nodes.iter().position(|p| Arc::ptr_eq(p, parameter)))
            .unwrap_or(0);
        if parameter_declaration_dot_dot_dot_token(parameter).is_some() {
            self.get_rest_type_at_position(&signature, pos)
        } else {
            Some(self.get_type_at_position(&signature, pos))
        }
    }

    pub fn get_parent_element_access(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_parent_element_access"); 
        let Some(ancestor) = node.parent().and_then(|p| p.parent()) else {
            return None;
        };
        match ancestor.kind {
            SyntaxKind::BindingElement | SyntaxKind::PropertyAssignment => {
                self.get_synthetic_element_access(&ancestor)
            }
            SyntaxKind::ArrayLiteralExpression => {
                let parent = node.parent().unwrap_or_else(|| panic!("parent should exist"));
                self.get_synthetic_element_access(&parent)
            }
            SyntaxKind::VariableDeclaration => ancestor.initializer().cloned(),
            SyntaxKind::BinaryExpression => binary_expression_right(&ancestor),
            _ => None,
        }
    }

    pub fn get_resolved_symbol_or_nil(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_resolved_symbol_or_nil"); 
        self.symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone())
    }

    pub fn get_synthetic_element_access(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_synthetic_element_access"); 
        let parent_access = self.get_parent_element_access(node)?;
        if get_flow_node_of_node(&parent_access).is_some() {
            if let Some(prop_name) = self.get_destructuring_property_name(node) {
                let mut literal = self.factory.new_string_literal(&prop_name, 0);
                if let Some(literal) = Arc::get_mut(&mut literal) {
                    literal.loc = node.loc;
                }
                let lhs_expr = if !ast::is_left_hand_side_expression(&parent_access) {
                    let mut parenthesized = self.factory.new_parenthesized_expression(&parent_access);
                    if let Some(parenthesized) = Arc::get_mut(&mut parenthesized) {
                        parenthesized.loc = node.loc;
                    }
                    parenthesized
                } else {
                    parent_access.clone()
                };
                let mut result = self
                    .factory
                    .new_element_access_expression(&lhs_expr, None, &literal, ast::NodeFlags::default());
                if let Some(result) = Arc::get_mut(&mut result) {
                    result.loc = node.loc;
                }
                literal.set_parent(&result);
                result.set_parent(node);
                if !Arc::ptr_eq(&lhs_expr, &parent_access) {
                    lhs_expr.set_parent(&result);
                }
                set_flow_node_of(&result, get_flow_node_of_node(&parent_access));
                return Some(result);
            }
        }
        None
    }

    pub fn get_type_of_accessors(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_accessors"); 
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(t) = &links.resolved_type {
                return Arc::clone(t);
            }
        }
        if !self.push_type_resolution(Arc::as_ptr(symbol), TypeResolutionProperty::Type) {
            return self.error_type();
        }
        let getter = get_declaration_of_kind(symbol, SyntaxKind::GetAccessor);
        let setter = get_declaration_of_kind(symbol, SyntaxKind::SetAccessor);
        let accessor = symbol
            .declarations
            .iter()
            .find(|d| is_auto_accessor_property_declaration(d))
            .cloned();
        let mut t = getter.as_ref().and_then(|g| self.get_annotated_accessor_type(g));
        if t.is_none() {
            t = setter.as_ref().and_then(|s| self.get_annotated_accessor_type(s));
        }
        if t.is_none() {
            t = accessor.as_ref().and_then(|a| self.get_annotated_accessor_type(a));
        }
        if t.is_none() {
            if let Some(getter) = &getter {
                if getter.body().is_some() {
                    t = Some(self.get_return_type_from_body(getter, CheckMode::Normal));
                }
            }
        }
        if t.is_none() {
            if let Some(accessor) = &accessor {
                t = self.get_widened_type_for_variable_like_declaration(accessor, true);
            }
        }
        if t.is_none() {
            if let Some(setter) = &setter {
                if !is_private_within_ambient(setter) {
                    let symbol_name = self.symbol_to_string(symbol);
                    self.error_or_suggestion_message(
                        self.no_implicit_any,
                        setter,msg::PROPERTY_0_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_ITS_SET_ACCESSOR_LACKS_A_PARAMETER_TYPE_ANNOTATION,
                        &[symbol_name],
                    );
                }
            } else if let Some(getter) = &getter {
                if !is_private_within_ambient(getter) {
                    let symbol_name = self.symbol_to_string(symbol);
                    self.error_or_suggestion_message(
                        self.no_implicit_any,
                        getter,msg::PROPERTY_0_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_ITS_GET_ACCESSOR_LACKS_A_RETURN_TYPE_ANNOTATION,
                        &[symbol_name],
                    );
                }
            }
            t = Some(self.any_type());
        }
        if !self.pop_type_resolution() {
            if let Some(getter) = &getter {
                if self.get_annotated_accessor_type_node(getter).is_some() {
                    let symbol_name = self.symbol_to_string(symbol);
                    self.error_message(getter,msg::X_0_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_TYPE_ANNOTATION, &[symbol_name]);
                    t = Some(self.any_type());
                }
            }
        }
        let t = t.unwrap_or_else(|| self.any_type());
        if self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
            .is_none()
        {
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
        }
        self.value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
            .unwrap_or(t)
    }

    pub fn get_type_of_alias(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_alias"); 
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(t) = &links.resolved_type {
                return Arc::clone(t);
            }
        }
        if !self.push_type_resolution(Arc::as_ptr(symbol), TypeResolutionProperty::Type) {
            return self.error_type();
        }
        let target_symbol = self.resolve_alias(symbol);
        if self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
            .is_none()
        {
            let resolved = if self
                .get_symbol_flags(&target_symbol)
                .intersects(SymbolFlags::VALUE)
            {
                self.get_type_of_symbol(&target_symbol)
            } else {
                self.error_type()
            };
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(resolved);
        }
        if !self.pop_type_resolution() {
            self.report_circularity_error(symbol);
            if self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
                .is_none()
            {
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(self.error_type());
            }
        }
        self.value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
            .unwrap_or_else(|| self.error_type())
    }

    pub fn get_type_of_concrete_property_of_contextual_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_of_concrete_property_of_contextual_type"); 
        let prop = self.get_property_of_type(t, &name.to_string())?;
        if self.is_circular_mapped_property(&prop) {
            return None;
        }
        let optional = prop.flags.intersects(SymbolFlags::Optional);
        let prop_type = self.get_type_of_symbol(&prop);
        Some(self.remove_missing_type(prop_type, optional))
    }

    pub fn get_type_of_func_class_enum_module_worker(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_func_class_enum_module_worker"); 
        if symbol.flags.intersects(SymbolFlags::MODULE) && is_shorthand_ambient_module_symbol(symbol) {
            return self.any_type();
        }
        if symbol.flags.intersects(SymbolFlags::ValueModule) {
            if let Some(value_declaration) = &symbol.value_declaration {
                if ast::is_source_file(value_declaration)
                    && self.source_file_common_js_module_indicator(value_declaration).is_some()
                {
                    let resolved_module = self.resolve_external_module_symbol(symbol, false);
                    if !Arc::ptr_eq(&resolved_module, symbol) {
                        return self.get_type_of_symbol(&resolved_module);
                    }
                }
            }
        }
        let t = self.new_object_type(ObjectFlags::Anonymous, Some(Arc::clone(symbol)));
        if symbol.flags.intersects(SymbolFlags::Class) {
            if let Some(base_type_variable) = self.get_base_type_variable_of_class(symbol) {
                return self.get_intersection_type(vec![t, base_type_variable]);
            }
            return t;
        }
        if self.strict_null_checks && symbol.flags.intersects(SymbolFlags::Optional) {
            return self.get_optional_type(Arc::clone(&t));
        }
        t
    }

    pub fn get_type_of_mapped_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_mapped_symbol"); 
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(t) = &links.resolved_type {
                return Arc::clone(t);
            }
        }
        let mapped_type = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.containing_type.clone())
            .unwrap_or_else(|| self.error_type());
        if !self.push_type_resolution(Arc::as_ptr(symbol), TypeResolutionProperty::Type) {
            self.set_mapped_type_contains_error(&mapped_type, true);
            return self.error_type();
        }
        let target = mapped_type_target(&mapped_type).unwrap_or_else(|| Arc::clone(&mapped_type));
        let template_type = self
            .get_template_type_from_mapped_type(&target)
            .unwrap_or_else(|| self.error_type());
        let key_type = self
            .mapped_symbol_links
            .get(symbol)
            .and_then(|l| l.key_type.clone())
            .unwrap_or_else(|| self.error_type());
        let type_parameter = self
            .get_type_parameter_from_mapped_type(&mapped_type)
            .unwrap_or_else(|| self.unknown_type());
        let mapper = Arc::new(append_type_mapping(
            mapped_type.mapper().map(|v| &**v),
            type_parameter,
            key_type,
        ));
        let mut prop_type = self.instantiate_type(&template_type, Some(&mapper));
        if self.strict_null_checks
            && symbol.flags.intersects(SymbolFlags::Optional)
            && !self.maybe_type_of_kind(&prop_type, TypeFlags::Undefined | TypeFlags::Void)
        {
            prop_type = self.get_optional_type(prop_type);
        } else if symbol.check_flags.intersects(ast::CheckFlags::StripOptional) {
            prop_type = self.remove_missing_or_undefined_type(&prop_type);
        }
        if self.pop_type_resolution() {
            if self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
                .is_none()
            {
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&prop_type));
            }
        } else {
            if self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
                .is_none()
            {
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(self.error_type());
            }
            let current_node = self.current_node();
            if let Some(current_node) = current_node {
                let symbol_str = self.symbol_to_string(symbol);
                let mapped_type_str = self.type_to_string(&mapped_type);
                self.error_message(
                    &current_node,msg::TYPE_OF_PROPERTY_0_CIRCULARLY_REFERENCES_ITSELF_IN_MAPPED_TYPE_1,
                    &[symbol_str, mapped_type_str],
                );
            }
        }
        self.value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
            .unwrap_or(prop_type)
    }

    pub fn get_type_of_parameter(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_parameter"); 
        let declaration = symbol.value_declaration.clone();
        let has_initializer_or_optional = declaration
            .as_ref()
            .map(|d| d.initializer().is_some() || is_optional_declaration(d))
            .unwrap_or(false);
        let symbol_type = self.get_type_of_symbol(symbol);
        self.add_optionality_ex(&symbol_type, false, has_initializer_or_optional)
    }

    pub fn get_type_of_property_in_base_class(&mut self, property: &Arc<Symbol>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_of_property_in_base_class"); 
        let class_type = self.get_declaring_class(property)?;
        let base_class_types = self.get_base_types(&class_type);
        if !base_class_types.is_empty() {
            return self.get_type_of_property_of_type(&base_class_types[0], &property.name);
        }
        None
    }

    pub fn get_type_of_property_or_index_signature_of_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_of_property_or_index_signature_of_type"); 
        if let Some(prop_type) = self.get_type_of_property_of_type(t, &name.to_string()) {
            return Some(prop_type);
        }
        if let Some(index_info) = self.get_applicable_index_info_for_name(t, &name.to_string()) {
            return index_info
            .value_type
            .as_ref()
            .map(|v| self.add_optionality_ex(v, true, true));
        }
        None
    }

    pub fn get_type_of_variable_or_parameter_or_property(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_variable_or_parameter_or_property"); 
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(t) = &links.resolved_type {
                return Arc::clone(t);
            }
        }
        let t = self.get_type_of_variable_or_parameter_or_property_worker(symbol);
        if self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
            .is_none()
            && !self.is_parameter_of_context_sensitive_signature(symbol)
        {
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
        }
        t
    }

    pub fn get_type_of_variable_or_parameter_or_property_worker(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_variable_or_parameter_or_property_worker"); 
        if symbol.flags.intersects(SymbolFlags::Prototype) {
            return self.get_type_of_prototype_property(symbol);
        }
        if let Some(require_symbol) = self.require_symbol() {
            if Arc::ptr_eq(symbol, &require_symbol) {
                return self.any_type();
            }
        }
        let declaration = symbol
            .value_declaration
            .clone()
            .unwrap_or_else(|| panic!("ValueDeclaration should be defined"));
        if ast::is_source_file(&declaration) {
            if let Some(file) = self.get_source_file_of_node(&declaration) {
                if tsox_frontend::ast::is_json_source_file(&file) {
                    let statements = tsox_frontend::ast::mig::m3c::statements(&file.node);
                    if statements.is_empty() {
                        return self.empty_object_type();
                    }
                    let expression = statements[0].expression().unwrap_or(&statements[0]);
                    let expr_type = self.check_expression_ex(&expression, CheckMode::Normal);
                    let widened_literal = self.get_widened_literal_type(&expr_type);
                    return self.get_widened_type(&widened_literal);
                }
            }
        }
        if !self.push_type_resolution(Arc::as_ptr(symbol), TypeResolutionProperty::Type) {
            return self.report_circularity_error(symbol);
        }
        if symbol.flags.intersects(SymbolFlags::ModuleExports) {
            if symbol.name == "exports" {
                let decl_symbol = self.get_symbol_of_declaration(&declaration).unwrap_or_else(|| panic!("missing symbol"));
                return self.get_type_of_symbol(&self.resolve_external_module_symbol(&decl_symbol, false));
            }
            return self.new_anonymous_type(symbol, symbol.members.clone(), vec![], vec![], vec![]);
        }
        let result = match declaration.kind {
            SyntaxKind::Parameter
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::BindingElement => {
                let variable_type =
                    self.get_type_for_variable_like_declaration(&declaration, true, CheckMode::Normal);
                let widen_in_context = !self.is_parameter_of_context_sensitive_signature(symbol);
                self.widen_type_for_variable_like_declaration(
                    variable_type,
                    &declaration,
                    widen_in_context,
                )
            }
            SyntaxKind::PropertyAssignment => self.check_property_assignment(&declaration, CheckMode::Normal),
            SyntaxKind::ShorthandPropertyAssignment => {
                self.check_shorthand_property_assignment(&declaration, true, CheckMode::Normal)
            }
            SyntaxKind::MethodDeclaration => self.check_object_literal_method(&declaration, CheckMode::Normal),
            SyntaxKind::ExportAssignment => match declaration.type_() {
                Some(type_node) => self.get_type_from_type_node(&type_node),
                None => {
                    let expression = declaration.expression().unwrap_or(&declaration);
                    let expr_type = self.check_expression_cached(&expression);
                    self.widen_type_for_variable_like_declaration(Some(expr_type), &declaration, false)
                }
            },
            SyntaxKind::BinaryExpression | SyntaxKind::CallExpression => {
                self.get_widened_type_for_assignment_declaration(symbol)
            }
            SyntaxKind::JsxAttribute => self.check_jsx_attribute_with_mode(&declaration, CheckMode::Normal),
            SyntaxKind::EnumMember => self.get_type_of_enum_member(symbol),
            _ => panic!("Unhandled case in getTypeOfVariableOrParameterOrPropertyWorker"),
        };
        if !self.pop_type_resolution() {
            return self.report_circularity_error(symbol);
        }
        result
    }

    pub fn get_type_only_alias_declaration_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_only_alias_declaration_ex"); 
        let mut symbol = Arc::clone(symbol);
        while symbol.flags.intersects(SymbolFlags::Alias) && !symbol.flags.intersects(meaning) {
            let resolved = self.resolve_alias(&symbol);
            if let Some(links) = self.alias_symbol_links.get(&symbol) {
                if let Some(decl) = &links.type_only_declaration {
                    return Some(Arc::clone(decl));
                }
            }
            if Arc::ptr_eq(&resolved, &symbol) {
                break;
            }
            symbol = resolved;
        }
        None
    }

    pub fn get_type_only_declaration_of_entity_name(&mut self, name: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_only_declaration_of_entity_name"); 
        let symbol = self.resolve_entity_name(
            name,
            SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
            true,
            true,
            None,
        )?;
        self.get_type_only_alias_declaration(&symbol)
    }

    pub fn get_type_parameters_for_type_and_symbol(
        &mut self,
        t: &Arc<Type>,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_parameters_for_type_and_symbol"); 
        if !self.is_error_type(t) {
            if symbol.flags.intersects(SymbolFlags::TypeAlias) {
                if let Some(links) = self.type_alias_links.get(symbol) {
                    if !links.type_parameters.is_empty() {
                        return links.type_parameters.clone();
                    }
                }
            }
            if t.object_flags.intersects(OBJECT_FLAGS_REFERENCE) {
                if let Some(target) = t.target() {
                    return interface_local_type_parameters(target);
                }
            }
        }
        vec![]
    }

    pub fn get_type_parameters_for_type_reference_or_import(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_parameters_for_type_reference_or_import"); 
        let t = self.get_type_from_type_node(node);
        if !self.is_error_type(&t) {
            if let Some(symbol) = self.get_resolved_symbol_or_nil(node) {
                return self.get_type_parameters_for_type_and_symbol(&t, &symbol);
            }
        }
        vec![]
    }

    pub fn get_type_parameters_from_declaration(&mut self, declaration: &Arc<Node>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_parameters_from_declaration"); 
        if let Some(sig) = self.get_signature_of_full_signature_type(declaration) {
            return sig.type_parameters.clone();
        }
        let mut result: Vec<Arc<Type>> = vec![];
        let own_type_parameters = node_type_parameters(declaration)
            .map(|list| list.nodes.clone())
            .unwrap_or_default();
        for node in &own_type_parameters {
            let symbol = self.get_symbol_of_declaration(node).unwrap_or_else(|| panic!("missing symbol"));
            let declared = self.get_declared_type_of_type_parameter(&symbol);
            if !result.iter().any(|t| Arc::ptr_eq(t, &declared)) {
                result.push(declared);
            }
        }
        result
    }

    pub fn get_type_reference_arity(&self, t: &Arc<Type>) -> i32 { ::tsox_core::fntrace::enter("get_type_reference_arity"); 
        target_interface_type_parameters(t).len() as i32
    }

    pub fn get_type_with_facts(&mut self, t: &Arc<Type>, include: TypeFacts) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_with_facts"); 
        filter_type_self(self, t, |c, t| c.has_type_facts(t, include))
    }

    pub fn get_type_with_synthetic_default_import_type(
        &mut self,
        t: &Arc<Type>,
        symbol: &Arc<Symbol>,
        original_symbol: &Arc<Symbol>,
        module_specifier: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_with_synthetic_default_import_type"); 
        if !self.is_error_type(t) {
            let key = CachedTypeKey {
                kind: CachedTypeKind::SyntheticType,
                type_id: t.id(),
            };
            if let Some(cached) = self.cached_types.get(&key) {
                return Arc::clone(cached);
            }
            let file = original_symbol
                .declarations
                .iter()
                .find(|d| ast::is_source_file(d))
                .cloned();
            let has_synthetic_default = match module_specifier {
                Some(specifier) => self.can_have_synthetic_default(file.as_ref(), original_symbol, false, specifier),
                None => false,
            };
            let synthetic_type = if has_synthetic_default {
                let anonymous_symbol = self.new_symbol(SymbolFlags::TypeLiteral, internal_symbol_name_type());
                let mut anonymous_symbol = anonymous_symbol;
                if let Some(s) = Arc::get_mut(&mut anonymous_symbol) {
                    s.declarations = original_symbol.declarations.clone();
                }
                let default_containing_object =
                    self.create_default_property_wrapper_for_module(symbol, original_symbol, Some(&anonymous_symbol));
                self.value_symbol_links
                    .get_or_default(&anonymous_symbol)
                    .resolved_type = Some(Arc::clone(&default_containing_object));
                if self.is_valid_spread_type(t) {
                    self.get_spread_type(
                        t,
                        &default_containing_object,
                        Some(Arc::clone(&anonymous_symbol)),
                        ObjectFlags::None,
                        false,
                    )
                } else {
                    default_containing_object
                }
            } else {
                Arc::clone(t)
            };
            self.cached_types.insert(key, Arc::clone(&synthetic_type));
            return synthetic_type;
        }
        Arc::clone(t)
    }

    pub fn get_type_with_this_argument(
        &mut self,
        t: &Arc<Type>,
        this_argument: Option<&Arc<Type>>,
        need_apparent_type: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_with_this_argument"); 
        if t.object_flags.intersects(OBJECT_FLAGS_REFERENCE) {
            let Some(target) = t.target() else {
                return Arc::clone(t);
            };
            let type_arguments = self.get_type_arguments(t);
            if target_interface_type_parameters(target).len() == type_arguments.len() {
                let this_argument = match this_argument {
                    Some(a) => Arc::clone(a),
                    None => interface_this_type(target).unwrap_or_else(|| self.any_type()),
                };
                let mut args = type_arguments;
                args.push(this_argument);
                return self.create_type_reference(target, &args);
            }
            return Arc::clone(t);
        }
        if t.flags.intersects(TypeFlags::Intersection) {
            let types = t.types().unwrap_or(&[]).to_vec();
            let new_types: Vec<Arc<Type>> = types
                .iter()
                .map(|u| self.get_type_with_this_argument(u, this_argument, need_apparent_type))
                .collect();
            if same_types(&new_types, &types) {
                return Arc::clone(t);
            }
            return self.get_intersection_type(new_types);
        }
        if need_apparent_type {
            return self.get_apparent_type(t);
        }
        Arc::clone(t)
    }

    pub fn get_unary_result_type(&mut self, operand_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_unary_result_type"); 
        if self.maybe_type_of_kind(operand_type, TYPE_FLAGS_BIG_INT_LIKE) {
            if self.is_type_assignable_to_kind(operand_type, TYPE_FLAGS_ANY_OR_UNKNOWN)
                || self.maybe_type_of_kind(operand_type, TYPE_FLAGS_NUMBER_LIKE)
            {
                return self.number_or_big_int_type();
            }
            return self.bigint_type();
        }
        self.number_type()
    }

    pub fn get_undefined_property(&mut self, prop: &Arc<Symbol>) -> Arc<Symbol> { ::tsox_core::fntrace::enter("get_undefined_property"); 
        if let Some(cached) = self.undefined_properties.get(&prop.name) {
            return Arc::clone(cached);
        }
        let mut result = self.create_symbol_with_type(prop, &self.undefined_or_missing_type());
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.flags.insert(SymbolFlags::Optional);
        }
        self.undefined_properties.insert(prop.name.clone(), Arc::clone(&result));
        result
    }

    pub fn get_union_or_intersection_type(
        &mut self,
        types: Vec<Arc<Type>>,
        is_union: bool,
        union_reduction: UnionReduction,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_union_or_intersection_type"); 
        if is_union {
            self.get_union_type_ex(types, union_reduction)
        } else {
            self.get_intersection_type(types)
        }
    }

    pub fn get_union_type_from_sorted_list(
        &mut self,
        types: Vec<Arc<Type>>,
        precomputed_object_flags: ObjectFlags,
        alias: Option<&TypeAlias>,
        origin: Option<&Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_union_type_from_sorted_list"); 
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().unwrap();
        }
        let key = get_union_key(&types, origin, alias);
        if let Some(t) = self.union_types.get(&key) {
            return Arc::clone(t);
        }
        let mut t = self.new_union_type(
            precomputed_object_flags | self.get_propagating_flags_of_types(&types, TYPE_FLAGS_NULLABLE),
            &types,
        );
        Arc::get_mut(&mut t).map(|u| {
            u.set_union_origin(origin.cloned());
            u.set_alias(alias.cloned());
            if types.len() == 2
                && types[0].flags.intersects(TypeFlags::BooleanLiteral)
                && types[1].flags.intersects(TypeFlags::BooleanLiteral)
            {
                u.flags.insert(TypeFlags::Boolean);
            }
            if let Some(alias_symbol) = alias
                .as_ref()
                .and_then(|a| a.symbol.as_ref())
                .filter(|s| s.flags.intersects(SymbolFlags::ENUM))
            {
                u.flags.insert(TypeFlags::EnumLiteral);
                u.symbol = Some(Arc::clone(alias_symbol));
            }
        });
        self.union_types.insert(key, Arc::clone(&t));
        t
    }

    pub fn get_union_type_worker(
        &mut self,
        types: Vec<Arc<Type>>,
        union_reduction: UnionReduction,
        alias: Option<&TypeAlias>,
        origin: Option<&Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_union_type_worker"); 
        let (mut type_set, includes) = self.add_types_to_union(&types);
        if union_reduction != UnionReduction::None {
            if includes.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN) {
                if includes.intersects(TypeFlags::Any) {
                    if includes.intersects(TYPE_FLAGS_INCLUDES_WILDCARD) {
                        return self.wildcard_type();
                    }
                    if includes.intersects(TYPE_FLAGS_INCLUDES_ERROR) {
                        return self.error_type();
                    }
                    return self.any_type();
                }
                return self.unknown_type();
            }
            if includes.intersects(TypeFlags::Undefined) {
                if type_set.len() >= 2
                    && Arc::ptr_eq(&type_set[0], &self.undefined_type())
                    && Arc::ptr_eq(&type_set[1], &self.missing_type())
                {
                    type_set.remove(1);
                }
            }
            if includes.intersects(
                TypeFlags::Enum
                    | TYPE_FLAGS_LITERAL
                    | TypeFlags::UniqueESSymbol
                    | TypeFlags::TemplateLiteral
                    | TypeFlags::StringMapping,
            ) || (includes.intersects(TypeFlags::Void) && includes.intersects(TypeFlags::Undefined))
            {
                type_set = self.remove_redundant_literal_types(&type_set, includes, union_reduction == UnionReduction::Subtype);
            }
            if includes.intersects(TypeFlags::StringLiteral)
                && includes.intersects(TypeFlags::TemplateLiteral | TypeFlags::StringMapping)
            {
                type_set = self.remove_string_literals_matched_by_template_literals(&type_set);
            }
            if includes.intersects(TYPE_FLAGS_INCLUDES_CONSTRAINED_TYPE_VARIABLE) {
                type_set = self.remove_constrained_type_variables(&type_set);
            }
            if union_reduction == UnionReduction::Subtype {
                match self.remove_subtypes(&type_set, includes.intersects(TypeFlags::Object)) {
                    Some(reduced) => type_set = reduced,
                    None => return self.error_type(),
                }
            }
            if type_set.is_empty() {
                if includes.intersects(TypeFlags::Null) {
                    if includes.intersects(TYPE_FLAGS_INCLUDES_NON_WIDENING_TYPE) {
                        return self.null_type();
                    }
                    return self.null_widening_type();
                }
                if includes.intersects(TypeFlags::Undefined) {
                    if includes.intersects(TYPE_FLAGS_INCLUDES_NON_WIDENING_TYPE) {
                        return self.undefined_type();
                    }
                    return self.undefined_widening_type();
                }
                return self.never_type();
            }
        }
        let mut origin = origin.cloned();
        if origin.is_none() && includes.intersects(TypeFlags::Union) {
            let named_unions = self.add_named_unions(vec![], &types);
            let mut reduced_types: Vec<Arc<Type>> = vec![];
            for t in &type_set {
                let contained = named_unions.iter().any(|u| {
                    u.types()
                        .map(|us| us.iter().any(|x| Arc::ptr_eq(x, t)))
                        .unwrap_or(false)
                });
                if !contained {
                    reduced_types.push(Arc::clone(t));
                }
            }
            if alias.is_none() && named_unions.len() == 1 && reduced_types.is_empty() {
                return Arc::clone(&named_unions[0]);
            }
            let mut named_types_count = 0;
            for u in &named_unions {
                named_types_count += u.types().map(|us| us.len()).unwrap_or(0);
            }
            if named_types_count + reduced_types.len() == type_set.len() {
                for u in &named_unions {
                    reduced_types.push(Arc::clone(u));
                }
                origin = Some(self.new_union_type(ObjectFlags::None, &reduced_types));
            }
        }
        let object_flags = if includes.intersects(TYPE_FLAGS_NOT_PRIMITIVE_UNION) {
            ObjectFlags::None
        } else {
            ObjectFlags::PrimitiveUnion
        } | if includes.intersects(TypeFlags::Intersection) {
            ObjectFlags::ContainsIntersections
        } else {
            ObjectFlags::None
        };
        self.get_union_type_from_sorted_list(type_set, object_flags, alias, origin.as_ref())
    }

    pub fn get_uniq_associated_names_from_tuple_type(
        &mut self,
        t: &Arc<Type>,
        rest_symbol: &Arc<Symbol>,
    ) -> Vec<String> { ::tsox_core::fntrace::enter("get_uniq_associated_names_from_tuple_type"); 
        let element_infos = t
            .target_tuple_type()
            .map(|d| d.element_infos.clone())
            .unwrap_or_default();
        let mut names: Vec<String> = Vec::with_capacity(element_infos.len());
        let mut counters: HashMap<String, i32> = HashMap::new();
        for (i, info) in element_infos.iter().enumerate() {
            let name = self.get_tuple_element_label(info, Some(rest_symbol), i);
            *counters.entry(name.clone()).or_insert(0) -= 1;
            names.push(name);
        }
        for i in 0..names.len() {
            let name = names[i].clone();
            if counters.get(&name).copied().unwrap_or(0) == -1 {
                continue;
            }
            let mut name = name;
            loop {
                if counters.get(&name).copied().unwrap_or(0) < 0 {
                    counters.insert(name.clone(), 0);
                }
                let next = counters.get(&name).copied().unwrap_or(0) + 1;
                counters.insert(name.clone(), next);
                let candidate_name = format!("{}_{}", name, next);
                if counters.get(&candidate_name).copied().unwrap_or(0) == 0 {
                    names[i] = candidate_name;
                    break;
                }
            }
        }
        names
    }

    pub fn get_unique_literal_type_for_type_parameter(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_unique_literal_type_for_type_parameter"); 
        if t.flags.intersects(TypeFlags::TypeParameter) {
            return self.unique_literal_type();
        }
        Arc::clone(t)
    }

    pub fn get_unique_type_parameters(
        &mut self,
        context: &InferenceContext,
        type_parameters: &[Arc<Type>],
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_unique_type_parameters"); 
        let mut old_type_parameters: Vec<Arc<Type>> = vec![];
        let mut new_type_parameters: Vec<Arc<Type>> = vec![];
        let mut result: Vec<Arc<Type>> = Vec::with_capacity(type_parameters.len());
        let inferred: Vec<Arc<Type>> = context
            .inferred_type_parameters
            .iter()
            .flatten()
            .cloned()
            .collect();
        for tp in type_parameters {
            let name = tp.symbol().map(|s| s.name.clone()).unwrap_or_default();
            if has_type_parameter_by_name(&inferred, &name) || has_type_parameter_by_name(&result, &name) {
                let mut combined = inferred.clone();
                combined.extend(result.iter().cloned());
                let new_name = get_unique_type_parameter_name(&combined, &name);
                let symbol = self.new_symbol(SymbolFlags::TypeParameter, &new_name);
                let mut new_type_parameter = self.new_type_parameter(Some(symbol));
                Arc::get_mut(&mut new_type_parameter).map(|n| n.set_type_parameter_target(tp));
                old_type_parameters.push(Arc::clone(tp));
                new_type_parameters.push(Arc::clone(&new_type_parameter));
                result.push(new_type_parameter);
            } else {
                result.push(Arc::clone(tp));
            }
        }
        if !new_type_parameters.is_empty() {
            let mapper = new_type_mapper(old_type_parameters, new_type_parameters.clone());
            for tp in &new_type_parameters {
                Arc::get_mut(&mut Arc::clone(tp)).map(|n| n.set_type_parameter_mapper(&mapper));
            }
        }
        result
    }

    pub fn get_widened_literal_like_type_for_contextual_iteration_type_if_needed(
        &mut self,
        t: Option<&Arc<Type>>,
        contextual_signature_return_type: Option<&Arc<Type>>,
        kind: IterationTypeKind,
        is_async_generator: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_widened_literal_like_type_for_contextual_iteration_type_if_needed"); 
        let t = t?;
        if is_unit_type(t) {
            let contextual_type = contextual_signature_return_type.and_then(|c| {
                self.get_iteration_type_of_generator_function_return_type(kind, c, is_async_generator)
            });
            return Some(self.get_widened_literal_like_type_for_contextual_type(t, contextual_type.as_ref()));
        }
        Some(Arc::clone(t))
    }

    pub fn get_widened_literal_like_type_for_contextual_return_type_if_needed(
        &mut self,
        t: Option<&Arc<Type>>,
        contextual_signature_return_type: Option<&Arc<Type>>,
        is_async: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_widened_literal_like_type_for_contextual_return_type_if_needed"); 
        let t = t?;
        if is_unit_type(t) {
            let contextual_type = match contextual_signature_return_type {
                None => None,
                Some(c) if is_async => self.get_promised_type_of_promise(c),
                Some(c) => Some(Arc::clone(c)),
            };
            return Some(self.get_widened_literal_like_type_for_contextual_type(t, contextual_type.as_ref()));
        }
        Some(Arc::clone(t))
    }

    pub fn get_widened_literal_like_type_for_contextual_type(
        &mut self,
        t: &Arc<Type>,
        contextual_type: Option<&Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_widened_literal_like_type_for_contextual_type"); 
        let mut t = Arc::clone(t);
        if let Some(contextual_type) = contextual_type {
            if !self.is_literal_of_contextual_type(&t, contextual_type) {
                t = self.get_widened_unique_es_symbol_type(&self.get_widened_literal_type(&t));
            }
        }
        self.get_regular_type_of_literal_type(&t)
    }

    pub fn get_widened_property(&mut self, prop: &Arc<Symbol>, context: Option<&WideningContext>) -> Arc<Symbol> { ::tsox_core::fntrace::enter("get_widened_property"); 
        if !prop.flags.intersects(SymbolFlags::Property) {
            return Arc::clone(prop);
        }
        let original = self.get_type_of_symbol(prop);
        let prop_context = context.map(|c| c.get_child_context(&prop.name));
        let widened = self.get_widened_type_with_context(&original, prop_context.as_ref());
        if Arc::ptr_eq(&widened, &original) {
            return Arc::clone(prop);
        }
        self.create_symbol_with_type(prop, &widened)
    }

    pub fn get_widened_type_with_context(&mut self, t: &Arc<Type>, context: Option<&WideningContext>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_widened_type_with_context"); 
        if !t.object_flags.intersects(OBJECT_FLAGS_REQUIRES_WIDENING) {
            return Arc::clone(t);
        }
        if context.is_none() {
            let key = CachedTypeKey {
                kind: CachedTypeKind::Widened,
                type_id: t.id(),
            };
            if let Some(cached) = self.cached_types.get(&key) {
                return Arc::clone(cached);
            }
        }
        let result = if t.flags.intersects(TypeFlags::Any | TYPE_FLAGS_NULLABLE) {
            Some(self.any_type())
        } else if is_object_literal_type(t) {
            Some(self.get_widened_type_of_object_literal(t, context))
        } else if t.flags.intersects(TypeFlags::Union) {
            let union_context_owned;
            let union_context = match context {
                Some(c) => c,
                None => {
                    union_context_owned = WideningContext::with_siblings(t.types().unwrap_or(&[]).to_vec());
                    &union_context_owned
                }
            };
            let widened_types: Vec<Arc<Type>> = t
                .types()
                .unwrap_or(&[])
                .iter()
                .map(|u| {
                    if u.flags.intersects(TYPE_FLAGS_NULLABLE) {
                        Arc::clone(u)
                    } else {
                        self.get_widened_type_with_context(u, Some(union_context))
                    }
                })
                .collect();
            let reduction = if widened_types.iter().any(|u| self.is_empty_object_type(u)) {
                UnionReduction::Subtype
            } else {
                UnionReduction::Literal
            };
            Some(self.get_union_type_ex(widened_types, reduction))
        } else if t.flags.intersects(TypeFlags::Intersection) {
            let mapped: Vec<Arc<Type>> = t
                .types()
                .unwrap_or(&[])
                .iter()
                .map(|u| self.get_widened_type(u))
                .collect();
            Some(self.get_intersection_type(mapped))
        } else if self.is_array_or_tuple_type(t) {
            let Some(target) = t.target() else {
                return Arc::clone(t);
            };
            let mapped: Vec<Arc<Type>> = self
                .get_type_arguments(t)
                .iter()
                .map(|u| self.get_widened_type(u))
                .collect();
            Some(self.create_type_reference(target, &mapped))
        } else {
            None
        };
        if let Some(result) = &result {
            if context.is_none() {
                let key = CachedTypeKey {
                    kind: CachedTypeKind::Widened,
                    type_id: t.id(),
                };
                self.cached_types.insert(key, Arc::clone(result));
            }
        }
        result.unwrap_or_else(|| Arc::clone(t))
    }

    pub fn get_widened_unique_es_symbol_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_widened_unique_es_symbol_type"); 
        if t.flags.intersects(TypeFlags::UniqueESSymbol) {
            return self.es_symbol_type();
        }
        if t.flags.intersects(TypeFlags::Union) {
            return map_type_self(self, t, &mut |c, t| Some(c.get_widened_unique_es_symbol_type(t)))
                .unwrap_or_else(|| Arc::clone(t));
        }
        Arc::clone(t)
    }

    pub fn get_write_type_of_accessors(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_write_type_of_accessors"); 
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(t) = &links.write_type {
                return Arc::clone(t);
            }
        }
        if !self.push_type_resolution(Arc::as_ptr(symbol), TypeResolutionProperty::WriteType) {
            return self.error_type();
        }
        let mut setter = get_declaration_of_kind(symbol, SyntaxKind::SetAccessor);
        if setter.is_none() {
            let prop_declaration = get_declaration_of_kind(symbol, SyntaxKind::PropertyDeclaration);
            if let Some(prop_declaration) = &prop_declaration {
                if is_auto_accessor_property_declaration(prop_declaration) {
                    setter = Some(Arc::clone(prop_declaration));
                }
            }
        }
        let write_type = setter.as_ref().and_then(|s| self.get_annotated_accessor_type(s));
        if !self.pop_type_resolution() {
            if let Some(setter) = &setter {
                if self.get_annotated_accessor_type_node(setter).is_some() {
                    let symbol_str = self.symbol_to_string(symbol);
                    self.error_message(
                        setter,msg::X_0_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_TYPE_ANNOTATION,
                        &[symbol_str],
                    );
                }
            }
        }
        if self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.write_type.clone())
            .is_none()
        {
            let write_type = write_type.unwrap_or_else(|| self.get_type_of_accessors(symbol));
            self.value_symbol_links.get_or_default(symbol).write_type = Some(write_type);
        }
        self.value_symbol_links
            .get(symbol)
            .and_then(|l| l.write_type.clone())
            .unwrap_or_else(|| self.any_type())
    }

    pub fn get_write_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_write_type_of_symbol"); 
        if symbol.check_flags.intersects(ast::CheckFlags::SyntheticProperty) {
            if symbol.check_flags.intersects(ast::CheckFlags::DeferredType) {
                return self.get_write_type_of_symbol_with_deferred_type(symbol);
            }
            let links = self.value_symbol_links.get_or_default(symbol);
            return links
                .write_type
                .clone()
                .or_else(|| links.resolved_type.clone())
                .unwrap_or_else(|| self.any_type());
        }
        if symbol.flags.intersects(SymbolFlags::Property) {
            let optional = symbol.flags.intersects(SymbolFlags::Optional);
            let t = self.get_type_of_symbol(symbol);
            return self.remove_missing_type(t, optional);
        }
        if symbol.flags.intersects(SymbolFlags::ACCESSOR) {
            if symbol.check_flags.intersects(ast::CheckFlags::Instantiated) {
                return self.get_write_type_of_instantiated_symbol(symbol);
            }
            return self.get_write_type_of_accessors(symbol);
        }
        self.get_type_of_symbol(symbol)
    }

    pub fn get_yielded_type_of_yield_expression(
        &mut self,
        node: &Arc<Node>,
        expression_type: &Arc<Type>,
        sent_type: &Arc<Type>,
        is_async: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_yielded_type_of_yield_expression"); 
        let error_node = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let is_yield_star = yield_expression_asterisk_token(node).is_some();
        let mut yielded_type = Arc::clone(expression_type);
        if is_yield_star {
            let iteration_use = if is_async {
                IterationUse::YieldStar { is_async: true }
            } else {
                IterationUse::YieldStar { is_async: false }
            };
            yielded_type = self.check_iterated_type_or_element_type(iteration_use, &yielded_type, Some(&error_node));
        }
        if !is_async {
            return yielded_type;
        }
        let diagnostic = if is_yield_star {
            &msg::TYPE_OF_ITERATED_ELEMENTS_OF_A_YIELD_ASTERISK_OPERAND_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER
        } else {
            &msg::TYPE_OF_YIELD_OPERAND_IN_AN_ASYNC_GENERATOR_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER
        };
        self.get_awaited_type_ex(&yielded_type, Some(&error_node), Some(diagnostic), &[])
            .unwrap_or_else(|| Arc::clone(&yielded_type))
    }

    pub fn has_context_sensitive_return_expression(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_context_sensitive_return_expression"); 
        if node_type_parameters(node).map_or(true, |list| list.is_empty()) && node.type_().is_none() {
            if let Some(body) = node.body() {
                if !ast::is_block(&body) {
                    return self.is_context_sensitive(&body);
                }
                return for_each_return_statement(&body, |statement| {
                    statement
                        .expression()
                        .map(|e| self.is_context_sensitive(&e))
                        .unwrap_or(false)
                });
            }
            return false;
        }
        false
    }
}

impl Checker {
    pub fn check_jsx_attribute_with_mode(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_jsx_attribute_with_mode"); 
        if let Some(initializer) = tsox_frontend::ast::mig::m3b::initializer(node) {
            return self.check_expression_for_mutable_location(initializer, check_mode);
        }
        self.true_type()
    }
}
