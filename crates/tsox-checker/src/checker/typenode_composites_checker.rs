#![allow(unused_imports)]

use crate::checker::typenode_composites::*;

impl Checker {
    pub(crate) fn get_type_from_array_or_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ArrayTypeNode(d) => {
                let elem_type = self.get_type_from_type_node(&d.element_type);
                self.create_array_type(elem_type)
            }
            NodeData::TupleTypeNode(d) => {
                let mut element_types = Vec::new();

                let mut element_infos = Vec::new();
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
                    element_infos.push(self.get_tuple_element_info(elem));
                    element_types.push(self.get_type_from_type_node(elem));
                }
                if has_variadic_union && !self.check_cross_product_union(node, &variadic_types) {
                    return self.error_type();
                }
                let readonly = node.parent().as_ref().is_some_and(|p| {
                    matches!(&p.data, NodeData::TypeOperatorNode(op)
                        if op.operator == SyntaxKind::ReadonlyKeyword)
                });
                self.create_tuple_type_ex(element_types, element_infos, readonly)
            }
            _ => self.error_type(),
        }
    }


    pub(crate) fn get_type_from_optional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("OptionalType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.add_optionality(&t)
    }

    pub(crate) fn get_type_from_union_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
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

    pub(crate) fn get_type_from_intersection_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
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
            if !all_undefined && !all_null && !self.check_cross_product_union(node, &member_types) {
                let e = self.error_type();
                self.cache_type(node, e.clone());
                return e;
            }
        }
        let result = self.get_intersection_type(member_types);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_named_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("NamedTupleMember has type").clone();
        self.get_type_from_type_node(&inner)
    }

    pub(crate) fn get_type_from_rest_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("RestType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.create_array_type(t)
    }
}
