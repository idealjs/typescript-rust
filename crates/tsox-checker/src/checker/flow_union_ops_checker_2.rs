#![allow(unused_imports)]

use crate::checker::flow_union_ops::*;

impl Checker {
    pub(crate) fn remove_falsy_from_union(
        &self,
        type_: &Arc<Type>,
        falsy_flags: TypeFlags,
    ) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let remaining: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {
                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(true));
                        }
                    }

                    if t.flags.contains(TypeFlags::StringLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::String(s) = &lit.value {
                                return !s.is_empty();
                            }
                        }
                        return false;
                    }

                    if t.flags.contains(TypeFlags::NumberLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            if let LiteralValue::Number(n) = &lit.value {
                                return n.0 != 0.0;
                            }
                        }
                        return false;
                    }
                    return false;
                }
                true
            })
            .collect();
        if remaining.is_empty() {
            return self.never_type();
        }
        if remaining.len() == 1 {
            return remaining.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: remaining,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn filter_to_falsy(&self, type_: &Arc<Type>) -> Arc<Type> {
        let constituents = self.constituent_types(type_);
        let falsy_flags =
            TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Void | TypeFlags::BooleanLiteral;
        let matching: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                if t.flags.intersects(falsy_flags) {
                    if t.flags.contains(TypeFlags::BooleanLiteral) {
                        if let TypeData::Literal(lit) = &t.data {
                            return matches!(lit.value, LiteralValue::Boolean(false));
                        }
                    }
                    return true;
                }

                if t.flags.contains(TypeFlags::StringLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::String(s) = &lit.value {
                            return s.is_empty();
                        }
                    }
                }

                if t.flags.contains(TypeFlags::NumberLiteral) {
                    if let TypeData::Literal(lit) = &t.data {
                        if let LiteralValue::Number(n) = &lit.value {
                            return n.0 == 0.0;
                        }
                    }
                }
                false
            })
            .collect();
        if matching.is_empty() {
            return self.never_type();
        }
        if matching.len() == 1 {
            return matching.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: matching,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: std::collections::HashMap::new(),
            }),
        ))
    }

    pub(crate) fn intersect_or_narrow(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> {
        // Go getNarrowedTypeWorker（assumeTrue）：对候选（谓词）成分逐个与源
        // 成分做 strictSubtype/subtype 四级阶梯，取更具体侧且谓词侧优先
        //（{} 与全可选属性类型双向可赋值，须靠 strictSubtype 分出谓词侧）；
        // 全空时实例化成分按约束交集，最后按可赋值性收尾或交集兜底
        if Arc::ptr_eq(type_, value_type) {
            return Arc::clone(value_type);
        }
        if type_.flags.intersects(TypeFlags::Any | TypeFlags::Unknown) {
            return Arc::clone(value_type);
        }
        let sources = self.constituent_types(type_);
        let candidates = self.constituent_types(value_type);
        let mut picked: Vec<Arc<Type>> = Vec::new();
        for n in &candidates {
            let mut directly: Vec<Arc<Type>> = Vec::new();
            for t in &sources {
                if self.is_type_strict_subtype_of(t, n) {
                    directly.push(Arc::clone(t));
                } else if self.is_type_strict_subtype_of(n, t) {
                    directly.push(Arc::clone(n));
                } else if self.is_type_subtype_of(t, n) {
                    directly.push(Arc::clone(t));
                } else if self.is_type_subtype_of(n, t) {
                    directly.push(Arc::clone(n));
                }
            }
            if !directly.is_empty() {
                picked.extend(directly);
                continue;
            }
            for t in &sources {
                if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE) {
                    let related = match self.get_base_constraint_of_type(t) {
                        None => true,
                        Some(constraint) => self.is_type_subtype_of(n, &constraint),
                    };
                    if related {
                        picked.push(self.get_intersection_type(vec![Arc::clone(t), Arc::clone(n)]));
                    }
                }
            }
        }
        if !picked.is_empty() {
            return self.get_union_type(picked);
        }
        if self.is_type_subtype_of(value_type, type_) {
            return Arc::clone(value_type);
        }
        if self.is_type_assignable_to(type_, value_type) {
            return Arc::clone(type_);
        }
        // Go getNarrowedTypeWorker 兜底序：candidate→t 可赋取 candidate，
        // t→candidate 可赋取 t（never 可赋给一切，谓词真分支保 never），
        // 否则交集
        if self.is_type_assignable_to(value_type, type_) {
            return Arc::clone(value_type);
        }
        self.get_intersection_type(vec![Arc::clone(type_), Arc::clone(value_type)])
    }

    pub(crate) fn types_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        if a.flags.contains(TypeFlags::Union)
            || b.flags.contains(TypeFlags::Union)
            || a.flags.contains(TypeFlags::Intersection)
            || b.flags.contains(TypeFlags::Intersection)
        {
            let a_types = self.constituent_types(a);
            let b_types = self.constituent_types(b);
            for at in &a_types {
                for bt in &b_types {
                    if self.literals_overlap(at, bt) {
                        return true;
                    }
                }
            }
            return false;
        }
        self.literals_overlap(a, b)
    }

    pub(crate) fn literals_overlap(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        let a_is_literal = a.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        let b_is_literal = b.flags.intersects(
            TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BigIntLiteral
                | TypeFlags::BooleanLiteral,
        );
        if a_is_literal && b_is_literal {
            return match (&a.data, &b.data) {
                (TypeData::Literal(a_lit), TypeData::Literal(b_lit)) => a_lit.value == b_lit.value,
                _ => false,
            };
        }
        if a_is_literal {
            return a.flags.intersects(b.flags);
        }
        if b_is_literal {
            return a.flags.intersects(b.flags);
        }

        a.flags.intersects(b.flags)
    }

    pub(crate) fn is_symbol_identifier(&self, node: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
        if matches!(
            node.kind,
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
        ) {
            return self
                .program
                .symbol_map()
                .symbol_of(node)
                .is_some_and(|s| Arc::ptr_eq(s, symbol));
        }
        if node.kind != SyntaxKind::Identifier {
            return false;
        }

        let symbol_map = self.program.symbol_map();
        if let Some(sym) = symbol_map.symbol_of(node) {
            if Arc::ptr_eq(sym, symbol) {
                return true;
            }
        }

        let node_name = match &node.data {
            NodeData::Identifier(data) => &data.text,
            _ => return false,
        };
        node_name == &symbol.name
    }

    pub(crate) fn expr_matches_target(&self, node: &Arc<Node>, target: &FlowRef) -> bool {
        match target {
            FlowRef::Symbol(symbol) => self.is_symbol_identifier(node, symbol),
            FlowRef::Node(reference) => self.is_matching_reference(reference, node),
        }
    }
}
