use std::sync::Arc;

use crate::ast::{
    Node, NodeData, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};
use crate::core::text::TextRange;


use super::*;

impl Checker {
    pub fn get_widened_type_of_literal(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(crate::checker::TypeFlags::StringLiteral)
            || t.flags.contains(crate::checker::TypeFlags::NumberLiteral)
            || t.flags.contains(crate::checker::TypeFlags::BigIntLiteral)
            || t.flags.contains(crate::checker::TypeFlags::BooleanLiteral)
        {
            return self.get_base_type_of_literal_type(t);
        }
        Arc::clone(t)
    }

    pub(crate) fn types_are_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        if a.flags != b.flags {
            return false;
        }

        match (&a.data, &b.data) {
            (crate::checker::TypeData::Intrinsic(a), crate::checker::TypeData::Intrinsic(b)) => {
                a.intrinsic_name == b.intrinsic_name
            }
            _ => false,
        }
    }

    pub(crate) fn infer_number_literal_type(&mut self, text: &str) -> Arc<Type> {

        let num = crate::jsnum::Number::from_string(text);
        if num.is_nan() {
            return self.number_type();
        }
        self.get_number_literal_type(num)
    }

    pub(crate) fn infer_string_literal_type(&mut self, text: &str) -> Arc<Type> {
        self.get_string_literal_type(text)
    }
    pub(crate) fn find_object_literal_property_name_node(
        &self,
        init: &Arc<Node>,
        prop_name: &str,
    ) -> Option<TextRange> {
        let crate::ast::NodeData::ObjectLiteralExpression(data) = &init.data else {
            return None;
        };
        for prop in data.properties.iter() {
            let name = match &prop.data {
                NodeData::PropertyAssignment(p) => &p.name,
                NodeData::ShorthandPropertyAssignment(p) => &p.name,
                _ => continue,
            };
            if self.get_property_name_from_node(name) == prop_name {
                return Some(name.loc);
            }
        }
        None
    }
    pub(crate) fn get_const_assertion_type(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        match expr.kind {
            SyntaxKind::ArrayLiteralExpression => {

                let elements = match &expr.data {
                    crate::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
                    _ => return self.get_any_type(),
                };
                let mut element_types: Vec<Arc<Type>> = Vec::new();
                for elem in elements.iter() {
                    if elem.kind == SyntaxKind::SpreadElement {

                        let t = self.get_type_of_node(elem);
                        element_types.push(t);
                    } else {
                        element_types.push(self.get_type_of_node(elem));
                    }
                }
                self.create_tuple_type(element_types)
            }
            _ => {

                self.get_type_of_node(expr)
            }
        }
    }

    pub(crate) fn get_type_of_object_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let properties = match &node.data {
            crate::ast::NodeData::ObjectLiteralExpression(data) => &data.properties,
            _ => return self.get_any_type(),
        };

        let contextual =
            self.get_contextual_type(node, ContextFlags::empty());
        let mut prop_pairs: Vec<(String, Arc<Type>, Option<Arc<Node>>)> = Vec::new();
        let mut fell_back_to_any = false;
        for prop in properties.iter() {
            match &prop.data {
                NodeData::PropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }

                    let mut t = self.get_type_of_node(&data.initializer);
                    if let Some(ctx) = &contextual
                        && let Some(prop_ctx) = self.get_type_of_property_of_type(ctx, &name)
                        && crate::checker::is_fresh_literal_type(&t)
                    {

                        if !self.is_literal_of_contextual_type(&t, &prop_ctx) {
                            t = self.get_widened_literal_type(&t);
                        } else {
                            t = self.get_regular_type_of_literal_type(&t);
                        }
                    }
                    prop_pairs.push((name, t, Some(Arc::clone(prop))));
                }
                NodeData::ShorthandPropertyAssignment(data) => {
                    let name = self.get_property_name_from_node(&data.name);
                    if name.is_empty() {
                        fell_back_to_any = true;
                        break;
                    }

                    let t = self.get_type_of_node(&data.name);
                    prop_pairs.push((name, t, Some(Arc::clone(prop))));
                }
                NodeData::SpreadAssignment(_) => {

                    fell_back_to_any = true;
                    break;
                }
                _ => {
                    fell_back_to_any = true;
                    break;
                }
            }
        }
        if fell_back_to_any {
            return self.get_any_type();
        }

        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(prop_pairs.len());
        for (name, t, decl) in prop_pairs {
            let mut sym = Symbol::new(SymbolFlags::Property, name.clone());
            if let Some(d) = decl {
                sym.declarations.push(d);
            }
            let symbol = Arc::new(sym);
            members.insert(name, Arc::clone(&symbol));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous | ObjectFlags::ObjectLiteral,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn get_excess_property_name(&self, source: &Arc<Type>, target: &Arc<Type>) -> Option<String> {

        if !crate::checker::is_object_literal_type(source) {
            return None;
        }
        let source_struct = source.as_structured()?;
        let target_struct = target.as_structured()?;

        if !target_struct.index_infos.is_empty() {
            return None;
        }
        for prop in &source_struct.properties {

            if !self.target_has_property(target, &prop.name) {
                return Some(prop.name.clone());
            }
        }
        None
    }

    fn target_has_property(&self, t: &Arc<Type>, name: &str) -> bool {

        if matches!(&t.data, TypeData::Mapped(m) if m.type_parameter.is_some()) {
            return true;
        }
        if let Some(structured) = t.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }

            if !structured.index_infos.is_empty() {
                return true;
            }
        }

        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|ct| self.target_has_property(ct, name));
            }
        }

        if t.flags.contains(TypeFlags::Intersection) {
            if let TypeData::Intersection(i) = &t.data {
                return i
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|ct| self.target_has_property(ct, name));
            }
        }
        false
    }
    pub(crate) fn get_constant_numeric_value(&self, node: &Arc<Node>) -> Option<f64> {
        match &node.data {
            crate::ast::NodeData::NumericLiteral(data) => data.text.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub(crate) fn get_type_of_array_literal(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let elements = match &node.data {
            crate::ast::NodeData::ArrayLiteralExpression(data) => &data.elements,
            _ => return self.get_any_type(),
        };
        if elements.is_empty() {

            let elem = if self.strict_null_checks {
                self.never_type()
            } else {
                self.undefined_type()
            };
            return self.create_array_type(elem);
        }

        let mut element_types: Vec<Arc<Type>> = Vec::new();
        for elem in elements.iter() {

            if elem.kind == SyntaxKind::SpreadElement {
                return self.create_array_type(self.get_any_type());
            }
            let t = self.get_type_of_node(elem);

            let widened = if crate::checker::is_object_literal_type(&t) {
                self.widen_initializer_type(&t)
            } else {
                self.get_widened_type_of_literal(&t)
            };
            element_types.push(widened);
        }

        let first = &element_types[0];
        let all_same = element_types[1..]
            .iter()
            .all(|t| Arc::ptr_eq(t, first) || self.types_are_equal(t, first));
        if all_same {
            return self.create_array_type(Arc::clone(first));
        }

        let elem_union = self.get_union_type(element_types);
        self.create_array_type(elem_union)
    }
}
