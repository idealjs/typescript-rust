#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if node.kind == SyntaxKind::ThisKeyword {
            return self.compute_type_of_node(node);
        }

        if let Some(links) = self.type_node_links.get(node) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }
        let result = self.compute_type_of_node(node);
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }

    pub(crate) fn compute_type_of_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match node.kind {
            SyntaxKind::NumericLiteral => {
                if let crate::ast::NodeData::NumericLiteral(data) = &node.data {
                    let lit = self.infer_number_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.number_type()
            }
            SyntaxKind::StringLiteral => {
                if let crate::ast::NodeData::StringLiteral(data) = &node.data {
                    let lit = self.infer_string_literal_type(&data.text);
                    return self.get_fresh_type_of_literal_type(&lit);
                }
                self.string_type()
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => self.string_type(),
            SyntaxKind::TrueKeyword => self.get_fresh_type_of_literal_type(&self.true_type()),
            SyntaxKind::FalseKeyword => self.get_fresh_type_of_literal_type(&self.false_type()),
            SyntaxKind::NullKeyword => self.nullish_widening_type(self.null_type()),
            SyntaxKind::UndefinedKeyword => self.nullish_widening_type(self.undefined_type()),
            SyntaxKind::BigIntLiteral => self.get_fresh_type_of_literal_type(&self.bigint_type()),
            SyntaxKind::ArrayLiteralExpression => {
                return self.get_type_of_array_literal(node);
            }
            SyntaxKind::ObjectLiteralExpression => {
                return self.get_type_of_object_literal(node);
            }
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
                self.get_type_of_function_like(node)
            }
            SyntaxKind::FunctionDeclaration => self.get_type_of_function_like(node),
            SyntaxKind::Identifier => self.get_type_of_identifier(node),
            SyntaxKind::MetaProperty => self.get_type_of_meta_property(node),

            SyntaxKind::BinaryExpression => self.get_type_of_binary_expression(node),
            SyntaxKind::PrefixUnaryExpression => {
                if let crate::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    match data.operator {
                        SyntaxKind::ExclamationToken => return self.boolean_type(),

                        SyntaxKind::DeleteKeyword => return self.boolean_type(),

                        SyntaxKind::VoidKeyword => return self.undefined_type(),

                        _ => return self.number_type(),
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::PostfixUnaryExpression => self.number_type(),
            SyntaxKind::CallExpression => self.get_return_type_of_call_expression(node),
            SyntaxKind::NewExpression => self.get_return_type_of_new_expression(node),
            SyntaxKind::PropertyAccessExpression => self.get_type_of_property_access(node),
            SyntaxKind::ElementAccessExpression => self.get_type_of_element_access(node),
            SyntaxKind::ParenthesizedExpression => {
                if let crate::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::AsExpression => {
                if let crate::ast::NodeData::AsExpression(data) = &node.data {
                    if data.type_node.kind == SyntaxKind::ConstKeyword {
                        return self.get_const_assertion_type(&data.expression);
                    }
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::SatisfiesExpression => {
                if let crate::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    return self.get_type_of_node(&data.expression);
                }
                self.get_any_type()
            }
            SyntaxKind::TypeAssertionExpression => {
                if let crate::ast::NodeData::TypeAssertion(data) = &node.data {
                    return self.get_type_from_type_node(&data.type_node);
                }
                self.get_any_type()
            }
            SyntaxKind::NonNullExpression => {
                if let crate::ast::NodeData::NonNullExpression(data) = &node.data {
                    let operand_type = self.get_type_of_node(&data.expression);
                    return self.remove_flags_from_union(
                        &operand_type,
                        TypeFlags::Undefined | TypeFlags::Null,
                    );
                }
                self.get_any_type()
            }
            SyntaxKind::ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(data) = &node.data {
                    let true_type = self.get_type_of_node(&data.when_true);
                    let false_type = self.get_type_of_node(&data.when_false);
                    let true_widened = self.get_widened_type_of_literal(&true_type);
                    let false_widened = self.get_widened_type_of_literal(&false_type);
                    return self.get_union_type(vec![true_widened, false_widened]);
                }
                self.get_any_type()
            }
            SyntaxKind::TemplateExpression => self.string_type(),
            SyntaxKind::TaggedTemplateExpression => {
                if let crate::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    let tag_type = self.get_type_of_node(&data.tag);
                    if let Some(structured) = tag_type.as_structured() {
                        for sig in structured.call_signatures() {
                            if let Some(rt) = self.get_return_type_of_signature(sig) {
                                return rt;
                            }
                            return self.get_any_type();
                        }
                    }
                }
                self.get_any_type()
            }
            SyntaxKind::DeleteExpression => self.boolean_type(),
            SyntaxKind::VoidExpression => self.undefined_type(),
            SyntaxKind::AwaitExpression => {
                if let crate::ast::NodeData::AwaitExpression(data) = &node.data {
                    let operand = Arc::clone(&data.expression);

                    if let Some(ns) = self.type_of_dynamic_import(&operand) {
                        return ns;
                    }
                    let operand_type = self.get_type_of_node(&operand);
                    return match self.get_awaited_type(&operand_type) {
                        Some(awaited) => awaited,
                        None => operand_type,
                    };
                }
                self.get_any_type()
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                if node.kind == SyntaxKind::SuperKeyword
                    && self.super_in_computed_name_of_innermost_class(node)
                    && self.enclosing_class_stack.len() >= 2
                {
                    return self
                        .this_type_stack
                        .get(self.this_type_stack.len() - 2)
                        .cloned()
                        .unwrap_or_else(|| self.get_any_type());
                }

                if self.this_container_stack.last() == Some(&ThisContainerKind::StaticMember)
                    && let Some(class) = self.enclosing_class_stack.last().cloned()
                {
                    return self.get_type_of_class_declaration(&class);
                }
                let r = self
                    .this_type_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(|| self.get_any_type());
                r
            }
            _ => self.get_any_type(),
        }
    }

    pub(crate) fn get_type_of_identifier(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(symbol) = self.resolve_identifier(node) {
            if symbol.flags == SymbolFlags::Alias {
                if let Some(t) = self.type_of_imported_symbol(&symbol) {
                    return t;
                }
            }
            let flow = self.program.symbol_map().flow_node_of(node).map(Arc::clone);
            let narrowed = self.get_narrowed_type_of_symbol(&symbol, flow.as_ref());

            if narrowed.object_flags.contains(ObjectFlags::EvolvingArray)
                && self.is_evolving_array_operation_target(node)
            {
                return self.auto_array_type();
            }

            let final_type = self.finalize_evolving_array_type(&narrowed);

            let target_kind = get_assignment_target_kind(node);
            let compound_like =
                target_kind == AssignmentKind::Definite && is_in_compound_like_assignment(node);
            if compound_like || target_kind == AssignmentKind::Compound {
                return self.get_base_type_of_literal_type(&final_type);
            }
            final_type
        } else {
            self.get_any_type()
        }
    }

    pub(crate) fn is_evolving_array_operation_target(&self, node: &Arc<Node>) -> bool {
        let root = self.get_reference_root(node);
        let Some(parent) = &root.parent else {
            return false;
        };

        if let NodeData::PropertyAccessExpression(pa) = &parent.data {
            if Arc::ptr_eq(&pa.expression, root) {
                let name = pa.name.text();
                if name == "length" {
                    return true;
                }
                if name == "push" || name == "unshift" {
                    if let Some(grandparent) = &parent.parent {
                        if matches!(grandparent.kind, SyntaxKind::CallExpression) {
                            return true;
                        }
                    }
                }
            }
        }

        if let NodeData::ElementAccessExpression(ea) = &parent.data {
            if Arc::ptr_eq(&ea.expression, root) {
                if let Some(grandparent) = &parent.parent {
                    if let NodeData::BinaryExpression(bin) = &grandparent.data {
                        if bin.operator_token.kind == SyntaxKind::EqualsToken
                            && Arc::ptr_eq(&bin.left, parent)
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub(crate) fn get_reference_root<'a>(&self, node: &'a Arc<Node>) -> &'a Arc<Node> {
        let Some(parent) = &node.parent else {
            return node;
        };
        let recurse = match &parent.data {
            NodeData::ParenthesizedExpression(_) => true,
            NodeData::BinaryExpression(bin) => {
                (bin.operator_token.kind == SyntaxKind::EqualsToken && Arc::ptr_eq(&bin.left, node))
                    || (bin.operator_token.kind == SyntaxKind::CommaToken
                        && Arc::ptr_eq(&bin.right, node))
            }
            _ => false,
        };
        if recurse {
            self.get_reference_root(parent)
        } else {
            node
        }
    }
}
