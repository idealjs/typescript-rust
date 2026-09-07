#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn narrow_by_switch_on_true(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
        target: &FlowRef,
    ) -> Arc<Type> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Arc::clone(type_);
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return Arc::clone(type_);
        };
        let clauses = &case_block.clauses.nodes;

        let has_default = clause_start == clause_end
            || clauses[clause_start..clause_end]
                .iter()
                .any(|c| c.kind == SyntaxKind::DefaultClause);

        let narrow_away = |checker: &mut Self, t: &Arc<Type>, clauses: &[Arc<Node>]| {
            let mut t = Arc::clone(t);
            for clause in clauses {
                if clause.kind == SyntaxKind::CaseClause
                    && let NodeData::CaseOrDefaultClause(cd) = &clause.data
                {
                    t = checker.narrow_by_expression(
                        &t,
                        &cd.expression,
                        target,
                        NarrowKind::FalseBranch,
                        0,
                    );
                }
            }
            t
        };

        let mut t = narrow_away(self, type_, &clauses[..clause_start.min(clauses.len())]);

        if has_default {
            let end = clause_end.min(clauses.len());
            if end < clauses.len() {
                t = narrow_away(self, &t, &clauses[end..]);
            }
            return t;
        }

        let mut parts: Vec<Arc<Type>> = Vec::new();
        for clause in &clauses[clause_start..clause_end.min(clauses.len())] {
            if clause.kind == SyntaxKind::CaseClause
                && let NodeData::CaseOrDefaultClause(cd) = &clause.data
            {
                let narrowed = self.narrow_by_expression(
                    &t,
                    &cd.expression,
                    target,
                    NarrowKind::TrueBranch,
                    0,
                );
                if !parts.iter().any(|p| Arc::ptr_eq(p, &narrowed)) {
                    parts.push(narrowed);
                }
            }
        }
        if parts.is_empty() {
            return t;
        }
        if parts.len() == 1 {
            return parts.into_iter().next().expect("exactly one");
        }
        self.get_union_type(parts)
    }

    pub(crate) fn narrow_by_switch_on_typeof(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
    ) -> Arc<Type> {
        let witnesses = self.get_switch_clause_typeof_witnesses(switch_stmt);
        let Some(witnesses) = witnesses else {
            return Arc::clone(type_);
        };
        let start = clause_start.min(witnesses.len());
        let end = clause_end.min(witnesses.len());
        let has_default = clause_start == clause_end
            || clauses_of_range(switch_stmt, clause_start, clause_end)
                .iter()
                .any(|c| c.kind == SyntaxKind::DefaultClause);
        if has_default {
            let mut outside_implied: Vec<Arc<Type>> = Vec::new();
            for (i, w) in witnesses.iter().enumerate() {
                if (i < start || i >= end) && !w.is_empty() {
                    outside_implied.push(self.typeof_string_to_type(w));
                }
            }
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !outside_implied.iter().any(|it| self.types_overlap(t, it)))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }

        let group_witnesses: Vec<(String, Arc<Type>)> = witnesses[start..end]
            .iter()
            .filter(|w| !w.is_empty())
            .map(|w| (w.clone(), self.typeof_string_to_type(w)))
            .collect();
        if group_witnesses.is_empty() {
            return Arc::clone(type_);
        }

        if type_.is_union() {
            let constituents = self.constituent_types(type_);
            let matching: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| {
                    group_witnesses.iter().any(|(text, implied)| {
                        if text == "function" {
                            return self.types_overlap(t, implied)
                                && !self
                                    .get_signatures_of_type(t, SignatureKind::Call)
                                    .is_empty();
                        }
                        self.types_overlap(t, implied)
                    })
                })
                .collect();
            return self.rebuild_union_or_never(type_, matching);
        }

        let overlapped: Vec<Arc<Type>> = group_witnesses
            .iter()
            .filter(|(_, implied)| self.types_overlap(type_, implied))
            .map(|(_, implied)| Arc::clone(implied))
            .collect();
        if overlapped.is_empty() {
            return self.never_type();
        }
        if overlapped.len() == 1 {
            let implied = overlapped.into_iter().next().expect("exactly one");
            if self.is_type_assignable_to(type_, &implied) {
                return Arc::clone(type_);
            }
            return implied;
        }
        self.get_union_type(overlapped)
    }

    pub(crate) fn get_switch_clause_typeof_witnesses(
        &mut self,
        switch_stmt: &Arc<Node>,
    ) -> Option<Vec<String>> {
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return None;
        };
        let NodeData::CaseBlock(case_block) = &switch_data.case_block.data else {
            return None;
        };
        let mut witnesses = Vec::with_capacity(case_block.clauses.len());
        for clause in &case_block.clauses.nodes {
            if clause.kind == SyntaxKind::CaseClause {
                if let NodeData::CaseOrDefaultClause(cd) = &clause.data {
                    let text = self.literal_text_of(&cd.expression);
                    match text {
                        Some(t) => witnesses.push(t),
                        None => return None,
                    }
                } else {
                    witnesses.push(String::new());
                }
            } else {
                witnesses.push(String::new());
            }
        }
        Some(witnesses)
    }

    pub(crate) fn typeof_string_to_type(&mut self, text: &str) -> Arc<Type> {
        match text {
            "string" => self.string_type(),
            "number" => self.number_type(),
            "bigint" => self.bigint_type(),
            "boolean" => self.boolean_type(),
            "symbol" => self.es_symbol_type(),
            "undefined" => self.undefined_type(),
            "object" => {
                let non_primitive = self.non_primitive_type();
                let null = self.null_type();
                self.get_union_type(vec![non_primitive, null])
            }
            "function" => {
                if let Some(f) = self.any_function_type.get() {
                    Arc::clone(f)
                } else {
                    self.any_type()
                }
            }
            _ => self.non_primitive_type(),
        }
    }

    pub(crate) fn literal_text_of(&self, node: &Arc<Node>) -> Option<String> {
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(data) = &node.data {
                    Some(data.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                if let NodeData::NoSubstitutionTemplateLiteral(data) = &node.data {
                    Some(data.text.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn narrow_by_switch_on_discriminant(
        &mut self,
        type_: &Arc<Type>,
        switch_stmt: &Arc<Node>,
        (clause_start, clause_end): (usize, usize),
    ) -> Arc<Type> {
        let case_types = self.get_switch_clause_types(switch_stmt);
        let group_clauses = clauses_of_range(switch_stmt, clause_start, clause_end);

        let group_case_types: Vec<Arc<Type>> = group_clauses
            .iter()
            .filter(|c| c.kind == SyntaxKind::CaseClause)
            .filter_map(|c| match &c.data {
                NodeData::CaseOrDefaultClause(cd) => Some(self.get_type_of_node(&cd.expression)),
                _ => None,
            })
            .collect();
        if group_case_types.is_empty() {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !case_types.iter().any(|ct| self.types_overlap(t, ct)))
                .collect();
            return self.rebuild_union_or_never(type_, remaining);
        }
        let group_union = if group_case_types.len() == 1 {
            group_case_types.into_iter().next().expect("exactly one")
        } else {
            self.get_union_type(group_case_types)
        };

        let case_part = self.intersect_or_narrow(type_, &group_union);

        let has_default_in_group = group_clauses
            .iter()
            .any(|c| c.kind == SyntaxKind::DefaultClause);
        if has_default_in_group {
            let constituents = self.constituent_types(type_);
            let remaining: Vec<Arc<Type>> = constituents
                .into_iter()
                .filter(|t| !case_types.iter().any(|ct| self.types_overlap(t, ct)))
                .collect();
            let default_part = self.rebuild_union_or_never(type_, remaining);
            return self.get_union_type(vec![case_part, default_part]);
        }
        case_part
    }
}
