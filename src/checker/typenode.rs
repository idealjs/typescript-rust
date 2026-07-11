//! Type node resolution: converting AST type nodes to `Type` objects.
//!
//! Ported from `getTypeFromTypeNode` and related functions in
//! `internal/checker/checker.go` (lines ~22713–22910).
//!
//! This is the primary entry point for converting a type annotation in the
//! AST (e.g. `string`, `number[]`, `Array<T>`, `A | B`, `keyof T`) into the
//! corresponding `Type` used by the checker.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, SyntaxKind};

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
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
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
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
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
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    fn get_type_from_infer_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
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
            .map(|_| TupleElementInfo {
                flags: ElementFlags::None,
                labeled_declaration: None,
            })
            .collect();
        let fixed_length = element_types.len();
        Arc::new(Type::new(
            TypeFlags::Object,
            TypeData::Tuple(TupleTypeData {
                interface_data: InterfaceTypeData::default(),
                element_infos,
                min_length: fixed_length,
                fixed_length,
                combined_flags: ElementFlags::None,
                readonly: false,
            }),
        ))
    }

    pub fn get_index_type(&mut self, _type: &Arc<Type>) -> Arc<Type> {
        self.error_type()
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
