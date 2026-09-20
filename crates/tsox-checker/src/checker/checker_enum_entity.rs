use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};
use tsox_frontend::evaluator::{EvalResult, EvalValue};

use crate::checker::checker::*;

impl Checker {
    // Go evaluateEntity：枚举初始化式内标识符/限定名/元素访问的常量求值
    pub(crate) fn evaluate_entity(
        &mut self,
        expr: &Arc<Node>,
        location: Option<&Arc<Node>>,
    ) -> EvalResult {
        match expr.kind {
            SyntaxKind::Identifier | SyntaxKind::PropertyAccessExpression => {
                let Some(mut symbol) = self.resolve_entity_for_enum(expr) else {
                    return EvalResult::none();
                };
                if symbol.flags == SymbolFlags::Alias
                    && let Some(resolved) = self.resolve_import_alias_target_symbol(&symbol)
                    && !Arc::ptr_eq(&resolved, &symbol)
                {
                    symbol = resolved;
                }
                if expr.kind == SyntaxKind::Identifier {
                    let text = expr.text();
                    if is_infinity_or_nan_text(&text)
                        && self
                            .globals
                            .get(text)
                            .is_some_and(|g| Arc::ptr_eq(g, &symbol))
                    {
                        let v = match text {
                            "NaN" => f64::NAN,
                            _ => f64::INFINITY,
                        };
                        return EvalResult::new(
                            Some(EvalValue::Number(tsox_core::jsnum::Number(v))),
                            false,
                            false,
                            false,
                        );
                    }
                }
                if symbol.flags.contains(SymbolFlags::EnumMember) {
                    return match location {
                        Some(loc) => self.evaluate_enum_member(expr, &symbol, loc),
                        None => symbol
                            .value_declaration
                            .as_ref()
                            .map(|d| self.get_enum_member_value(d))
                            .unwrap_or_else(EvalResult::none),
                    };
                }
                if self.is_constant_variable(&symbol)
                    && let Some(declaration) = symbol.value_declaration.clone()
                {
                    let init_ok = match &declaration.data {
                        NodeData::VariableDeclaration(d) => {
                            d.type_node.is_none() && d.initializer.is_some()
                        }
                        _ => false,
                    };
                    let use_ok = location.is_none_or(|loc| {
                        !Arc::ptr_eq(&declaration, loc)
                            && self.is_block_scoped_name_declared_before_use(&declaration, loc)
                    });
                    if init_ok && use_ok {
                        let initializer = match &declaration.data {
                            NodeData::VariableDeclaration(d) => {
                                d.initializer.as_ref().map(Arc::clone)
                            }
                            _ => None,
                        };
                        if let Some(initializer) = initializer {
                            let mut entity_fn =
                                |expr: &Arc<Node>, loc: Option<&Arc<Node>>| {
                                    self.evaluate_entity(expr, loc)
                                };
                            let result = tsox_frontend::evaluator::evaluate_expression(
                                &initializer,
                                Some(&declaration),
                                &mut entity_fn,
                            );
                            let cross_file = match (location, self.get_source_file_of_node(&declaration)) {
                                (Some(loc), Some(df)) => self
                                    .get_source_file_of_node(loc)
                                    .is_some_and(|uf| !Arc::ptr_eq(&uf, &df)),
                                _ => false,
                            };
                            if cross_file {
                                return EvalResult::new(result.value, false, true, true);
                            }
                            return EvalResult::new(
                                result.value,
                                result.is_syntactically_string,
                                result.resolved_other_files,
                                true,
                            );
                        }
                    }
                }
                EvalResult::none()
            }
            SyntaxKind::ElementAccessExpression => {
                let (root, argument) = match &expr.data {
                    NodeData::ElementAccessExpression(d) => (d.expression.clone(), d.argument_expression.clone()),
                    _ => return EvalResult::none(),
                };
                if !is_entity_name_expression(&root) || !is_string_literal_like(&argument) {
                    return EvalResult::none();
                }
                let Some(root_symbol) = self.resolve_entity_for_enum(&root) else {
                    return EvalResult::none();
                };
                if !root_symbol.flags.intersects(SymbolFlags::ENUM) {
                    return EvalResult::none();
                }
                let name = argument.text();
                let member = self
                    .member_of_enum_symbol(&root_symbol, &name);
                let Some(member) = member else {
                    return EvalResult::none();
                };
                match location {
                    Some(loc) => self.evaluate_enum_member(expr, &member, loc),
                    None => member
                        .value_declaration
                        .as_ref()
                        .map(|d| self.get_enum_member_value(d))
                        .unwrap_or_else(EvalResult::none),
                }
            }
            _ => EvalResult::none(),
        }
    }

    fn resolve_entity_for_enum(&mut self, expr: &Arc<Node>) -> Option<Arc<Symbol>> {
        match expr.kind {
            SyntaxKind::Identifier => {
                let name = expr.text();
                let mut cursor = expr.parent();
                while let Some(node) = cursor {
                    if node.kind == SyntaxKind::EnumDeclaration {
                        let symbol_map = self.program.symbol_map();
                        if let Some(enum_sym) = symbol_map.symbol_of(&node) {
                            if let Some(member) = self.member_of_enum_symbol(&enum_sym, &name) {
                                return Some(member);
                            }
                        }
                        break;
                    }
                    if node.kind == SyntaxKind::SourceFile {
                        break;
                    }
                    cursor = node.parent();
                }
                self.resolve_identifier_with_meaning(expr, SymbolFlags::VALUE)
            }
            _ => self.resolve_qualified_symbol_traced(expr).ok(),
        }
    }

    fn member_of_enum_symbol(
        &self,
        enum_symbol: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        enum_symbol
            .members
            .get(name)
            .or_else(|| enum_symbol.exports.get(name))
            .filter(|s| s.flags.contains(SymbolFlags::EnumMember))
            .cloned()
    }

    // Go evaluateEnumMember：先于赋值引用报 TS2565，后置成员引用报 TS18041
    fn evaluate_enum_member(
        &mut self,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
        location: &Arc<Node>,
    ) -> EvalResult {
        let Some(declaration) = symbol.value_declaration.clone() else {
            self.report_used_before_assignment(expr, symbol);
            return EvalResult::none();
        };
        if Arc::ptr_eq(&declaration, location) {
            self.report_used_before_assignment(expr, symbol);
            return EvalResult::none();
        }
        if !self.is_block_scoped_name_declared_before_use(&declaration, location) {
            let file = self.get_source_file_of_node(expr).or_else(|| self.current_file.clone());
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                expr.loc,
                tsox_core::diagnostics::messages_generated::
                    A_MEMBER_INITIALIZER_IN_A_ENUM_DECLARATION_CANNOT_REFERENCE_MEMBERS_DECLARED_AFTER_IT_INCLUDING_MEMBERS_DEFINED_IN_OTHER_ENUMS,
                vec![],
            ));
            return EvalResult::new(Some(EvalValue::Number(0.0.into())), false, false, false);
        }
        let value = self.get_enum_member_value(&declaration);
        let different_enum = location
            .parent()
            .zip(declaration.parent())
            .is_some_and(|(lp, dp)| lp.id() != dp.id());
        if different_enum {
            return EvalResult::new(
                value.value,
                value.is_syntactically_string,
                value.resolved_other_files,
                true,
            );
        }
        value
    }

    fn report_used_before_assignment(&mut self, expr: &Arc<Node>, symbol: &Arc<Symbol>) {
        let file = self.get_source_file_of_node(expr).or_else(|| self.current_file.clone());
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            expr.loc,
            tsox_core::diagnostics::messages_generated::PROPERTY_0_IS_USED_BEFORE_BEING_ASSIGNED,
            vec![symbol.name.clone()],
        ));
    }

    fn is_constant_variable(&self, symbol: &Arc<Symbol>) -> bool {
        symbol.flags.intersects(SymbolFlags::VARIABLE)
            && symbol
                .value_declaration
                .as_ref()
                .is_some_and(|d| {
                    self.root_declaration_flags(d)
                        .intersects(tsox_frontend::ast::NodeFlags::Constant)
                })
    }

    // Go getDeclarationNodeFlagsFromSymbol：根声明（VariableStatement）的标志
    fn root_declaration_flags(&self, declaration: &Arc<Node>) -> tsox_frontend::ast::NodeFlags {
        let mut root = Arc::clone(declaration);
        while let Some(parent) = root.parent() {
            match parent.kind {
                SyntaxKind::VariableDeclarationList
                | SyntaxKind::VariableDeclaration
                | SyntaxKind::BindingElement
                | SyntaxKind::ArrayBindingPattern
                | SyntaxKind::ObjectBindingPattern => root = parent,
                _ => break,
            }
        }
        root.flags
    }

    fn is_block_scoped_name_declared_before_use(
        &self,
        declaration: &Arc<Node>,
        usage: &Arc<Node>,
    ) -> bool {
        let decl_file = self.get_source_file_of_node(declaration);
        let use_file = self.get_source_file_of_node(usage);
        match (decl_file, use_file) {
            (Some(d), Some(u)) if !Arc::ptr_eq(&d, &u) => true,
            _ => declaration.loc.pos <= usage.loc.pos,
        }
    }
}

fn is_infinity_or_nan_text(text: &str) -> bool {
    matches!(text, "Infinity" | "-Infinity" | "NaN")
}

fn is_entity_name_expression(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::Identifier => true,
        SyntaxKind::PropertyAccessExpression => {
            matches!(&node.data, NodeData::PropertyAccessExpression(d) if is_entity_name_expression(&d.expression))
        }
        _ => false,
    }
}

fn is_string_literal_like(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
    )
}
