use std::sync::Arc;

use crate::ast::{FlowNode, Node, NodeData, Symbol, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::types::*;

use super::clauses_of_range;

use super::FlowRef;

use super::NarrowKind;

impl Checker {
    pub(crate) fn try_narrow_by_discriminant_property(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Option<Arc<Type>> {

        let (symbol, node_reference): (Option<Arc<Symbol>>, Option<Arc<Node>>) = match target {
            FlowRef::Symbol(symbol) => (Some(Arc::clone(symbol)), None),
            FlowRef::Node(reference) => (None, Some(Arc::clone(reference))),
        };
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return None;
        };
        let op = bin.operator_token.kind;

        let is_strict_eq = op == SyntaxKind::EqualsEqualsEqualsToken
            || op == SyntaxKind::ExclamationEqualsEqualsToken;
        if !is_strict_eq {
            return None;
        }

        let (access_node, value_node) = if let Some(symbol) = &symbol {
            if let Some(alias) = self.discriminant_alias_access(&bin.left, symbol) {
                (alias, &bin.right)
            } else if let Some(alias) = self.discriminant_alias_access(&bin.right, symbol) {
                (alias, &bin.left)
            } else if self.is_property_access_on_symbol(&bin.left, symbol) {
                (Arc::clone(&bin.left), &bin.right)
            } else if self.is_property_access_on_symbol(&bin.right, symbol) {
                (Arc::clone(&bin.right), &bin.left)
            } else {
                return None;
            }
        } else if let Some(reference) = node_reference.as_ref() {
            if self.is_property_access_on_reference(&bin.left, reference) {
                (Arc::clone(&bin.left), &bin.right)
            } else if self.is_property_access_on_reference(&bin.right, reference) {
                (Arc::clone(&bin.right), &bin.left)
            } else {
                return None;
            }
        } else {
            unreachable!()
        };
        let prop_name = Self::get_accessed_property_name_from_node(&access_node)?;
        let value_type = self.get_type_of_node(value_node);
        let is_equality = op == SyntaxKind::EqualsEqualsEqualsToken;
        let keep_matching = if is_equality {
            kind == NarrowKind::TrueBranch
        } else {
            kind == NarrowKind::FalseBranch
        };

        if !type_.is_union() {
            let Some(prop_type) = self.get_property_type_of_type(type_, &prop_name) else {
                return Some(Arc::clone(type_));
            };
            if prop_type.flags.contains(TypeFlags::Any) {
                return Some(Arc::clone(type_));
            };

            let could_equal = self.is_type_assignable_to(&prop_type, &value_type)
                || self.is_type_assignable_to(&value_type, &prop_type);
            if keep_matching {
                return Some(if could_equal {
                    Arc::clone(type_)
                } else {
                    self.never_type()
                });
            }

            if value_type.flags.intersects(TYPE_FLAGS_UNIT)
                && self.is_type_assignable_to(&prop_type, &value_type)
            {
                return Some(self.never_type());
            }
            return Some(Arc::clone(type_));
        }
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);

                if prop_type
                    .as_ref()
                    .is_some_and(|pt| pt.flags.contains(TypeFlags::Never))
                {
                    return false;
                }
                if keep_matching {

                    prop_type
                        .map(|pt| {
                            self.is_type_assignable_to(&pt, &value_type)
                                || self.is_type_assignable_to(&value_type, &pt)
                        })
                        .unwrap_or(false)
                } else {

                    prop_type
                        .map(|pt| !self.is_type_assignable_to(&pt, &value_type))
                        .unwrap_or(true)
                }
            })
            .collect();
        Some(self.rebuild_union_or_never(type_, filtered))
    }

    pub(crate) fn try_narrow_by_typeof_discriminant(
        &mut self,
        type_: &Arc<Type>,
        typeof_expr: &Arc<Node>,
        type_name_node: &Arc<Node>,
        target: &FlowRef,
        narrow_to_value: bool,
    ) -> Option<Arc<Type>> {
        let FlowRef::Symbol(symbol) = target else {
            return None;
        };
        let NodeData::TypeOfExpression(typeof_data) = &typeof_expr.data else {
            return None;
        };
        let target = &typeof_data.expression;

        let owned = match self.discriminant_alias_access(target, symbol) {
            Some(alias) => alias,
            None if self.is_property_access_on_symbol(target, symbol) => Arc::clone(target),
            None => return None,
        };
        let prop_name = Self::get_accessed_property_name_from_node(&owned)?;

        if !type_.is_union() {
            return Some(Arc::clone(type_));
        }
        let type_name = match &type_name_node.data {
            NodeData::StringLiteral(data) => data.text.as_str(),
            _ => return None,
        };
        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let prop_type = self.get_property_type_of_type(t, &prop_name);
                let Some(prop_type) = prop_type else {
                    return false;
                };
                if narrow_to_value {

                    self.type_matches_typeof_any(&prop_type, type_name)
                } else {

                    !self.type_matches_typeof_all(&prop_type, type_name)
                }
            })
            .collect();
        Some(self.rebuild_union_or_never(type_, filtered))
    }

    fn type_matches_typeof_any(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        constituents
            .iter()
            .any(|c| self.constituent_matches_typeof(c, type_name))
    }

    fn type_matches_typeof_all(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        !constituents.is_empty()
            && constituents
                .iter()
                .all(|c| self.constituent_matches_typeof(c, type_name))
    }

    fn constituent_matches_typeof(&self, t: &Arc<Type>, type_name: &str) -> bool {
        match type_name {
            "string" => t.flags.intersects(TYPE_FLAGS_STRING_LIKE),
            "number" => t.flags.intersects(TYPE_FLAGS_NUMBER_LIKE),
            "boolean" => t.flags.intersects(TYPE_FLAGS_BOOLEAN_LIKE),
            "bigint" => t.flags.intersects(TYPE_FLAGS_BIG_INT_LIKE),
            "symbol" => t.flags.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE),
            "undefined" => t.flags.contains(TypeFlags::Undefined),
            "function" => !self
                .get_signatures_of_type(t, SignatureKind::Call)
                .is_empty(),
            "object" => t.flags.contains(TypeFlags::Object) || t.flags.contains(TypeFlags::Null),
            _ => false,
        }
    }

    pub(crate) fn narrow_by_switch_clause(
        &mut self,
        type_: &Arc<Type>,
        flow: &Arc<FlowNode>,
        target: &FlowRef,
    ) -> Arc<Type> {
        let Some(switch_stmt) = &flow.switch_statement else {
            return Arc::clone(type_);
        };
        let NodeData::SwitchStatement(switch_data) = &switch_stmt.data else {
            return Arc::clone(type_);
        };
        let discriminant = &switch_data.expression;

        let (clause_start, clause_end) = match flow.clause_range {
            Some(r) => r,
            None => {

                match (&flow.node, switch_stmt) {
                    (Some(clause), _)
                        if let NodeData::SwitchStatement(sd) = &switch_stmt.data
                            && let NodeData::CaseBlock(cb) = &sd.case_block.data
                            && let Some(idx) = cb
                                .clauses
                                .nodes
                                .iter()
                                .position(|c| Arc::ptr_eq(c, clause)) =>
                    {
                        (idx, idx + 1)
                    }
                    _ => (0, 0),
                }
            }
        };
        let range = (clause_start, clause_end);

        if self.expr_matches_target(discriminant, target) {
            return self.narrow_by_switch_on_discriminant(type_, switch_stmt, range);
        }

        if let FlowRef::Symbol(symbol) = target {
        if let Some(access) = self
            .discriminant_alias_access(discriminant, symbol)
            .or_else(|| {
                self.is_property_access_on_symbol(discriminant, symbol)
                    .then(|| Arc::clone(discriminant))
            })
        {
            return self.narrow_by_switch_on_discriminant_property(
                type_,
                switch_stmt,
                range,
                &access,
            );
        }
        }

        if discriminant.kind == SyntaxKind::TypeOfExpression {
            if let NodeData::TypeOfExpression(typeof_data) = &discriminant.data {
                if self.expr_matches_target(&typeof_data.expression, target) {
                    return self.narrow_by_switch_on_typeof(type_, switch_stmt, range);
                }
            }
        }

        if discriminant.kind == SyntaxKind::TrueKeyword {
            return self.narrow_by_switch_on_true(type_, switch_stmt, range, target);
        }

        Arc::clone(type_)
    }

    fn narrow_by_switch_on_true(
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

    fn narrow_by_switch_on_typeof(
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
                .filter(|t| {
                    !outside_implied.iter().any(|it| self.types_overlap(t, it))
                })
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

    fn get_switch_clause_typeof_witnesses(
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

    fn typeof_string_to_type(&mut self, text: &str) -> Arc<Type> {
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

    fn literal_text_of(&self, node: &Arc<Node>) -> Option<String> {
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

    fn narrow_by_switch_on_discriminant(
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
                NodeData::CaseOrDefaultClause(cd) => {
                    Some(self.get_type_of_node(&cd.expression))
                }
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

    fn narrow_by_switch_on_discriminant_property(
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
                NodeData::CaseOrDefaultClause(cd) => {
                    Some(self.get_type_of_node(&cd.expression))
                }
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

    fn get_switch_clause_types(&mut self, switch_stmt: &Arc<Node>) -> Vec<Arc<Type>> {
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

    pub(crate) fn optional_chain_contains_target(&self, source: &Arc<Node>, target: &FlowRef) -> bool {
        let symbol = match target {
            FlowRef::Symbol(symbol) => symbol,
            FlowRef::Node(reference) => {
                return self.optional_chain_contains_reference(source, reference)
            }
        };
        self.optional_chain_contains_symbol(source, symbol)
    }

    fn optional_chain_contains_symbol(&self, source: &Arc<Node>, symbol: &Arc<Symbol>) -> bool {
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

    fn optional_chain_contains_reference(
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

    fn type_contains_flags(&self, type_: &Arc<Type>, flags: TypeFlags) -> bool {
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
