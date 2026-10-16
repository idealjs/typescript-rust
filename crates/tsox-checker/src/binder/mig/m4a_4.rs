use crate::binder::nameresolver::NameResolver;
use crate::binder::nameresolver_get_local_symbol_for_export_default::node_symbol;
use crate::binder::*;
use crate::binder::referenceresolver_resolver_impl::ReferenceResolverImpl;
use std::sync::Arc;
use super::r19k5_ext::SymbolFlagsExt;
use tsox_frontend::ast::mig::m3f_3::is_alias_symbol_declaration;
use tsox_frontend::ast::mig::m3g::{is_logical_expression, is_logical_or_coalescing_assignment_expression};
use tsox_frontend::ast::mig::m3b::is_type_only;
use tsox_frontend::ast::mig::m3b::postfix_token as node_postfix_token;
use tsox_frontend::ast::mig::w7a::is_expando_initializer;
use tsox_frontend::ast::*;
use tsox_frontend::scanner::mig::m3i::get_source_text_of_node_from_source_file;

impl ReferenceResolverImpl {
    pub(crate) fn get_declaration_of_alias_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_declaration_of_alias_symbol"); 
        symbol
            .declarations
            .iter()
            .rev()
            .find(|d| is_alias_symbol_declaration(d))
            .cloned()
    }

    pub(crate) fn get_parent_of_symbol(&self, symbol: Option<&Arc<Symbol>>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_parent_of_symbol"); 
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_parent_of_symbol_fn {
                return callback(symbol);
            }
            return symbol.parent();
        }
        None
    }

    pub(crate) fn get_referenced_value_symbol(
        &mut self,
        reference: &Arc<Node>,
        start_in_declaration_container: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_referenced_value_symbol"); 
        if let Some(resolved_symbol) = self.get_resolved_symbol(Some(reference)) {
            return Some(resolved_symbol);
        }
        let mut location = Arc::clone(reference);
        if start_in_declaration_container
            && let Some(parent) = reference.parent()
            && is_declaration(&parent)
            && parent.name().is_some_and(|n| Arc::ptr_eq(&n, reference))
        {
            location = crate::checker::checker_checker_checker::Checker::get_declaration_container(&parent)
                .unwrap_or_else(|| Arc::clone(&parent));
        }
        let meaning =
            SymbolFlags::ExportValue | SymbolFlags::Value | SymbolFlags::Alias;
        if let Some(resolve_name) = &self.hooks.resolve_name_fn {
            return resolve_name(&location, reference.text(), meaning, None, false, false);
        }
        if self.resolver.is_none() {
            let mut resolver = NameResolver::new();
            resolver.compiler_options = self.options.clone();
            self.resolver = Some(resolver);
        }
        self.resolver.as_mut().unwrap().resolve(
            &location,
            reference.text(),
            meaning,
            None,
            false,
            false,
        )
    }

    pub(crate) fn is_type_only_alias_declaration(&self, symbol: Option<&Arc<Symbol>>) -> bool { ::tsox_core::fntrace::enter("is_type_only_alias_declaration"); 
        if let Some(symbol) = symbol {
            if let Some(callback) = &self.hooks.get_type_only_alias_declaration_fn {
                return callback(symbol, SymbolFlags::Value).is_some();
            }
            let mut node = self.get_declaration_of_alias_symbol(symbol);
            while let Some(current) = node {
                match current.kind {
                    SyntaxKind::ImportEqualsDeclaration | SyntaxKind::ExportDeclaration => {
                        return is_type_only(&current);
                    }
                    SyntaxKind::ImportClause
                    | SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportSpecifier => {
                        if is_type_only(&current) {
                            return true;
                        }
                        node = current.parent();
                        continue;
                    }
                    SyntaxKind::NamedImports | SyntaxKind::NamedExports => {
                        node = current.parent();
                        continue;
                    }
                    _ => break,
                }
            }
        }
        false
    }
}

pub(crate) fn get_initializer_symbol(symbol: Option<&Arc<Symbol>>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_initializer_symbol"); 
    let symbol = symbol?;
    let declaration = symbol.value_declaration.as_ref()?;
    if is_function_declaration(declaration)
        || is_in_js_file(declaration) && is_class_declaration(declaration)
    {
        return Some(Arc::clone(symbol));
    }
    if is_variable_declaration(declaration) {
        let is_const = declaration
            .parent()
            .is_some_and(|p| p.flags.contains(NodeFlags::Const));
        if is_const || is_in_js_file(declaration) {
            let initializer = declaration.initializer();
            if let Some(init) = initializer
                && is_expando_initializer(declaration, Some(init.as_ref()))
            {
                return node_symbol(init);
            }
        }
    }
    if is_binary_expression(declaration) && is_in_js_file(declaration) {
        if let NodeData::BinaryExpression(expr) = &declaration.data {
            let initializer = &expr.right;
            if is_expando_initializer(declaration, Some(initializer.as_ref())) {
                return node_symbol(initializer);
            }
        }
    }
    None
}

pub(crate) fn get_optional_symbol_flag_for_node(node: &Arc<Node>) -> SymbolFlags { ::tsox_core::fntrace::enter("get_optional_symbol_flag_for_node"); 
    match node_postfix_token(node) {
        Some(token) if token.kind == SyntaxKind::QuestionToken => SymbolFlags::Optional,
        _ => SymbolFlags::empty(),
    }
}

pub(crate) fn get_parent_of_property_assignment(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_parent_of_property_assignment"); 
    match node.kind {
        SyntaxKind::BinaryExpression => match &node.data {
            NodeData::BinaryExpression(expr) => expr.left.expression().cloned(),
            _ => None,
        },
        SyntaxKind::CallExpression => node.arguments().and_then(|a| a.nodes.first().cloned()),
        _ => panic!("Unhandled case in getParentOfPropertyAssignment"),
    }
}

pub(crate) fn is_assignment_declaration(decl: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_assignment_declaration"); 
    is_binary_expression(decl)
        || is_access_expression(decl)
        || is_identifier(decl)
        || is_call_expression(decl)
}

pub(crate) fn is_effective_module_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_effective_module_declaration"); 
    is_module_declaration(node) || is_identifier(node)
}

pub(crate) fn is_eval_or_arguments_identifier(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_eval_or_arguments_identifier"); 
    is_identifier(node) && (node.text() == "eval" || node.text() == "arguments")
}

pub(crate) fn is_function_symbol(symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_function_symbol"); 
    if let Some(d) = &symbol.value_declaration {
        if is_function_declaration(d) {
            return true;
        }
        if is_variable_declaration(d)
            && let NodeData::VariableDeclaration(var_decl) = &d.data
            && let Some(initializer) = &var_decl.initializer
        {
            return is_function_like(initializer);
        }
    }
    false
}

pub(crate) fn is_generator_function_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_generator_function_expression"); 
    is_function_expression(node)
        && matches!(
            &node.data,
            NodeData::FunctionExpression(data) if data.asterisk_token.is_some()
        )
}

pub(crate) fn is_logical_assignment_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_logical_assignment_expression"); 
    is_logical_or_coalescing_assignment_expression(&skip_parentheses(node))
}

pub(crate) fn is_signed_numeric_literal(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_signed_numeric_literal"); 
    if node.kind == SyntaxKind::PrefixUnaryExpression {
        if let NodeData::PrefixUnaryExpression(expr) = &node.data {
            return matches!(expr.operator, SyntaxKind::PlusToken | SyntaxKind::MinusToken)
                && is_numeric_literal(&expr.operand);
        }
    }
    false
}

pub(crate) fn is_statement_condition(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_statement_condition"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::IfStatement | SyntaxKind::WhileStatement | SyntaxKind::DoStatement => {
            parent.expression().is_some_and(|e| Arc::ptr_eq(&e, node))
        }
        SyntaxKind::ForStatement => matches!(
            &parent.data,
            NodeData::ForStatement(data) if data.condition.as_ref().is_some_and(|c| Arc::ptr_eq(c, node))
        ),
        SyntaxKind::ConditionalExpression => matches!(
            &parent.data,
            NodeData::ConditionalExpression(data) if Arc::ptr_eq(&data.condition, node)
        ),
        _ => false,
    }
}

pub(crate) fn is_top_level_logical_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_top_level_logical_expression"); 
    let mut node = Arc::clone(node);
    loop {
        let Some(parent) = node.parent() else {
            break;
        };
        let ascend = is_parenthesized_expression(&parent)
            || is_prefix_unary_expression(&parent)
                && matches!(
                    &parent.data,
                    NodeData::PrefixUnaryExpression(data) if data.operator == SyntaxKind::ExclamationToken
                );
        if !ascend {
            break;
        }
        node = parent;
    }
    let Some(parent) = node.parent() else {
        return !is_statement_condition(&node);
    };
    !is_statement_condition(&node)
        && !is_logical_expression(&parent)
        && !(is_optional_chain(&parent) && parent.expression().is_some_and(|e| Arc::ptr_eq(&e, &node)))
}

pub(crate) fn is_use_strict_prologue_directive(
    source_file: &Arc<SourceFile>,
    node: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_use_strict_prologue_directive"); 
    let Some(expression) = node.expression() else {
        return false;
    };
    let node_text = get_source_text_of_node_from_source_file(source_file, &expression, false);
    node_text == "\"use strict\"" || node_text == "'use strict'"
}
