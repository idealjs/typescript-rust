#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::checker_iteration::IterationUse;

impl Checker {
    pub(crate) fn check_destructuring_assignment(
        &mut self,
        node: &Arc<Node>,
        source_type: &Arc<Type>,
    ) {
        self.check_destructuring_assignment_ex(node, source_type, false);
    }

    pub(crate) fn check_destructuring_assignment_ex(
        &mut self,
        node: &Arc<Node>,
        source_type: &Arc<Type>,
        right_is_this: bool,
    ) {
        let mut target = Arc::clone(node);
        let mut source_type = Arc::clone(source_type);
        let mut shorthand_initializer: Option<Arc<Node>> = None;
        if let NodeData::ShorthandPropertyAssignment(data) = &node.data {
            if let Some(initializer) = &data.object_assignment_initializer {
                self.check_expression(initializer);
                let init_type = self.get_type_of_node(initializer);
                if self.strict_null_checks
                    && !self.type_is_or_contains_undefined(&init_type)
                {
                    source_type = self.remove_undefined_from_union(&source_type);
                }
                shorthand_initializer = Some(Arc::clone(initializer));
            }
            target = Arc::clone(&data.name);
        }
        if let NodeData::BinaryExpression(bin) = &target.data
            && bin.operator_token.kind == SyntaxKind::EqualsToken
        {
            self.check_binary_expression(&target);
            target = Arc::clone(&bin.left);
            if self.strict_null_checks {
                source_type = self.remove_undefined_from_union(&source_type);
            }
        }
        match target.kind {
            SyntaxKind::ObjectLiteralExpression => {
                self.check_object_literal_assignment(&target, &source_type, right_is_this);
            }
            SyntaxKind::ArrayLiteralExpression => {
                self.check_array_literal_assignment(&target, &source_type);
            }
            _ => {
                self.check_reference_assignment(&target, &source_type);
            }
        }
        // 简写绑定默认值 { x = expr }：expr 另行对目标声明型比较
        //（Go 对默认表达式带目标上下文型检查；与源比较先后同基线顺序）
        if let Some(initializer) = shorthand_initializer {
            let target_type = self.get_type_of_node(&target);
            let init_type = self.get_type_of_node(&initializer);
            if !self.is_type_assignable_to(&init_type, &target_type) {
                self.check_type_assignable_to_and_optionally_elaborate(
                    &init_type,
                    &target_type,
                    Some(&target),
                    Some(&target),
                    None,
                    None,
                );
            }
        }
    }

    fn check_object_literal_assignment(
        &mut self,
        node: &Arc<Node>,
        source_type: &Arc<Type>,
        right_is_this: bool,
    ) {
        let NodeData::ObjectLiteralExpression(data) = &node.data else {
            return;
        };
        for property in data.properties.iter() {
            match &property.data {
                NodeData::PropertyAssignment(_) | NodeData::ShorthandPropertyAssignment(_) => {
                    let name = property
                        .name()
                        .map(|n| self.property_assignment_name(n))
                        .unwrap_or_default();
                    if let Some(prop) = self.get_property_of_type(source_type, &name) {
                        self.mark_property_as_referenced_ex(
                            &prop,
                            Some(property),
                            Some(right_is_this),
                        );
                    }
                    let element_type = self.get_property_type_of_type(source_type, &name)
                        .unwrap_or_else(|| self.get_any_type());
                    let expr = match &property.data {
                        NodeData::PropertyAssignment(pa) => Arc::clone(&pa.initializer),
                        _ => Arc::clone(&property),
                    };
                    self.check_destructuring_assignment(&expr, &element_type);
                }
                NodeData::SpreadAssignment(_) | NodeData::SpreadElement(_) => {
                    let last = Arc::ptr_eq(
                        property,
                        data.properties
                            .nodes
                            .last()
                            .expect("property within list"),
                    );
                    if !last {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            property.loc,
                            tsox_core::diagnostics::messages_generated::
                                A_REST_ELEMENT_MUST_BE_LAST_IN_A_DESTRUCTURING_PATTERN,
                            Vec::new(),
                        ));
                    } else if let Some(inner) = property.expression() {
                        self.check_destructuring_assignment(&inner, source_type);
                    }
                }
                _ => {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        property.loc,
                        tsox_core::diagnostics::messages_generated::PROPERTY_ASSIGNMENT_EXPECTED,
                        Vec::new(),
                    ));
                }
            }
        }
    }

    fn check_array_literal_assignment(&mut self, node: &Arc<Node>, source_type: &Arc<Type>) {
        let NodeData::ArrayLiteralExpression(data) = &node.data else {
            return;
        };
        let possibly_out_of_bounds = self.check_iterated_type_or_element_type(
            IterationUse::Destructuring,
            source_type,
            Some(node),
        );
        for (index, _element) in data.elements.iter().enumerate() {
            let element_type = possibly_out_of_bounds.clone();
            self.check_array_literal_destructuring_element(
                node,
                source_type,
                index,
                &element_type,
            );
        }
    }

    fn check_array_literal_destructuring_element(
        &mut self,
        node: &Arc<Node>,
        source_type: &Arc<Type>,
        element_index: usize,
        element_type: &Arc<Type>,
    ) {
        let NodeData::ArrayLiteralExpression(data) = &node.data else {
            return;
        };
        let Some(element) = data.elements.nodes.get(element_index) else {
            return;
        };
        if element.kind == SyntaxKind::OmittedExpression {
            return;
        }
        if !matches!(&element.data, NodeData::SpreadElement(_)) {
            if self.is_array_like_type(source_type) {
                let indexed = if self.is_tuple_type(source_type) {
                    self.get_tuple_element_type(source_type, element_index)
                } else {
                    self.get_number_index_type(source_type)
                };
                let mut assigned = indexed.unwrap_or_else(|| self.get_any_type());
                if self.element_has_default_value(element) {
                    assigned = self.remove_undefined_from_union(&assigned);
                }
                self.check_destructuring_assignment(element, &assigned);
            } else {
                self.check_destructuring_assignment(element, element_type);
            }
            return;
        }
        let last = element_index + 1 == data.elements.nodes.len();
        if !last {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                element.loc,
                tsox_core::diagnostics::messages_generated::
                    A_REST_ELEMENT_MUST_BE_LAST_IN_A_DESTRUCTURING_PATTERN,
                Vec::new(),
            ));
            return;
        }
        let Some(rest_expression) = element.expression() else {
            return;
        };
        if let NodeData::BinaryExpression(bin) = &rest_expression.data
            && bin.operator_token.kind == SyntaxKind::EqualsToken
        {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                bin.operator_token.loc,
                tsox_core::diagnostics::messages_generated::
                    A_REST_ELEMENT_CANNOT_HAVE_AN_INITIALIZER,
                Vec::new(),
            ));
            return;
        }
        let rest_type = if self.is_tuple_type(source_type) {
            let mut rest: Vec<Arc<Type>> = Vec::new();
            let mut index = element_index;
            while let Some(t) = self.get_tuple_element_type(source_type, index) {
                rest.push(t);
                index += 1;
            }
            let element_t = if rest.is_empty() {
                self.never_type()
            } else {
                self.get_union_type(rest)
            };
            self.create_array_type(element_t)
        } else {
            self.create_array_type(Arc::clone(element_type))
        };
        self.check_destructuring_assignment(&rest_expression, &rest_type);
    }

    fn check_reference_assignment(&mut self, target: &Arc<Node>, source_type: &Arc<Type>) {
        self.check_expression(target);
        let mut node = Arc::clone(target);
        while node.kind == SyntaxKind::ParenthesizedExpression
            && let Some(inner) = node.expression()
        {
            node = Arc::clone(inner);
        }
        if node.kind != SyntaxKind::Identifier && !tsox_frontend::ast::is_access_expression(&node) {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                target.loc,
                tsox_core::diagnostics::messages_generated::
                    THE_LEFT_HAND_SIDE_OF_AN_ASSIGNMENT_EXPRESSION_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
                Vec::new(),
            ));
            return;
        }
        let target_type = self.get_type_of_node(target);
        self.check_type_assignable_to_and_optionally_elaborate(
            source_type,
            &target_type,
            Some(target),
            Some(target),
            None,
            None,
        );
    }

    fn property_assignment_name(&self, name: &Arc<Node>) -> String {
        match &name.data {
            NodeData::StringLiteral(s) => s.text.clone(),
            NodeData::NumericLiteral(n) => n.text.clone(),
            _ => name.text().to_string(),
        }
    }

    fn element_has_default_value(&self, element: &Arc<Node>) -> bool {
        matches!(&element.data, NodeData::BinaryExpression(bin)
            if bin.operator_token.kind == SyntaxKind::EqualsToken)
    }

    fn type_is_or_contains_undefined(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Undefined) {
            return true;
        }
        t.types()
            .is_some_and(|parts| {
                parts.iter().any(|p| p.flags.contains(TypeFlags::Undefined))
            })
    }

    pub(crate) fn remove_undefined_from_union(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if !t.is_union() {
            return Arc::clone(t);
        }
        let Some(parts) = t.types() else {
            return Arc::clone(t);
        };
        let kept: Vec<Arc<Type>> = parts
            .iter()
            .filter(|p| !p.flags.contains(TypeFlags::Undefined))
            .cloned()
            .collect();
        if kept.len() == parts.len() {
            return Arc::clone(t);
        }
        self.remove_flags_from_union(t, TypeFlags::Undefined)
    }
}
