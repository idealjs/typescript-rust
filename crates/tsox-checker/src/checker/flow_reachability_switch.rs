#![allow(unused_imports)]

use crate::checker::flow_impl_chunk::*;
use crate::checker::utilities_token_is_identifier_or_keyword::is_unit_type;
use tsox_frontend::ast::{NodeData, SyntaxKind};

impl Checker {
    pub fn is_exhaustive_switch_statement(&mut self, node: &Arc<Node>) -> bool {
        let key = Arc::as_ptr(node) as usize as u64;
        match self.switch_exhaustive_state.get(&key) {
            Some(&2) => return true,
            Some(&3) => return false,
            Some(&1) => return false,
            _ => {}
        }
        self.switch_exhaustive_state.insert(key, 1);
        let exhaustive = self.compute_exhaustive_switch_statement(node);
        self.switch_exhaustive_state
            .insert(key, if exhaustive { 2 } else { 3 });
        exhaustive
    }

    fn compute_exhaustive_switch_statement(&mut self, node: &Arc<Node>) -> bool {
        let NodeData::SwitchStatement(data) = &node.data else {
            return false;
        };
        if data.expression.kind == SyntaxKind::TypeOfExpression {
            return false;
        }
        let expression = Arc::clone(&data.expression);
        let expression_type = self.get_type_of_node(&expression);
        let t = self.get_base_constraint_or_type(&expression_type);
        if !is_literal_switch_type(&t) {
            return false;
        }
        let switch_types = self.get_switch_clause_types(node);
        if switch_types.is_empty()
            || switch_types
                .iter()
                .any(|ct| !is_unit_type(ct) && !ct.flags.contains(TypeFlags::Never))
        {
            return false;
        }
        let t = self.get_regular_type_of_literal_type(&t);
        let switch_types: Vec<Arc<Type>> = switch_types
            .iter()
            .map(|st| self.get_regular_type_of_literal_type(st))
            .collect();
        if t.flags.contains(TypeFlags::Boolean) {
            return switch_types.iter().any(|st| is_boolean_literal(st, true))
                && switch_types
                    .iter()
                    .any(|st| is_boolean_literal(st, false));
        }
        let constituents = self.constituent_types(&t);
        constituents
            .iter()
            .all(|c| switch_types.iter().any(|st| same_unit_type(c, st)))
    }
}

fn is_literal_switch_type(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::Boolean) {
        return true;
    }
    if t.flags.contains(TypeFlags::Union) {
        return t.flags.contains(TypeFlags::EnumLiteral) || constituent_flags_all_unit(t);
    }
    is_unit_type(t)
}

fn constituent_flags_all_unit(t: &Arc<Type>) -> bool {
    match &t.data {
        TypeData::Union(u) => u
            .union_or_intersection
            .types
            .iter()
            .all(|ct| is_unit_type(ct)),
        _ => false,
    }
}

fn same_unit_type(a: &Arc<Type>, b: &Arc<Type>) -> bool {
    if Arc::ptr_eq(a, b) {
        return true;
    }
    match (&a.data, &b.data) {
        (TypeData::Literal(la), TypeData::Literal(lb)) => la.value == lb.value,
        _ => false,
    }
}

fn is_boolean_literal(t: &Arc<Type>, value: bool) -> bool {
    matches!(
        &t.data,
        TypeData::Literal(lit) if lit.value == LiteralValue::Boolean(value)
    )
}
