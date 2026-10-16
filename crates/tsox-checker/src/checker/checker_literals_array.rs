use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::{
    CachedTypeKey, CachedTypeKind, ElementFlags, ObjectFlags, TupleElementInfo, Type,
    TypeData, TypeFlags, UnionReduction,
};

pub(crate) fn is_spread_into_call_or_new(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_spread_into_call_or_new"); 
    let Some(parent) = walk_up_parenthesized(node.parent()) else {
        return false;
    };
    if parent.kind != SyntaxKind::SpreadElement {
        return false;
    }
    matches!(
        parent.parent().map(|p| p.kind),
        Some(SyntaxKind::CallExpression | SyntaxKind::NewExpression)
    )
}

fn walk_up_parenthesized(parent: Option<Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("walk_up_parenthesized"); 
    let mut current = parent?;
    while current.kind == SyntaxKind::ParenthesizedExpression {
        current = current.parent()?;
    }
    Some(current)
}

impl Checker {
    pub(crate) fn is_const_context(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_const_context"); 
        let Some(parent) = node.parent() else {
            return false;
        };
        if tsox_frontend::ast::mig::m3f_4::is_const_assertion(&parent) {
            return true;
        }
        if self.is_inline_import_attributes(node) {
            return true;
        }
        if self.is_valid_const_assertion_argument(node)
            && self
                .get_contextual_type(node, crate::checker::inference::ContextFlags::empty())
                .is_some_and(|t| self.is_const_type_variable(&t, 0))
        {
            return true;
        }
        if matches!(
            parent.kind,
            SyntaxKind::ParenthesizedExpression
                | SyntaxKind::ArrayLiteralExpression
                | SyntaxKind::SpreadElement
        ) && self.is_const_context(&parent)
        {
            return true;
        }
        matches!(
            parent.kind,
            SyntaxKind::PropertyAssignment
                | SyntaxKind::ShorthandPropertyAssignment
                | SyntaxKind::TemplateSpan
        ) && parent
            .parent()
            .is_some_and(|grandparent| self.is_const_context(&grandparent))
    }

    pub(crate) fn is_tuple_like_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_tuple_like_type"); 
        if crate::checker::utilities::is_tuple_type(t) {
            return true;
        }
        let saved = self.property_lookup_skips_index_synthesis;
        self.property_lookup_skips_index_synthesis = true;
        let prop0 = self.get_property_of_type(t, "0");
        self.property_lookup_skips_index_synthesis = saved;
        if prop0.is_some() {
            return true;
        }
        if self.is_array_like_type(t)
            && let Some(length_type) = self.get_type_of_property_of_type(t, "length")
        {
            return match &length_type.data {
                TypeData::Union(u) => u
                    .union_or_intersection
                    .types
                    .iter()
                    .all(|c| c.flags.contains(TypeFlags::NumberLiteral)),
                _ => length_type.flags.contains(TypeFlags::NumberLiteral),
            };
        }
        false
    }

    fn contextual_member_is_tuple_like(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("contextual_member_is_tuple_like");
        if self.is_tuple_like_type(t) {
            return true;
        }
        matches!(&t.data, TypeData::Mapped(m) if m.name_type.is_none())
            && matches!(&t.data, TypeData::Mapped(m)
                if m.type_parameter.as_ref().is_some_and(|tp| {
                    self.get_constraint_of_type_parameter(tp).is_some()
                }))
    }

    fn some_contextual_member_tuple_like(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("some_contextual_member_tuple_like");
        match &t.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.contextual_member_is_tuple_like(c)),
            _ => self.contextual_member_is_tuple_like(t),
        }
    }

    pub(crate) fn tuple_type_arguments(t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("tuple_type_arguments");
        match &t.data {
            TypeData::Tuple(tuple) => tuple
                .element_infos
                .iter()
                .map(|e| e.type_.clone())
                .collect::<Option<Vec<_>>>()
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn create_array_literal_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_array_literal_type");
        if !t.object_flags.intersects(ObjectFlags::Reference) {
            return Arc::clone(t);
        }
        let key = CachedTypeKey {
            kind: CachedTypeKind::ArrayLiteralType,
            type_id: t.id(),
        };
        if let Some(cached) = self.cached_types.get(&key) {
            return Arc::clone(cached);
        }
        let mut literal_type = self.clone_type_reference(t);
        if let Some(t_mut) = Arc::get_mut(&mut literal_type) {
            t_mut.object_flags |=
                ObjectFlags::ArrayLiteral | ObjectFlags::ContainsObjectOrArrayLiteral;
        }
        self.cached_types.insert(key, literal_type.clone());
        literal_type
    }

    fn array_literal_element_type(
        &mut self,
        elem: &Arc<Node>,
        in_const_context: bool,
    ) -> (Arc<Type>, ElementFlags) { ::tsox_core::fntrace::enter("array_literal_element_type");
        if elem.kind == SyntaxKind::SpreadElement {
            let inner = match &elem.data {
                NodeData::SpreadElement(s) => Arc::clone(&s.expression),
                _ => return (self.get_any_type(), ElementFlags::Rest),
            };
            let spread_type = self.get_type_of_node(&inner);
            if self.is_array_like_type(&spread_type) {
                return (spread_type, ElementFlags::Variadic);
            }
            return (self.get_any_type(), ElementFlags::Rest);
        }
        let t = self.get_type_of_node(elem);
        let widened = if in_const_context {
            t
        } else {
            let contextual = self.get_contextual_type(
                elem,
                crate::checker::inference::ContextFlags::empty(),
            );
            let literal_of_ctx = contextual
                .as_ref()
                .is_some_and(|c| self.is_literal_of_contextual_type(&t, c));
            if literal_of_ctx {
                self.get_regular_type_of_literal_type(&t)
            } else if crate::checker::is_object_literal_type(&t) {
                // Go checkExpressionForMutableLocation 默认臂（checker.go:14639-14642）
                // 经 getWidenedLiteralLikeTypeForContextualType（checker.go:27047-27051）：
                // getWidenedLiteralType 对对象字面量恒等（checker.go:27017-27031），
                // 终点 getRegularTypeOfObjectLiteral 仅剥 FreshLiteral 保留
                // ObjectLiteral（checker.go:29842-29843）——union SubtypeReduction
                // 的 propertiesRelatedTo 对象字面量目标规则（relater.go:4434-4443）
                // 依赖该标志拦截成员吞并；加宽路径（getWidenedTypeOfObjectLiteral
                // checker.go:19507-19509 剥 ObjectLiteral）属 var 初始化语义，
                // 不适用于数组元素
                self.get_regular_type_of_object_literal(&t)
            } else if t.flags.intersects(TypeFlags::Null | TypeFlags::Undefined) {
                t
            } else {
                let instantiated_contextual = contextual.as_ref().map(|ct| {
                    self.instantiate_contextual_type(
                        ct,
                        elem,
                        crate::checker::inference::ContextFlags::empty(),
                    )
                });
                self.get_widened_literal_like_type_for_contextual_type(
                    &t,
                    instantiated_contextual.as_ref(),
                )
            }
        };
        (widened, ElementFlags::Required)
    }

    pub(crate) fn get_type_of_array_literal(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_array_literal");
        let elements = match &node.data {
            NodeData::ArrayLiteralExpression(data) => &data.elements,
            _ => return self.get_any_type(),
        };

        let in_destructuring_pattern =
            crate::checker::checker_object_literal_is_destructuring_target::is_assignment_target(
                node,
            );
        let in_const_context = self.is_const_context(node);
        let contextual_type =
            self.get_contextual_type(node, crate::checker::inference::ContextFlags::None);
        let in_tuple_context = is_spread_into_call_or_new(node)
            || contextual_type
                .as_ref()
                .is_some_and(|ct| self.some_contextual_member_tuple_like(ct));


        let mut element_types: Vec<Arc<Type>> = Vec::with_capacity(elements.len());
        let mut element_infos: Vec<TupleElementInfo> = Vec::with_capacity(elements.len());
        let mut has_omitted = false;
        for elem in elements.iter() {
            if elem.kind == SyntaxKind::OmittedExpression && self.exact_optional_property_types {
                has_omitted = true;
                element_types.push(self.undefined_type());
                element_infos.push(TupleElementInfo {
                    label: None,
                    flags: ElementFlags::Optional,
                    labeled_declaration: None,
                    type_: None,
                });
                continue;
            }
            let (t, base_flags) = self.array_literal_element_type(elem, in_const_context);
            let flags = if has_omitted {
                ElementFlags::Optional
            } else {
                base_flags
            };
            element_types.push(t);
            element_infos.push(TupleElementInfo {
                label: None,
                flags,
                labeled_declaration: None,
                type_: None,
            });
        }

        if in_destructuring_pattern {
            return self.create_tuple_type_ex(element_types, element_infos, false);
        }

        if in_const_context || in_tuple_context {
            let readonly = in_const_context && !self.contextual_has_mutable_array_like(&contextual_type);
            return self.create_tuple_type_ex(element_types, element_infos, readonly);
        }

        if element_types.is_empty() {
            let elem = if self.strict_null_checks {
                self.never_type()
            } else {
                self.nullish_widening_type(self.undefined_type())
            };
            let array_type = self.create_array_type_ex(elem, false);
            return self.create_array_literal_type(&array_type);
        }

        let mut element_union_types: Vec<Arc<Type>> = Vec::with_capacity(element_types.len());
        for (i, t) in element_types.iter().enumerate() {
            if element_infos[i].flags.contains(ElementFlags::Variadic) {
                let number = self.number_type();
                element_union_types.push(self.get_indexed_access_type(t, &number));
            } else {
                element_union_types.push(Arc::clone(t));
            }
        }

        let elem_union = self.get_union_type_ex(element_union_types, UnionReduction::Subtype, None, None);
        let array_type = self.create_array_type_ex(elem_union, false);
        self.create_array_literal_type(&array_type)
    }

    fn contextual_has_mutable_array_like(&mut self, contextual_type: &Option<Arc<Type>>) -> bool { ::tsox_core::fntrace::enter("contextual_has_mutable_array_like");
        contextual_type.as_ref().is_some_and(|ct| match &ct.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.is_mutable_array_like_type(c)),
            _ => self.is_mutable_array_like_type(ct),
        })
    }

    pub(crate) fn is_mutable_array_like_type(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_mutable_array_like_type");
        self.is_array_like_type(t)
            && !matches!(&t.data, TypeData::Tuple(tuple) if tuple.readonly)
    }
}
