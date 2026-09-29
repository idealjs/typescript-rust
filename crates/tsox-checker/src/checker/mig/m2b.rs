use tsox_frontend::ast::mig::m3f_3::is_auto_accessor_property_declaration;
#[path = "r22k6_defs.rs"]
pub mod r22k6_defs;

#[path = "r23k6_defs.rs"]
pub mod r23k6_defs;
use crate::checker::mig::m2a::r19k11_defs::*;
use crate::checker::mig::m2a::r20k6_defs::*;
use crate::checker::mig::wc3::NodeAccessExt;
use tsox_frontend::ast::mig::m3g_2::is_type_only_import_or_export_declaration;
use crate::checker::mig::m1e::get_entity_name_from_type_node;
use crate::checker::mig::m3a_2::new_diagnostic_for_node;
use crate::checker::relater_relation::Relation;
use crate::checker::checker::*;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

impl Checker {
    pub fn mark_type_node_as_referenced(&mut self, node: Option<&Arc<Node>>) {
        if let Some(node) = node {
            if let Some(name) = get_type_reference_name_arc(node) {
                self.mark_entity_name_or_entity_expression_as_reference(&name, false);
            }
        }
    }

    pub fn may_resolve_type_alias(&mut self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::TypeReference => self
                .resolve_type_reference_name(node, SymbolFlags::TYPE, false)
                .map(|s| s.flags.contains(SymbolFlags::TypeAlias))
                .unwrap_or(false),
            SyntaxKind::TypeQuery => true,
            SyntaxKind::TypeOperator => {
                node.as_type_operator_node().operator != SyntaxKind::UniqueKeyword
                    && self.may_resolve_type_alias(&node.type_().unwrap())
            }
            SyntaxKind::ParenthesizedType
            | SyntaxKind::OptionalType
            | SyntaxKind::NamedTupleMember => self.may_resolve_type_alias(&node.type_().unwrap()),
            SyntaxKind::RestType => {
                let element_type = node.type_().unwrap();
                element_type.kind != SyntaxKind::ArrayType
                    || self.may_resolve_type_alias(&element_type.as_array_type_node().element_type)
            }
            SyntaxKind::UnionType => node
                .as_union_type_node()
                .types
                .nodes
                .iter()
                .any(|t| self.may_resolve_type_alias(t)),
            SyntaxKind::IntersectionType => node
                .as_intersection_type_node()
                .types
                .nodes
                .iter()
                .any(|t| self.may_resolve_type_alias(t)),
            SyntaxKind::IndexedAccessType => {
                let access = node.as_indexed_access_type_node();
                self.may_resolve_type_alias(&access.object_type)
                    || self.may_resolve_type_alias(&access.index_type)
            }
            SyntaxKind::ConditionalType => {
                let conditional = node.as_conditional_type_node();
                self.may_resolve_type_alias(&conditional.check_type.clone())
                    || self.may_resolve_type_alias(&conditional.extends_type.clone())
                    || self.may_resolve_type_alias(&conditional.true_type.clone())
                    || self.may_resolve_type_alias(&conditional.false_type.clone())
            }
            _ => false,
        }
    }

    pub fn maybe_add_missing_await_info(
        &mut self,
        error_node: Option<&Arc<Node>>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: &Relation,
        report_errors: bool,
        diagnostic_output: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) {
        if let (Some(error_node), true, Some(diagnostic_output)) =
            (error_node, report_errors, diagnostic_output)
        {
            if diagnostic_output.is_empty() {
                return;
            }
            if self.get_awaited_type_of_promise(target).is_some() {
                return;
            }
            if let Some(awaited_type_of_source) = self.get_awaited_type_of_promise(source) {
                if self.is_type_related_to(&awaited_type_of_source, target, relation.kind) {
                    diagnostic_output[0].add_related_info(new_diagnostic_for_node(
                        Some(error_node),
                        tsox_core::diagnostics::messages_generated::DID_YOU_FORGET_TO_USE_AWAIT,
                        Vec::new(),
                    ));
                }
            }
        }
    }

    pub fn maybe_mapped_type(&mut self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        let mut node = Arc::clone(node);
        loop {
            let parent = match node.parent() {
                Some(parent) => parent,
                None => break,
            };
            if !(tsox_frontend::ast::is_computed_property_name(&parent)
                || tsox_frontend::ast::is_property_signature_declaration(&parent))
            {
                break;
            }
            node = parent;
        }
        if tsox_frontend::ast::is_type_literal_node(&node) {
            if let tsox_frontend::ast::NodeData::TypeLiteralNode(d) = &node.data {
                if d.members.nodes.len() == 1 {
                    let t = self.get_declared_type_of_symbol(symbol);
                    return t.flags.intersects(TypeFlags::Union)
                        && self.all_types_assignable_to_kind_ex(
                            &t,
                            TypeFlags::STRING_OR_NUMBER_LITERAL,
                            true,
                        );
                }
            }
        }
        false
    }

    pub fn maybe_type_of_kind_considering_base_constraint(
        &mut self,
        t: &Arc<Type>,
        kind: TypeFlags,
    ) -> bool {
        if self.maybe_type_of_kind(t, kind) {
            return true;
        }
        let base_constraint = self.get_base_constraint_or_type(t);
        self.maybe_type_of_kind(&base_constraint, kind)
    }

    pub fn merge_pattern_ambient_modules(&mut self) {
        let modules = r22k6_defs::take_pattern_ambient_modules();
        let mut groups_by_pattern: HashMap<String, Vec<usize>> = HashMap::new();
        let mut grouped: Vec<PatternAmbientModule> =
            Vec::with_capacity(modules.len());
        for module in &modules {
            let attributes_type = self.get_type_of_module_import_attributes(&module.symbol);
            let mut group_index: Option<usize> = None;
            if let Some(indices) = groups_by_pattern.get(&module.pattern.text) {
                for index in indices {
                    let group_attributes_type =
                        self.get_type_of_module_import_attributes(&grouped[*index].symbol);
                    if self.is_type_identical_to(&attributes_type, &group_attributes_type) {
                        group_index = Some(*index);
                        break;
                    }
                }
            }
            match group_index {
                None => {
                    groups_by_pattern
                        .entry(module.pattern.text.clone())
                        .or_default()
                        .push(grouped.len());
                    grouped.push(PatternAmbientModule {
                        pattern: module.pattern.clone(),
                        symbol: Arc::clone(&module.symbol),
                    });
                }
                Some(group_index) => {
                    let merged =
                        self.merge_symbol(&grouped[group_index].symbol, &module.symbol, false);
                    grouped[group_index].symbol = merged;
                }
            }
        }
        for module in &modules {
            if self.globals.entries.contains_key(&module.symbol.name) {
                let merged = self.get_merged_symbol(&module.symbol);
                self.globals.insert(module.symbol.name.clone(), merged);
            }
        }
        r22k6_defs::set_pattern_ambient_modules(grouped);
    }

    pub fn need_collision_check_for_identifier(
        &self,
        node: &Arc<Node>,
        identifier: Option<&Arc<Node>>,
        name: &str,
    ) -> bool {
        if let Some(identifier) = identifier {
            if identifier.text() != name {
                return false;
            }
        }
        match node.kind {
            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::PropertyAssignment => return false,
            _ => {}
        }
        if node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient) {
            return false;
        }
        if tsox_frontend::ast::is_import_clause(node)
            || tsox_frontend::ast::is_import_equals_declaration(node)
            || tsox_frontend::ast::is_import_specifier(node)
        {
            if is_type_only_import_or_export_declaration(node) {
                return false;
            }
        }
        let root = tsox_frontend::ast::get_root_declaration(node);
        if tsox_frontend::ast::is_parameter_declaration(&root)
            && root
                .parent()
                .and_then(|p| p.body().cloned())
                .is_some_and(|body| tsox_frontend::ast::node_is_missing(Some(&body)))
        {
            return false;
        }
        true
    }

    pub fn new_anonymous_type(
        &mut self,
        symbol: &Arc<Symbol>,
        members: SymbolTable,
        call_signatures: Vec<Arc<Signature>>,
        construct_signatures: Vec<Arc<Signature>>,
        index_infos: Vec<Arc<IndexInfo>>,
    ) -> Arc<Type> {
        let t = self.new_object_type(ObjectFlags::Anonymous, Some(Arc::clone(symbol)));
        self.set_structured_type_members(
            &t,
            Some(members),
            call_signatures,
            construct_signatures,
            index_infos,
        );
        t
    }

    pub fn new_class_accessor_decorator_context_type(
        &mut self,
        this_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_accessor_decorator_context_type();
        self.try_create_type_reference(&global, &[this_type.clone(), value_type.clone()])
    }

    pub fn new_class_accessor_decorator_result_type(
        &mut self,
        this_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_accessor_decorator_result_type();
        self.try_create_type_reference(&global, &[this_type.clone(), value_type.clone()])
    }

    pub fn new_class_accessor_decorator_target_type(
        &mut self,
        this_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_accessor_decorator_target_type();
        self.try_create_type_reference(&global, &[this_type.clone(), value_type.clone()])
    }

    pub fn new_class_decorator_context_type(
        &mut self,
        class_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_decorator_context_type();
        self.try_create_type_reference(&global, &[class_type.clone()])
    }

    pub fn new_class_field_decorator_context_type(
        &mut self,
        this_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_field_decorator_context_type();
        self.try_create_type_reference(&global, &[this_type.clone(), value_type.clone()])
    }

    pub fn new_class_field_decorator_initializer_mutator_type(
        &mut self,
        this_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let this_param = self.new_parameter("this", this_type);
        let value_param = self.new_parameter("value", value_type);
        self.new_function_type(&[], None, &[this_param, value_param], value_type)
    }

    pub fn new_class_getter_decorator_context_type(
        &mut self,
        class_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_getter_decorator_context_type();
        self.try_create_type_reference(&global, &[class_type.clone(), value_type.clone()])
    }

    pub fn new_class_member_decorator_context_type_for_node(
        &mut self,
        node: &Arc<Node>,
        this_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let is_static = tsox_frontend::ast::has_static_modifier(node);
        let is_private = node.name().is_some_and(|n| tsox_frontend::ast::is_private_identifier(&n));
        let name_node = node.name().unwrap();
        let name_type = if is_private {
            self.get_string_literal_type(&name_node.text())
        } else {
            self.get_literal_type_from_property_name(&name_node)
                .expect("literal type from property name")
        };
        let context_type = if tsox_frontend::ast::is_method_declaration(node) {
            self.new_class_method_decorator_context_type(this_type, value_type)
        } else if tsox_frontend::ast::is_get_accessor_declaration(node) {
            self.new_class_getter_decorator_context_type(this_type, value_type)
        } else if tsox_frontend::ast::is_set_accessor_declaration(node) {
            self.new_class_setter_decorator_context_type(this_type, value_type)
        } else if is_auto_accessor_property_declaration(node) {
            self.new_class_accessor_decorator_context_type(this_type, value_type)
        } else if tsox_frontend::ast::is_property_declaration(node) {
            self.new_class_field_decorator_context_type(this_type, value_type)
        } else {
            panic!("Unhandled case in createClassMemberDecoratorContextTypeForNode")
        };
        let override_type =
            self.get_class_member_decorator_context_override_type(&name_type, is_private, is_static);
        self.get_intersection_type(vec![context_type, override_type])
    }

    pub fn new_class_method_decorator_context_type(
        &mut self,
        class_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_method_decorator_context_type();
        self.try_create_type_reference(&global, &[class_type.clone(), value_type.clone()])
    }

    pub fn new_class_setter_decorator_context_type(
        &mut self,
        class_type: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        let global = self.get_global_class_setter_decorator_context_type();
        self.try_create_type_reference(&global, &[class_type.clone(), value_type.clone()])
    }

    pub fn new_conditional_type(
        &mut self,
        root: ConditionalRoot,
        mapper: &Arc<TypeMapper>,
        combined_mapper: Option<&Arc<TypeMapper>>,
    ) -> Arc<Type> {
        let mut data = empty_conditional_type_data();
        data.check_type = Some(self.instantiate_type(&root.check_type.clone().unwrap(), Some(mapper)));
        data.extends_type =
            Some(self.instantiate_type(&root.extends_type.clone().unwrap(), Some(mapper)));
        data.root = Some(Box::new(root));
        data.mapper = Some(Arc::clone(mapper));
        data.combined_mapper = combined_mapper.cloned();
        self.new_type(
            TypeFlags::Conditional,
            ObjectFlags::empty(),
            TypeData::Conditional(data),
        )
    }

    pub fn new_es_decorator_call_signature(
        &mut self,
        target_type: &Arc<Type>,
        context_type: &Arc<Type>,
        non_optional_return_type: &Arc<Type>,
    ) -> Arc<Signature> {
        let target_param = self.new_parameter("target", target_type);
        let context_param = self.new_parameter("context", context_type);
        let void_type = self.void_type();
        let return_type = self.get_union_type(vec![non_optional_return_type.clone(), void_type]);
        self.new_call_signature(None, None, vec![target_param, context_param], Some(return_type))
    }

    pub fn new_getter_function_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        self.new_function_type(&[], None, &[], t)
    }

    pub fn new_index_info(
        &mut self,
        key_type: &Arc<Type>,
        value_type: &Arc<Type>,
        is_readonly: bool,
        declaration: Option<&Arc<Node>>,
        components: &[Arc<Node>],
    ) -> Arc<IndexInfo> {
        Arc::new(IndexInfo {
            key_type: Some(Arc::clone(key_type)),
            value_type: Some(Arc::clone(value_type)),
            is_readonly,
            declaration: declaration.cloned(),
            index_symbol: None,
            components: components.to_vec(),
        })
    }

    pub fn new_index_type(&mut self, target: &Arc<Type>, index_flags: IndexFlags) -> Arc<Type> {
        let mut data = empty_index_type_data();
        data.target = Some(Arc::clone(target));
        data.index_flags = index_flags;
        self.new_type(TypeFlags::Index, ObjectFlags::empty(), TypeData::Index(data))
    }

    pub fn new_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        access_flags: AccessFlags,
    ) -> Arc<Type> {
        let mut data = empty_indexed_access_type_data();
        data.object_type = Some(Arc::clone(object_type));
        data.index_type = Some(Arc::clone(index_type));
        data.access_flags = access_flags;
        self.new_type(
            TypeFlags::IndexedAccess,
            ObjectFlags::empty(),
            TypeData::IndexedAccess(data),
        )
    }

    pub fn new_intersection_type(
        &mut self,
        object_flags: ObjectFlags,
        types: &[Arc<Type>],
    ) -> Arc<Type> {
        let mut data = IntersectionTypeData::default();
        data.union_or_intersection.types = types.to_vec();
        self.new_type(
            TypeFlags::Intersection,
            object_flags,
            TypeData::Intersection(data),
        )
    }

    pub fn new_intrinsic_type(&mut self, flags: TypeFlags, intrinsic_name: &str) -> Arc<Type> {
        self.new_intrinsic_type_ex(flags, intrinsic_name, ObjectFlags::empty())
    }

    pub fn new_intrinsic_type_ex(
        &mut self,
        flags: TypeFlags,
        intrinsic_name: &str,
        object_flags: ObjectFlags,
    ) -> Arc<Type> {
        let mut data = empty_intrinsic_type_data();
        data.intrinsic_name = intrinsic_name.to_string();
        self.new_type(flags, object_flags, TypeData::Intrinsic(data))
    }

    pub fn new_literal_type(
        &mut self,
        flags: TypeFlags,
        value: LiteralValue,
        regular_type: Option<&Arc<Type>>,
    ) -> Arc<Type> {
        let data = empty_literal_type_data(value);
        let mut t = self.new_type(flags, ObjectFlags::empty(), TypeData::Literal(data));
        let regular = match regular_type {
            Some(regular_type) => Arc::clone(regular_type),
            None => Arc::clone(&t),
        };
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            if let TypeData::Literal(literal) = &mut t_mut.data {
                let _ = literal.regular_type.set(regular);
            }
        }
        t
    }

    pub fn new_object_type(
        &mut self,
        object_flags: ObjectFlags,
        symbol: Option<Arc<Symbol>>,
    ) -> Arc<Type> {
        let data = if object_flags.intersects(ObjectFlags::CLASS_OR_INTERFACE) {
            TypeData::Interface(InterfaceTypeData::default())
        } else if object_flags.intersects(ObjectFlags::Tuple) {
            TypeData::Tuple(TupleTypeData::default())
        } else if object_flags.intersects(ObjectFlags::Reference) {
            TypeData::Object(ObjectTypeData::default())
        } else if object_flags.intersects(ObjectFlags::Mapped) {
            TypeData::Mapped(empty_mapped_type_data())
        } else if object_flags.intersects(ObjectFlags::ReverseMapped) {
            TypeData::ReverseMapped(empty_reverse_mapped_type_data())
        } else if object_flags.intersects(ObjectFlags::EvolvingArray) {
            TypeData::EvolvingArray(empty_evolving_array_type_data())
        } else if object_flags.intersects(ObjectFlags::InstantiationExpressionType) {
            TypeData::InstantiationExpression(empty_instantiation_expression_type_data())
        } else if object_flags.intersects(ObjectFlags::Anonymous) {
            TypeData::Object(ObjectTypeData::default())
        } else {
            panic!("Unhandled case in newObjectType")
        };
        let mut t = self.new_type(TypeFlags::Object, object_flags, data);
        if let Some(symbol) = symbol {
            if let Some(t_mut) = Arc::get_mut(&mut t) {
                t_mut.symbol = Some(symbol);
            }
        }
        t
    }

    pub fn new_parameter(&mut self, name: &str, t: &Arc<Type>) -> Arc<Symbol> {
        let symbol = self.new_symbol(SymbolFlags::FunctionScopedVariable, name);
        if let Some(links) = self.value_symbol_links.get_mut(&symbol) {
            links.resolved_type = Some(Arc::clone(t));
        }
        symbol
    }

    pub fn new_property(&mut self, name: &str, t: &Arc<Type>) -> Arc<Symbol> {
        let symbol = self.new_symbol(SymbolFlags::Property, name);
        if let Some(links) = self.value_symbol_links.get_mut(&symbol) {
            links.resolved_type = Some(Arc::clone(t));
        }
        symbol
    }

    pub fn new_setter_function_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let value_param = self.new_parameter("value", t);
        let void_type = self.void_type();
        self.new_function_type(&[], None, &[value_param], &void_type)
    }

    pub fn new_signature(
        &mut self,
        flags: SignatureFlags,
        declaration: Option<&Arc<Node>>,
        type_parameters: &[Arc<Type>],
        this_parameter: Option<&Arc<Symbol>>,
        parameters: &[Arc<Symbol>],
        resolved_return_type: &Arc<Type>,
        resolved_type_predicate: Option<TypePredicate>,
        min_argument_count: usize,
    ) -> Arc<Signature> {
        self.signature_count += 1;
        Arc::new(Signature {
            id: self.signature_count,
            flags,
            min_argument_count: min_argument_count as i32,
            resolved_min_argument_count: -1,
            declaration: declaration.cloned(),
            type_parameters: type_parameters.to_vec(),
            parameters: parameters.to_vec(),
            this_parameter: this_parameter.cloned(),
            resolved_return_type: OnceLock::from(Arc::clone(resolved_return_type)),
            resolved_type_predicate: resolved_type_predicate.map(Box::new),
            target: None,
            mapper: None,
            isolated_signature_type: OnceLock::new(),
            instantiated_parameter_types: None,
        })
    }

    pub fn new_string_mapping_type(
        &mut self,
        symbol: &Arc<Symbol>,
        target: &Arc<Type>,
    ) -> Arc<Type> {
        let mut data = empty_string_mapping_type_data();
        data.target = Some(Arc::clone(target));
        let mut t = self.new_type(
            TypeFlags::StringMapping,
            ObjectFlags::empty(),
            TypeData::StringMapping(data),
        );
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.symbol = Some(Arc::clone(symbol));
        }
        t
    }

    pub fn new_substitution_type(
        &mut self,
        base_type: &Arc<Type>,
        constraint: &Arc<Type>,
    ) -> Arc<Type> {
        let mut data = empty_substitution_type_data();
        data.base_type = Some(Arc::clone(base_type));
        data.constraint = Some(Arc::clone(constraint));
        self.new_type(
            TypeFlags::Substitution,
            ObjectFlags::empty(),
            TypeData::Substitution(data),
        )
    }

    pub fn new_symbol(&mut self, flags: SymbolFlags, name: &str) -> Arc<Symbol> {
        self.symbol_count += 1;
        Arc::new(Symbol::new(
            flags | SymbolFlags::Transient,
            name.to_string(),
        ))
    }

    pub fn new_symbol_ex(
        &mut self,
        flags: SymbolFlags,
        name: &str,
        check_flags: CheckFlags,
    ) -> Arc<Symbol> {
        let mut result = self.new_symbol(flags, name);
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.check_flags = check_flags;
        }
        result
    }
}
