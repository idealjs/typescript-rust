use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    CheckFlags, ModifierFlags, ModifierList, Node, NodeList, Symbol, SymbolFlags, SymbolTable,
    SyntaxKind,
};
use crate::jsnum;

use super::checker::Checker;
use super::types::*;

fn index_type_less_than_fixed(index_type: &Arc<Type>, limit: usize) -> bool {
    let constituents: Vec<Arc<Type>> = if index_type.flags.contains(TypeFlags::Union) {
        index_type
            .types()
            .map(|ts| ts.to_vec())
            .unwrap_or_default()
    } else {
        vec![Arc::clone(index_type)]
    };
    if constituents.is_empty() {
        return false;
    }
    constituents.iter().all(|c| {
        if let Some(LiteralValue::Number(n)) = c.literal_value() {
            let text = n.to_string();
            if let Ok(index) = text.parse::<f64>() {
                return index >= 0.0 && index < limit as f64;
            }
        }
        false
    })
}

fn is_static_modifier(modifiers: &Option<Arc<ModifierList>>) -> bool {
    modifiers
        .as_ref()
        .map(|m| m.modifier_flags.contains(ModifierFlags::Static))
        .unwrap_or(false)
}

fn template_token_text(node: &Arc<Node>) -> String {
    match &node.data {
        NodeData::TemplateHead(d) => d.text.clone(),
        NodeData::TemplateMiddle(d) => d.text.clone(),
        NodeData::TemplateTail(d) => d.text.clone(),
        _ => String::new(),
    }
}

impl Checker {

    pub fn get_type_from_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {

        let key = (node.id() as usize, self.type_argument_stack_hash());
        if let Some(t) = self.type_node_subst_cache.get(&key) {
            return Arc::clone(t);
        }

        let degraded_epoch = self.heritage_degraded_events;

        if !self.type_node_resolving.insert(key) {
            return self.error_type();
        }
        self.type_node_query_epochs.push(degraded_epoch);

        let over_budget = !self.type_argument_stack.is_empty() && {
            self.type_instantiation_count += 1;
            self.type_instantiation_count >= 5_000_000
        };
        let result = if self.type_resolution_depth >= 500 || over_budget {
            if !self.type_instantiation_limit_reported {
                self.type_instantiation_limit_reported = true;
                let file = self.current_file.clone();
                let loc = self
                    .current_node
                    .as_ref()
                    .map(|n| n.loc)
                    .unwrap_or(node.loc);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    loc,
                    crate::diagnostics::messages_generated::
                        TYPE_INSTANTIATION_IS_EXCESSIVELY_DEEP_AND_POSSIBLY_INFINITE,
                    Vec::new(),
                ));
            }
            self.error_type()
        } else {
            self.type_resolution_depth += 1;
            let r = self.get_type_from_type_node_worker(node);
            self.type_resolution_depth -= 1;
            r
        };
        self.type_node_resolving.remove(&key);
        self.type_node_query_epochs.pop();
        if self.heritage_degraded_events == degraded_epoch {

            if self.type_node_subst_cache.len() >= self.type_node_subst_cache_limit {
                self.type_node_subst_cache.clear();
            }
            self.type_node_subst_cache.insert(key, Arc::clone(&result));
        }
        result
    }

    fn type_argument_stack_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        if self.type_argument_stack.is_empty() {
            return 0;
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for map in &self.type_argument_stack {
            let mut entries: Vec<(usize, usize)> = map
                .iter()
                .map(|(k, v)| (*k as usize, Arc::as_ptr(v) as usize))
                .collect();
            entries.sort_unstable();
            entries.len().hash(&mut h);
            for e in entries {
                e.hash(&mut h);
            }
        }
        h.finish()
    }

    fn get_type_from_type_node_worker(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match node.kind {
            SyntaxKind::AnyKeyword | SyntaxKind::JSDocAllType => self.any_type(),
            SyntaxKind::JSDocNonNullableType => {
                let inner = node
                    .type_node()
                    .expect("JSDocNonNullableType has type")
                    .clone();
                self.get_type_from_type_node(&inner)
            }
            SyntaxKind::JSDocNullableType => {
                let inner = node
                    .type_node()
                    .expect("JSDocNullableType has type")
                    .clone();
                let t = self.get_type_from_type_node(&inner);
                if self.strict_null_checks {
                    self.get_nullable_type(&t, TypeFlags::Null)
                } else {
                    t
                }
            }
            SyntaxKind::JSDocVariadicType => {
                let inner = node
                    .type_node()
                    .expect("JSDocVariadicType has type")
                    .clone();
                let elem_type = self.get_type_from_type_node(&inner);
                self.create_array_type(elem_type)
            }
            SyntaxKind::JSDocOptionalType => {
                let inner = node
                    .type_node()
                    .expect("JSDocOptionalType has type")
                    .clone();
                let t = self.get_type_from_type_node(&inner);
                self.add_optionality(&t)
            }
            SyntaxKind::UnknownKeyword => self.unknown_type(),
            SyntaxKind::StringKeyword => self.string_type(),
            SyntaxKind::NumberKeyword => self.number_type(),
            SyntaxKind::BigIntKeyword => self.bigint_type(),
            SyntaxKind::BooleanKeyword => self.boolean_type(),
            SyntaxKind::SymbolKeyword => self.es_symbol_type(),
            SyntaxKind::VoidKeyword => self.void_type(),
            SyntaxKind::UndefinedKeyword => self.undefined_type(),
            SyntaxKind::NullKeyword => self.null_type(),
            SyntaxKind::NeverKeyword => self.never_type(),
            SyntaxKind::ObjectKeyword => self.non_primitive_type(),

            SyntaxKind::ConstKeyword => self.any_type(),
            SyntaxKind::ThisType | SyntaxKind::ThisKeyword => {
                self.get_type_from_this_type_node(node)
            }
            SyntaxKind::LiteralType => self.get_type_from_literal_type_node(node),
            SyntaxKind::TypeReference | SyntaxKind::ExpressionWithTypeArguments => {
                self.get_type_from_type_reference(node)
            }
            SyntaxKind::TypePredicate => {
                if let NodeData::TypePredicateNode(data) = &node.data {
                    if data.asserts_modifier.is_some() {
                        return self.void_type();
                    }
                }
                self.boolean_type()
            }
            SyntaxKind::TypeQuery => self.get_type_from_type_query_node(node),
            SyntaxKind::ArrayType | SyntaxKind::TupleType => {
                self.get_type_from_array_or_tuple_type_node(node)
            }
            SyntaxKind::OptionalType => self.get_type_from_optional_type_node(node),
            SyntaxKind::UnionType => self.get_type_from_union_type_node(node),
            SyntaxKind::IntersectionType => self.get_type_from_intersection_type_node(node),
            SyntaxKind::NamedTupleMember => self.get_type_from_named_tuple_type_node(node),
            SyntaxKind::ParenthesizedType => {
                let inner = node
                    .type_node()
                    .expect("ParenthesizedType has type")
                    .clone();
                self.get_type_from_type_node(&inner)
            }
            SyntaxKind::RestType => self.get_type_from_rest_type_node(node),
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType | SyntaxKind::TypeLiteral => {
                self.get_type_from_type_literal_or_function_or_constructor_type_node(node)
            }
            SyntaxKind::TypeOperator => self.get_type_from_type_operator_node(node),
            SyntaxKind::IndexedAccessType => self.get_type_from_indexed_access_type_node(node),
            SyntaxKind::TemplateLiteralType => self.get_type_from_template_type_node(node),
            SyntaxKind::MappedType => self.get_type_from_mapped_type_node(node),
            SyntaxKind::ConditionalType => self.get_type_from_conditional_type_node(node),
            SyntaxKind::InferType => self.get_type_from_infer_type_node(node),
            SyntaxKind::ImportType => self.get_type_from_import_type_node(node),
            _ => self.error_type(),
        }
    }

    fn get_cached_type(&self, node: &Arc<Node>) -> Option<Arc<Type>> {
        if !self.type_argument_stack.is_empty() {
            return None;
        }
        self.type_node_links
            .get(node)
            .and_then(|l| l.resolved_type.clone())
    }

    fn cache_type(&mut self, node: &Arc<Node>, t: Arc<Type>) {
        if !self.type_argument_stack.is_empty() {
            return;
        }
        if let Some(epoch) = self.type_node_query_epochs.last()
            && *epoch != self.heritage_degraded_events
        {
            return;
        }
        self.type_node_links.get_or_default(node).resolved_type = Some(t);
    }

    fn get_type_from_this_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_literal_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let literal = match &node.data {
            NodeData::LiteralTypeNode(data) => &data.literal,
            _ => return self.error_type(),
        };
        if literal.kind == SyntaxKind::NullKeyword {
            return self.null_type();
        }
        let result = self.literal_type_from_literal_node(literal);
        self.cache_type(node, result.clone());
        result
    }

    fn literal_type_from_literal_node(&mut self, literal: &Arc<Node>) -> Arc<Type> {
        match literal.kind {
            SyntaxKind::StringLiteral => self.get_string_literal_type(literal.text()),
            SyntaxKind::NumericLiteral => {
                if let Ok(n) = literal.text().parse::<f64>() {
                    self.get_number_literal_type(crate::jsnum::Number::from(n))
                } else {
                    self.number_type()
                }
            }
            SyntaxKind::BigIntLiteral => {
                let text = literal.text();
                if let Some(t) = self.bigint_literal_types.get(text).cloned() {
                    return t;
                }
                let (neg, digits) = if let Some(rest) = text.strip_prefix('-') {
                    (true, rest.trim_end_matches('n'))
                } else {
                    (false, text.trim_end_matches('n'))
                };
                let t = Arc::new(Type::new(
                    TypeFlags::BigIntLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::BigInt(crate::jsnum::PseudoBigInt::new(digits, neg)),
                        fresh_type: std::sync::OnceLock::new(),
                        regular_type: std::sync::OnceLock::new(),
                    }),
                ));
                self.bigint_literal_types
                    .insert(text.to_string(), Arc::clone(&t));
                t
            }
            SyntaxKind::TrueKeyword => self.true_type(),
            SyntaxKind::FalseKeyword => self.false_type(),
            _ => self.error_type(),
        }
    }

    fn get_type_from_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_reference(node);
        self.cache_type(node, result.clone());
        result
    }

    fn resolve_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(data) => (&data.type_name, data.type_arguments.clone()),
            NodeData::ExpressionWithTypeArguments(data) => {
                (&data.expression, data.type_arguments.clone())
            }
            _ => return self.error_type(),
        };

        if type_name.kind == SyntaxKind::Identifier && type_name.text() == "intrinsic" {
            return self.error_type();
        }
        let mut symbol = if type_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(type_name) {
                Some(s) => s,
                None => {

                    if self.ts2304_reporting_allowed_for(type_name) {
                        use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                        let name_text = type_name.text();

                        let file = self
                            .get_source_file_of_node(type_name)
                            .or_else(|| self.current_file.clone());
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            CANNOT_FIND_NAME_0,
                            vec![name_text.to_string()],
                        ));
                    }
                    return self.error_type();
                }
            }
        } else if matches!(
            type_name.kind,
            SyntaxKind::Identifier
                | SyntaxKind::QualifiedName

                | SyntaxKind::PropertyAccessExpression
        ) {
            match self.resolve_qualified_symbol_traced(type_name) {
                Ok(s) => s,
                Err((segment, ns_path, member)) => {

                    let attributed_file = self
                        .get_source_file_of_node(type_name)
                        .or_else(|| self.current_file.clone());
                    let reportable = type_name.kind == SyntaxKind::QualifiedName;
                    if reportable
                        && self.ts2304_reporting_allowed_for(type_name)
                        && attributed_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = attributed_file;
                        if ns_path.is_empty() {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                segment.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_FIND_NAMESPACE_0,
                                vec![segment.text().to_string()],
                            ));
                        } else {

                            let leftmost = super::checker::base_identifier_of(type_name);
                            let left_hit = self
                                .resolve_identifier(&leftmost)
                                .map(|s| self.resolve_alias_base(s));
                            let left_non_namespace = left_hit
                                .as_ref()
                                .is_some_and(|b| !b.flags.intersects(SymbolFlags::NAMESPACE));
                            if left_non_namespace {
                                let name_text = leftmost.text().to_string();
                                if let Some(sugg) =
                                    self.find_name_suggestion(&name_text, SymbolFlags::NAMESPACE)
                                {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1,
                                        vec![name_text.clone(), sugg],
                                    ));
                                } else if left_hit
                                    .as_ref()
                                    .is_some_and(|b| b.flags.intersects(SymbolFlags::TYPE))
                                {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            X_0_ONLY_REFERS_TO_A_TYPE_BUT_IS_BEING_USED_AS_A_NAMESPACE_HERE,
                                        vec![name_text],
                                    ));
                                } else {
                                    self.diagnostics.add(crate::ast::Diagnostic::new(
                                        file,
                                        leftmost.loc,
                                        crate::diagnostics::messages_generated::
                                            CANNOT_FIND_NAMESPACE_0,
                                        vec![name_text],
                                    ));
                                }
                            } else {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    file,
                                    segment.loc,
                                    crate::diagnostics::messages_generated::
                                        NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                                    vec![ns_path, member],
                                ));
                            }
                        }
                    }
                    return self.error_type();
                }
            }
        } else {
            return self.error_type();
        };

        if symbol.flags == SymbolFlags::Alias {
            let alias_name = type_name.text().to_string();
            if let Some(target) = self.resolve_import_alias_target_symbol(&symbol) {
                let target_has_type_meaning = target.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::ENUM
                        | SymbolFlags::TypeParameter,
                );
                if target_has_type_meaning {
                    symbol = target;
                } else {
                    if type_name.kind == SyntaxKind::Identifier
                        && self.ts2304_reporting_allowed_for(type_name)
                        && !self.has_same_named_type_symbol(&alias_name)
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            crate::diagnostics::messages_generated::
                                X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                            vec![alias_name.clone(), alias_name],
                        ));
                    }
                    return self.error_type();
                }
            }
        }

        if !self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
            && symbol
                .flags
                .intersects(SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::TypeAlias)
            && !type_name_inside_conditional_branch(type_name)
            && !type_name_shadowed_by_type_parameter(type_name)
            && !self.check_type_reference_arguments(node, type_name, &symbol)
        {

            return self.error_type();
        }

        if symbol.flags.contains(SymbolFlags::TypeParameter) {

            if self.in_static_member_type {

                let tp_decl = symbol.value_declaration.clone().or_else(|| {
                    symbol.declarations.first().cloned()
                });
                let owned_by_class = tp_decl.is_some_and(|d| {
                    let mut cur = d.parent.as_ref();
                    while let Some(a) = cur {
                        match a.kind {
                            crate::ast::SyntaxKind::ClassDeclaration
                            | crate::ast::SyntaxKind::ClassExpression => return true,
                            crate::ast::SyntaxKind::SourceFile => return false,
                            _ => cur = a.parent.as_ref(),
                        }
                    }
                    false
                });
                if owned_by_class {
                    use crate::diagnostics::messages_generated::STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS;
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        type_name.loc,
                        STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS,
                        Vec::new(),
                    ));
                }
            }

            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            for map in self.type_argument_stack.iter().rev() {
                if let Some(t) = map.get(&key) {
                    return Arc::clone(t);
                }
            }

            for frame in self.type_argument_name_frames.iter().rev() {
                for (frame_sym, t) in frame.iter().rev() {
                    if Arc::ptr_eq(frame_sym, &symbol)
                        || (frame_sym.name == symbol.name
                            && self.type_param_symbols_share_container(frame_sym, &symbol))
                    {
                        return Arc::clone(t);
                    }
                }
            }
            return self.get_type_parameter_from_symbol(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Interface) {

            return self.resolve_interface_type(&symbol, type_arguments);
        }
        if symbol.flags.intersects(SymbolFlags::ENUM) {

            return self.resolve_enum_type(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Class) {

            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            let merged_with_ns = symbol.flags.contains(SymbolFlags::ValueModule);
            if !merged_with_ns {
                if let Some(cached) = self
                    .type_alias_links
                    .get(&symbol)
                    .and_then(|l| l.declared_type.clone())
                {
                    return cached;
                }
            }
            if !self.resolving_type_aliases.insert(key) {
                return self.error_type();
            }
            let class_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                .cloned();
            let instance_type = match class_node {
                Some(node) => self.build_class_instance_type_with_base(&node),
                None => self.error_type(),
            };
            self.resolving_type_aliases.remove(&key);
            if !merged_with_ns {
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&instance_type));
            }

            let arg_types: Option<Vec<Arc<Type>>> = type_arguments.map(|nodes| {
                nodes
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect()
            });
            if let Some(arg_types) = arg_types {
                let tps = self.declared_type_parameter_types(&symbol);
                if !tps.is_empty() && tps.len() == arg_types.len() {
                    return self.attach_explicit_type_arguments_cached(&instance_type, arg_types);
                }
            }
            return instance_type;
        }
        if !symbol.flags.contains(SymbolFlags::TypeAlias) {

            if matches!(&node.data, NodeData::ExpressionWithTypeArguments(_))
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::TypeParameter,
                )
            {
                let value_type = self.get_type_of_symbol(&symbol);
                if let Some(structured) = value_type.as_structured() {
                    for sig in structured.construct_signatures() {
                        if let Some(ret) = self.get_return_type_of_signature(sig) {
                            return ret;
                        }
                    }
                }
                return self.get_any_type();
            }

            if type_name.kind == SyntaxKind::Identifier
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeParameter
                        | SymbolFlags::TypeAlias

                        | SymbolFlags::Alias,
                )
                && self.ts2304_reporting_allowed_for(type_name)
                && !self.has_same_named_type_symbol(type_name.text())

                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let name_text = type_name.text().to_string();
                let file = self.current_file.clone();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    crate::diagnostics::messages_generated::
                        X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                    vec![name_text.clone(), name_text],
                ));
            }

            return self.error_type();
        }

        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }

        let has_type_args = type_arguments.is_some();
        let resolved = if !has_type_args {

            let cached = self
                .type_alias_links
                .get(&symbol)
                .and_then(|l| l.declared_type.clone());
            cached.unwrap_or_else(|| {

                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let found = self.resolve_alias_body(&symbol);
                self.in_static_member_type = saved_static;
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&found));
                found
            })
        } else {

            let (tp_symbols, type_node) = self.collect_alias_type_params_and_body(&symbol);
            let arg_types: Vec<Arc<Type>> = match &type_arguments {
                Some(args) => args
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect(),
                None => Vec::new(),
            };
            let mut mapping = HashMap::new();
            for (i, tp_sym) in tp_symbols.iter().enumerate() {
                if i < arg_types.len() {
                    let tp_key = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                    mapping.insert(tp_key, Arc::clone(&arg_types[i]));
                }
            }
            self.type_argument_stack.push(mapping);

            let alias_decl = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::TypeAliasDeclaration)
                .cloned();
            if let Some(decl) = &alias_decl {
                self.push_scope(decl);
            }
            let saved_static = self.in_static_member_type;
            self.in_static_member_type = false;
            let found = self.get_type_from_type_node(&type_node);
            self.in_static_member_type = saved_static;
            if alias_decl.is_some() {
                self.pop_scope();
            }
            self.type_argument_stack.pop();
            found
        };
        self.pop_type_resolution();
        resolved
    }

    pub fn resolve_alias_body(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {

                self.push_scope(decl);
                let result = self.get_type_from_type_node(&data.type_node);
                self.pop_scope();
                return result;
            }
        }
        self.error_type()
    }

    fn collect_alias_type_params_and_body(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> (Vec<Arc<Symbol>>, Arc<Node>) {
        let mut tp_symbols = Vec::new();
        let mut type_node = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                type_node = Some(Arc::clone(&data.type_node));
                if let Some(tps) = &data.type_parameters {
                    for tp in tps.iter() {
                        if let Some(tp_sym) = self.program.symbol_map().symbol_of(tp) {
                            tp_symbols.push(Arc::clone(tp_sym));
                        }
                    }
                }
                break;
            }
        }
        (
            tp_symbols,
            type_node.unwrap_or_else(|| Arc::clone(&symbol.declarations[0])),
        )
    }

    pub(crate) fn resolve_interface_type(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Type> {
        let arg_types = type_arguments.map(|nodes| {
            nodes
                .iter()
                .map(|a| self.get_type_from_type_node(a))
                .collect()
        });
        self.resolve_interface_type_ex(symbol, arg_types)
    }

    pub(crate) fn resolve_interface_type_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        type_args: Option<Vec<Arc<Type>>>,
    ) -> Arc<Type> {

        let has_type_args = type_args.is_some();
        if !has_type_args {
            if let Some(cached) = self
                .type_alias_links
                .get(symbol)
                .and_then(|l| l.declared_type.clone())
            {
                if !crate::checker::utilities::is_type_error(&cached) {
                    return cached;
                }
            }
        }

        let instantiation_key: Option<Vec<usize>> = type_args.as_ref().map(|args| {
            let mut key = Vec::with_capacity(args.len() + 1);
            key.push(Arc::as_ptr(symbol) as *const Symbol as usize);
            key.extend(
                args.iter()
                    .map(|t| Arc::as_ptr(t) as *const crate::checker::types::Type as usize),
            );
            key
        });

        let pinned_args: Option<Vec<Arc<Type>>> = type_args.clone();
        if let Some(key) = &instantiation_key
            && let Some(cached) = self.interface_instantiation_cache.get(key)
        {
            return Arc::clone(&cached.1);
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {

            self.heritage_degraded_events += 1;
            return self.error_type();
        }

        let interface_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))
            .cloned()
            .collect();

        let epoch_at_entry = self.heritage_degraded_events;
        let mut heritage_degraded = false;
        let result = match interface_decls.first() {
            Some(first) => {
                let data = match &first.data {
                    NodeData::InterfaceDeclaration(d) => d,
                    _ => unreachable!(),
                };

                let tp_symbols = match &data.type_parameters {
                    Some(tps) => {
                        let sym_map = self.program.symbol_map();
                        let collected: Vec<Arc<Symbol>> = tps
                            .iter()
                            .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                            .collect();
                        collected
                    }
                    None => Vec::new(),
                };

                let arg_types: Vec<Arc<Type>> = type_args.unwrap_or_default();
                if has_type_args {
                    let mut mapping = HashMap::new();
                    for (i, tp_sym) in tp_symbols.iter().enumerate() {
                        if let Some(arg) = arg_types.get(i) {
                            let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                            mapping.insert(k, Arc::clone(arg));
                        }
                    }
                    for decl in &interface_decls {
                        let NodeData::InterfaceDeclaration(d) = &decl.data else {
                            continue;
                        };
                        let Some(tps) = &d.type_parameters else {
                            continue;
                        };
                        let sym_map = self.program.symbol_map();
                        for (i, tp) in tps.iter().enumerate() {
                            let Some(tp_sym) = sym_map.symbol_of(tp) else {
                                continue;
                            };

                            let idx = if let Some(first_sym) = tp_symbols.get(i) {
                                if first_sym.name == tp_sym.name { i } else {
                                    tp_symbols.iter().position(|s| s.name == tp_sym.name).unwrap_or(i)
                                }
                            } else {
                                i
                            };
                            if let Some(arg) = arg_types.get(idx) {
                                let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                                mapping.insert(k, Arc::clone(arg));
                            }
                        }
                    }
                    self.type_argument_stack.push(mapping);
                }

                self.push_scope(
                    symbol
                        .declarations
                        .iter()
                        .next()
                        .expect("interface has a declaration"),
                );

                let merged_members: Vec<Arc<Node>> = interface_decls
                    .iter()
                    .flat_map(|decl| match &decl.data {
                        NodeData::InterfaceDeclaration(d) => d.members.iter().cloned(),
                        _ => unreachable!(),
                    })
                    .collect();
                let merged_list = Arc::new(NodeList::new(merged_members));

                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let own_result = self.build_interface_type_from_members(&merged_list);
                self.in_static_member_type = saved_static;

                let mut base_types: Vec<(Arc<Node>, Arc<Type>)> = Vec::new();
                for decl in &interface_decls {
                    if let NodeData::InterfaceDeclaration(d) = &decl.data {
                        if let Some(heritage) = &d.heritage_clauses {
                            for clause in heritage.iter() {
                                if let NodeData::HeritageClause(hc) = &clause.data
                                    && hc.token == SyntaxKind::ExtendsKeyword
                                {
                                    for type_ref in hc.types.iter() {
                                        let bt = self.get_type_from_type_node(type_ref);

                                        if crate::checker::utilities::is_type_error(&bt)
                                            && !heritage_degraded
                                        {
                                            heritage_degraded = true;
                                            self.heritage_degraded_events += 1;
                                        }
                                        base_types.push((Arc::clone(type_ref), bt));
                                    }
                                }
                            }
                        }
                    }
                }
                self.pop_scope();
                if has_type_args {
                    self.type_argument_stack.pop();
                }
                let result = if base_types.is_empty() {
                    own_result.clone()
                } else {
                    let mut merged = own_result.clone();
                    for (_, base) in &base_types {
                        merged = self.merge_interface_type_with_base(&merged, base);
                    }
                    merged
                };

                if !has_type_args && !base_types.is_empty() {
                    let own_structured = match &own_result.data {
                        TypeData::Object(o) => Some(&o.structured),
                        _ => None,
                    };
                    let name_loc = interface_decls.first().and_then(|d| {
                        match &d.data {
                            NodeData::InterfaceDeclaration(d) => Some(d.name.loc),
                            _ => None,
                        }
                    });
                                            if let (Some(own), Some(name_loc)) = (own_structured, name_loc) {
                        for (type_ref_node, base) in &base_types {

                            let dedup_key = (
                                Arc::as_ptr(symbol) as *const crate::ast::Symbol,
                                Arc::as_ptr(type_ref_node) as *const crate::ast::Node,
                            );
                            if self.interface_extends_reported.contains(&dedup_key) {
                                continue;
                            }
                            let base_structured = match &base.data {
                                TypeData::Object(o) => Some(&o.structured),
                                _ => None,
                            };
                            let Some(base_structured) = base_structured else {
                                continue;
                            };
                            for own_prop in &own.properties {
                                let Some(base_prop) = base_structured
                                    .members
                                    .get(&own_prop.name)
                                else {
                                    continue;
                                };
                                let derived_type = self
                                    .value_symbol_links
                                    .get(own_prop)
                                    .and_then(|l| l.resolved_type.clone());
                                let base_type = self
                                    .value_symbol_links
                                    .get(base_prop)
                                    .and_then(|l| l.resolved_type.clone());
                                if let (Some(dt), Some(bt)) = (derived_type, base_type) {

                                    let bt = match bt.symbol.as_ref() {
                                        Some(bsym) => {
                                            let tps = self.declared_type_parameter_types(bsym);
                                            if !tps.is_empty()
                                                && bt.as_object().is_none_or(|o| {
                                                    o.type_arguments.is_empty()
                                                })
                                            {
                                                let anys: Vec<Arc<Type>> = std::iter::repeat(
                                                    self.get_any_type(),
                                                )
                                                .take(tps.len())
                                                .collect();
                                                self.resolve_interface_type_ex(
                                                    bsym,
                                                    Some(anys),
                                                )
                                            } else {
                                                bt
                                            }
                                        }
                                        None => bt,
                                    };

                                    let saved_chain =
                                        std::mem::take(&mut self.relater_error_chain);
                                    let was_active = self.relater_chain_active;
                                    self.relater_chain_active = true;
                                    let incompatible = !self.is_type_assignable_to(&dt, &bt);
                                    let captured = std::mem::replace(
                                        &mut self.relater_error_chain,
                                        saved_chain,
                                    );
                                    self.relater_chain_active = was_active;
                                    if incompatible {
                                        self.interface_extends_reported.insert(dedup_key);
                                        let base_name = base
                                            .symbol
                                            .as_ref()
                                            .map(|s| s.name.clone())
                                            .unwrap_or_default();
                                        let file = self.current_file.clone();
                                        let mut diag = crate::ast::Diagnostic::new(
                                            file,
                                            name_loc,
                                            crate::diagnostics::messages_generated::
                                                INTERFACE_0_INCORRECTLY_EXTENDS_INTERFACE_1,
                                            vec![symbol.name.clone(), base_name],
                                        );

                                        let dt_str = self.type_to_string(&dt);
                                        let bt_str = self.type_to_string(&bt);
                                        self.relater_error_chain = captured;
                                        self.relater_chain_active = true;
                                        self.push_relation_head_with_tp_note(
                                            &dt,
                                            &bt,
                                            crate::diagnostics::messages_generated::
                                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                            vec![dt_str, bt_str],
                                        );
                                        self.relater_report_error(
                                            crate::diagnostics::messages_generated::
                                                TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                                            vec![self.chain_property_arg_name(own_prop)],
                                        );
                                        let entries = std::mem::take(
                                            &mut self.relater_error_chain,
                                        );
                                        self.relater_chain_active = was_active;

                                        let mut child: Option<
                                            crate::ast::Diagnostic,
                                        > = None;
                                        for entry in entries
                                            .iter()
                                            .filter(|e| {
                                                !e.message.elided_in_compatibility_pyramid
                                            })
                                        {
                                            let mut d = crate::ast::Diagnostic::new(
                                                None,
                                                name_loc,
                                                entry.message,
                                                entry.args.clone(),
                                            );
                                            if let Some(c) = child.take() {
                                                d.message_chain = vec![c];
                                            }
                                            child = Some(d);
                                        }
                                        if let Some(c) = child {
                                            diag.message_chain = vec![c];
                                        }
                                        self.diagnostics.add(diag);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                {
                    let result_mut = Arc::as_ptr(&result) as *mut crate::checker::types::Type;
                    unsafe {
                        (*result_mut).symbol = Some(Arc::clone(symbol));
                        if has_type_args
                            && let TypeData::Object(o) = &mut (*result_mut).data
                        {
                            o.type_arguments = arg_types.clone();
                        }
                    }
                }
                result
            }
            None => self.error_type(),
        };
        self.pop_type_resolution();

        if self.heritage_degraded_events != epoch_at_entry {
            heritage_degraded = true;
        }

        let mut degraded_accepted = false;
        if heritage_degraded {
            let sym_key = Arc::as_ptr(symbol) as *const crate::ast::Symbol as usize;
            let retries = self.heritage_retry_counts.entry(sym_key).or_insert(0);
            *retries += 1;
            degraded_accepted = *retries > crate::checker::checker::HERITAGE_RETRY_LIMIT;
        }
        let cache_result = !heritage_degraded || degraded_accepted;
        if degraded_accepted && self.heritage_degraded_events != epoch_at_entry {
            self.heritage_degraded_events = epoch_at_entry;
        }
        if heritage_degraded {

            self.degraded_type_ptrs
                .insert(Arc::as_ptr(&result) as *const crate::checker::types::Type as usize);
        }
        if !has_type_args && cache_result {
            self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        }
        if let Some(key) = instantiation_key {
            if cache_result {

                let pin = pinned_args.clone().unwrap_or_default();
                self.interface_instantiation_cache
                    .insert(key, (pin, Arc::clone(&result)));
            }
        }
        result
    }

    pub(crate) fn build_interface_type_from_members(
        &mut self,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();

        let mut call_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut construct_signatures: Vec<Arc<Signature>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    let mut prop_type = self.get_type_from_type_node(&data.type_node);
                    let is_optional = data
                        .postfix_token
                        .as_ref()
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);

                    if is_optional {
                        prop_type = self.get_optional_type(prop_type);
                    }
                    let mut flags = SymbolFlags::Property;
                    if is_optional {
                        flags |= SymbolFlags::Optional;
                    }
                    let mut symbol = Symbol::new(flags, name.clone());

                    symbol.declarations = vec![Arc::clone(member)];

                    if let Some(m) = &data.modifiers {
                        if m.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                    }
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::MethodSignatureDeclaration(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         false,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();

                    if let Some(existing) = symbol_table.get(&name).cloned() {
                        let existing_type = self
                            .value_symbol_links
                            .get(&existing)
                            .and_then(|l| l.resolved_type.clone());
                        let merged_sigs = existing_type
                            .as_ref()
                            .and_then(|t| t.as_structured().map(|s| s.call_signatures().to_vec()))
                            .unwrap_or_default();
                        let mut all_sigs = merged_sigs;
                        all_sigs.push(sig);
                        let fn_type = self.create_function_or_constructor_type(all_sigs, false);
                        self.value_symbol_links.insert(
                            &existing,
                            ValueSymbolLinks {
                                resolved_type: Some(fn_type),
                                ..Default::default()
                            },
                        );
                        continue;
                    }
                    let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                    let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::IndexSignatureDeclaration(data) => {
                    let mut key_type = None;
                    let value_type;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd
                                .type_node
                                .as_ref()
                                .map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    let is_readonly = member
                        .modifiers()
                        .as_ref()
                        .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly,
                        declaration: Some(Arc::clone(member)),
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }

                NodeData::PropertyDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }
                    let mut prop_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => match data.initializer.as_ref() {
                            Some(init) => {

                                let raw = match &init.data {
                                    NodeData::Identifier(_) => {
                                        match self.resolve_identifier(init) {
                                            Some(sym) if sym.flags.intersects(
                                                SymbolFlags::BlockScopedVariable
                                                    | SymbolFlags::FunctionScopedVariable,
                                            ) => self.get_type_of_symbol(&sym),
                                            _ => self.get_type_of_node(init),
                                        }
                                    }
                                    _ => self.get_type_of_node(init),
                                };
                                let is_readonly = data
                                    .modifiers
                                    .as_ref()
                                    .is_some_and(|m| {
                                        m.modifier_flags.contains(ModifierFlags::Readonly)
                                    });
                                let widened = if is_readonly {
                                    raw
                                } else if self.is_empty_array_literal(init) {

                                    if self.strict_null_checks {
                                        self.get_widened_literal_type(&raw)
                                    } else {
                                        self.create_array_type(self.get_any_type())
                                    }
                                } else {
                                    self.get_widened_literal_type(&raw)
                                };
                                let regularized =
                                    self.get_regular_type_of_literal_type(&widened);
                                self.widen_initializer_type(&regularized)
                            }
                            None => self.get_any_type(),
                        },
                    };
                    let is_optional = data
                        .postfix_token
                        .as_ref()
                        .map(|t| t.kind == SyntaxKind::QuestionToken)
                        .unwrap_or(false);
                    if is_optional {
                        prop_type = self.get_optional_type(prop_type);
                    }
                    let mut flags = SymbolFlags::Property;
                    if is_optional {
                        flags |= SymbolFlags::Optional;
                    }
                    let mut symbol = Symbol::new(flags, name.clone());

                    symbol.declarations.push(Arc::clone(member));

                    if let Some(m) = &data.modifiers {
                        if m.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                    }
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::MethodDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         false,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();

                    if let Some(existing) = symbol_table.get(&name).cloned() {
                        if data.body.is_some() {

                            let existing_mut =
                                Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                            continue;
                        }
                        let existing_type = self
                            .value_symbol_links
                            .get(&existing)
                            .and_then(|l| l.resolved_type.clone());
                        let merged_sigs = existing_type
                            .as_ref()
                            .and_then(|t| {
                                t.as_structured().map(|s| s.call_signatures().to_vec())
                            })
                            .unwrap_or_default();
                        let mut all_sigs = merged_sigs;
                        all_sigs.push(sig);
                        let fn_type =
                            self.create_function_or_constructor_type(all_sigs, false);
                        self.value_symbol_links.insert(
                            &existing,
                            ValueSymbolLinks {
                                resolved_type: Some(fn_type),
                                ..Default::default()
                            },
                        );
                        continue;
                    }
                    let fn_type = self.create_function_or_constructor_type(vec![sig], false);
                    let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());

                    symbol.declarations.push(Arc::clone(member));
                    let symbol = Arc::new(symbol);
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(fn_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::GetAccessorDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    let prop_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    match symbol_table.get(&name).cloned() {
                        Some(existing) => {

                            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).flags |= SymbolFlags::GetAccessor;
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                            self.value_symbol_links.insert(
                                &existing,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                        }
                        None => {
                            let mut symbol =
                                Symbol::new(SymbolFlags::Property | SymbolFlags::GetAccessor, name.clone());
                            symbol.declarations.push(Arc::clone(member));
                            let symbol = Arc::new(symbol);
                            self.value_symbol_links.insert(
                                &symbol,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                            symbol_table.insert(name, Arc::clone(&symbol));
                            props.push(symbol);
                        }
                    }
                }
                NodeData::SetAccessorDeclaration(data) => {
                    if is_static_modifier(&data.modifiers) {
                        continue;
                    }
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        continue;
                    }

                    let prop_type = data
                        .parameters
                        .iter()
                        .next()
                        .and_then(|p| {
                            if let NodeData::ParameterDeclaration(pd) = &p.data {
                                pd.type_node.as_ref().map(|tn| self.get_type_from_type_node(tn))
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| self.get_any_type());
                    match symbol_table.get(&name).cloned() {
                        Some(existing) => {

                            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                            unsafe {
                                (*existing_mut).flags |= SymbolFlags::SetAccessor;
                                (*existing_mut).declarations.push(Arc::clone(member));
                            }
                        }
                        None => {
                            let mut symbol =
                                Symbol::new(SymbolFlags::Property | SymbolFlags::SetAccessor, name.clone());
                            symbol.declarations.push(Arc::clone(member));
                            let symbol = Arc::new(symbol);
                            self.value_symbol_links.insert(
                                &symbol,
                                ValueSymbolLinks {
                                    resolved_type: Some(prop_type),
                                    ..Default::default()
                                },
                            );
                            symbol_table.insert(name, Arc::clone(&symbol));
                            props.push(symbol);
                        }
                    }
                }
                NodeData::CallSignatureDeclaration(data) => {

                    let suppress = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.file_name.starts_with("bundled://"));
                    if suppress {
                        self.push_ts2304_suppression();
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         false,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    if suppress {
                        self.pop_ts2304_suppression();
                    }
                    call_signatures.push(sig);
                }
                NodeData::ConstructSignatureDeclaration(data) => {
                    let suppress = self
                        .current_file
                        .as_ref()
                        .is_some_and(|f| f.file_name.starts_with("bundled://"));
                    if suppress {
                        self.push_ts2304_suppression();
                    }

                    self.push_scope(member);
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                         true,
                         None,
                         Some(Arc::clone(member)),
                    );
                    self.pop_scope();
                    if suppress {
                        self.pop_ts2304_suppression();
                    }
                    construct_signatures.push(sig);
                }
                NodeData::ConstructorDeclaration(data) => {

                    for param in data.parameters.iter() {
                        let NodeData::ParameterDeclaration(pd) = &param.data else {
                            continue;
                        };
                        if pd.name.kind != SyntaxKind::Identifier {
                            continue;
                        }
                        let Some(modifiers) = &pd.modifiers else {
                            continue;
                        };
                        if !modifiers.modifier_flags.intersects(
                            ModifierFlags::Public
                                | ModifierFlags::Private
                                | ModifierFlags::Protected
                                | ModifierFlags::Readonly,
                        ) {
                            continue;
                        }
                        let name = pd.name.text().to_string();
                        if name.is_empty() || symbol_table.get(&name).is_some() {
                            continue;
                        }
                        let prop_type = match pd.type_node.as_ref() {
                            Some(tn) => self.get_type_from_type_node(tn),
                            None => match pd.initializer.as_ref() {
                                Some(init) => self.get_type_of_node(init),
                                None => self.get_any_type(),
                            },
                        };
                        let mut symbol = Symbol::new(SymbolFlags::Property, name.clone());

                        symbol.declarations.push(Arc::clone(param));
                        if modifiers.modifier_flags.contains(ModifierFlags::Readonly) {
                            symbol.check_flags |= CheckFlags::Readonly;
                        }
                        let symbol = Arc::new(symbol);
                        self.value_symbol_links.insert(
                            &symbol,
                            ValueSymbolLinks {
                                resolved_type: Some(prop_type),
                                ..Default::default()
                            },
                        );
                        symbol_table.insert(name, Arc::clone(&symbol));
                        props.push(symbol);
                    }
                }
                _ => {}
            }
        }

        let call_signature_count = call_signatures.len();
        let mut signatures = call_signatures;
        signatures.extend(construct_signatures);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    fn merge_interface_type_with_base(
        &mut self,
        derived: &Arc<Type>,
        base: &Arc<Type>,
    ) -> Arc<Type> {
        if base.flags.contains(TypeFlags::Any) {
            return Arc::clone(derived);
        }
        let derived_data = match &derived.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let base_data = match &base.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();

        for prop in &derived_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        for prop in &base_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                continue;
            }
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }
        let mut index_infos = derived_data.index_infos.clone();
        index_infos.extend(base_data.index_infos.iter().cloned());

        let mut call_signatures: Vec<Arc<Signature>> =
            derived_data.call_signatures().to_vec();
        let derived_call_count = call_signatures.len();
        call_signatures.extend(base_data.call_signatures().iter().cloned());
        let mut signatures = call_signatures;
        signatures.extend(derived_data.construct_signatures().iter().cloned());
        signatures.extend(base_data.construct_signatures().iter().cloned());
        let merged = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count: derived_call_count
                        + base_data.call_signatures().len(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        });

        let merged_degraded = {
            let p = |t: &Arc<Type>| Arc::as_ptr(t) as *const Type as usize;
            self.degraded_type_ptrs.contains(&p(base))
                || self.degraded_type_ptrs.contains(&p(derived))
        };
        if merged_degraded {
            self.degraded_type_ptrs
                .insert(Arc::as_ptr(&merged) as *const Type as usize);
        }
        merged
    }

    pub fn resolve_enum_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }

        let sym_map = self.program.symbol_map();
        let mut entries: Vec<(Option<Arc<Symbol>>, String, Option<Arc<Node>>)> = Vec::new();
        for decl in symbol.declarations.iter() {
            if let NodeData::EnumDeclaration(data) = &decl.data {
                for member_node in data.members.iter() {
                    let NodeData::EnumMember(member) = &member_node.data else {
                        continue;
                    };
                    let member_name = member.name.text().to_string();
                    let member_sym = sym_map.symbol_of(member_node).map(Arc::clone);
                    entries.push((member_sym, member_name, member.initializer.clone()));
                }
            }
        }
        let result = if entries.is_empty() {
            self.error_type()
        } else {
            let mut member_types: Vec<Arc<Type>> = Vec::new();
            let mut next_value: Option<f64> = Some(0.0);
            for (member_sym, member_name, initializer) in &entries {
                let base = match initializer {
                    Some(init) => {

                        let t = self.get_type_of_node(init);

                        if t.flags.contains(TypeFlags::NumberLiteral) {
                            if let TypeData::Literal(LiteralTypeData {
                                value: LiteralValue::Number(n),
                                ..
                            }) = &t.data
                            {
                                next_value = Some(n.0 + 1.0);
                            }
                        } else if t.flags.contains(TypeFlags::StringLiteral) {

                            next_value = None;
                        }
                        t
                    }
                    None => match next_value {
                        Some(v) => {
                            next_value = Some(v + 1.0);
                            self.get_number_literal_type(jsnum::Number::from(v))
                        }
                        None => {

                            self.get_any_type()
                        }
                    },
                };

                let member_type = if base
                    .flags
                    .intersects(TypeFlags::NumberLiteral | TypeFlags::StringLiteral)
                {
                    let value = match &base.data {
                        TypeData::Literal(lit) => lit.value.clone(),
                        _ => LiteralValue::None,
                    };
                    let enum_literal_flags = base.flags | TypeFlags::EnumLiteral;
                    let mut regular_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value: value.clone(),
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::new(),
                        }),
                    );
                    regular_ty.symbol = member_sym.clone();
                    let regular_ty = Arc::new(regular_ty);
                    let mut fresh_ty = Type::new(
                        enum_literal_flags,
                        TypeData::Literal(LiteralTypeData {
                            value,
                            fresh_type: OnceLock::new(),
                            regular_type: OnceLock::from(Arc::clone(&regular_ty)),
                        }),
                    );
                    fresh_ty.symbol = member_sym.clone();
                    let fresh_ty = Arc::new(fresh_ty);

                    if let TypeData::Literal(reg_lit) = &regular_ty.data {
                        let _ = reg_lit.fresh_type.set(Arc::clone(&fresh_ty));
                    }
                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(fresh_ty),
                                ..Default::default()
                            },
                        );
                    }
                    regular_ty
                } else {

                    if let Some(ms) = member_sym {
                        self.value_symbol_links.insert(
                            ms,
                            ValueSymbolLinks {
                                resolved_type: Some(Arc::clone(&base)),
                                ..Default::default()
                            },
                        );
                    }
                    base
                };
                let _ = member_name;
                member_types.push(member_type);
            }
            match member_types.len() {
                0 => self.never_type(),
                1 => member_types.into_iter().next().unwrap(),
                _ => self.get_union_type(member_types),
            }
        };
        self.pop_type_resolution();
        self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        result
    }

    pub(crate) fn get_type_of_prototype_property(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        let Some(parent) = symbol.parent.clone() else {
            return self.get_any_type();
        };
        let Some(class_decl) = parent
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ClassDeclaration(_)))
            .cloned()
        else {
            return self.get_any_type();
        };
        let ctor_type = self.get_type_of_class_declaration(&class_decl);
        let instance_type = ctor_type
            .as_structured()
            .and_then(|s| s.construct_signatures().first().cloned())
            .and_then(|sig| self.get_return_type_of_signature(&sig))
            .unwrap_or_else(|| self.get_any_type());
        let tp_types: Vec<Arc<Type>> = match &class_decl.data {
            NodeData::ClassDeclaration(d) => match &d.type_parameters {
                Some(tps) => {
                    let sym_map = self.program.symbol_map();
                    let tp_syms: Vec<Arc<Symbol>> = tps
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect();
                    tp_syms
                        .iter()
                        .map(|s| self.get_type_parameter_from_symbol(s))
                        .collect()
                }
                None => Vec::new(),
            },
            _ => Vec::new(),
        };
        if tp_types.is_empty() {
            return instance_type;
        }
        let any_t = self.get_any_type();
        let anys: Vec<Arc<Type>> = tp_types.iter().map(|_| Arc::clone(&any_t)).collect();
        self.substitute_infer_type_parameters(&instance_type, &tp_types, &anys)
    }

    fn get_type_parameter_from_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(ref t) = links.declared_type {
                return Arc::clone(t);
            }
        }
        let sym_key = Arc::as_ptr(symbol) as usize;
        if !self.type_parameter_resolving.insert(sym_key) {

            return Arc::new(Type {
                flags: TypeFlags::TypeParameter,
                object_flags: ObjectFlags::None,
                id: 0,
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::TypeParameter(TypeParameterData {
                    constrained: ConstrainedTypeData::default(),
                    constraint: None,
                    target: None,
                    mapper: None,
                    is_this_type: false,
                    resolved_default_type: OnceLock::new(),
                }),
            });
        }
        let mut constraint: Option<Arc<Type>> = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeParameterDeclaration(data) = &decl.data {
                if let Some(constraint_node) = &data.constraint {
                    constraint = Some(self.get_type_from_type_node(constraint_node));
                }
                break;
            }
        }

        if let Some(c) = &constraint
            && self.constraint_chain_is_circular(sym_key, c)
        {
            if self.ts2313_reported.insert(sym_key) {
                let loc = symbol
                    .declarations
                    .iter()
                    .find_map(|d| match &d.data {
                        NodeData::TypeParameterDeclaration(td) => {
                            td.constraint.as_ref().map(|cn| cn.loc)
                        }
                        _ => None,
                    })
                    .or_else(|| symbol.declarations.first().map(|d| d.loc))
                    .unwrap_or_default();
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    crate::diagnostics::messages_generated::
                        TYPE_PARAMETER_0_HAS_A_CIRCULAR_CONSTRAINT,
                    vec![symbol.name.clone()],
                ));
            }
            constraint = None;
        }
        let tp = Arc::new(Type {
            flags: TypeFlags::TypeParameter,
            object_flags: ObjectFlags::None,
            id: 0,
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::TypeParameter(TypeParameterData {
                constrained: ConstrainedTypeData::default(),
                constraint,
                target: None,
                mapper: None,
                is_this_type: false,
                resolved_default_type: OnceLock::new(),
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&tp));
        self.type_parameter_resolving.remove(&sym_key);
        tp
    }

    fn constraint_chain_is_circular(&self, start_key: usize, constraint: &Arc<Type>) -> bool {
        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut current = constraint;
        for _ in 0..50 {
            let TypeData::TypeParameter(tp) = &current.data else {
                return false;
            };
            let Some(sym) = &current.symbol else { return false };
            let key = Arc::as_ptr(sym) as usize;
            if !visited.insert(key) {
                return true;
            }
            match &tp.constraint {
                Some(next) => current = next,
                None => return key == start_key,
            }
        }
        false
    }

    pub fn resolve_namespace_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if let Some(cached) = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let mut members: Vec<(String, Arc<Symbol>)> = symbol
            .exports
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();

        if self.ambient_namespace_locals_visible(symbol) {
            let local_members: Vec<(String, Arc<Symbol>)> = symbol
                .declarations
                .iter()
                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                .filter_map(|d| {
                    self.program
                        .symbol_map()
                        .locals
                        .get(&d.id())
                        .map(|l| {
                            l.iter()
                                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                                .collect::<Vec<(String, Arc<Symbol>)>>()
                        })
                })
                .flatten()
                .collect();
            for (k, v) in local_members {
                if !members.iter().any(|(mk, _)| *mk == k) {
                    members.push((k, v));
                }
            }
        }

        let mut file_exported: Vec<(String, Arc<Symbol>)> = Vec::new();
        if symbol
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
            && members.is_empty()
        {
            let sym_map = self.program.symbol_map();

            let mut wanted: Vec<(String, Option<String>)> = Vec::new();
            let mut default_node: Option<Arc<Node>> = None;
            self.for_each_module_statement(symbol, |stmt| {
                let has_export =
                    stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export);
                match &stmt.data {
                    NodeData::ExportDeclaration(d) => {
                        if let Some(clause) = &d.export_clause
                            && let NodeData::NamedExports(ne) = &clause.data
                        {
                            for el in ne.elements.iter() {
                                if let NodeData::ExportSpecifier(spec) = &el.data {
                                    let exported = spec
                                        .name
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    let local = spec
                                        .property_name
                                        .as_ref()
                                        .unwrap_or(&spec.name)
                                        .text()
                                        .trim_matches(['"', '\'', '`'])
                                        .to_string();
                                    wanted.push((exported, Some(local)));
                                }
                            }
                        }
                    }
                    NodeData::ExportAssignment(ea) => {
                        if !ea.is_export_equals && default_node.is_none() {
                            default_node = Some(Arc::clone(stmt));
                        }
                    }
                    NodeData::VariableStatement(vs) if has_export => {
                        if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                            for decl in vdl.declarations.iter() {
                                if let Some(name) = decl.name() {
                                    wanted.push((name.text().to_string(), None));
                                }
                            }
                        }
                    }
                    _ if has_export => {
                        if let Some(name) = stmt.name() {
                            wanted.push((name.text().to_string(), None));
                        }
                    }
                    _ => {}
                }
                false
            });
            let file_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SourceFile);
            let locals = file_node.and_then(|f| sym_map.locals.get(&f.id()));
            for (exported, clause_local) in wanted.iter() {
                if members.iter().any(|(k, _)| *k == *exported)
                    || file_exported.iter().any(|(k, _)| *k == *exported)
                {
                    continue;
                }
                let lookup = clause_local.as_deref().unwrap_or(exported);
                if let Some(s) = locals.and_then(|l| l.get(lookup).cloned()) {
                    file_exported.push((exported.clone(), s));
                } else if let Some(s) = symbol.members.get(lookup).cloned() {
                    file_exported.push((exported.clone(), s));
                }
            }
            if let Some(node) = default_node
                && !members.iter().any(|(k, _)| k == "default")
                && !file_exported.iter().any(|(k, _)| k == "default")
            {

                let s = sym_map
                    .symbol_of(&node)
                    .cloned()
                    .or_else(|| node.expression().and_then(|e| sym_map.symbol_of(e).cloned()));
                if let Some(s) = s {
                    file_exported.push(("default".to_string(), s));
                }
            }
        }

        let mut reexported: Vec<(String, Arc<Symbol>)> = Vec::new();
        {
            let mut clause_specs: Vec<(String, String, String)> = Vec::new();
            self.for_each_module_statement(symbol, |stmt| {
                if let NodeData::ExportDeclaration(d) = &stmt.data
                    && let Some(clause) = &d.export_clause
                    && let NodeData::NamedExports(ne) = &clause.data
                    && let Some(module_spec) = &d.module_specifier
                {
                    for el in ne.elements.iter() {
                        if let NodeData::ExportSpecifier(spec) = &el.data {
                            let exported = spec
                                .name
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let imported = spec
                                .property_name
                                .as_ref()
                                .unwrap_or(&spec.name)
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            let module_text = module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string();
                            if !exported.is_empty() && !module_text.is_empty() {
                                clause_specs.push((exported, imported, module_text));
                            }
                        }
                    }
                }
                false
            });
            for (exported, imported, module_text) in clause_specs {
                if members.iter().any(|(k, _)| *k == exported)
                    || reexported.iter().any(|(k, _)| *k == exported)
                {
                    continue;
                }
                let target = self
                    .resolve_module_spec_from(symbol, &module_text)
                    .and_then(|m| self.resolve_module_member_symbol(&m, &imported, 8));
                if let Some(t) = target {
                    reexported.push((exported, t));
                }
            }
        }
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in members.iter().chain(file_exported.iter()).chain(reexported.iter()) {

            if name.starts_with(crate::ast::INTERNAL_SYMBOL_NAME_PREFIX)
                || name == crate::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            {
                continue;
            }
            let member_type = self.get_type_of_symbol(member_sym);

            let prop_sym = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
            self.value_symbol_links.insert(
                &prop_sym,
                ValueSymbolLinks {
                    resolved_type: Some(member_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name.clone(), Arc::clone(&prop_sym));
            props.push(prop_sym);
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        result
    }

    fn get_type_from_type_query_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_query(node);
        self.cache_type(node, result.clone());
        result
    }

    fn resolve_import_alias_target_symbol(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let (member_name, import_decl): (String, Arc<Node>) = {
            let decl = alias
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::ImportClause | SyntaxKind::ImportSpecifier
                    )
                })?
                .clone();
            match &decl.data {
                NodeData::ImportClause(_) => ("default".to_string(), decl),
                NodeData::ImportSpecifier(d) => (
                    d.property_name
                        .as_ref()
                        .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
                    decl,
                ),
                _ => return None,
            }
        };
        let mut import_decl = import_decl.parent.as_ref()?;
        while !matches!(import_decl.data, NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_sym = self
            .resolve_module_file_symbol(&module_spec)
            .or_else(|| {
                let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let cur = self.current_file.clone()?;
                let path = self.program.resolve_external_module_path(
                    &trimmed,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()
            })?;
        let resolved = self
            .resolve_module_member_symbol(&module_sym, &member_name, 8)
            .or_else(|| {

                self.file_module_exported_member(&module_sym, &member_name)
            });
        let resolved = match resolved {
            Some(t)

                if !t.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                ) =>
            {
                let mut cur = Arc::clone(&t);
                for _ in 0..4 {
                    if cur.flags != crate::ast::SymbolFlags::Alias {
                        break;
                    }

                    let next = cur
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .and_then(|d| match &d.data {
                            NodeData::ExportAssignment(ea)
                                if matches!(
                                    ea.expression.kind,
                                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                ) =>
                            {
                                Some(ea.expression.text().to_string())
                            }
                            _ => None,
                        })
                        .and_then(|n| {
                            module_sym
                                .members
                                .get(&n)
                                .cloned()
                                .or_else(|| module_sym.exports.get(&n).cloned())
                        });
                    match next {
                        Some(n) => cur = n,
                        None => break,
                    }
                }
                let has_type_meaning = cur.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                );
                if has_type_meaning {
                    Some(cur)
                } else {
                    Some(t)
                }
            }
            other => other,
        };
        resolved
    }

    fn file_module_exported_member(
        &self,
        module_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if !module_sym
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
        {
            return None;
        }
        if let Some(s) = module_sym.exports.get(name) {
            return Some(Arc::clone(s));
        }
        let sym_map = self.program.symbol_map();
        let mut found: Option<Arc<Symbol>> = None;
        self.for_each_module_statement(module_sym, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(ea) => {
                    if !ea.is_export_equals && name == "default" && found.is_none() {

                        let by_name = match &ea.expression.kind {
                            SyntaxKind::Identifier => module_sym
                                .members
                                .get(ea.expression.text())
                                .cloned()
                                .or_else(|| module_sym.exports.get(ea.expression.text()).cloned()),
                            _ => None,
                        };
                        found = by_name.or_else(|| {
                            sym_map
                                .symbol_of(stmt)
                                .cloned()
                                .or_else(|| {
                                    stmt.expression()
                                        .and_then(|e| sym_map.symbol_of(e).cloned())
                                })
                        });
                    }
                }
                NodeData::VariableStatement(vs) => {
                    if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                        for decl in vdl.declarations.iter() {
                            if decl.name().is_some_and(|n| n.text() == name) {
                                found = sym_map.symbol_of(decl).cloned();
                            }
                        }
                    }
                }
                _ => {
                    if stmt.name().is_some_and(|n| n.text() == name)
                        && (stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
                            || stmt
                                .has_syntactic_modifier(crate::ast::ModifierFlags::Default))
                    {
                        found = sym_map.symbol_of(stmt).cloned();
                    }
                }
            }
            false
        });
        found
    }

    fn resolve_type_query(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let NodeData::TypeQueryNode(d) = &node.data else {
            return self.error_type();
        };

        fn report_unresolved(c: &mut Checker, seg: &Arc<Node>) {
            if c.ts2304_reporting_allowed_for(seg) {
                use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                let file = c
                    .get_source_file_of_node(seg)
                    .or_else(|| c.current_file.clone());
                c.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    seg.loc,
                    CANNOT_FIND_NAME_0,
                    vec![seg.text().to_string()],
                ));
            }
        }
        let symbol = if d.expr_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(&d.expr_name) {
                Some(s) => s,
                None => {
                    report_unresolved(self, &d.expr_name);
                    return self.error_type();
                }
            }
        } else {
            match self.resolve_qualified_symbol(&d.expr_name) {
                Some(s) => s,
                None => {
                    report_unresolved(self, &d.expr_name);
                    return self.error_type();
                }
            }
        };

        if symbol.flags == crate::ast::SymbolFlags::Alias {

            if d.expr_name.kind == SyntaxKind::Identifier {
                let base = self.resolve_alias_base(Arc::clone(&symbol));
                let is_true_namespace = base.declarations.iter().any(|dd| {
                    dd.kind == SyntaxKind::ModuleDeclaration
                        && dd.name().is_some_and(|n| {
                            !matches!(n.kind, SyntaxKind::StringLiteral)
                        })
                });
                if base.flags.contains(crate::ast::SymbolFlags::ValueModule)
                    && is_true_namespace
                    && !self.namespace_usable_as_value(&base)
                {
                    let file = self
                        .get_source_file_of_node(&d.expr_name)
                        .or_else(|| self.current_file.clone());
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        d.expr_name.loc,
                        crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                        vec![d.expr_name.text().to_string()],
                    ));
                    return self.error_type();
                }
            }
            if let Some(t) = self.type_of_imported_symbol(&symbol) {
                return t;
            }
        }

        if symbol.flags.contains(crate::ast::SymbolFlags::ValueModule)
            && d.expr_name.kind == SyntaxKind::Identifier
            && symbol.declarations.iter().any(|dd| {
                dd.kind == SyntaxKind::ModuleDeclaration
                    && dd.name().is_some_and(|n| !matches!(n.kind, SyntaxKind::StringLiteral))
            })
            && !self.namespace_usable_as_value(&symbol)
        {
            let file = self
                .get_source_file_of_node(&d.expr_name)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                d.expr_name.loc,
                crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                vec![d.expr_name.text().to_string()],
            ));
            return self.error_type();
        }

        let class_decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned();
        if let Some(decl) = class_decl {
            if d.type_arguments.is_none() {
                return self.get_type_of_class_declaration(&decl);
            }
            let sym_map = self.program.symbol_map();
            let tp_symbols: Vec<Arc<crate::ast::Symbol>> = match &decl.data {
                NodeData::ClassDeclaration(cd) => match &cd.type_parameters {
                    Some(tps) => tps
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect(),
                    None => Vec::new(),
                },
                _ => Vec::new(),
            };
            let arg_types: Vec<Arc<Type>> = d
                .type_arguments
                .as_ref()
                .unwrap()
                .iter()
                .map(|a| self.get_type_from_type_node(a))
                .collect();
            if tp_symbols.is_empty() || arg_types.len() > tp_symbols.len() {
                return self.get_type_of_class_declaration(&decl);
            }
            let mut key = Vec::with_capacity(arg_types.len() + 1);
            key.push(decl.id() as usize);
            key.extend(
                arg_types
                    .iter()
                    .map(|t| Arc::as_ptr(t) as *const Type as usize),
            );
            if let Some(cached) = self.typequery_instantiation_cache.get(&key) {
                return Arc::clone(&cached.1);
            }
            let mut mapping = HashMap::new();
            for (i, tp) in tp_symbols.iter().enumerate() {
                if i < arg_types.len() {
                    mapping.insert(
                        Arc::as_ptr(tp) as *const crate::ast::Symbol,
                        Arc::clone(&arg_types[i]),
                    );
                }
            }
            self.type_argument_stack.push(mapping);
            let members = match &decl.data {
                NodeData::ClassDeclaration(data) => Arc::clone(&data.members),
                _ => unreachable!("class decl checked above"),
            };

            let built = if self.class_type_resolution_stack.contains(&decl.id()) {

                self.get_type_of_class_declaration(&decl)
            } else {
                self.class_type_resolution_stack.push(decl.id());
                let built = self.build_type_of_class_declaration(&decl, &members);
                self.class_type_resolution_stack.pop();
                built
            };
            self.type_argument_stack.pop();

            let stripped = Arc::new(Type::new(
                built.flags,
                crate::checker::types::TypeData::Object(
                    match &built.data {
                        crate::checker::types::TypeData::Object(o) => ObjectTypeData {
                            structured: StructuredTypeData {
                                members: o.structured.members.clone(),
                                properties: o.structured.properties.clone(),
                                signatures: o.structured.signatures.clone(),
                                call_signature_count: o.structured.call_signature_count,
                                index_infos: o.structured.index_infos.clone(),
                                ..Default::default()
                            },
                            target: o.target.clone(),
                            mapper: o.mapper.clone(),
                            type_arguments: o.type_arguments.clone(),
                        },
                        _ => unreachable!("class build yields an object type"),
                    },
                ),
            ));
            {
                let t_mut = Arc::as_ptr(&stripped) as *mut Type;
                unsafe {
                    (*t_mut).object_flags =
                        built.object_flags | crate::checker::types::ObjectFlags::Instantiated;
                }
            }

            self.typequery_instantiation_cache
                .insert(key, (arg_types, Arc::clone(&stripped)));
            return stripped;
        }

        let value_type = self
            .value_symbol_links
            .get(&symbol)
            .and_then(|l| l.resolved_type.clone());
        if let Some(t) = value_type {
            if d.type_arguments.is_some() {
                let arg_types: Vec<Arc<Type>> = d
                    .type_arguments
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect();
                return self.instantiate_value_type_for_type_query(&t, &arg_types);
            }
            return t;
        }
        self.error_type()
    }

    fn instantiate_value_type_for_type_query(
        &mut self,
        base: &Arc<Type>,
        arg_types: &[Arc<Type>],
    ) -> Arc<Type> {
        if arg_types.is_empty() {
            return Arc::clone(base);
        }
        match &base.data {
            TypeData::Intersection(i) => {
                let parts: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_intersection_type(parts)
            }
            TypeData::Union(u) => {
                let parts: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_union_type(parts)
            }
            TypeData::Object(o) => {
                let call_count = o.structured.call_signature_count;
                let sigs = &o.structured.signatures;
                let mut changed = false;
                let mut new_sigs: Vec<Arc<crate::checker::types::Signature>> =
                    Vec::with_capacity(sigs.len());
                for (idx, sig) in sigs.iter().enumerate() {
                    let is_construct = idx >= call_count;

                    let mut params: Vec<Arc<Type>> = sig.type_parameters.clone();
                    if params.is_empty() && is_construct {
                        if let Some(rt) = self.get_return_type_of_signature(sig)
                            && let Some(class_sym) = &rt.symbol
                        {
                            let class_tps = self.declared_type_parameter_types(class_sym);
                            if !class_tps.is_empty() {
                                params = class_tps;
                            }
                        }
                    }
                    if params.is_empty() || arg_types.len() > params.len() {
                        new_sigs.push(Arc::clone(sig));
                        continue;
                    }
                    let inst = self.get_signature_instantiation(sig, arg_types);

                    let rt0 = self.get_return_type_of_signature(&inst);
                    let inst = match rt0 {
                        Some(rt0) => {
                            let deep = self
                                .substitute_object_properties_deep(&rt0, &params, arg_types);
                            let mut rebuilt = crate::checker::types::Signature::new();
                            rebuilt.flags = inst.flags;
                            rebuilt.min_argument_count = inst.min_argument_count;
                            rebuilt.resolved_min_argument_count =
                                inst.resolved_min_argument_count;
                            rebuilt.declaration = inst.declaration.clone();
                            rebuilt.parameters = inst.parameters.clone();
                            rebuilt.this_parameter = inst.this_parameter.clone();
                            rebuilt.resolved_type_predicate =
                                inst.resolved_type_predicate.clone();
                            rebuilt.target = inst.target.clone();
                            rebuilt.mapper = inst.mapper.clone();
                            rebuilt.instantiated_parameter_types =
                                inst.instantiated_parameter_types.clone();
                            if let Some(it) = inst.isolated_signature_type.get() {
                                let _ = rebuilt.isolated_signature_type.set(it.clone());
                            }
                            let _ = rebuilt.resolved_return_type.set(deep);
                            Arc::new(rebuilt)
                        }
                        None => inst,
                    };
                    changed = true;
                    new_sigs.push(inst);
                }
                if !changed {
                    return Arc::clone(base);
                }
                let shell = Arc::new(Type::new(
                    base.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData {
                            members: o.structured.members.clone(),
                            properties: o.structured.properties.clone(),
                            signatures: new_sigs,
                            call_signature_count: call_count,
                            index_infos: o.structured.index_infos.clone(),
                            ..Default::default()
                        },
                        target: o.target.clone(),
                        mapper: o.mapper.clone(),
                        type_arguments: o.type_arguments.clone(),
                    }),
                ));
                {
                    let t_mut = Arc::as_ptr(&shell) as *mut Type;
                    unsafe {
                        (*t_mut).object_flags =
                            base.object_flags | crate::checker::types::ObjectFlags::Instantiated;
                        (*t_mut).symbol = base.symbol.clone();
                    }
                }
                shell
            }
            _ => Arc::clone(base),
        }
    }

    fn get_type_from_array_or_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ArrayTypeNode(d) => {
                let elem_type = self.get_type_from_type_node(&d.element_type);
                self.create_array_type(elem_type)
            }
            NodeData::TupleTypeNode(d) => {
                let mut element_types = Vec::new();

                let mut variadic_types: Vec<Arc<Type>> = Vec::new();
                let mut has_variadic_union = false;
                for elem in d.elements.iter() {
                    if let NodeData::RestTypeNode(rd) = &elem.data {
                        let inner = Arc::clone(&rd.type_node);
                        let inner_t = self.get_type_from_type_node(&inner);
                        has_variadic_union |= inner_t.flags.contains(TypeFlags::Never)
                            || matches!(&inner_t.data, TypeData::Union(_));
                        variadic_types.push(inner_t);
                    }
                    element_types.push(self.get_type_from_type_node(elem));
                }
                if has_variadic_union
                    && !self.check_cross_product_union(node, &variadic_types)
                {
                    return self.error_type();
                }
                self.create_tuple_type(element_types)
            }
            _ => self.error_type(),
        }
    }

    fn check_type_reference_arguments(
        &mut self,
        _node: &Arc<Node>,
        type_name: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {

        let params: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .find_map(|d| {
                let tps = match &d.data {
                    NodeData::InterfaceDeclaration(i) => i.type_parameters.as_ref(),
                    NodeData::ClassDeclaration(c) => c.type_parameters.as_ref(),
                    NodeData::TypeAliasDeclaration(t) => t.type_parameters.as_ref(),
                    _ => None,
                }?;
                Some(tps.iter().cloned().collect())
            })
            .unwrap_or_default();
        if params.is_empty() {
            return true;
        }
        let provided: Vec<Arc<Node>> = type_name
            .parent
            .as_ref()
            .and_then(|p| match &p.data {
                NodeData::TypeReferenceNode(tr) => tr.type_arguments.clone(),

                NodeData::ExpressionWithTypeArguments(e) => e.type_arguments.clone(),
                _ => None,
            })
            .map(|list| list.iter().cloned().collect())
            .unwrap_or_default();

        let required = params
            .iter()
            .rposition(|p| {
                !matches!(&p.data, NodeData::TypeParameterDeclaration(d) if d.default_type.is_some())
            })
            .map_or(0, |i| i + 1);
        let file = self
            .get_source_file_of_node(type_name)
            .or_else(|| self.current_file.clone());
        if provided.len() < required || provided.len() > params.len() {
            let display = format!(
                "{}<{}>",
                symbol.name,
                params
                    .iter()
                    .filter_map(|p| p.name().map(|n| n.text().to_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2314 && d.loc == type_name.loc);
            if !already {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    crate::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_1_TYPE_ARGUMENT_S,
                    vec![display, params.len().to_string()],
                ));
            }

            return false;
        }

        for (i, arg_node) in provided.iter().enumerate() {
            let Some(param) = params.get(i) else { continue };
            let NodeData::TypeParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let Some(constraint_node) = &pd.constraint else {
                continue;
            };

            let param_names: Vec<String> = params
                .iter()
                .filter_map(|p| p.name().map(|n| n.text().to_string()))
                .collect();
            if type_node_references_names(constraint_node, &param_names) {
                continue;
            }
            let arg_type = self.get_type_from_type_node(arg_node);

            if arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                || arg_type.is_type_parameter()
            {
                continue;
            }
            let constraint_type = self.get_type_from_type_node(constraint_node);
            if constraint_type.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
                continue;
            }
            if self.is_type_assignable_to(&arg_type, &constraint_type) {
                continue;
            }

            let primitive_like = |t: &Arc<Type>| {
                t.flags.intersects(
                    TypeFlags::String
                        | TypeFlags::Number
                        | TypeFlags::Boolean
                        | TypeFlags::BigInt
                        | TypeFlags::ESSymbol
                        | TypeFlags::Enum
                        | TypeFlags::StringLiteral
                        | TypeFlags::NumberLiteral
                        | TypeFlags::BooleanLiteral
                        | TypeFlags::EnumLiteral
                        | TypeFlags::Null
                        | TypeFlags::Undefined,
                )
            };
            let object_like = |t: &Arc<Type>| {
                t.flags.contains(TypeFlags::Object)
                    && t.as_structured().is_some()
                    && !t.object_flags.contains(ObjectFlags::Tuple)
                    && !t.object_flags.contains(ObjectFlags::Reference)
                    && t.as_structured()
                        .is_some_and(|s| s.call_signature_count == 0)
            };
            let clear_cut = (primitive_like(&arg_type) && (primitive_like(&constraint_type) || object_like(&constraint_type)))
                || (object_like(&arg_type) && object_like(&constraint_type));
            if !clear_cut {
                continue;
            }

            {
                let ap = Arc::as_ptr(&arg_type) as *const Type as usize;
                let cp = Arc::as_ptr(&constraint_type) as *const Type as usize;
                if self.degraded_type_ptrs.contains(&ap)
                    || self.degraded_type_ptrs.contains(&cp)
                {
                    continue;
                }
            }
            let arg_str = self.type_to_string(&arg_type);
            let constraint_str = self.type_to_string(&constraint_type);
            let mut diag = crate::ast::Diagnostic::new(
                file.clone(),
                arg_node.loc,
                crate::diagnostics::messages_generated::TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1,
                vec![arg_str.clone(), constraint_str.clone()],
            );

            let missing = self.get_missing_required_properties(&arg_type, &constraint_type);
            if missing.len() == 1 {
                diag.message_chain.push(crate::ast::Diagnostic::new(
                    None,
                    arg_node.loc,
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), arg_str.clone(), constraint_str.clone()],
                ));
            }
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2344 && d.loc == arg_node.loc);
            if !already {
                self.diagnostics.add(diag);
            }
        }
        true
    }

    fn get_type_from_optional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("OptionalType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.add_optionality(&t)
    }

    fn get_type_from_union_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::UnionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }
        let result = self.get_union_type(member_types);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_intersection_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::IntersectionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }

        let has_union = member_types
            .iter()
            .any(|t| matches!(&t.data, TypeData::Union(_)));
        if has_union {
            let all_undefined = member_types.iter().all(|t| {
                let TypeData::Union(u) = &t.data else {
                    return false;
                };
                u.union_or_intersection
                    .types
                    .first()
                    .is_some_and(|f| f.flags.contains(TypeFlags::Undefined))
            });
            let all_null = member_types.iter().all(|t| {
                let TypeData::Union(u) = &t.data else {
                    return false;
                };
                u.union_or_intersection
                    .types
                    .iter()
                    .take(2)
                    .any(|f| f.flags.contains(TypeFlags::Null))
            });
            if !all_undefined
                && !all_null
                && !self.check_cross_product_union(node, &member_types)
            {
                let e = self.error_type();
                self.cache_type(node, e.clone());
                return e;
            }
        }
        let result = self.get_intersection_type(member_types);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_named_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("NamedTupleMember has type").clone();
        self.get_type_from_type_node(&inner)
    }

    fn get_type_from_rest_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("RestType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.create_array_type(t)
    }

    fn get_type_from_type_literal_or_function_or_constructor_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        self.cache_type(node, self.error_type());
        let result = match &node.data {
            NodeData::TypeLiteralNode(data) => {

                self.build_interface_type_from_members(&data.members)
            }
            NodeData::FunctionTypeNode(_) => self.get_type_from_function_type_node(node),
            NodeData::ConstructorTypeNode(_) => self.get_type_from_constructor_type_node(node),
            _ => self.error_type(),
        };
        self.cache_type(node, result.clone());
        result
    }

    #[allow(dead_code)]
    fn get_type_from_type_literal_members(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(data) => {

                    let name = data.name.text().to_string();
                    if name.is_empty() {
                        continue;
                    }
                    let prop_type = self.get_type_from_type_node(&data.type_node);
                    let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::IndexSignatureDeclaration(data) => {

                    let mut key_type = None;
                    let value_type;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd
                                .type_node
                                .as_ref()
                                .map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    let is_readonly = member
                        .modifiers()
                        .as_ref()
                        .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly,
                        declaration: Some(Arc::clone(member)),
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
                _ => {}
            }
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    fn get_type_from_function_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::FunctionTypeNode(data) => {

                self.push_scope(node);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                     false,
                     None,
                     Some(Arc::clone(node)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], false)
            }
            _ => self.error_type(),
        }
    }

    fn get_type_from_constructor_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ConstructorTypeNode(data) => {

                self.push_scope(node);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                     true,
                     None,
                     Some(Arc::clone(node)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], true)
            }
            _ => self.error_type(),
        }
    }

    fn iife_with_too_few_arguments(
        declaration: &Option<Arc<Node>>,
        parameter_count: usize,
    ) -> bool {
        let Some(decl) = declaration else {
            return false;
        };
        if !matches!(
            decl.kind,
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
        ) {
            return false;
        }
        let mut prev: Arc<Node> = Arc::clone(decl);
        let mut parent: Option<Arc<Node>> = decl.parent.clone();
        while matches!(
            parent.as_ref().map(|p| p.kind),
            Some(SyntaxKind::ParenthesizedExpression)
        ) {
            prev = parent.clone().expect("checked Some above");
            parent = prev.parent.clone();
        }
        let Some(parent) = parent else {
            return false;
        };
        if parent.kind != SyntaxKind::CallExpression {
            return false;
        }
        let crate::ast::NodeData::CallExpression(call) = &parent.data else {
            return false;
        };

        Arc::ptr_eq(&call.expression, &prev) && parameter_count > call.arguments.nodes.len()
    }

    pub fn build_signature_from_function_like_type_node(
        &mut self,
        parameters: &Arc<NodeList>,
        return_type: Arc<Type>,
        is_construct: bool,
        contextual_signature: Option<&Arc<Signature>>,
        declaration: Option<Arc<Node>>,
    ) -> Arc<Signature> {

        let type_parameters = self.type_parameters_of_declaration(&declaration);
        let mut param_symbols: Vec<Arc<Symbol>> = Vec::with_capacity(parameters.len());
        let mut flags = SignatureFlags::None;
        if is_construct {
            flags |= SignatureFlags::Construct;
        }

        let mut min_argument_count: i32 = 0;
        let mut reached_optional_or_rest = false;

        let mut this_parameter: Option<Arc<Symbol>> = None;
        for (i, param) in parameters.iter().enumerate() {
            let NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let is_rest = pd.dot_dot_dot_token.is_some();
            let is_optional = pd.question_token.is_some();
            let is_this_param = i == 0
                && !is_rest
                && matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this");

            let param_type = match pd.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => {
                    let mut t = None;
                    if let Some(ctx_sig) = contextual_signature {
                        if i < ctx_sig.parameters.len() {

                            t = self
                                .signature_instantiated_param_type(ctx_sig, i)
                                .or_else(|| {
                                    Some(self.get_type_of_symbol(&ctx_sig.parameters[i]))
                                });
                        }
                    }
                    t.unwrap_or_else(|| self.get_any_type())
                }
            };

            let param_type = if pd.question_token.is_some() && pd.initializer.is_none() {
                self.add_optional_undefined(param_type)
            } else {
                param_type
            };

            let name = pd.name.text().to_string();
            let name = if name.is_empty() {
                format!("__arg{}", i)
            } else {
                name
            };

            let sym = match self.program.symbol_map().symbol_of(param) {
                Some(s) => Arc::clone(s),
                None => Arc::new(Symbol::new(SymbolFlags::Property, name)),
            };
            self.value_symbol_links.insert(
                &sym,
                ValueSymbolLinks {
                    resolved_type: Some(param_type),
                    ..Default::default()
                },
            );
            param_symbols.push(sym);
            if is_this_param && this_parameter.is_none() {

                this_parameter = param_symbols.pop();
                continue;
            }
            if is_rest {
                flags |= SignatureFlags::HasRestParameter;
                reached_optional_or_rest = true;
            } else if is_optional
                || pd.initializer.is_some()
                || (pd.type_node.is_none()
                    && Self::iife_with_too_few_arguments(&declaration, parameters.len()))
            {

                reached_optional_or_rest = true;
            }
            if !reached_optional_or_rest {
                min_argument_count += 1;
            }
        }
        let sig = Arc::new(Signature {
            id: 0,
            flags,
            min_argument_count,
            resolved_min_argument_count: -1,
            declaration,
            type_parameters,
            parameters: param_symbols,
            this_parameter,
            resolved_return_type: std::sync::OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: std::sync::OnceLock::new(),
            instantiated_parameter_types: None,
        });

        let _ = sig.resolved_return_type.set(return_type);
        sig
    }

    fn type_parameters_of_declaration(
        &mut self,
        declaration: &Option<Arc<Node>>,
    ) -> Vec<Arc<Type>> {
        let Some(decl) = declaration else {
            return Vec::new();
        };
        let tp_list = match &decl.data {
            NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::FunctionExpression(d) => d.type_parameters.as_ref(),
            NodeData::ArrowFunction(d) => d.type_parameters.as_ref(),
            NodeData::MethodDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::MethodSignatureDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::SetAccessorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::FunctionTypeNode(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorTypeNode(d) => d.type_parameters.as_ref(),

            NodeData::CallSignatureDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructSignatureDeclaration(d) => d.type_parameters.as_ref(),
            _ => None,
        };
        let Some(list) = tp_list else {
            return Vec::new();
        };

        let symbols: Vec<Arc<Symbol>> = list
            .iter()
            .filter_map(|tp| self.program.symbol_map().symbol_of(tp).map(Arc::clone))
            .collect();
        symbols
            .iter()
            .map(|s| self.get_type_parameter_from_symbol(s))
            .collect()
    }

    pub fn create_function_or_constructor_type(
        &self,
        sigs: Vec<Arc<Signature>>,
        is_construct: bool,
    ) -> Arc<Type> {
        let call_signature_count = if is_construct { 0 } else { sigs.len() };
        let mut structured = StructuredTypeData::default();
        structured.signatures = sigs;
        structured.call_signature_count = call_signature_count;
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured,
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        })
    }

    fn get_type_from_type_operator_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = match &node.data {
            NodeData::TypeOperatorNode(data) => match data.operator {
                SyntaxKind::KeyOfKeyword => {
                    let arg_type = self.get_type_from_type_node(&data.type_node);
                    self.get_index_type(&arg_type)
                }
                SyntaxKind::UniqueKeyword => {
                    if data.type_node.kind == SyntaxKind::SymbolKeyword {
                        self.es_symbol_type()
                    } else {
                        self.error_type()
                    }
                }
                SyntaxKind::ReadonlyKeyword => {
                    let inner = self.get_type_from_type_node(&data.type_node);

                    if let TypeData::Tuple(tuple) = &inner.data {
                        if !tuple.readonly {
                            return Arc::new(Type {
                                flags: inner.flags,
                                object_flags: inner.object_flags,
                                id: 0,
                                symbol: None,
                                alias: None,
                                data: TypeData::Tuple(TupleTypeData {
                                    interface_data: InterfaceTypeData::default(),
                                    element_infos: tuple.element_infos.clone(),
                                    min_length: tuple.min_length,
                                    fixed_length: tuple.fixed_length,
                                    combined_flags: tuple.combined_flags,
                                    readonly: true,
                                }),
                            });
                        }
                    }
                    inner
                }
                _ => self.error_type(),
            },
            _ => self.error_type(),
        };
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_indexed_access_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = {
            let (object_type_node, index_type_node) = match &node.data {
                NodeData::IndexedAccessTypeNode(data) => {
                    (Arc::clone(&data.object_type), Arc::clone(&data.index_type))
                }
                _ => return self.error_type(),
            };
            let object_type = self.get_type_from_type_node(&object_type_node);
            let index_type = self.get_type_from_type_node(&index_type_node);

            if self.should_defer_indexed_access_type(&object_type, &index_type) {
                Arc::new(Type::new(
                    TypeFlags::IndexedAccess,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: Some(Arc::clone(&object_type)),
                        index_type: Some(Arc::clone(&index_type)),
                        access_flags: AccessFlags::None,
                    }),
                ))
            } else {

                if !self.index_type_is_kind_usable(&index_type)
                    && self
                        .indexed_access_2538_reported
                        .insert(Arc::as_ptr(&index_type_node) as *const crate::ast::Node)
                {

                    let ip = Arc::as_ptr(&index_type) as *const Type as usize;
                    let op = Arc::as_ptr(&object_type) as *const Type as usize;
                    let degraded = self.degraded_type_ptrs.contains(&ip)
                        || self.degraded_type_ptrs.contains(&op);
                    if !degraded {
                        let type_str = if index_type_node.kind == SyntaxKind::BigIntLiteral {
                            "bigint".to_string()
                        } else {
                            self.type_to_string(&index_type)
                        };
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            index_type_node.loc,
                            crate::diagnostics::messages_generated::
                                TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                            vec![type_str],
                        ));
                    }
                }

                self.get_indexed_access_type(&object_type, &index_type)
            }
        };
        self.cache_type(node, result.clone());
        result
    }

    fn should_defer_indexed_access_type(
        &self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> bool {
        if self.type_flags_is_generic_index_type(index_type) {
            return true;
        }
        if self.type_flags_is_generic_object_type(object_type) {
            if let TypeData::Tuple(tup) = &object_type.data {
                if index_type_less_than_fixed(index_type, tup.fixed_length) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    fn index_type_is_kind_usable(&mut self, t: &Arc<Type>) -> bool {
        let primitive_index_kinds = TypeFlags::from_bits_truncate(
            TypeFlags::Any.bits()
                | TypeFlags::Unknown.bits()
                | TypeFlags::Never.bits()
                | TypeFlags::String.bits()
                | TypeFlags::StringLiteral.bits()
                | TypeFlags::StringMapping.bits()
                | TypeFlags::TemplateLiteral.bits()
                | TypeFlags::Number.bits()
                | TypeFlags::NumberLiteral.bits()
                | TypeFlags::ESSymbol.bits()
                | TypeFlags::UniqueESSymbol.bits()
                | TypeFlags::Enum.bits()
                | TypeFlags::EnumLiteral.bits(),
        );
        let constituents: Vec<Arc<Type>> = if t.flags.contains(TypeFlags::Union) {
            t.types().map(|ts| ts.to_vec()).unwrap_or_default()
        } else {
            vec![Arc::clone(t)]
        };
        if constituents.is_empty() {
            return true;
        }
        for c in &constituents {
            if c.flags.intersects(primitive_index_kinds) {
                continue;
            }
            let ok = self.is_type_assignable_to(c, &self.string_type())
                || self.is_type_assignable_to(c, &self.number_type())
                || self.is_type_assignable_to(c, &self.es_symbol_type());
            if !ok {
                return false;
            }
        }
        true
    }

    fn get_type_from_template_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_template_literal_type(node);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_mapped_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_mapped_type(node);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_conditional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.build_conditional_type(node);
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_infer_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        {
            let mut in_extends_clause = false;
            let mut cur = Some(Arc::clone(node));
            while let Some(n) = cur {
                if let Some(p) = n.parent.clone()
                    && let NodeData::ConditionalTypeNode(cd) = &p.data
                    && Arc::ptr_eq(&cd.extends_type, &n)
                {
                    in_extends_clause = true;
                    break;
                }
                cur = n.parent.clone();
            }
            if !in_extends_clause {
                let already = self
                    .diagnostics
                    .get_all()
                    .iter()
                    .any(|d| d.code == 1338 && d.loc.pos() == node.loc.pos());
                if !already {
                    self.grammar_error_on_node(
                        node,
                        &crate::diagnostics::messages_generated::
                            X_INFER_DECLARATIONS_ARE_ONLY_PERMITTED_IN_THE_EXTENDS_CLAUSE_OF_A_CONDITIONAL_TYPE,
                    );
                }
            }
        }

        let result = {
            let tp_node = match &node.data {
                NodeData::InferTypeNode(data) => &data.type_parameter,
                _ => return self.error_type(),
            };
            let symbol = self.program.symbol_map().symbol_of(tp_node).map(Arc::clone);
            match symbol {
                Some(sym) => self.get_type_parameter_from_symbol(&sym),
                None => self.error_type(),
            }
        };
        self.cache_type(node, result.clone());
        result
    }

    fn build_conditional_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (check_type_node, extends_type_node) = match &node.data {
            NodeData::ConditionalTypeNode(data) => {
                (Arc::clone(&data.check_type), Arc::clone(&data.extends_type))
            }
            _ => return self.error_type(),
        };

        let check_type = self.get_type_from_type_node(&check_type_node);
        let extends_type = self.get_type_from_type_node(&extends_type_node);

        let infer_type_parameters = self.collect_infer_type_parameters(node);

        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let saved_name_frames = std::mem::take(&mut self.type_argument_name_frames);
        let unmapped_check_type = self.get_type_from_type_node(&check_type_node);
        self.type_argument_stack = saved_stack;
        self.type_argument_name_frames = saved_name_frames;
        let is_distributive = unmapped_check_type.flags.contains(TypeFlags::TypeParameter);
        let check_type_parameter_symbol = if is_distributive {
            unmapped_check_type.symbol.clone()
        } else {
            None
        };

        let root = Box::new(ConditionalRoot {
            node: Some(Arc::clone(node)),
            check_type: Some(Arc::clone(&check_type)),
            extends_type: Some(Arc::clone(&extends_type)),
            is_distributive,
            check_type_parameter_symbol,
            infer_type_parameters: infer_type_parameters.clone(),
            outer_type_parameters: Vec::new(),
            alias: None,
            creation_scopes: self.scope_stack.clone(),
        });

        let cond_type = Arc::new(Type::new(
            TypeFlags::Conditional,
            TypeData::Conditional(ConditionalTypeData {
                constrained: ConstrainedTypeData::default(),
                root: Some(root),
                check_type: Some(Arc::clone(&check_type)),
                extends_type: Some(Arc::clone(&extends_type)),
                resolved_true_type: OnceLock::new(),
                resolved_false_type: OnceLock::new(),
                resolved_inferred_true_type: OnceLock::new(),
                resolved_default_constraint: OnceLock::new(),
                resolved_constraint_of_distributive: OnceLock::new(),
                mapper: None,
                combined_mapper: None,
                creation_type_argument_stack: self
                    .type_argument_stack
                    .iter()
                    .map(|frame| {
                        frame
                            .iter()
                            .map(|(k, v)| (*k as usize, Arc::clone(v)))
                            .collect::<HashMap<_, _>>()
                    })
                    .collect(),
            }),
        ));

        if let Some(resolved) = self.resolve_conditional_type(&cond_type) {
            resolved
        } else {
            cond_type
        }
    }

    fn collect_infer_type_parameters(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> {

        let symbols: Vec<Arc<Symbol>> = self
            .program
            .symbol_map()
            .locals_of(node)
            .map(|locals| {
                locals
                    .iter()
                    .filter(|(_, sym)| sym.flags.contains(SymbolFlags::TypeParameter))
                    .map(|(_, sym)| Arc::clone(sym))
                    .collect()
            })
            .unwrap_or_default();
        symbols
            .into_iter()
            .map(|sym| self.get_type_parameter_from_symbol(&sym))
            .collect()
    }

    fn get_type_from_import_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        if let NodeData::ImportTypeNode(d) = &node.data
            && let Some(attrs) = &d.attributes
        {
            let attrs = Arc::clone(attrs);
            let _ = self.get_resolution_mode_override(&attrs, true);
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    fn cross_product_union_size(types: &[Arc<Type>]) -> u64 {
        let mut size: u64 = 1;
        for t in types {
            if let TypeData::Union(u) = &t.data {
                size = size.saturating_mul(u.union_or_intersection.types.len() as u64);
            } else if t.flags.contains(TypeFlags::Never) {
                return 0;
            }
        }
        size
    }

    fn check_cross_product_union(&mut self, node: &Arc<Node>, types: &[Arc<Type>]) -> bool {
        if Self::cross_product_union_size(types) < 100_000 {
            return true;
        }
        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == 2590 && d.loc.pos() == node.loc.pos());
        if !already {
            let file = self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    EXPRESSION_PRODUCES_A_UNION_TYPE_THAT_IS_TOO_COMPLEX_TO_REPRESENT,
                vec![],
            ));
        }
        false
    }

    pub fn get_optional_type(&mut self, t: Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            self.get_union_type(vec![t, self.undefined_type()])
        } else {
            t
        }
    }

    pub fn get_union_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
        }

        let types: Vec<Arc<Type>> = types
            .into_iter()
            .filter(|t| !t.flags.contains(TypeFlags::Never))
            .collect();
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }

        let mut flattened: Vec<Arc<Type>> = Vec::with_capacity(types.len());
        for t in types {
            if let TypeData::Union(u) = &t.data {
                for inner in &u.union_or_intersection.types {
                    if !flattened.iter().any(|s| Arc::ptr_eq(s, inner)) {
                        flattened.push(Arc::clone(inner));
                    }
                }
            } else if !flattened.iter().any(|s| Arc::ptr_eq(s, &t)) {
                flattened.push(t);
            }
        }
        if flattened.is_empty() {
            return self.never_type();
        }
        if flattened.len() == 1 {
            return flattened.into_iter().next().expect("exactly one");
        }

        {
            let rank = |t: &Arc<Type>| -> u32 {
                if t.flags
                    .intersects(TypeFlags::EnumLiteral | TypeFlags::Enum)
                {
                    return TypeFlags::Enum.bits();
                }
                let b = t.flags.bits();
                b & b.wrapping_neg()
            };
            flattened.sort_by_key(rank);
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: flattened,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub fn get_intersection_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.unknown_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Intersection,
            TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_apparent_type: std::sync::OnceLock::new(),
                unique_literal_filled_instantiation: std::sync::OnceLock::new(),
            }),
        ))
    }

    pub fn create_array_type(&mut self, element_type: Arc<Type>) -> Arc<Type> {

        let array_symbol = self.globals.get("Array").cloned();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Reference,
            id: 0,
            symbol: array_symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                target: None,
                mapper: None,
                type_arguments: vec![element_type],
            }),
        })
    }

    pub(crate) fn array_type_parameter_symbols(&mut self) -> Vec<Arc<Symbol>> {
        if let Some(cached) = &self.array_type_parameter_symbols {
            return cached.clone();
        }
        let collected = self
            .globals
            .get("Array")
            .and_then(|sym| {
                let decl = sym
                    .declarations
                    .iter()
                    .find(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))?;
                let NodeData::InterfaceDeclaration(d) = &decl.data else {
                    return None;
                };
                let sym_map = self.program.symbol_map();
                Some(
                    d.type_parameters
                        .as_ref()?
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap_or_default();
        self.array_type_parameter_symbols = Some(collected.clone());
        collected
    }

    pub(crate) fn instantiate_array_member_type(
        &mut self,
        obj_type: &Arc<Type>,
        member: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {

        let is_evolving = obj_type
            .object_flags
            .contains(ObjectFlags::EvolvingArray);
        if !self.is_array_type(obj_type) && !is_evolving {
            return None;
        }
        if let Some(structured) = obj_type.as_structured()
            && structured.members.get(&member.name).is_some()
        {

            return None;
        }
        let element = match &obj_type.data {
            TypeData::Object(o) => match o.type_arguments.first() {
                Some(e) => Arc::clone(e),
                None => return None,
            },
            TypeData::EvolvingArray(e) => e
                .element_type
                .clone()
                .unwrap_or_else(|| self.never_type()),
            _ => return None,
        };

        let declared = match self
            .globals
            .get("Array")
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            }) {
            Some(d) => Some(d),

            None => self.globals.get("Array").cloned().map(|sym| {
                self.resolve_interface_type(&sym, None)
            }),
        };
        let raw = declared
            .as_ref()
            .and_then(|d| d.as_structured())
            .and_then(|s| s.members.get(&member.name).cloned())
            .map(|synthetic| self.get_type_of_symbol(&synthetic))?;
        let key = (
            Arc::as_ptr(&element) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(member) as *const crate::ast::Symbol as usize,
        );
        if let Some(cached) = self.array_member_type_cache.get(&key) {
            return Some(Arc::clone(cached));
        }

        let mut free_tps: Vec<Arc<Type>> = Vec::new();
        for sig in self.get_signatures_of_type(&raw, SignatureKind::Call) {

            for param in &sig.parameters {
                let pt = self.get_type_of_symbol(param);
                self.collect_free_type_parameters_deep(&pt, &mut free_tps);
            }
            if let Some(rt) = self.get_return_type_of_signature(&sig) {
                self.collect_free_type_parameters_deep(&rt, &mut free_tps);
            }
        }

        let array_tps = self.array_type_parameter_symbols();
        let subst_tps: Vec<Arc<Type>> = free_tps
            .iter()
            .filter(|tp| {
                tp.symbol
                    .as_ref()
                    .is_some_and(|s| array_tps.iter().any(|a| Arc::ptr_eq(a, s)))
            })
            .cloned()
            .collect();
        if subst_tps.is_empty() {

            return Some(raw);
        }
        let substitutions: Vec<Arc<Type>> = std::iter::repeat(Arc::clone(&element))
            .take(subst_tps.len())
            .collect();
        let substituted =
            self.substitute_infer_type_parameters(&raw, &subst_tps, &substitutions);
        self.array_member_type_cache
            .insert(key, Arc::clone(&substituted));
        Some(substituted)
    }

    pub(crate) fn declared_array_member_symbol(&mut self, name: &str) -> Option<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            })?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
    }

    pub(crate) fn declared_array_member_symbols(&mut self) -> Vec<Arc<Symbol>> {
        let array_sym = self.globals.get("Array").cloned();
        let declared = array_sym
            .as_ref()
            .and_then(|sym| {
                self.type_alias_links
                    .get(sym)
                    .and_then(|l| l.declared_type.clone())
            })
            .or_else(|| {
                array_sym
                    .as_ref()
                    .map(|sym| self.resolve_interface_type(&sym, None))
            });
        declared
            .and_then(|t| t.as_structured().map(|s| s.properties.clone()))
            .unwrap_or_default()
    }

    pub(crate) fn global_interface_member_symbol(
        &mut self,
        interface_name: &str,
        member: &str,
    ) -> Option<Arc<Symbol>> {
        let sym = self.globals.get(interface_name).cloned()?;
        let declared = self
            .type_alias_links
            .get(&sym)
            .and_then(|l| l.declared_type.clone())
            .or_else(|| Some(self.resolve_interface_type(&sym, None)))?;
        declared
            .as_structured()
            .and_then(|s| s.members.get(member).cloned())
    }

    pub(crate) fn substituted_member_type_of(
        &mut self,
        owner: &Arc<Type>,
        prop: &Arc<Symbol>,
    ) -> Arc<Type> {
        let Some(obj) = owner.as_object() else {
            return self.get_type_of_symbol(prop);
        };
        if obj.type_arguments.is_empty() {
            return self.get_type_of_symbol(prop);
        }
        let Some(owner_sym) = owner.symbol.clone() else {
            return self.get_type_of_symbol(prop);
        };
        let key = (
            Arc::as_ptr(owner) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(prop) as *const crate::ast::Symbol as usize,
        );
        if let Some(cached) = self.instantiated_member_type_cache.get(&key) {
            return Arc::clone(&cached.1);
        }

        let result = if owner_sym.flags.contains(SymbolFlags::Interface) {
            let proper =
                self.resolve_interface_type_ex(&owner_sym, Some(obj.type_arguments.clone()));
            let prop_sym = proper
                .as_structured()
                .and_then(|s| s.members.get(&prop.name).cloned());
            match prop_sym {
                Some(ps) => self.get_type_of_symbol(&ps),
                None => self.get_type_of_symbol(prop),
            }
        } else {
            self.substitute_member_type_fallback(&owner_sym, prop, &obj.type_arguments)
        };

        if self.instantiated_member_type_cache.len() >= self.instantiated_member_type_cache_limit
        {
            self.instantiated_member_type_cache.clear();
        }

        self.instantiated_member_type_cache
            .insert(key, (Arc::clone(owner), Arc::clone(&result)));
        result
    }

    fn substitute_member_type_fallback(
        &mut self,
        owner_sym: &Arc<Symbol>,
        prop: &Arc<Symbol>,
        args: &[Arc<Type>],
    ) -> Arc<Type> {
        let decl_tps = self.declared_type_parameter_types(owner_sym);
        if decl_tps.len() == args.len() && !decl_tps.is_empty() {
            let raw = self.get_type_of_symbol(prop);
            let substitutions = args.to_vec();
            let r = self.substitute_infer_type_parameters(&raw, &decl_tps, &substitutions);
            r
        } else {
            self.get_type_of_symbol(prop)
        }
    }

    pub(crate) fn declared_type_parameter_types(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Type>> {
        let decl = symbol.declarations.iter().find(|d| {
            matches!(
                d.data,
                NodeData::InterfaceDeclaration(_) | NodeData::ClassDeclaration(_)
            )
        });
        let Some(decl) = decl else {
            return Vec::new();
        };
        let tps = match &decl.data {
            NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
            _ => None,
        };
        let Some(tps) = tps else {
            return Vec::new();
        };
        let tp_syms: Vec<Arc<Symbol>> = {
            let sym_map = self.program.symbol_map();
            tps.iter()
                .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                .collect()
        };

        self.push_ts2304_suppression();
        let types = tp_syms
            .iter()
            .map(|tp_sym| self.get_type_parameter_from_symbol(tp_sym))
            .collect();
        self.pop_ts2304_suppression();
        types
    }

    fn collect_free_type_parameters_deep(&mut self, t: &Arc<Type>, out: &mut Vec<Arc<Type>>) {
        match &t.data {
            TypeData::TypeParameter(_) => {
                if !out.iter().any(|p| Arc::ptr_eq(p, t)) {
                    out.push(Arc::clone(t));
                }
            }
            TypeData::Union(u) => {
                for ty in &u.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Intersection(i) => {
                for ty in &i.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Object(o) => {
                for ty in &o.type_arguments {
                    self.collect_free_type_parameters_deep(ty, out);
                }

                for sig in o.structured.signatures.clone() {
                    for param in sig.parameters.iter() {
                        let pt = self.get_type_of_symbol(param);
                        self.collect_free_type_parameters_deep(&pt, out);
                    }
                    if let Some(rt) = sig.resolved_return_type.get() {
                        let rt = Arc::clone(rt);
                        self.collect_free_type_parameters_deep(&rt, out);
                    }
                }
            }
            TypeData::Tuple(tu) => {
                for ei in &tu.element_infos {
                    if let Some(ty) = &ei.type_ {
                        self.collect_free_type_parameters_deep(ty, out);
                    }
                }
            }
            TypeData::IndexedAccess(ia) => {
                if let Some(obj) = &ia.object_type {
                    self.collect_free_type_parameters_deep(obj, out);
                }
                if let Some(idx) = &ia.index_type {
                    self.collect_free_type_parameters_deep(idx, out);
                }
            }
            _ => {}
        }
    }

    pub fn create_tuple_type(&mut self, element_types: Vec<Arc<Type>>) -> Arc<Type> {
        let element_infos: Vec<TupleElementInfo> = element_types
            .iter()
            .map(|t| TupleElementInfo {
                flags: ElementFlags::Required,
                labeled_declaration: None,
                type_: Some(Arc::clone(t)),
            })
            .collect();
        let fixed_length = element_types.len();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Tuple,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Tuple(TupleTypeData {
                interface_data: InterfaceTypeData::default(),
                element_infos,
                min_length: fixed_length,
                fixed_length,
                combined_flags: ElementFlags::Required,
                readonly: false,
            }),
        })
    }

    pub fn get_index_type(&mut self, t: &Arc<Type>) -> Arc<Type> {

        if t.flags.contains(TypeFlags::Never) {
            return self.never_type();
        }
        if t.flags.contains(TypeFlags::Any) {

            return self.string_type();
        }

        if t.flags.contains(TypeFlags::Union) {
            let types = match &t.data {
                TypeData::Union(u) => &u.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let mut common: Option<Vec<String>> = None;
            for constituent in types {
                let k = self.get_index_type(constituent);
                let names = self.string_literal_values(&k);
                common = Some(match common.take() {
                    None => names,
                    Some(acc) => acc.into_iter().filter(|n| names.contains(n)).collect(),
                });
            }
            let names = common.unwrap_or_default();
            if names.is_empty() {
                return self.never_type();
            }
            let literals: Vec<Arc<Type>> = names
                .into_iter()
                .map(|n| self.get_string_literal_type(&n))
                .collect();
            return self.get_union_type(literals);
        }

        if t.flags.contains(TypeFlags::Intersection) {
            let types = match &t.data {
                TypeData::Intersection(i) => &i.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let keys: Vec<Arc<Type>> = types.iter().map(|c| self.get_index_type(c)).collect();
            return self.get_union_type(keys);
        }

        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.get_index_type(&constraint);
            }

            return self.string_type();
        }

        if let TypeData::Mapped(m) = &t.data
            && let Some(constraint) = &m.constraint_type
        {
            let generic = constraint.flags.intersects(
                TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index,
            ) || matches!(&constraint.data, TypeData::IndexedAccess(_));
            let domain = if generic {
                match self.constraint_of_indexed_access(constraint) {
                    Some(reduced) => reduced,
                    None => Arc::clone(constraint),
                }
            } else {
                Arc::clone(constraint)
            };
            if domain.flags.contains(TypeFlags::String) {
                let keys = vec![domain, self.number_type()];
                return self.get_union_type(keys);
            }
            return domain;
        }

        if let Some(structured) = t.as_structured() {
            let mut keys: Vec<Arc<Type>> = structured
                .properties
                .iter()

                .filter(|p| !p.name.starts_with('#'))
                .map(|p| self.get_string_literal_type(&p.name))
                .collect();
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    keys.push(Arc::clone(key));

                    if key.flags.contains(TypeFlags::String) {
                        keys.push(self.number_type());
                    }
                }
            }
            if keys.is_empty() {
                return self.never_type();
            }
            return self.get_union_type(keys);
        }

        self.never_type()
    }

    fn type_node_references_name(node: &Arc<Node>, name: &str) -> bool {
        if node.kind == SyntaxKind::Identifier && node.text() == name {
            return true;
        }
        let mut found = false;
        crate::ast::node_data_generated::for_each_child(node, |c| {
            found = found || Self::type_node_references_name(c, name);
            found
        });
        found
    }

    pub fn get_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> Arc<Type> {

        if object_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }
        if object_type.flags.contains(TypeFlags::Unknown) {
            return self.unknown_type();
        }
        if index_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }

        if index_type.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &index_type.data {
                let prop_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|c| self.get_indexed_access_type(object_type, c))
                    .collect();
                if prop_types.is_empty() {
                    return self.any_type();
                }
                return self.get_union_type(prop_types);
            }
        }

        if object_type.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(object_type) {
                return self.get_indexed_access_type(&constraint, index_type);
            }
            return self.any_type();
        }

        if let TypeData::Mapped(m) = &object_type.data
            && let Some(constraint) = &m.constraint_type
        {
            let generic = constraint.flags.intersects(
                TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index,
            ) || matches!(&constraint.data, TypeData::IndexedAccess(_));
            let domain = if generic {
                match self.constraint_of_indexed_access(constraint) {
                    Some(reduced) => reduced,
                    None => Arc::clone(constraint),
                }
            } else {
                Arc::clone(constraint)
            };
            if self.is_type_assignable_to(index_type, &domain) {

                let substituted = m
                    .declaration
                    .as_ref()
                    .and_then(|decl| match &decl.data {
                        crate::ast::NodeData::MappedTypeNode(d) => d.type_node.as_ref().map(
                            |tn| {
                                (
                                    Arc::clone(&d.type_parameter),
                                    Arc::clone(tn),
                                    Arc::clone(decl),
                                )
                            },
                        ),
                        _ => None,
                    })
                    .and_then(|(tp_node, template_node, decl)| {
                        let tp_sym = self
                            .program
                            .symbol_map()
                            .symbol_of(&tp_node)
                            .cloned()?;

                        if !Self::type_node_references_name(
                            &template_node,
                            &tp_sym.name,
                        ) {
                            return None;
                        }
                        let mut mapping = std::collections::HashMap::new();
                        mapping.insert(
                            Arc::as_ptr(&tp_sym) as *const crate::ast::Symbol,
                            Arc::clone(index_type),
                        );
                        self.push_scope(&decl);
                        self.type_argument_stack.push(mapping);
                        let t = self.get_type_from_type_node(&template_node);
                        self.type_argument_stack.pop();
                        self.pop_scope();
                        Some(t)
                    });
                return substituted.unwrap_or_else(|| {
                    Arc::clone(m.template_type.as_ref().expect("template present"))
                });
            }
        }

        if index_type.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &index_type.data {
                if let LiteralValue::String(name) = &lit.value {
                    if let Some(structured) = object_type.as_structured() {
                        if let Some(sym) = structured.members.get(name) {
                            return self.get_type_of_symbol(sym);
                        }

                        if let Some(value_type) =
                            self.lookup_index_signature_value(structured, index_type)
                        {
                            return value_type;
                        }
                    }
                    return self.any_type();
                }
            }
        }

        if index_type.flags.contains(TypeFlags::Number)
            || index_type.flags.contains(TypeFlags::NumberLiteral)
        {
            if self.is_array_type(object_type) {
                return self.get_array_element_type(object_type);
            }

            if self.is_tuple_type(object_type) {
                if let Some(structured) = object_type.as_structured() {
                    let elem_types: Vec<Arc<Type>> = structured
                        .properties
                        .iter()
                        .map(|p| self.get_type_of_symbol(p))
                        .collect();
                    if !elem_types.is_empty() {
                        return self.get_union_type(elem_types);
                    }
                }
            }
        }

        if let Some(structured) = object_type.as_structured() {
            if let Some(value_type) = self.lookup_index_signature_value(structured, index_type) {
                return value_type;
            }
        }
        self.any_type()
    }

    pub fn lookup_index_signature_value(
        &mut self,
        structured: &StructuredTypeData,
        index_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        for info in &structured.index_infos {
            let key_matches = match info.key_type.as_ref() {
                Some(key) => {

                    if key.flags.contains(TypeFlags::String) {
                        index_type.flags.contains(TypeFlags::String)
                            || index_type.flags.contains(TypeFlags::StringLiteral)
                    } else if key.flags.contains(TypeFlags::Number) {
                        index_type.flags.contains(TypeFlags::Number)
                            || index_type.flags.contains(TypeFlags::NumberLiteral)
                    } else {
                        false
                    }
                }
                None => true,
            };
            if key_matches {
                return info.value_type.clone();
            }
        }
        None
    }

    fn build_template_literal_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (head, spans) = match &node.data {
            NodeData::TemplateLiteralTypeNode(data) => {
                (Arc::clone(&data.head), Arc::clone(&data.template_spans))
            }
            _ => return self.error_type(),
        };
        let head_text = template_token_text(&head);

        let mut span_types: Vec<Arc<Type>> = Vec::new();
        let mut span_texts: Vec<String> = Vec::new();
        for span_node in spans.iter() {
            let (type_node, literal_node) = match &span_node.data {
                NodeData::TemplateLiteralTypeSpan(data) => {
                    (Arc::clone(&data.type_node), Arc::clone(&data.literal))
                }
                _ => return self.error_type(),
            };
            span_types.push(self.get_type_from_type_node(&type_node));
            span_texts.push(template_token_text(&literal_node));
        }

        if span_types
            .iter()
            .any(|t| t.flags.contains(TypeFlags::Never) || matches!(&t.data, TypeData::Union(_)))
            && !self.check_cross_product_union(node, &span_types)
        {
            return self.error_type();
        }

        let all_literal = span_types.iter().all(|t| {
            t.flags
                .intersects(TYPE_FLAGS_LITERAL | TypeFlags::Null | TypeFlags::Undefined)
        });
        if all_literal {
            let mut sb = String::new();
            sb.push_str(&head_text);
            for (t, text) in span_types.iter().zip(span_texts.iter()) {
                sb.push_str(&self.template_string_for_type(t));
                sb.push_str(text);
            }
            return self.get_string_literal_type(&sb);
        }

        let mut texts = Vec::with_capacity(span_types.len() + 1);
        texts.push(head_text);
        for t in span_texts {
            texts.push(t);
        }
        Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(TemplateLiteralTypeData {
                constrained: ConstrainedTypeData::default(),
                texts,
                types: span_types,
            }),
        ))
    }

    fn template_string_for_type(&self, t: &Arc<Type>) -> String {
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return s.clone();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Number(n) = &lit.value {
                    return n.to_string();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Boolean(b) = &lit.value {
                    return if *b { "true".into() } else { "false".into() };
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::Null) {
            return "null".into();
        }
        if t.flags.contains(TypeFlags::Undefined) {
            return "undefined".into();
        }
        String::new()
    }

    fn build_mapped_type(&mut self, node: &Arc<Node>) -> Arc<Type> {

        self.push_scope(node);
        let result = self.build_mapped_type_inner(node);
        self.pop_scope();
        result
    }

    fn build_mapped_type_inner(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let data = match &node.data {
            NodeData::MappedTypeNode(data) => data,
            _ => return self.error_type(),
        };

        let constraint_node = match &data.type_parameter.data {
            NodeData::TypeParameterDeclaration(tp) => match &tp.constraint {
                Some(c) => Arc::clone(c),
                None => return self.error_type(),
            },
            _ => return self.error_type(),
        };
        let constraint_type = self.get_type_from_type_node(&constraint_node);

        if data.type_node.is_none()
            && self.no_implicit_any
            && self
                .current_file
                .as_ref()
                .is_some_and(|f| !f.file_name.starts_with("bundled://"))
        {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    MAPPED_OBJECT_TYPE_IMPLICITLY_HAS_AN_ANY_TEMPLATE_TYPE,
                Vec::new(),
            ));
        }

        let keys = self.string_literal_values(&constraint_type);

        let constraint_all_literals = self.union_is_all_string_literals(&constraint_type);
        if keys.is_empty() || !constraint_all_literals {

            let tp_type = self.get_type_from_type_node(&data.type_parameter);
            let template_type = match &data.type_node {
                Some(tn) => {

                    self.get_type_from_type_node(tn)
                }
                None => self.get_any_type(),
            };
            let name_type = data
                .name_type
                .as_ref()
                .map(|n| self.get_type_from_type_node(n));
            return Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: crate::checker::types::ObjectFlags::Mapped,
                id: 0,
                symbol: None,
                alias: None,
                data: TypeData::Mapped(MappedTypeData {
                    object: ObjectTypeData {
                        structured: StructuredTypeData::default(),
                        ..Default::default()
                    },
                    declaration: Some(Arc::clone(node)),
                    type_parameter: Some(tp_type),
                    constraint_type: Some(constraint_type),
                    name_type,
                    template_type: Some(template_type),
                    modifiers_type: None,
                    resolved_apparent_type: OnceLock::new(),
                    contains_error: false,
                }),
            });
        }

        let tp_symbol = self
            .program
            .symbol_map()
            .symbol_of(&data.type_parameter)
            .map(Arc::clone);
        let tp_key = tp_symbol
            .as_ref()
            .map(|s| Arc::as_ptr(s) as *const crate::ast::Symbol);

        let is_optional = data
            .question_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);

        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for key in &keys {
            let mut prop_type = match &data.type_node {
                Some(tn) => {

                    if let Some(k) = tp_key {
                        let mut mapping = HashMap::new();
                        mapping.insert(k, self.get_string_literal_type(key));
                        self.type_argument_stack.push(mapping);
                    }
                    let t = self.get_type_from_type_node(tn);
                    if tp_key.is_some() {
                        self.type_argument_stack.pop();
                    }
                    t
                }
                None => self.get_any_type(),
            };
            if is_optional {
                prop_type = self.get_optional_type(prop_type);
            }
            let mut flags = SymbolFlags::Property;
            if is_optional {
                flags |= SymbolFlags::Optional;
            }
            let symbol = Arc::new(Symbol::new(flags, key.clone()));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(key.clone(), Arc::clone(&symbol));
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos: Vec::new(),
                    signatures: Vec::new(),
                    call_signature_count: 0,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    fn string_literal_values(&self, t: &Arc<Type>) -> Vec<String> {
        if t.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return vec![s.clone()];
                }
            }
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .flat_map(|c| self.string_literal_values(c))
                    .collect();
            }
        }
        Vec::new()
    }

    fn union_is_all_string_literals(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::StringLiteral) {
            return true;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .all(|c| self.union_is_all_string_literals(c));
            }
        }
        false
    }

    pub fn add_optionality(&self, t: &Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            self.make_union_two(Arc::clone(t), self.undefined_type())
        } else {
            Arc::clone(t)
        }
    }

    fn make_union_two(&self, a: Arc<Type>, b: Arc<Type>) -> Arc<Type> {
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: vec![a, b],
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub fn get_nullable_type(&self, t: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let mut types = vec![Arc::clone(t)];
        if flags.contains(TypeFlags::Null) {
            types.push(self.null_type());
        }
        if flags.contains(TypeFlags::Undefined) {
            types.push(self.undefined_type());
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub fn collect_return_types_from_node(&mut self, node: &Arc<Node>, types: &mut Vec<Arc<Type>>) {
        use crate::ast::node_data_generated::for_each_child;
        match node.kind {
            SyntaxKind::ReturnStatement => {
                if let crate::ast::NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        types.push(self.get_type_of_node(expr));
                    }
                }
                return;
            }

            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => return,
            _ => {}
        }
        for_each_child(node, |child| {
            self.collect_return_types_from_node(child, types);
            false
        });
    }

    pub fn infer_function_return_type(
        &mut self,
        body: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        if let Some(type_node) = type_node {
            return self.get_type_from_type_node(type_node);
        }
        let Some(body) = body else {
            return self.void_type();
        };

        if body.kind != SyntaxKind::Block {
            let t = self.get_type_of_node(body);
            return self.get_widened_type(&t);
        }
        let mut types: Vec<Arc<Type>> = Vec::new();
        self.collect_return_types_from_node(body, &mut types);
        if types.is_empty() {
            return self.void_type();
        }
        let inferred = if types.len() == 1 {
            types.into_iter().next().expect("exactly one")
        } else {
            self.get_union_type(types)
        };

        self.get_widened_type(&inferred)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nullable_flags_correct() {
        assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Undefined));
        assert!(TYPE_FLAGS_NULLABLE.contains(TypeFlags::Null));
    }

    #[test]
    fn union_flags_set() {
        let t = Type::new(
            TypeFlags::Union,
            TypeData::Intrinsic(IntrinsicTypeData {
                intrinsic_name: "test".to_string(),
            }),
        );
        assert!(t.flags.contains(TypeFlags::Union));
    }
}

fn type_node_references_names(node: &Arc<Node>, names: &[String]) -> bool {
    let mut found = false;
    NodeWalker { names, found: &mut found }.walk(node);
    found
}

struct NodeWalker<'a> {
    names: &'a [String],
    found: &'a mut bool,
}

impl<'a> NodeWalker<'a> {
    fn walk(&mut self, node: &Arc<Node>) {
        if *self.found {
            return;
        }
        if node.kind == SyntaxKind::Identifier && names_contain(self.names, node.text()) {
            *self.found = true;
            return;
        }
        crate::ast::node_data_generated::for_each_child(node, |c| {
            self.walk(c);
            *self.found
        });
    }
}

fn names_contain(names: &[String], text: &str) -> bool {
    names.iter().any(|n| n == text)
}

fn type_name_inside_conditional_branch(node: &Arc<Node>) -> bool {
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if matches!(&a.data, NodeData::ConditionalTypeNode(_)) {

            if let NodeData::ConditionalTypeNode(c) = &a.data {
                if node_inside(node, &c.check_type) || node_inside(node, &c.extends_type) {
                    cur = a.parent.as_ref();
                    continue;
                }
            }
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}

fn node_inside(node: &Arc<Node>, root: &Arc<Node>) -> bool {
    if Arc::ptr_eq(node, root) {
        return true;
    }
    let mut cur = node.parent.as_ref();
    while let Some(a) = cur {
        if Arc::ptr_eq(a, root) {
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}

fn type_name_shadowed_by_type_parameter(type_name: &Arc<Node>) -> bool {
    let name = type_name.text();
    let mut cur = type_name.parent.as_ref();
    while let Some(a) = cur {
        let tps = match &a.data {
            NodeData::TypeAliasDeclaration(t) => t.type_parameters.as_ref(),
            NodeData::InterfaceDeclaration(i) => i.type_parameters.as_ref(),
            NodeData::ClassDeclaration(c) => c.type_parameters.as_ref(),
            NodeData::MethodDeclaration(m) => m.type_parameters.as_ref(),
            NodeData::FunctionDeclaration(f) => f.type_parameters.as_ref(),
            _ => None,
        };
        if let Some(list) = tps
            && list.iter().any(|p| {
                p.name().is_some_and(|n| n.text() == name)
            })
        {
            return true;
        }
        cur = a.parent.as_ref();
    }
    false
}
