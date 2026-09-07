#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn narrow_by_switch_on_discriminant_property(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
        access: &Arc<Node>,
    ) -> Arc<Type> {
        let Some(prop_name) = Self::get_accessed_property_name_from_node(access) else {
            return Arc::clone(type_);
        };
        let group_clauses = clauses_of_range(switch_stmt, clause_start, clause_end);
        let is_default = group_clauses.is_empty()
            || group_clauses
                .iter()
                .all(|c| c.kind == SyntaxKind::DefaultClause);

        if !type_.is_union() {
            let mut any_overlap = is_default;
            for clause in &group_clauses {
                if clause.kind == SyntaxKind::DefaultClause {
                    continue;
                }
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    let case_type = self.get_type_of_node(&cd.expression);
                    if let Some(prop_type) = self.get_property_type_of_type(type_, &prop_name)
                        && self.types_overlap(&prop_type, &case_type)
                    {
                        any_overlap = true;
                    }
                }
            }
            if !any_overlap {
                return self.never_type();
            }
            return Arc::clone(type_);
        }

        let group_case_types: Vec<Arc<Type>> = group_clauses
            .iter()
            .filter(|c| c.kind == SyntaxKind::CaseClause)
            .filter_map(|c| match &c.data {
                NodeData::CaseOrDefaultClause(cd) => Some(self.get_type_of_node(&cd.expression)),
                _ => None,
            })
            .collect();
        let all_case_types = if is_default {
            self.get_switch_clause_types(switch_stmt)
        } else {
            Vec::new()
        };
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);
                let Some(prop_type) = prop_type else {
                    return is_default;
                };
                if is_default {
                    !all_case_types
                        .iter()
                        .any(|ct| self.types_overlap(&prop_type, ct))
                } else {
                    group_case_types
                        .iter()
                        .any(|ct| self.types_overlap(&prop_type, ct))
                }
            })
            .collect();
        self.rebuild_union_or_never(type_, filtered)
    }

    pub(crate) fn get_switch_clause_types(&mut self, switch_stmt: &Arc<Node>) -> Vec<Arc<Type>> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Vec::new();
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return Vec::new();
        };
        let mut types = Vec::with_capacity(case_block.clauses.len());
        for clause in &case_block.clauses.nodes {
            if clause.kind == SyntaxKind::CaseClause {
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    types.push(self.get_type_of_node(&cd.expression));
                    continue;
                }
            }
            types.push(self.never_type());
        }
        types
    }

    pub(crate) fn optional_chain_contains_target(
        &self,
        source: &Arc<Node>,
        target: &FlowRef,
    ) -> bool {
        let symbol = match target {
            FlowRef::Symbol(symbol) => symbol,
            FlowRef::Node(reference) => {
                return self.optional_chain_contains_reference(source, reference);
            }
        };
        self.optional_chain_contains_symbol(source, symbol)
    }

    pub(crate) fn optional_chain_contains_symbol(
        &self,
        source: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        let mut current = Arc::clone(source);
        loop {
            let (inner, is_optional) = match &current.data {
                NodeData::PropertyAccessExpression(pa) => {
                    (&pa.expression, pa.question_dot_token.is_some())
                }
                NodeData::ElementAccessExpression(ea) => {
                    (&ea.expression, ea.question_dot_token.is_some())
                }
                NodeData::CallExpression(ce) => (&ce.expression, ce.question_dot_token.is_some()),
                NodeData::NonNullExpression(ne) => (&ne.expression, false),
                NodeData::ParenthesizedExpression(pe) => (&pe.expression, false),
                _ => return false,
            };
            if is_optional && self.is_symbol_identifier(inner, symbol) {
                return true;
            }
            if !is_optional
                && !matches!(
                    &current.data,
                    NodeData::NonNullExpression(_) | NodeData::ParenthesizedExpression(_)
                )
            {
                return false;
            }
            current = Arc::clone(inner);
        }
    }

    pub(crate) fn optional_chain_contains_reference(
        &self,
        source: &Arc<Node>,
        reference: &Arc<Node>,
    ) -> bool {
        let mut current = Arc::clone(source);
        loop {
            let (inner, is_optional) = match &current.data {
                NodeData::PropertyAccessExpression(pa) => {
                    (&pa.expression, pa.question_dot_token.is_some())
                }
                NodeData::ElementAccessExpression(ea) => {
                    (&ea.expression, ea.question_dot_token.is_some())
                }
                NodeData::CallExpression(ce) => (&ce.expression, ce.question_dot_token.is_some()),
                NodeData::NonNullExpression(ne) => (&ne.expression, false),
                NodeData::ParenthesizedExpression(pe) => (&pe.expression, false),
                _ => return false,
            };
            if is_optional && self.is_matching_reference(reference, inner) {
                return true;
            }
            if !is_optional
                && !matches!(
                    &current.data,
                    NodeData::NonNullExpression(_) | NodeData::ParenthesizedExpression(_)
                )
            {
                return false;
            }
            current = Arc::clone(inner);
        }
    }

    pub(crate) fn narrow_by_optional_chain_containment(
        &mut self,
        type_: &Arc<Type>,
        op: SyntaxKind,
        value_node: &Arc<Node>,
        kind: NarrowKind,
    ) -> Arc<Type> {
        let is_equality =
            op == SyntaxKind::EqualsEqualsEqualsToken || op == SyntaxKind::EqualsEqualsToken;
        let is_loose =
            op == SyntaxKind::EqualsEqualsToken || op == SyntaxKind::ExclamationEqualsToken;

        let nullable_flags = if is_loose {
            TypeFlags::Undefined | TypeFlags::Null
        } else {
            TypeFlags::Undefined
        };
        let value_type = self.get_type_of_node(value_node);

        let value_is_nullable = self.type_contains_flags(&value_type, nullable_flags);
        let value_excludes_nullable = !value_is_nullable;
        let remove_nullable = if is_equality {
            (kind == NarrowKind::TrueBranch && value_excludes_nullable)
                || (kind == NarrowKind::FalseBranch && value_is_nullable)
        } else {
            (kind == NarrowKind::FalseBranch && value_excludes_nullable)
                || (kind == NarrowKind::TrueBranch && value_is_nullable)
        };
        if remove_nullable {
            self.remove_nullable_from_union(type_)
        } else {
            Arc::clone(type_)
        }
    }

    pub(crate) fn remove_nullable_from_union(&self, type_: &Arc<Type>) -> Arc<Type> {
        self.remove_flags_from_union(type_, TypeFlags::Undefined | TypeFlags::Null)
    }

    pub(crate) fn type_contains_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> bool {
        if type_.flags.intersects(flags) {
            return true;
        }
        if type_.is_union() {
            if let TypeData::Union(u) = &type_.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|t| t.flags.intersects(flags));
            }
        }
        false
    }
}
