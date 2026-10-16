#![allow(unused_imports)]

use crate::checker::emitresolver::*;

impl Checker {
    pub fn is_optional_parameter(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_optional_parameter");
        let NodeData::ParameterDeclaration(data) = &node.data else {
            return false;
        };
        if data.question_token.is_some() {
            return true;
        }
        let Some(parent) = node.parent() else {
            return false;
        };
        if data.initializer.is_some() {
            let Some(signature) = self.get_signature_from_declaration(&parent) else {
                return false;
            };
            let parameter_index = parameter_declaration_index(&parent, node);
            return parameter_index
                >= self.get_min_argument_count_ex(
                    &signature,
                    crate::checker::relater_relation::MinArgumentCountFlags::StrongArityForUntypedJS
                        | crate::checker::relater_relation::MinArgumentCountFlags::VoidIsNonOptional,
                );
        }
        if let Some(iife) =
            tsox_frontend::ast::mig::m3e_4::get_immediately_invoked_function_expression(&parent)
        {
            let parameter_index = parameter_declaration_index(&parent, node);
            return data.type_node.is_none()
                && data.dot_dot_dot_token.is_none()
                && parameter_index >= self.get_effective_call_arguments(&iife).len();
        }
        false
    }

    pub fn is_literal_const_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_literal_const_declaration"); 
        if node.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let NodeData::VariableDeclaration(data) = &node.data else {
            return false;
        };

        if data.initializer.is_none() {
            return false;
        }
        let initializer = data.initializer.as_ref().unwrap();
        matches!(
            initializer.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::PrefixUnaryExpression
        )
    }

    pub fn get_constant_value(&mut self, node: &Arc<Node>) -> Option<String> { ::tsox_core::fntrace::enter("get_constant_value"); 
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value_string(node);
        }
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &node.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &node.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::TrueKeyword => Some("true".to_string()),
            SyntaxKind::FalseKeyword => Some("false".to_string()),
            SyntaxKind::NullKeyword => Some("null".to_string()),
            _ => None,
        }
    }

    pub fn is_referenced_alias_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_referenced_alias_declaration"); 
        if let Some(links) = self.declaration_links.get(node) {
            if links.is_visible.is_true() {
                return true;
            }
        }

        true
    }

    pub fn is_value_alias_declaration(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_value_alias_declaration"); 
        match &node.data {
            NodeData::ImportSpecifier(data) => !data.is_type_only,
            _ => true,
        }
    }

    pub fn get_symbol_of_declaration(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_of_declaration"); 
        self.program.symbol_map().symbol_of(node).cloned()
    }

    // Go getSymbolOfNode/getSymbolOfDeclaration（checker.go:15192-15199、
    // 15181-15188）：声明节点符号经 getLateBoundSymbol 揭示计算属性名成员、
    // 再 getMergedSymbol 归并跨文件同名声明（script 文件顶层声明与 lib 全局
    // 符合并）；节点侧符号缺其他文件声明集，getTypeOfNode 声明名臂
    //（GetSymbolOfNameOrPropertyAccessExpression → getSymbolOfNode）在合并
    // 符号上取型
    pub fn get_symbol_of_declaration_merged(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_of_declaration_merged"); 
        let symbol = self.program.symbol_map().symbol_of(node).cloned()?;
        let symbol = self.get_late_bound_symbol(&symbol);
        Some(self.get_merged_symbol(&symbol))
    }

    pub fn is_const_enum_member(&self, symbol: &Symbol) -> bool { ::tsox_core::fntrace::enter("is_const_enum_member"); 
        symbol.flags.contains(SymbolFlags::ConstEnum)
    }
}

pub(crate) fn parameter_declaration_index(parent: &Arc<Node>, parameter: &Arc<Node>) -> usize { ::tsox_core::fntrace::enter("parameter_declaration_index");
    tsox_frontend::ast::mig::m3b::parameters(parent)
        .iter()
        .position(|p| Arc::ptr_eq(p, parameter))
        .unwrap_or(usize::MAX)
}

pub(crate) fn collect_children(node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("collect_children"); 
    let mut children: Vec<Arc<Node>> = Vec::new();
    for_each_child(node, |child| {
        children.push(Arc::clone(child));
        false
    });
    children
}
