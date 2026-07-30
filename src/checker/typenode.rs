//! Type node resolution: converting AST type nodes to `Type` objects.
//!
//! Ported from `getTypeFromTypeNode` and related functions in
//! `internal/checker/checker.go` (lines ~22713–22910).
//!
//! This is the primary entry point for converting a type annotation in the
//! AST (e.g. `string`, `number[]`, `Array<T>`, `A | B`, `keyof T`) into the
//! corresponding `Type` used by the checker.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, NodeList, Symbol, SymbolFlags, SymbolTable, SyntaxKind};

use super::checker::Checker;
use super::types::*;

impl Checker {
    /// Convert an AST type node into a `Type`.
    ///
    /// Mirrors `Checker.getTypeFromTypeNode` in Go. This is the public
    /// entry point (from `exports.go`).
    pub fn get_type_from_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.get_type_from_type_node_worker(node)
    }

    /// Worker for `get_type_from_type_node`.
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

    // ────────────────────────────────────────────────────────────────────────
    // Cached resolution helpers
    //
    // These use a check-then-compute-then-store pattern to avoid borrow
    // conflicts: we first check the cache with an immutable borrow, then
    // compute the type (which may need &mut self), then store it back.
    // ────────────────────────────────────────────────────────────────────────

    /// Check if a type node has already been resolved.
    fn get_cached_type(&self, node: &Arc<Node>) -> Option<Arc<Type>> {
        self.type_node_links
            .get(node)
            .and_then(|l| l.resolved_type.clone())
    }

    /// Store a resolved type for a type node.
    fn cache_type(&mut self, node: &Arc<Node>, t: Arc<Type>) {
        self.type_node_links.get_or_default(node).resolved_type = Some(t);
    }

    // ────────────────────────────────────────────────────────────────────────
    // Individual type node resolvers
    // ────────────────────────────────────────────────────────────────────────

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

    /// Resolve a `TypeReference` (`Foo`, `Foo<T>`) or
    /// `ExpressionWithTypeArguments` to its `Type`.
    ///
    /// Currently resolves type aliases (`type Foo = ...`) by recursively
    /// resolving the alias's declared type. Interface/class/enum references
    /// fall back to `error_type` (= `any`) so no false positives are produced
    /// until their object-type construction is ported.
    fn resolve_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(data) => (&data.type_name, data.type_arguments.clone()),
            NodeData::ExpressionWithTypeArguments(data) => {
                (&data.expression, data.type_arguments.clone())
            }
            _ => return self.error_type(),
        };
        // Only Identifier names are handled for now; QualifiedName
        // (e.g. `A.B`) needs module/namespace resolution.
        if type_name.kind != SyntaxKind::Identifier {
            return self.error_type();
        }
        let symbol = match self.resolve_identifier(type_name) {
            Some(s) => s,
            None => return self.error_type(),
        };
        // Type parameter: build a TypeParameter type with the constraint
        // resolved from the declaration (`<T extends Constraint>`).
        if symbol.flags.contains(SymbolFlags::TypeParameter) {
            // Check the type-argument substitution stack first — if this
            // type parameter is being instantiated with a concrete type,
            // return the substitution instead of the TypeParameter type.
            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            for map in self.type_argument_stack.iter().rev() {
                if let Some(t) = map.get(&key) {
                    return Arc::clone(t);
                }
            }
            return self.get_type_parameter_from_symbol(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Interface) {
            // Interface: build an anonymous object type from the interface's
            // members (PropertySignature, MethodSignature, IndexSignature).
            return self.resolve_interface_type(&symbol, type_arguments);
        }
        if !symbol.flags.contains(SymbolFlags::TypeAlias) {
            // Class/enum/etc.: defer to error_type (any) for now.
            return self.error_type();
        }
        // Cycle guard: a recursive alias (`type A = B; type B = A`) would
        // otherwise infinite-loop here.
        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return self.error_type();
        }
        // For non-generic aliases (no type arguments on the reference), use
        // the cached declared type. For generic references (type arguments
        // present), we must re-resolve the alias body with the type
        // parameters substituted by the type arguments, bypassing the cache.
        let has_type_args = type_arguments.is_some();
        let resolved = if !has_type_args {
            // Reuse a previously-computed declared type if present.
            let cached = self
                .type_alias_links
                .get(&symbol)
                .and_then(|l| l.declared_type.clone());
            cached.unwrap_or_else(|| {
                let found = self.resolve_alias_body(&symbol);
                self.type_alias_links
                    .get_or_default(&symbol)
                    .declared_type = Some(Arc::clone(&found));
                found
            })
        } else {
            // Generic type alias instantiation: collect the alias's type
            // parameters, resolve the type arguments, push the mapping,
            // resolve the body, and pop.
            let (tp_symbols, type_node) = self.collect_alias_type_params_and_body(&symbol);
            let arg_types: Vec<Arc<Type>> = match &type_arguments {
                Some(args) => args.iter().map(|a| self.get_type_from_type_node(a)).collect(),
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
            let found = self.get_type_from_type_node(&type_node);
            self.type_argument_stack.pop();
            found
        };
        self.resolving_type_aliases.remove(&key);
        resolved
    }

    /// Resolve the declared type of a type alias symbol (the alias body).
    fn resolve_alias_body(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                return self.get_type_from_type_node(&data.type_node);
            }
        }
        self.error_type()
    }

    /// Collect a type alias's type-parameter symbols and its body type node.
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
        (tp_symbols, type_node.unwrap_or_else(|| Arc::clone(&symbol.declarations[0])))
    }

    /// Resolve an `interface` declaration to an anonymous object type.
    ///
    /// Builds the type from the interface's members (PropertySignature,
    /// MethodSignature, IndexSignature). For generic interfaces (`Box<T>`),
    /// the type arguments are pushed onto the substitution stack before
    /// resolving members, so type-parameter references inside member types
    /// resolve to the corresponding type arguments.
    ///
    /// Heritage clauses (`extends A`) are not yet merged — only the
    /// interface's own members are included.
    fn resolve_interface_type(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Type> {
        // For non-generic interfaces, reuse a cached declared type.
        let has_type_args = type_arguments.is_some();
        if !has_type_args {
            if let Some(cached) = self
                .type_alias_links
                .get(symbol)
                .and_then(|l| l.declared_type.clone())
            {
                return cached;
            }
        }
        // Cycle guard for recursive interface references.
        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return self.error_type();
        }
        // Find the InterfaceDeclaration node.
        let decl = symbol
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)));
        let result = match decl.map(|d| &d.data) {
            Some(NodeData::InterfaceDeclaration(data)) => {
                // Collect type-parameter symbols for substitution.
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
                // Push the type-argument substitution mapping for generic
                // interfaces.
                if has_type_args {
                    let arg_types: Vec<Arc<Type>> = type_arguments
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|a| self.get_type_from_type_node(a))
                        .collect();
                    let mut mapping = HashMap::new();
                    for (i, tp_sym) in tp_symbols.iter().enumerate() {
                        if let Some(arg) = arg_types.get(i) {
                            let k = Arc::as_ptr(tp_sym) as *const crate::ast::Symbol;
                            mapping.insert(k, Arc::clone(arg));
                        }
                    }
                    self.type_argument_stack.push(mapping);
                }
                // Push scope so type-parameter references in member types
                // resolve via the scope stack.
                self.push_scope(symbol.declarations.iter().next().expect("interface has a declaration"));
                let result = self.build_interface_type_from_members(&data.members);
                self.pop_scope();
                if has_type_args {
                    self.type_argument_stack.pop();
                }
                result
            }
            _ => self.error_type(),
        };
        self.resolving_type_aliases.remove(&key);
        if !has_type_args {
            self.type_alias_links
                .get_or_default(symbol)
                .declared_type = Some(result.clone());
        }
        result
    }

    /// Build an anonymous object type from an interface's member list.
    /// Handles PropertySignature, MethodSignature, and IndexSignature.
    fn build_interface_type_from_members(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();
        let mut signatures: Vec<Arc<Signature>> = Vec::new();
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
                NodeData::MethodSignatureDeclaration(data) => {
                    let name = data.name.text().to_string();
                    if name.is_empty() {
                        continue;
                    }
                    // Build a function type from the method signature.
                    let return_type = match data.type_node.as_ref() {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        &data.parameters,
                        return_type,
                        /* is_construct */ false,
                        /* contextual_signature */ None,
                        /* declaration */ Some(Arc::clone(member)),
                    );
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
                    let mut value_type = None;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd
                                .type_node
                                .as_ref()
                                .map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly: false,
                        declaration: Some(Arc::clone(member)),
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
                _ => {}
            }
        }
        let call_signature_count = signatures.len();
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

    /// Build a `TypeParameter` type from a `TypeParameter` symbol, resolving
    /// its constraint (if any) from the declaration.
    fn get_type_parameter_from_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        // Cached parameter type stored on the symbol's links.
        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(ref t) = links.declared_type {
                return Arc::clone(t);
            }
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
        self.type_alias_links
            .get_or_default(symbol)
            .declared_type = Some(Arc::clone(&tp));
        tp
    }

    fn get_type_from_type_query_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_array_or_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ArrayTypeNode(d) => {
                let elem_type = self.get_type_from_type_node(&d.element_type);
                self.create_array_type(elem_type)
            }
            NodeData::TupleTypeNode(d) => {
                let mut element_types = Vec::new();
                for elem in d.elements.iter() {
                    element_types.push(self.get_type_from_type_node(elem));
                }
                self.create_tuple_type(element_types)
            }
            _ => self.error_type(),
        }
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
        // Reserve a cache slot first to break cycles on recursive types
        // like `type Box = { value: number; next: Box | null }`. During
        // member resolution a back-reference to this node returns the
        // reserved `error_type` (≈ `any`) instead of infinite-looping.
        self.cache_type(node, self.error_type());
        let result = match &node.data {
            NodeData::TypeLiteralNode(data) => {
                self.get_type_from_type_literal_members(&data.members)
            }
            NodeData::FunctionTypeNode(_) => self.get_type_from_function_type_node(node),
            NodeData::ConstructorTypeNode(_) => self.get_type_from_constructor_type_node(node),
            _ => self.error_type(),
        };
        self.cache_type(node, result.clone());
        result
    }

    /// Build an anonymous object type from a `TypeLiteral`'s member list
    /// (e.g. `{ a: number; b: string }`).
    ///
    /// Handles `PropertySignature` and `IndexSignature` members. Other
    /// member kinds (MethodSignature, CallSignature, ConstructSignature)
    /// are skipped — their support requires signature construction which is
    /// part of P3.7.
    fn get_type_from_type_literal_members(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(data) => {
                    // `node.text()` returns the name for identifier/string/
                    // numeric literal names, and "" for computed property names.
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
                    // `[key: string]: number` — extract key and value types.
                    let mut key_type = None;
                    let mut value_type = None;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd.type_node.as_ref().map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly: false,
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

    /// Resolve a `FunctionType` node `(x: number) => string` to an anonymous
    /// object type with a single call signature. Parameter types and the
    /// return type are resolved from the AST. Rest parameters
    /// (`...args: number[]`) and optional parameters (`x?: number`) are
    /// honored via `SignatureFlags::HasRestParameter` and
    /// `min_argument_count` respectively. Generic type parameters
    /// (`<T>(x: T) => T`) are skipped for now (the signature is non-generic).
    /// Note: cache handling is done by the caller
    /// (`get_type_from_type_literal_or_function_or_constructor_type_node`).
    fn get_type_from_function_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::FunctionTypeNode(data) => {
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    /* is_construct */ false,
                    /* contextual_signature */ None,
                    /* declaration */ Some(Arc::clone(node)),
                );
                self.create_function_or_constructor_type(vec![sig], false)
            }
            _ => self.error_type(),
        }
    }

    /// Resolve a `ConstructorType` node `new (x: number) => Foo` to an
    /// anonymous object type with a single construct signature. Cache
    /// handling is done by the caller.
    fn get_type_from_constructor_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ConstructorTypeNode(data) => {
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    /* is_construct */ true,
                    /* contextual_signature */ None,
                    /* declaration */ Some(Arc::clone(node)),
                );
                self.create_function_or_constructor_type(vec![sig], true)
            }
            _ => self.error_type(),
        }
    }

    /// Build a `Signature` from a function-like type node's parameter list
    /// and a pre-resolved return type. Each parameter is turned into a
    /// `Symbol` whose resolved type is stored in `value_symbol_links` (so
    /// the relater's `get_type_of_symbol` returns it during signature
    /// comparison).
    ///
    /// The return type is passed in already-resolved (rather than as a type
    /// node) so that this helper can be shared by both type-annotation
    /// resolution (where the return type comes from a `TypeNode`) and
    /// function-expression type inference (where the return type is inferred
    /// from the body via `infer_function_return_type`).
    ///
    /// `contextual_signature` carries the contextual function type's call
    /// signature (when the function expression is the initializer of a
    /// variable/parameter/property with a function-type annotation). When a
    /// parameter lacks an explicit type annotation, its type is taken from
    /// the corresponding position in the contextual signature — this is what
    /// makes `let f: (x: number) => number = (x) => x + 1;` type-check `x`
    /// as `number` inside the body. Parameters beyond the contextual
    /// signature's length (or with no contextual signature) fall back to
    /// `any`.
    pub fn build_signature_from_function_like_type_node(
        &mut self,
        parameters: &Arc<NodeList>,
        return_type: Arc<Type>,
        is_construct: bool,
        contextual_signature: Option<&Arc<Signature>>,
        declaration: Option<Arc<Node>>,
    ) -> Arc<Signature> {
        let mut param_symbols: Vec<Arc<Symbol>> = Vec::with_capacity(parameters.len());
        let mut flags = SignatureFlags::None;
        if is_construct {
            flags |= SignatureFlags::Construct;
        }
        // `min_argument_count` = number of leading required (non-optional,
        // non-rest) parameters.
        let mut min_argument_count: i32 = 0;
        let mut reached_optional_or_rest = false;
        for (i, param) in parameters.iter().enumerate() {
            let NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let is_rest = pd.dot_dot_dot_token.is_some();
            let is_optional = pd.question_token.is_some();
            // Resolve the parameter's type annotation. When no annotation is
            // present, fall back to the contextual signature's parameter
            // type at the same position (contextual typing for arrow/function
            // expression parameters), then to `any`.
            let param_type = match pd.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => {
                    let mut t = None;
                    if let Some(ctx_sig) = contextual_signature {
                        if i < ctx_sig.parameters.len() {
                            t = Some(self.get_type_of_symbol(&ctx_sig.parameters[i]));
                        }
                    }
                    t.unwrap_or_else(|| self.get_any_type())
                }
            };
            // Use the parameter name when it's an identifier; otherwise
            // synthesize a positional name.
            let name = pd.name.text().to_string();
            let name = if name.is_empty() {
                format!("__arg{}", i)
            } else {
                name
            };
            // Prefer the binder's actual parameter symbol when present, so
            // that both the call signature and the function body share the
            // same symbol. The body resolves parameters via the scope stack
            // (which finds the binder's symbol), so storing the resolved
            // parameter type on that symbol makes `get_type_of_symbol` return
            // it from within the body too — this is what lets contextual
            // typing flow into the function body. For synthetic type-annotation
            // nodes (FunctionType/ConstructorType) that have no binder symbol,
            // fall back to a fresh `Property` symbol.
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
            if is_rest {
                flags |= SignatureFlags::HasRestParameter;
                reached_optional_or_rest = true;
            } else if is_optional {
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
            type_parameters: Vec::new(),
            parameters: param_symbols,
            this_parameter: None,
            resolved_return_type: std::sync::OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: std::sync::OnceLock::new(),
        });
        // Eagerly populate the resolved return type so
        // `get_return_type_of_signature` returns `Some(...)` without a
        // separate inference pass.
        let _ = sig.resolved_return_type.set(return_type);
        sig
    }

    /// Create an anonymous object type with the given signatures. When
    /// `is_construct` is false, all signatures are call signatures
    /// (`call_signature_count = sigs.len()`); when true, all are construct
    /// signatures (`call_signature_count = 0`).
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
                SyntaxKind::ReadonlyKeyword => self.get_type_from_type_node(&data.type_node),
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
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_template_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_mapped_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.error_type();
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
        // Mirrors Go's `getTypeFromInferTypeNode`: resolve to the declared
        // type of the infer type parameter's symbol.
        let result = {
            let tp_node = match &node.data {
                NodeData::InferTypeNode(data) => &data.type_parameter,
                _ => return self.error_type(),
            };
            let symbol = self
                .program
                .symbol_map()
                .symbol_of(tp_node)
                .map(Arc::clone);
            match symbol {
                Some(sym) => self.get_type_parameter_from_symbol(&sym),
                None => self.error_type(),
            }
        };
        self.cache_type(node, result.clone());
        result
    }

    /// Build a `Conditional` type from a `ConditionalTypeNode` and attempt to
    /// resolve it. Mirrors Go's `getTypeFromConditionalTypeNode` +
    /// `getConditionalType`.
    ///
    /// When the check type is concrete (no remaining type parameters), the
    /// conditional is resolved immediately to the true or false branch. When
    /// the check type is generic (deferred), the unresolved `Conditional`
    /// type is returned — it will be resolved later when the type
    /// parameters are substituted (e.g. during `is_type_assignable_to`).
    fn build_conditional_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (check_type_node, extends_type_node) = match &node.data {
            NodeData::ConditionalTypeNode(data) => (
                Arc::clone(&data.check_type),
                Arc::clone(&data.extends_type),
            ),
            _ => return self.error_type(),
        };

        let check_type = self.get_type_from_type_node(&check_type_node);
        let extends_type = self.get_type_from_type_node(&extends_type_node);

        // Collect `infer R` type parameters from the ConditionalType node's
        // locals. The binder declares them there (see `bind_type_parameter`).
        let infer_type_parameters = self.collect_infer_type_parameters(node);

        let is_distributive = check_type.flags.contains(TypeFlags::TypeParameter);

        let root = Box::new(ConditionalRoot {
            node: Some(Arc::clone(node)),
            check_type: Some(Arc::clone(&check_type)),
            extends_type: Some(Arc::clone(&extends_type)),
            is_distributive,
            infer_type_parameters: infer_type_parameters.clone(),
            outer_type_parameters: Vec::new(),
            alias: None,
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
            }),
        ));

        // Try to resolve the conditional immediately. If the check type is
        // still generic, `resolve_conditional_type` returns `None` and we
        // return the unresolved conditional type.
        if let Some(resolved) = self.resolve_conditional_type(&cond_type) {
            resolved
        } else {
            cond_type
        }
    }

    /// Collect the `infer R` type parameters declared as locals of a
    /// `ConditionalType` node. Mirrors Go's `getInferTypeParameters`.
    fn collect_infer_type_parameters(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> {
        // Collect the type-parameter symbols first to avoid holding an
        // immutable borrow of `self.program.symbol_map()` across the
        // mutable `get_type_parameter_from_symbol` call.
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
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    // ────────────────────────────────────────────────────────────────────────
    // Type creation helpers
    // ────────────────────────────────────────────────────────────────────────

    pub fn get_union_type(&mut self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
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
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Reference,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                target: None,
                mapper: None,
                type_arguments: vec![element_type],
            }),
        })
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

    /// Compute `keyof T` — the union of string-literal property names of `T`.
    ///
    /// Mirrors Go's `getIndexType`. Currently handles:
    /// - Object/Interface/Tuple/Mapped types: union of `properties[*].name`
    ///   as string-literal types. If there are no properties, returns `never`.
    /// - Union types: `keyof (A | B)` = `keyof A & keyof B` (common keys).
    /// - Intersection types: `keyof (A & B)` = `keyof A | keyof B`.
    /// - Type parameters: `keyof T` = `keyof Constraint<T>` (if constrained).
    /// - `never`: returns `never`.
    /// - `any`/`error`: returns `string` (simplified — Go returns
    ///   `string | number | symbol`).
    /// - Other types: `never`.
    pub fn get_index_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        // `keyof never` is `never`; `keyof any` is `string | number | symbol`
        // (approximated as `string` here).
        if t.flags.contains(TypeFlags::Never) {
            return self.never_type();
        }
        if t.flags.contains(TypeFlags::Any) {
            // `keyof any` = string | number | symbol. We approximate with
            // `string` since number/symbol literal keys are rare in tests.
            return self.string_type();
        }
        // Union: `keyof (A | B)` = `keyof A & keyof B` — only keys present in
        // ALL constituents are valid. Since keyof of an object type yields a
        // union of string-literal types, the intersection reduces to the set
        // of common literal names. We compute this directly to avoid leaving
        // an unsimplified `Union & Union` intersection type.
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
                    Some(acc) => acc
                        .into_iter()
                        .filter(|n| names.contains(n))
                        .collect(),
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
        // Intersection: `keyof (A & B)` = `keyof A | keyof B` — keys present
        // in ANY constituent are valid.
        if t.flags.contains(TypeFlags::Intersection) {
            let types = match &t.data {
                TypeData::Intersection(i) => &i.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let keys: Vec<Arc<Type>> =
                types.iter().map(|c| self.get_index_type(c)).collect();
            return self.get_union_type(keys);
        }
        // Type parameter: resolve through the constraint.
        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.get_index_type(&constraint);
            }
            // Unconstrained type parameter: `keyof T` = `string | number | symbol`,
            // approximated as `string`.
            return self.string_type();
        }
        // Object-like types: collect property names as string-literal types.
        if let Some(structured) = t.as_structured() {
            if structured.properties.is_empty() {
                return self.never_type();
            }
            // Collect names first to avoid borrowing `self` while iterating.
            let names: Vec<String> =
                structured.properties.iter().map(|p| p.name.clone()).collect();
            let literals: Vec<Arc<Type>> = names
                .into_iter()
                .map(|n| self.get_string_literal_type(&n))
                .collect();
            return self.get_union_type(literals);
        }
        // Other types (literals, etc.): `keyof` is `never`.
        self.never_type()
    }

    /// Flatten a type into its string-literal values.
    ///
    /// Used by `get_index_type` to compute common keys across union
    /// constituents: `keyof (A | B)` collects the keyof of each arm (a
    /// union of string-literal types, or `never`) and intersects the name
    /// sets. This helper extracts those names.
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

    // ────────────────────────────────────────────────────────────────────────
    // Function return type inference (P3.8b)
    // ────────────────────────────────────────────────────────────────────────

    /// Walk a function body collecting the types of every `return expr;`
    /// statement, skipping nested function bodies (those have their own
    /// return type). Used to infer the return type of unannotated
    /// functions. Mirrors the spirit of Go's `collectReturnStatements`
    /// + `getReturnTypeOfFunction` flow.
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
            // Don't descend into nested function-like bodies — they have
            // their own return type.
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

    /// Infer a function's return type. If an explicit return-type annotation
    /// is present (`type_node`), it wins. Otherwise the body is walked for
    /// `return expr;` statements and the union of their types is returned.
    /// If no return statements are found, the inferred type is `void` (or
    /// `any` in non-strict-null-checks mode, mirroring Go's `voidType`).
    ///
    /// Mirrors Go's `getReturnTypeFromBody` (checker.go ~L20031). Literal
    /// return types are widened to their primitive base (e.g. `42` →
    /// `number`, `"foo"` → `string`) when no explicit return-type annotation
    /// is present, matching TypeScript's literal-widening rules.
    pub fn infer_function_return_type(&mut self, body: Option<&Arc<Node>>, type_node: Option<&Arc<Node>>) -> Arc<Type> {
        if let Some(type_node) = type_node {
            return self.get_type_from_type_node(type_node);
        }
        let Some(body) = body else {
            return self.void_type();
        };
        // Arrow functions can have an expression body (`() => expr`) rather
        // than a block body. In that case the expression IS the return value.
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
        // Widen fresh literal types to their primitive base (e.g. `42` →
        // `number`) for unannotated function returns.
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
