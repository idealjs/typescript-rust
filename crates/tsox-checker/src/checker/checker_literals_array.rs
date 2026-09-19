use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::{
    ElementFlags, ObjectFlags, ObjectTypeData, StructuredTypeData, TupleElementInfo, Type,
    TypeData, TypeFlags,
};

pub(crate) fn is_spread_into_call_or_new(node: &Arc<Node>) -> bool {
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

fn walk_up_parenthesized(parent: Option<Arc<Node>>) -> Option<Arc<Node>> {
    let mut current = parent?;
    while current.kind == SyntaxKind::ParenthesizedExpression {
        current = current.parent()?;
    }
    Some(current)
}

fn entity_name_text(node: &Arc<Node>) -> String {
    match &node.data {
        NodeData::Identifier(id) => id.text.clone(),
        NodeData::QualifiedName(q) => entity_name_text(&q.right),
        _ => String::new(),
    }
}

fn is_const_type_reference_type(node: &Arc<Node>) -> bool {
    match &node.data {
        NodeData::TypeReferenceNode(r) => {
            r.type_arguments.is_none() && entity_name_text(&r.type_name) == "const"
        }
        _ => false,
    }
}

fn has_readonly_modifier(node: &Arc<Node>) -> bool {
    let modifiers = match &node.data {
        NodeData::PropertyDeclaration(p) => p.modifiers.as_ref(),
        NodeData::PropertySignatureDeclaration(p) => p.modifiers.as_ref(),
        _ => None,
    };
    modifiers
        .map(|m| {
            m.list
                .nodes
                .iter()
                .any(|k| k.kind == SyntaxKind::ReadonlyKeyword)
        })
        .unwrap_or(false)
}

fn is_readonly_declaration(node: &Arc<Node>) -> bool {
    match &node.data {
        NodeData::VariableDeclaration(_) => false,
        NodeData::PropertyDeclaration(_) | NodeData::PropertySignatureDeclaration(_) => {
            has_readonly_modifier(node)
        }
        NodeData::EnumMember(_) => true,
        NodeData::NamedTupleMember(_) => false,
        _ => false,
    }
}

fn is_const_type_reference(node: &Arc<Node>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind == SyntaxKind::TypeAssertionExpression
        && let NodeData::TypeAssertion(a) = &parent.data
        && is_const_type_reference_type(&a.type_node)
    {
        return true;
    }
    match parent.kind {
        SyntaxKind::ParenthesizedExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::SatisfiesExpression => is_const_type_reference(&parent),
        SyntaxKind::VariableDeclaration
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::EnumMember
        | SyntaxKind::NamedTupleMember => is_readonly_declaration(&parent),
        SyntaxKind::ArrayLiteralExpression => is_const_type_reference(&parent),
        _ => false,
    }
}

impl Checker {
    pub(crate) fn is_const_context(&self, node: &Arc<Node>) -> bool {
        is_const_type_reference(node)
    }

    pub(crate) fn is_tuple_like_type(&mut self, t: &Arc<Type>) -> bool {
        if crate::checker::utilities::is_tuple_type(t)
            || self.get_property_of_type(t, "0").is_some()
        {
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

    fn contextual_member_is_tuple_like(&mut self, t: &Arc<Type>) -> bool {
        if self.is_tuple_like_type(t) {
            return true;
        }
        matches!(&t.data, TypeData::Mapped(m) if m.name_type.is_none())
            && matches!(&t.data, TypeData::Mapped(m)
                if m.type_parameter.as_ref().is_some_and(|tp| {
                    self.get_constraint_of_type_parameter(tp).is_some()
                }))
    }

    fn some_contextual_member_tuple_like(&mut self, t: &Arc<Type>) -> bool {
        match &t.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.contextual_member_is_tuple_like(c)),
            _ => self.contextual_member_is_tuple_like(t),
        }
    }

    pub(crate) fn tuple_type_arguments(t: &Arc<Type>) -> Vec<Arc<Type>> {
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

    fn create_array_literal_type(&mut self, element_type: Arc<Type>) -> Arc<Type> {
        let Some(array_symbol) = self.globals.get("Array").cloned() else {
            return self.get_any_type();
        };
        let target = self.get_declared_type_of_symbol(&array_symbol);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Reference
                | ObjectFlags::ArrayLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral,
            id: crate::checker::types::next_type_id(),
            symbol: Some(array_symbol),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData::default(),
                target: Some(target),
                mapper: None,
                type_arguments: vec![element_type],
            }),
        })
    }

    fn array_literal_element_type(
        &mut self,
        elem: &Arc<Node>,
        in_const_context: bool,
    ) -> (Arc<Type>, ElementFlags) {
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
        } else if crate::checker::is_object_literal_type(&t) {
            self.widen_initializer_type(&t)
        } else if t.flags.intersects(TypeFlags::Null | TypeFlags::Undefined) {
            t
        } else {
            self.get_widened_type(&t)
        };
        (widened, ElementFlags::Required)
    }

    pub(crate) fn get_type_of_array_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
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
            return self.create_array_literal_type(elem);
        }

        let mut reduced_types: Vec<Arc<Type>> = Vec::with_capacity(element_types.len());
        let mut has_nullable = false;
        for (i, t) in element_types.iter().enumerate() {
            if element_infos[i].flags.contains(ElementFlags::Variadic) {
                let number = self.number_type();
                reduced_types.push(self.get_indexed_access_type(t, &number));
            } else {
                if t.flags.intersects(TypeFlags::Null | TypeFlags::Undefined) {
                    has_nullable = true;
                }
                reduced_types.push(Arc::clone(t));
            }
        }

        let reduced = self.remove_subtype_redundant_members(reduced_types);
        if reduced.is_empty() {
            let any = self.get_any_type();
            return self.create_array_literal_type(any);
        }
        if reduced.len() == 1 {
            let only = Arc::clone(&reduced[0]);
            if !self.strict_null_checks
                && only.flags.intersects(TypeFlags::Null | TypeFlags::Undefined)
            {
                let any = self.get_any_type();
                return self.create_array_literal_type(any);
            }
            return self.create_array_literal_type(only);
        }
        if !self.strict_null_checks && has_nullable {
            let any = self.get_any_type();
            return self.create_array_literal_type(any);
        }
        let elem_union = self.get_union_type(reduced);
        self.create_array_literal_type(elem_union)
    }

    fn contextual_has_mutable_array_like(&mut self, contextual_type: &Option<Arc<Type>>) -> bool {
        contextual_type.as_ref().is_some_and(|ct| match &ct.data {
            TypeData::Union(u) => u
                .union_or_intersection
                .types
                .iter()
                .any(|c| self.is_mutable_array_like_type(c)),
            _ => self.is_mutable_array_like_type(ct),
        })
    }

    fn is_mutable_array_like_type(&mut self, t: &Arc<Type>) -> bool {
        self.is_array_like_type(t)
            && !matches!(&t.data, TypeData::Tuple(tuple) if tuple.readonly)
    }
}
