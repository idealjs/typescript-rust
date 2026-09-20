#![allow(unused_imports)]

use crate::checker::checker_contextual::*;
use crate::checker::types::{SignatureKind, Type, TypeData, TypeFlags};
use tsox_frontend::ast::NodeData;

fn is_logical_or_coalescing(node: &Arc<Node>) -> bool {
    matches!(
        &node.data,
        NodeData::BinaryExpression(b)
            if matches!(
                b.operator_token.kind,
                SyntaxKind::AmpersandAmpersandToken
                    | SyntaxKind::BarBarToken
                    | SyntaxKind::QuestionQuestionToken
            )
    )
}

impl Checker {
    pub(crate) fn check_testing_known_truthy_callable_or_awaitable(
        &mut self,
        cond_expr: &Arc<Node>,
        cond_type: &Arc<Type>,
        body: Option<&Arc<Node>>,
    ) {
        if !self.strict_null_checks {
            return;
        }
        let cond = Self::skip_parentheses(cond_expr);
        self.check_truthy_callable_chain(&cond, cond_type, body);
    }

    fn check_truthy_callable_chain(
        &mut self,
        cond_expr: &Arc<Node>,
        cond_type: &Arc<Type>,
        body: Option<&Arc<Node>>,
    ) {
        let mut cur = Self::skip_parentheses(cond_expr);
        self.check_testing_known_truthy_type(&cur, cond_type, body);
        while let NodeData::BinaryExpression(b) = &cur.data {
            match b.operator_token.kind {
                SyntaxKind::BarBarToken | SyntaxKind::QuestionQuestionToken => {
                    cur = Self::skip_parentheses(&b.left);
                    self.check_testing_known_truthy_type(&cur, cond_type, body);
                }
                _ => break,
            }
        }
    }

    fn check_testing_known_truthy_type(
        &mut self,
        cond_expr: &Arc<Node>,
        cond_type: &Arc<Type>,
        body: Option<&Arc<Node>>,
    ) {
        let mut location = Arc::clone(cond_expr);
        if is_logical_or_coalescing(&location)
            && let NodeData::BinaryExpression(b) = &location.data
        {
            location = Self::skip_parentheses(&b.right);
        }
        if crate::binder::bind_js_assignment_declarations::is_module_exports_access_expression(
            &location,
        ) {
            return;
        }
        if is_logical_or_coalescing(&location) {
            self.check_truthy_callable_chain(&location, cond_type, body);
            return;
        }
        let mut t = Arc::clone(cond_type);
        if !Arc::ptr_eq(&location, cond_expr) {
            t = self.get_type_of_node(&location);
        }
        let property_expression_cast = matches!(
            &location.data,
            NodeData::PropertyAccessExpression(pa)
                if matches!(
                    Self::skip_parentheses(&pa.expression).kind,
                    SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression
                )
        );
        if !self.type_has_truthy_fact(&t) || property_expression_cast {
            return;
        }
        let call_signatures = self.get_signatures_of_type(&t, SignatureKind::Call);
        let is_promise = self.get_promised_type_of_promise(&t).is_some();
        if call_signatures.is_empty() && !is_promise {
            return;
        }
        let tested_node: Option<Arc<Node>> = match &location.data {
            NodeData::Identifier(_) => Some(Arc::clone(&location)),
            NodeData::PropertyAccessExpression(pa) => Some(Arc::clone(&pa.name)),
            _ => None,
        };
        let tested_symbol = tested_node
            .as_ref()
            .and_then(|n| self.truthy_symbol_at_location(n));
        if tested_symbol.is_none() && !is_promise {
            return;
        }
        if tested_symbol
            .as_ref()
            .is_some_and(|s| s.flags.contains(SymbolFlags::Optional))
        {
            return;
        }
        let mut is_used = false;
        if let (Some(sym), Some(parent)) = (tested_symbol.as_ref(), cond_expr.parent()) {
            if parent.kind == SyntaxKind::BinaryExpression
                && self.symbol_used_in_binary_chain(&parent, sym)
            {
                is_used = true;
            }
        }
        if !is_used
            && let (Some(sym), Some(body)) = (tested_symbol.as_ref(), body)
            && let Some(tn) = tested_node.as_ref()
        {
            is_used = self.symbol_used_in_condition_body(cond_expr, body, tn, sym);
        }
        if !is_used {
            let file = self.current_file.clone();
            if is_promise {
                let name = self.get_type_name_for_error_display(&t);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    location.loc,
                    THIS_CONDITION_WILL_ALWAYS_RETURN_TRUE_SINCE_THIS_0_IS_ALWAYS_DEFINED,
                    vec![name],
                ));
            } else {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    location.loc,
                    THIS_CONDITION_WILL_ALWAYS_RETURN_TRUE_SINCE_THIS_FUNCTION_IS_ALWAYS_DEFINED_DID_YOU_MEAN_TO_CALL_IT_INSTEAD,
                    Vec::new(),
                ));
            }
        }
    }

    fn type_has_truthy_fact(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .any(|c| self.type_has_truthy_fact(c));
            }
            return true;
        }
        !t.flags.intersects(
            TypeFlags::Undefined | TypeFlags::Null | TypeFlags::Void | TypeFlags::Never,
        )
    }

    fn truthy_symbol_at_location(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let is_property_name = node.parent().as_ref().is_some_and(|p| {
            matches!(&p.data, NodeData::PropertyAccessExpression(pa) if Arc::ptr_eq(&pa.name, node))
        });
        if is_property_name {
            return self.resolve_property_access_symbol(node);
        }
        if let NodeData::Identifier(id) = &node.data {
            return self.resolve_name_in_enclosure(node, &id.text, SymbolFlags::VALUE);
        }
        self.program.symbol_map().symbol_of(node).map(Arc::clone)
    }

    fn collect_identifiers(node: &Arc<Node>, out: &mut Vec<(Arc<Node>, Arc<Node>)>) {
        if node.kind == SyntaxKind::Identifier {
            let parent = node.parent().unwrap_or_else(|| Arc::clone(node));
            out.push((Arc::clone(node), parent));
        }
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            Self::collect_identifiers(child, out);
            false
        });
    }

    fn symbol_used_in_binary_chain(&mut self, node: &Arc<Node>, tested: &Arc<Symbol>) -> bool {
        let mut node = Arc::clone(node);
        loop {
            let NodeData::BinaryExpression(b) = &node.data else {
                return false;
            };
            if b.operator_token.kind != SyntaxKind::AmpersandAmpersandToken {
                return false;
            }
            let mut idents = Vec::new();
            Self::collect_identifiers(&b.right, &mut idents);
            for (id, _) in &idents {
                if self
                    .truthy_symbol_at_location(id)
                    .is_some_and(|s| s.id() == tested.id())
                {
                    return true;
                }
            }
            let Some(parent) = node.parent() else {
                return false;
            };
            node = parent;
        }
    }

    fn symbol_used_in_condition_body(
        &mut self,
        expr: &Arc<Node>,
        body: &Arc<Node>,
        tested_node: &Arc<Node>,
        tested: &Arc<Symbol>,
    ) -> bool {
        let simple_identifier_test = expr.kind == SyntaxKind::Identifier || {
            tested_node.kind == SyntaxKind::Identifier
                && tested_node
                    .parent()
                    .is_some_and(|p| p.kind == SyntaxKind::BinaryExpression)
        };
        let mut idents = Vec::new();
        Self::collect_identifiers(body, &mut idents);
        for (child, child_parent) in &idents {
            if !self
                .truthy_symbol_at_location(child)
                .is_some_and(|s| s.id() == tested.id())
            {
                continue;
            }
            if simple_identifier_test {
                return true;
            }
            let mut tested_expr = tested_node.parent();
            let mut child_expr = Some(Arc::clone(child_parent));
            while let (Some(te), Some(ce)) = (tested_expr, child_expr) {
                if te.kind == SyntaxKind::ThisKeyword && ce.kind == SyntaxKind::ThisKeyword {
                    return true;
                }
                match (&te.data, &ce.data) {
                    (NodeData::Identifier(_), NodeData::Identifier(_)) => {
                        return match (
                            self.truthy_symbol_at_location(&te),
                            self.truthy_symbol_at_location(&ce),
                        ) {
                            (Some(a), Some(b)) => a.id() == b.id(),
                            _ => false,
                        };
                    }
                    (
                        NodeData::PropertyAccessExpression(tp),
                        NodeData::PropertyAccessExpression(cp),
                    ) => {
                        let same = match (
                            self.truthy_symbol_at_location(&tp.name),
                            self.truthy_symbol_at_location(&cp.name),
                        ) {
                            (Some(a), Some(b)) => a.id() == b.id(),
                            _ => false,
                        };
                        if !same {
                            return false;
                        }
                        child_expr = Some(Arc::clone(&cp.expression));
                        tested_expr = Some(Arc::clone(&tp.expression));
                    }
                    (NodeData::CallExpression(tc), NodeData::CallExpression(cc)) => {
                        child_expr = Some(Arc::clone(&cc.expression));
                        tested_expr = Some(Arc::clone(&tc.expression));
                    }
                    _ => return false,
                }
            }
        }
        false
    }
}
