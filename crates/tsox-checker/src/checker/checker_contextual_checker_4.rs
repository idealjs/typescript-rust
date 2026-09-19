#![allow(unused_imports)]

use crate::checker::checker_contextual::*;

impl Checker {
    pub(crate) fn check_block_scoped_variable_used_before_declaration(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {
        if self.is_declared_as_plain_var(symbol) {
            return;
        }

        if self.has_const_enum_declarations(symbol) {
            return;
        }

        if self.is_used_in_type_position(node) {
            return;
        }

        if !symbol
            .flags
            .intersects(SymbolFlags::BlockScopedVariable | SymbolFlags::Class | SymbolFlags::ENUM)
        {
            return;
        }

        let declaration_for_scope = symbol.declarations.iter().find(|d| {
            matches!(
                d.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::BindingElement
                    | SyntaxKind::EnumDeclaration
            )
        });
        // 类装饰器用法在 Go 里走 isBlockScopedNameDeclaredBeforeUse 的类分支，
        // 不进入 isUsedInFunctionOrInstanceProperty 豁免，先行短路
        let class_like_declaration = declaration_for_scope.is_some_and(|d| {
            matches!(
                d.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            )
        });
        let class_decorator_or_computed = class_like_declaration
            && self.usage_in_class_computed_name_or_decorator(
                declaration_for_scope.unwrap(),
                node,
            );
        if !class_decorator_or_computed && self.is_scope_exempt(node, declaration_for_scope) {
            return;
        }

        let declaration = symbol.declarations.iter().find(|d| {
            matches!(
                d.kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::ClassDeclaration
                    | SyntaxKind::BindingElement
                    | SyntaxKind::EnumDeclaration
            )
        });
        let Some(declaration) = declaration else {
            return;
        };

        if declaration.kind == SyntaxKind::VariableDeclaration
            && !is_let_or_const_declaration(declaration)
        {
            return;
        }

        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
        {
            return;
        }

        let decl_name_pos = match &declaration.data {
            tsox_frontend::ast::NodeData::VariableDeclaration(d) => d.name.pos(),
            tsox_frontend::ast::NodeData::BindingElement(d) => d
                .name
                .as_ref()
                .map(|n| n.pos())
                .unwrap_or(declaration.pos()),
            _ => declaration.pos(),
        };
        // 类节点 pos 含装饰器：用法在该类计算名/装饰器内时不算声明先于使用（Go 类分支）
        if !class_decorator_or_computed && decl_name_pos <= node.pos() {
            let inside_own_initializer = {
                let mut cur = declaration.parent();
                let mut found = false;
                while let Some(a) = cur {
                    if matches!(&a.data, tsox_frontend::ast::NodeData::VariableDeclaration(vdd)
                        if vdd.initializer.as_ref().is_some_and(|init| init.loc.contains(node.loc.pos())))
                    {
                        found = true;
                        break;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::BindingElement
                            | SyntaxKind::ArrayBindingPattern
                            | SyntaxKind::ObjectBindingPattern
                    ) {
                        cur = a.parent();
                        continue;
                    }
                    break;
                }
                found
            };
            if !inside_own_initializer {
                return;
            }
        }

        let decl_file = self.get_source_file_of_node(declaration);
        let use_file = self.get_source_file_of_node(node);
        if let (Some(df), Some(uf)) = (&decl_file, &use_file) {
            if df.file_name != uf.file_name {
                return;
            }
        }
        let file = self.current_file.clone();

        let message = if symbol.flags.contains(SymbolFlags::Class) {
            tsox_core::diagnostics::messages_generated::CLASS_0_USED_BEFORE_ITS_DECLARATION
        } else if symbol.flags.intersects(SymbolFlags::RegularEnum)
            || (symbol.flags.intersects(SymbolFlags::ConstEnum)
                && self.compiler_options.isolated_modules.is_true())
        {
            tsox_core::diagnostics::messages_generated::ENUM_0_USED_BEFORE_ITS_DECLARATION
        } else {
            BLOCK_SCOPED_VARIABLE_0_USED_BEFORE_ITS_DECLARATION
        };
        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == message.code && d.loc == node.loc);
        if !already {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                message,
                vec![name.to_string()],
            ));
        }
    }
}
