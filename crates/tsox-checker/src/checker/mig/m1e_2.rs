use std::sync::Arc;

use tsox_frontend::ast::mig::m3b::{elements, is_import_declaration_or_js_import_declaration};
use tsox_frontend::ast::mig::m3c::type_arguments;
use tsox_frontend::ast::mig::m3e_4::get_import_attributes;
use tsox_frontend::ast::mig::m3f_2::{has_import_attributes, node_parameters};
use tsox_frontend::ast::mig::x1a::arguments;
use tsox_frontend::ast::node_data_generated::{
    is_binary_expression, is_conditional_type_node, is_element_access_expression, is_export_declaration,
    is_import_type_node, is_index_signature_declaration, is_infer_type_node, is_literal_type_node, is_mapped_type_node,
    is_named_tuple_member, is_parameter_declaration, is_parenthesized_type_node, is_rest_type_node,
    is_template_literal_type_span, is_type_parameter_declaration, is_type_reference_node,
};
use tsox_frontend::ast::{
    is_import_call, ModifierFlags, Node, NodeData, Symbol, SymbolFlags, SyntaxKind, skip_parentheses,
};

use crate::checker::checker::Checker;
use crate::checker::mapper::new_simple_type_mapper;
use crate::checker::types::{
    AccessFlags, CacheHashKey, ExternalEmitHelpers, IndexFlags, IndexInfo, IntersectionFlags, KeyBuilder, Signature, Type,
    TypeAlias, TypeData, TypeFlags, UnionReduction, ObjectFlags, CachedTypeKind, CachedTypeKey, InferenceContext,
};
use crate::checker::utilities_is_optional_symbol::has_readonly_modifier;

use super::m1c_7::find_index_info_free as find_index_info;
use super::m1b::is_node_descendant_of;
use super::m1d_4::for_each_type;
use super::m1e::r17k3_flags::*;
use super::m1e::r18k5_helpers::*;
use super::m1g_5::get_type_list_key;
use super::m1e::r26k3_defs::new_deferred_type_mapper;
use super::m2a::is_unary_tuple_type_node;
use super::m2c::r18k3_defs::append_type_mapping;
use super::wc3::NodeAccessExt;

#[path = "r31k3_defs.rs"]
pub mod r31k3_defs;
pub use r31k3_defs::*;

impl Checker {
    pub fn get_helper_names(&self, helper: ExternalEmitHelpers) -> Vec<&'static str> {
        match helper {
            ExternalEmitHelpers::Rest => vec!["__rest"],
            ExternalEmitHelpers::Decorate => {
                if self.legacy_decorators {
                    vec!["__decorate"]
                } else {
                    vec!["__esDecorate", "__runInitializers"]
                }
            }
            ExternalEmitHelpers::Metadata => vec!["__metadata"],
            ExternalEmitHelpers::Param => vec!["__param"],
            ExternalEmitHelpers::Awaiter => vec!["__awaiter"],
            ExternalEmitHelpers::Await => vec!["__await"],
            ExternalEmitHelpers::AsyncGenerator => vec!["__asyncGenerator"],
            ExternalEmitHelpers::AsyncDelegator => vec!["__asyncDelegator"],
            ExternalEmitHelpers::AsyncValues => vec!["__asyncValues"],
            ExternalEmitHelpers::ExportStar => vec!["__exportStar"],
            ExternalEmitHelpers::ImportStar => vec!["__importStar"],
            ExternalEmitHelpers::ImportDefault => vec!["__importDefault"],
            ExternalEmitHelpers::MakeTemplateObject => vec!["__makeTemplateObject"],
            ExternalEmitHelpers::ClassPrivateFieldGet => vec!["__classPrivateFieldGet"],
            ExternalEmitHelpers::ClassPrivateFieldSet => vec!["__classPrivateFieldSet"],
            ExternalEmitHelpers::ClassPrivateFieldIn => vec!["__classPrivateFieldIn"],
            ExternalEmitHelpers::SetFunctionName => vec!["__setFunctionName"],
            ExternalEmitHelpers::PropKey => vec!["__propKey"],
            ExternalEmitHelpers::AddDisposableResourceAndDisposeResources => vec!["__addDisposableResource", "__disposeResources"],
            ExternalEmitHelpers::RewriteRelativeImportExtension => vec!["__rewriteRelativeImportExtension"],
            _ => panic!("Unrecognized helper"),
        }
    }

    pub fn get_homomorphic_type_variable(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let Some(constraint_type) = self.get_constraint_type_from_mapped_type(t) else {
            return None;
        };
        if constraint_type.flags.intersects(TypeFlags::INDEX) {
            if let TypeData::Index(i) = &constraint_type.data
                && let Some(target) = i.target.as_ref()
            {
                let type_variable = self.get_actual_type_variable(target);
                if type_variable.flags.intersects(TypeFlags::TYPE_PARAMETER) {
                    return Some(type_variable);
                }
            }
        }
        None
    }

    pub fn get_implied_constraint(&mut self, t: &Arc<Type>, check_node: &Arc<Node>, extends_node: &Arc<Node>) -> Option<Arc<Type>> {
        if is_unary_tuple_type_node(check_node) && is_unary_tuple_type_node(extends_node) {
            let check_elements = elements(check_node);
            let extends_elements = elements(extends_node);
            return self.get_implied_constraint(t, &check_elements[0], &extends_elements[0]);
        }
        let check_type = self.get_type_from_type_node(check_node);
        let check_variable = self.get_actual_type_variable(&check_type);
        let t_variable = self.get_actual_type_variable(t);
        if Arc::ptr_eq(&check_variable, &t_variable) {
            return Some(self.get_type_from_type_node(extends_node));
        }
        None
    }

    pub fn get_import_attributes_type_for_module_specifier(&mut self, module_specifier: &Node) -> Option<Arc<Type>> {
        let parent = module_specifier.parent()?;
        if is_import_declaration_or_js_import_declaration(&parent) || is_export_declaration(&parent) {
            return self.get_type_from_import_attributes(get_import_attributes(&parent).as_ref());
        }
        if let Some(grandparent) = parent.parent()
            && is_literal_type_node(&parent)
            && is_import_type_node(&grandparent)
        {
            return self.get_type_from_import_attributes(get_import_attributes(&grandparent).as_ref());
        }
        if is_import_call(&parent) && arguments(&parent).len() > 1 {
            let options = &arguments(&parent)[1];
            let options_type = self.check_expression_cached(options);
            return self.get_type_of_property_of_type(&options_type, "with");
        }
        None
    }

    pub fn get_index_infos_of_index_symbol(&mut self, index_symbol: &Arc<Symbol>, sibling_symbols: &[Arc<Symbol>]) -> Vec<Arc<IndexInfo>> {
        let mut index_infos: Vec<Arc<IndexInfo>> = Vec::new();
        let mut has_computed_string_property = false;
        let mut has_computed_number_property = false;
        let mut has_computed_symbol_property = false;
        let mut readonly_computed_string_property = true;
        let mut readonly_computed_number_property = true;
        let mut readonly_computed_symbol_property = true;
        let mut property_symbols: Vec<Arc<Symbol>> = Vec::new();
        let string_number_symbol_type = self.string_number_symbol_type();
        for declaration in &index_symbol.declarations {
            if is_index_signature_declaration(declaration) {
                let parameters = node_parameters(declaration).map(|p| p.nodes.as_slice()).unwrap_or(&[]);
                let return_type_node = declaration.typ();
                if parameters.len() == 1 {
                    let type_node = parameters[0].typ();
                    if let Some(type_node) = type_node {
                        let mut value_type = self.any_type();
                        if let Some(return_type_node) = return_type_node {
                            value_type = self.get_type_from_type_node(return_type_node);
                        }
                        let key_src = self.get_type_from_type_node(type_node);
                        for_each_type(&key_src, &mut |key_type: &Arc<Type>| {
                            if self.is_valid_index_key_type(key_type) && find_index_info(&index_infos, key_type).is_none() {
                                let index_info = self.new_index_info(
                                    key_type,
                                    &value_type,
                                    has_readonly_modifier(declaration),
                                    Some(declaration),
                                    &[],
                                );
                                index_infos.push(index_info);
                            }
                        });
                    }
                }
            } else if self.has_late_bindable_index_signature(declaration) {
                let decl_name: Arc<Node> = if is_binary_expression(declaration) {
                    declaration.as_binary_expression().left.clone()
                } else {
                    declaration.name().unwrap().clone()
                };
                let key_type: Arc<Type> = if is_element_access_expression(&decl_name) {
                    self.check_expression_cached(&decl_name.as_element_access_expression().argument_expression)
                } else if decl_name.kind == SyntaxKind::ComputedPropertyName {
                    self.check_expression_cached(decl_name.expression().unwrap())
                } else {
                    self.check_expression_cached(&decl_name)
                };
                if find_index_info(&index_infos, &key_type).is_some() {
                    continue;
                }
                if self.is_type_assignable_to(&key_type, &string_number_symbol_type) {
                    if self.is_type_assignable_to(&key_type, &self.number_type()) {
                        has_computed_number_property = true;
                        if !has_readonly_modifier(declaration) {
                            readonly_computed_number_property = false;
                        }
                    } else if self.is_type_assignable_to(&key_type, &self.es_symbol_type()) {
                        has_computed_symbol_property = true;
                        if !has_readonly_modifier(declaration) {
                            readonly_computed_symbol_property = false;
                        }
                    } else {
                        has_computed_string_property = true;
                        if !has_readonly_modifier(declaration) {
                            readonly_computed_string_property = false;
                        }
                    }
                    if let Some(sym) = self.get_symbol_of_declaration(declaration) {
                        property_symbols.push(sym);
                    }
                }
            }
        }
        if has_computed_string_property || has_computed_number_property || has_computed_symbol_property {
            for sym in sibling_symbols {
                if !Arc::ptr_eq(sym, index_symbol) {
                    property_symbols.push(Arc::clone(sym));
                }
            }
            if has_computed_string_property && find_index_info(&index_infos, &self.string_type()).is_none() {
                let info = self.get_object_literal_index_info(readonly_computed_string_property, &property_symbols, &self.string_type());
                index_infos.push(Arc::new(info));
            }
            if has_computed_number_property && find_index_info(&index_infos, &self.number_type()).is_none() {
                let info = self.get_object_literal_index_info(readonly_computed_number_property, &property_symbols, &self.number_type());
                index_infos.push(Arc::new(info));
            }
            if has_computed_symbol_property && find_index_info(&index_infos, &self.es_symbol_type()).is_none() {
                let info = self.get_object_literal_index_info(readonly_computed_symbol_property, &property_symbols, &self.es_symbol_type());
                index_infos.push(Arc::new(info));
            }
        }
        index_infos
    }

    pub fn get_index_infos_of_structured_type(&mut self, t: &Arc<Type>) -> Vec<Arc<IndexInfo>> {
        if t.flags.intersects(TypeFlags::STRUCTURED_TYPE) {
            if let Some(s) = self.resolve_structured_type_members(t).as_structured_type() {
                return s.index_infos.clone();
            }
            return Vec::new();
        }
        Vec::new()
    }

    pub fn get_index_infos_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<IndexInfo>> {
        if let Some(index_symbol) = self.get_index_symbol(symbol) {
            let members: Vec<Arc<Symbol>> = self
                .get_resolved_members_or_exports_table(symbol)
                .iter()
                .map(|(_, s)| Arc::clone(s))
                .collect();
            return self.get_index_infos_of_index_symbol(&index_symbol, &members);
        }
        Vec::new()
    }

    pub fn get_index_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        self.get_resolved_members_or_exports_table(symbol)
            .get(internal_symbol_name_index)
            .cloned()
    }

    // Go getMembersOfSymbol：LateBindingContainer（Class|Interface|TypeLiteral|
    // ObjectLiteral|Function，symbolflags.go:82）走 resolved members（含
    // __index/__computed 晚绑定），其余取原始 members
    fn get_resolved_members_or_exports_table(&mut self, symbol: &Arc<Symbol>) -> tsox_frontend::ast::SymbolTable {
        if symbol
            .flags
            .intersects(SymbolFlags::Class
                .union(SymbolFlags::Interface)
                .union(SymbolFlags::TypeLiteral)
                .union(SymbolFlags::ObjectLiteral)
                .union(SymbolFlags::Function))
        {
            return self.get_resolved_members_or_exports_of_symbol(
                symbol,
                crate::checker::types_alias_symbol_links::MembersOrExportsResolutionKind::ResolvedMembers,
            );
        }
        symbol.members.clone()
    }

    pub fn get_index_type_ex(&mut self, t: &Arc<Type>, index_flags: IndexFlags) -> Arc<Type> {
        let t = self.get_reduced_type(t);
        if self.is_no_infer_type(&t) {
            if let TypeData::Substitution(s) = &t.data
                && let Some(base_type) = s.base_type.as_ref()
            {
                let base = self.get_index_type_ex(base_type, index_flags);
                if self.is_no_infer_target_type(&base) {
                    let unknown = self.unknown_type();
                    return self.get_or_create_substitution_type(&base, &unknown);
                }
                return base;
            }
            return t;
        }
        if self.should_defer_index_type(&t, index_flags) {
            return self.get_index_type_for_generic_type(&t, index_flags);
        }
        if t.flags.intersects(TypeFlags::UNION) {
            if let TypeData::Union(u) = &t.data {
                let parts: Vec<Arc<Type>> = u.union_or_intersection.types.iter().map(|ty| self.get_index_type_ex(ty, index_flags)).collect();
                return self.get_intersection_type(parts);
            }
            return t;
        }
        if t.flags.intersects(TypeFlags::INTERSECTION) {
            if let TypeData::Union(u) = &t.data {
                let parts: Vec<Arc<Type>> = u.union_or_intersection.types.iter().map(|ty| self.get_index_type_ex(ty, index_flags)).collect();
                return self.get_union_type(parts);
            }
            return t;
        }
        if t.object_flags.intersects(ObjectFlags::MAPPED) {
            return self.get_index_type_for_mapped_type(&t, index_flags);
        }
        if Arc::ptr_eq(&t, &self.wildcard_type()) {
            return self.wildcard_type();
        }
        if t.flags.intersects(TypeFlags::UNKNOWN) {
            return self.never_type();
        }
        if t.flags.intersects(TypeFlags::ANY | TypeFlags::NEVER) {
            return self.string_number_symbol_type();
        }
        let mut include = if index_flags.intersects(IndexFlags::NO_INDEX_SIGNATURES) {
            TypeFlags::STRING_LITERAL
        } else {
            TypeFlags::STRING_LIKE
        };
        if !index_flags.intersects(IndexFlags::STRINGS_ONLY) {
            include |= TypeFlags::NUMBER_LIKE | TypeFlags::ES_SYMBOL_LIKE;
        }
        self.get_literal_type_from_properties(&t, include, !index_flags.intersects(IndexFlags::NO_INDEX_SIGNATURES))
    }

    pub fn get_index_type_for_generic_type(&mut self, t: &Arc<Type>, index_flags: IndexFlags) -> Arc<Type> {
        let key = CachedTypeKey {
            kind: if index_flags.intersects(IndexFlags::STRINGS_ONLY) {
                CachedTypeKind::StringIndexType
            } else {
                CachedTypeKind::IndexType
            },
            type_id: t.id,
        };
        if let Some(index_type) = self.cached_types.get(&key) {
            return Arc::clone(index_type);
        }
        let index_type = self.new_index_type(t, index_flags & IndexFlags::STRINGS_ONLY);
        self.cached_types.insert(key, Arc::clone(&index_type));
        index_type
    }

    pub fn get_index_type_for_mapped_type(&mut self, t: &Arc<Type>, index_flags: IndexFlags) -> Arc<Type> {
        let type_parameter = self.get_type_parameter_from_mapped_type(t);
        let Some(constraint_type) = self.get_constraint_type_from_mapped_type(t) else {
            return self.never_type();
        };
        let name_type = self.get_name_type_from_mapped_type(&match &t.data {
            TypeData::Mapped(m) if m.object.target.is_some() => m.object.target.clone().unwrap(),
            _ => Arc::clone(t),
        });
        if name_type.is_none() && !index_flags.intersects(IndexFlags::NO_INDEX_SIGNATURES) {
            return constraint_type;
        }
        let mut key_types: Vec<Arc<Type>> = Vec::new();
        let t_clone = Arc::clone(t);
        let mut add_member_for_key_type = |c: &mut Checker, key_type: &Arc<Type>| {
            let mut prop_name_type = Arc::clone(key_type);
            if let (Some(name_type), Some(tp)) = (&name_type, type_parameter.as_ref()) {
                if let TypeData::Mapped(m) = &t_clone.data {
                    prop_name_type = c.instantiate_type(
                        name_type,
                        append_type_mapping(m.object.mapper.as_ref(), tp, key_type).as_ref(),
                    );
                }
            }
            if Arc::ptr_eq(&prop_name_type, &c.string_type()) {
                key_types.push(c.string_or_number_type());
            } else {
                key_types.push(prop_name_type);
            }
        };
        if self.is_generic_index_type(&constraint_type) {
            if Checker::is_mapped_type_with_keyof_constraint_declaration(t) {
                return self.get_index_type_for_generic_type(t, index_flags);
            }
            let constraint = Arc::clone(&constraint_type);
            for_each_type(&constraint, &mut |key_type| add_member_for_key_type(self, key_type));
        } else if Checker::is_mapped_type_with_keyof_constraint_declaration(t) {
            let modifiers_type = self.get_modifiers_type_from_mapped_type(t);
            let modifiers_type = self.get_apparent_type(&modifiers_type);
            let mut mapped_key_types: Vec<Arc<Type>> = Vec::new();
            self.for_each_mapped_type_property_key_type_and_index_signature_key_type(
                &modifiers_type,
                TypeFlags::STRING_OR_NUMBER_LITERAL_OR_UNIQUE,
                index_flags.intersects(IndexFlags::STRINGS_ONLY),
                &mut |key_type| mapped_key_types.push(Arc::clone(key_type)),
            );
            for key_type in &mapped_key_types {
                add_member_for_key_type(self, key_type);
            }
        } else {
            let lower_bound = self.get_lower_bound_of_key_type(&constraint_type);
            for_each_type(&lower_bound, &mut |key_type| add_member_for_key_type(self, key_type));
        }
        let result = if index_flags.intersects(IndexFlags::NO_INDEX_SIGNATURES) {
            let union = self.get_union_type(key_types.clone());
            self.filter_type(&union, &mut |t: &Arc<Type>| !t.flags.intersects(TypeFlags::ANY | TypeFlags::STRING))
        } else {
            self.get_union_type(key_types.clone())
        };
        if let TypeData::Union(ru) = &result.data
            && let TypeData::Union(cu) = &constraint_type.data
            && get_type_list_key(&ru.union_or_intersection.types) == get_type_list_key(&cu.union_or_intersection.types)
        {
            return constraint_type;
        }
        result
    }

    pub fn get_index_type_of_type_ex(&mut self, t: &Arc<Type>, key_type: &Arc<Type>, default_type: &Arc<Type>) -> Arc<Type> {
        if let Some(result) = self.get_index_info_of_type(t, key_type).and_then(|i| i.value_type.clone()) {
            return result;
        }
        Arc::clone(default_type)
    }

    pub fn get_index_type_or_string(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let index_type = self.get_index_type(t);
        let index_type = self.get_extract_string_type(&index_type);
        if index_type.flags.intersects(TypeFlags::NEVER) {
            return self.string_type();
        }
        index_type
    }

    pub fn get_indexed_access_type_or_undefined(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        mut access_flags: AccessFlags,
        access_node: Option<&Node>,
        alias: Option<&TypeAlias>,
    ) -> Option<Arc<Type>> {
        if Arc::ptr_eq(object_type, &self.wildcard_type()) || Arc::ptr_eq(index_type, &self.wildcard_type()) {
            return Some(self.wildcard_type());
        }
        let object_type = self.get_reduced_type(object_type);
        let mut object_type = object_type;
        let mut index_type = Arc::clone(index_type);
        if self.is_string_index_signature_only_type_worker(&object_type)
            && !index_type.flags.intersects(TypeFlags::NULLABLE)
            && self.is_type_assignable_to_kind(&index_type, TypeFlags::STRING | TypeFlags::NUMBER)
        {
            index_type = self.string_type();
        }
        if self.compiler_options.no_unchecked_indexed_access.is_true() && access_flags.intersects(AccessFlags::EXPRESSION_POSITION) {
            access_flags |= AccessFlags::INCLUDE_UNDEFINED;
        }
        if self.should_defer_indexed_access_type(&object_type, &index_type) {
            if object_type.flags.intersects(TypeFlags::ANY_OR_UNKNOWN) {
                return Some(object_type);
            }
            let persistent_access_flags = access_flags & AccessFlags::PERSISTENT;
            let key = get_indexed_access_key(&object_type, &index_type, access_flags, alias);
            if let Some(t) = self.indexed_access_types.get(&key) {
                return Some(Arc::clone(t));
            }
            let mut t = self.new_indexed_access_type(&object_type, &index_type, persistent_access_flags);
            if let Some(inner) = Arc::get_mut(&mut t) {
                inner.alias = alias.cloned().map(Box::new);
            }
            self.indexed_access_types.insert(key, Arc::clone(&t));
            return Some(t);
        }
        let apparent_object_type = self.get_reduced_apparent_type(&object_type);
        if index_type.flags.intersects(TypeFlags::UNION) && !index_type.flags.intersects(TypeFlags::BOOLEAN) {
            let mut prop_types: Vec<Arc<Type>> = Vec::new();
            let mut was_missing_prop = false;
            if let TypeData::Union(u) = &index_type.data {
                for t in &u.union_or_intersection.types {
                    let mut flags = access_flags;
                    if was_missing_prop {
                        flags |= AccessFlags::SUPPRESS_NO_IMPLICIT_ANY_ERROR;
                    }
                    match self.get_property_type_for_index_type(&object_type, &apparent_object_type, t, &index_type, access_node, flags) {
                        Some(prop_type) => prop_types.push(prop_type),
                        None => {
                            if access_node.is_none() {
                                return None;
                            }
                            was_missing_prop = true;
                        }
                    }
                }
            }
            if was_missing_prop {
                return None;
            }
            if access_flags.intersects(AccessFlags::WRITING) {
                return Some(self.get_intersection_type_ex(&prop_types, IntersectionFlags::empty(), alias));
            }
            return Some(self.get_union_type_ex(prop_types.clone(), UnionReduction::Literal));
        }
        self.get_property_type_for_index_type(
            &object_type,
            &apparent_object_type,
            &index_type,
            &index_type,
            access_node,
            access_flags | AccessFlags::CACHE_SYMBOL | AccessFlags::REPORT_DEPRECATED,
        )
    }

    pub fn get_indexed_mapped_type_substituted_type_of_contextual_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        name_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Type>> {
        let property_name_type = match name_type {
            Some(nt) => Arc::clone(nt),
            None => self.get_string_literal_type(name),
        };
        let Some(constraint) = self.get_constraint_type_from_mapped_type(t) else {
            return None;
        };
        if let TypeData::Mapped(m) = &t.data {
            let name_type_excluded = match &m.name_type {
                Some(mt_name_type) => self.is_excluded_mapped_property_name(mt_name_type, &property_name_type),
                None => false,
            };
            if name_type_excluded || self.is_excluded_mapped_property_name(&constraint, &property_name_type) {
                return None;
            }
        }
        let constraint_of_constraint = self.get_base_constraint_or_type(&constraint);
        if !self.is_type_assignable_to(&property_name_type, &constraint_of_constraint) {
            return None;
        }
        Some(self.substitute_indexed_mapped_type(t, &property_name_type))
    }

    pub fn get_inference_context(&self, node: &Node) -> Option<&InferenceContext> {
        for info in self.inference_context_infos.iter().rev() {
            if let Some(context) = info.context.as_deref()
                && info.node.as_ref().is_some_and(|n| is_node_descendant_of(node, n))
            {
                return Some(context);
            }
        }
        None
    }

    pub fn get_inferred_type_parameter_constraint(&mut self, t: &Arc<Type>, omit_type_references: bool) -> Option<Arc<Type>> {
        let mut inferences: Vec<Arc<Type>> = Vec::new();
        if let Some(symbol) = &t.symbol
            && !symbol.declarations.is_empty()
        {
            for declaration in &symbol.declarations {
                let Some(parent) = declaration.parent() else {
                    continue;
                };
                if !is_infer_type_node(&parent) {
                    continue;
                }
                let mut child = Arc::clone(&parent);
                let mut parent_node = child.parent();
                while parent_node.as_deref().is_some_and(is_parenthesized_type_node) {
                    child = parent_node.unwrap();
                    parent_node = child.parent();
                }
                if let Some(parent) = parent_node {
                    if is_type_reference_node(&parent) && !omit_type_references {
                        let type_parameters = self.get_type_parameters_for_type_reference_or_import(&parent);
                        if !type_parameters.is_empty() {
                            let index = type_arguments(&parent).iter().position(|a| Arc::ptr_eq(a, &child));
                            if let Some(index) = index.filter(|i| *i < type_parameters.len()) {
                                let declared_constraint = self.get_constraint_of_type_parameter(&type_parameters[index]);
                                if let Some(declared_constraint) = declared_constraint {
                                    let mapper = new_deferred_type_mapper(
                                        self,
                                        &type_parameters,
                                        type_parameters.iter().enumerate().map(|(i, _)| {
                                            let parent = Arc::clone(&parent);
                                            let type_parameters = type_parameters.clone();
                                            Box::new(move |c: &mut Checker| c.get_effective_type_argument_at_index(&parent, &type_parameters, i))
                                                as Box<dyn Fn(&mut Checker) -> Arc<Type> + Send + Sync>
                                        }).collect(),
                                    );
                                    let constraint = self.instantiate_type(&declared_constraint, Some(&mapper));
                                    if !Arc::ptr_eq(&constraint, t) {
                                        inferences.push(constraint);
                                    }
                                }
                            }
                        }
                    } else if is_parameter_declaration(&parent) && parent.as_parameter_declaration().dot_dot_dot_token.is_some()
                        || is_rest_type_node(&parent)
                        || is_named_tuple_member(&parent)
                            && match &parent.data {
                                NodeData::NamedTupleMember(d) => d.dot_dot_dot_token.is_some(),
                                _ => false,
                            }
                    {
                        inferences.push(self.create_array_type(self.unknown_type()));
                    } else if is_template_literal_type_span(&parent) {
                        inferences.push(self.string_type());
                    } else if is_type_parameter_declaration(&parent)
                        && parent.parent().as_deref().is_some_and(|gp| is_mapped_type_node(gp))
                    {
                        inferences.push(self.string_number_symbol_type());
                    } else if is_mapped_type_node(&parent) {
                        let mapped_ty = parent.typ().map(skip_parentheses);
                        let parent_parent = parent.parent();
                        let conditional = parent_parent.as_deref().filter(|gp| is_conditional_type_node(gp));
                        let extends_matches = conditional
                            .map(|gp| match &gp.data {
                                NodeData::ConditionalTypeNode(d) => Arc::ptr_eq(&d.extends_type, &parent),
                                _ => false,
                            })
                            .unwrap_or(false);
                        let check_mapped_type = conditional.and_then(|gp| match &gp.data {
                            NodeData::ConditionalTypeNode(d) => Some(Arc::clone(&d.check_type)),
                            _ => None,
                        });
                        let check_mapped_type = check_mapped_type
                            .filter(|ct| is_mapped_type_node(ct.as_ref()) && ct.typ().is_some());
                        let mapped_ty_matches = mapped_ty
                            .as_ref()
                            .zip(declaration.parent())
                            .is_some_and(|(a, b)| Arc::ptr_eq(a, &b));
                        if mapped_ty.is_some()
                            && mapped_ty_matches
                            && extends_matches
                            && let Some(check_mapped_type) = check_mapped_type
                        {
                            let node_type = self.get_type_from_type_node(check_mapped_type.typ().unwrap());
                            let check_mapped_type_parameter = match &check_mapped_type.data {
                                NodeData::MappedTypeNode(d) => Arc::clone(&d.type_parameter),
                                _ => unreachable!(),
                            };
                            let constraint_type = match &check_mapped_type_parameter.data {
                                NodeData::TypeParameterDeclaration(d) if d.constraint.is_some() => {
                                    self.get_type_from_type_node(d.constraint.as_ref().unwrap())
                                }
                                _ => self.string_number_symbol_type(),
                            };
                            let mapper = Arc::new(new_simple_type_mapper(
                                self.get_declared_type_of_type_parameter(
                                    &self.get_symbol_of_declaration(&check_mapped_type_parameter).unwrap(),
                                ),
                                constraint_type,
                            ));
                            let inferred = self.instantiate_type(&node_type, Some(&mapper));
                            if !Arc::ptr_eq(&inferred, t) {
                                inferences.push(inferred);
                            }
                        }
                    }
                }
            }
        }
        if !inferences.is_empty() {
            return Some(self.get_intersection_type(inferences));
        }
        None
    }
}

pub fn get_index_node_for_access_expression(access_node: &Node) -> &Node {
    match access_node.kind {
        SyntaxKind::ElementAccessExpression => &access_node.as_element_access_expression().argument_expression,
        SyntaxKind::IndexedAccessType => match &access_node.data {
            NodeData::IndexedAccessTypeNode(d) => &d.index_type,
            _ => unreachable!(),
        },
        SyntaxKind::ComputedPropertyName => access_node.expression().unwrap(),
        _ => access_node,
    }
}

pub fn get_indexed_access_key(object_type: &Arc<Type>, index_type: &Arc<Type>, access_flags: AccessFlags, alias: Option<&TypeAlias>) -> CacheHashKey {
    let mut b = KeyBuilder::new();
    b.write_type(object_type);
    b.write_type(index_type);
    b.write_u32(access_flags.bits());
    b.write_alias(alias);
    b.hash()
}
