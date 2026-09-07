#![allow(unused_imports)]

use super::*;

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

    pub(crate) fn type_matches_typeof_any(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        constituents
            .iter()
            .any(|c| self.constituent_matches_typeof(c, type_name))
    }

    pub(crate) fn type_matches_typeof_all(&self, t: &Arc<Type>, type_name: &str) -> bool {
        let constituents = self.constituent_types(t);
        !constituents.is_empty()
            && constituents
                .iter()
                .all(|c| self.constituent_matches_typeof(c, type_name))
    }

    pub(crate) fn constituent_matches_typeof(&self, t: &Arc<Type>, type_name: &str) -> bool {
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
            None => match (&flow.node, switch_stmt) {
                (Some(clause), _)
                    if let NodeData::SwitchStatement(sd) = &switch_stmt.data
                        && let NodeData::CaseBlock(cb) = &sd.case_block.data
                        && let Some(idx) =
                            cb.clauses.nodes.iter().position(|c| Arc::ptr_eq(c, clause)) =>
                {
                    (idx, idx + 1)
                }
                _ => (0, 0),
            },
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
}
