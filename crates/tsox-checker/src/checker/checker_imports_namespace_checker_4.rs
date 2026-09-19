#![allow(unused_imports)]

use crate::checker::checker_imports_namespace::*;

impl Checker {
    pub(crate) fn namespace_member_recursive(
        &mut self,
        namespace: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if let Some(s) = namespace
            .exports
            .get(name)
            .or_else(|| namespace.members.get(name))
        {
            return Some(Arc::clone(s));
        }
        for d in &namespace.declarations {
            if d.kind == SyntaxKind::ModuleDeclaration
                && let Some(s) = self
                    .program
                    .symbol_map()
                    .locals
                    .get(&d.id())
                    .and_then(|l| l.get(name))
            {
                return Some(Arc::clone(s));
            }
        }

        let export_equals = namespace.exports.get("export=")?;
        for d in &export_equals.declarations {
            let (expression, is_export_equals_form) = match &d.data {
                tsox_frontend::ast::NodeData::ExportAssignment(ea) => {
                    (Some(Arc::clone(&ea.expression)), ea.is_export_equals)
                }
                tsox_frontend::ast::NodeData::BinaryExpression(bin)
                    if bin.operator_token.kind == SyntaxKind::EqualsToken =>
                {
                    (Some(Arc::clone(&bin.right)), true)
                }
                _ => (None, false),
            };
            let Some(expression) = expression else {
                continue;
            };
            if is_export_equals_form {
                if let tsox_frontend::ast::NodeData::ObjectLiteralExpression(ol) = &expression.data
                {
                    for prop in ol.properties.iter() {
                        if prop.text() == name
                            && let Some(s) = self.program.symbol_map().symbol_of(prop)
                        {
                            return Some(Arc::clone(s));
                        }
                    }
                    continue;
                }
                if matches!(
                    expression.kind,
                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                ) {
                    let scope_decl = namespace
                        .declarations
                        .iter()
                        .find(|d| {
                            matches!(d.kind, SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile)
                        })
                        .cloned();
                    let target = scope_decl.and_then(|scope_decl| {
                        self.push_scope(&scope_decl);
                        let t = self.resolve_qualified_symbol(&expression);
                        self.pop_scope();
                        t
                    });
                    if let Some(mut target) = target {
                        for _ in 0..4 {
                            if target.flags.contains(SymbolFlags::ValueModule) {
                                break;
                            }
                            if target.flags != SymbolFlags::Alias {
                                break;
                            }
                            let next = target
                                .declarations
                                .iter()
                                .find(|d| d.kind == SyntaxKind::ImportEqualsDeclaration)
                                .and_then(|d| {
                                    if let tsox_frontend::ast::NodeData::ImportEqualsDeclaration(
                                        ied,
                                    ) = &d.data
                                        && matches!(
                                            ied.module_reference.kind,
                                            SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                        )
                                    {
                                        Some(self.resolve_qualified_symbol(&ied.module_reference))
                                    } else {
                                        None
                                    }
                                })
                                .flatten();
                            match next {
                                Some(n) => target = n,
                                None => break,
                            }
                        }
                        if target.flags.contains(SymbolFlags::ValueModule) {
                            return self.namespace_member_recursive(&target, name);
                        }
                        // export = <expr>：具名导入取 expr 类型的同名属性
                        //（Go getExternalModuleMember 的 export= 值属性；default
                        // 走 getTargetOfModuleDefault 的合成默认导出近似路径）
                        if name == "default" {
                            return Some(target);
                        }
                        let t = self.get_type_of_symbol(&target);
                        return self.get_property_of_type(&t, name);
                    }
                }
            }
        }
        None
    }

    pub(crate) fn namespace_full_path(symbol: &Arc<Symbol>) -> String {
        // Go getFullyQualifiedName：沿符号 parent 链拼点分限定名，根为模块
        // 文件符号时输出带引号 specifier（"mod".Ns 形态）
        if let Some(parent) = symbol.parent() {
            return format!("{}.{}", Self::namespace_full_path(&parent), symbol.name);
        }
        if symbol
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
        {
            return format!(
                "\"{}\"",
                crate::checker::nodebuilder::module_specifier_of_name(&symbol.name)
            );
        }
        symbol.name.clone()
    }
}
