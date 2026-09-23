#![allow(unused_imports)]

use crate::checker::checker_imports_namespace::*;

impl Checker {
    /// Go resolveESModuleSymbol：namespace import 的模块类型先揭示 export=
    /// 目标（resolveExternalModuleSymbol），目标带调用/构造签名或已有 default
    /// 时合成 {default: 目标} 包装并克隆符号承接显示身份（class → typeof Foo，
    /// function 无驻留签名 → 成员展开 { default: () => any; }），否则直接取
    /// 目标类型（interface 等纯类型目标 → any，值对象/模块 → 原成员表）
    pub(crate) fn namespace_import_module_type(&mut self, module_sym: &Arc<Symbol>) -> Arc<Type> {
        let export_equals = module_sym
            .exports
            .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
            .cloned();
        let base = match export_equals {
            Some(eq) => {
                let resolved = if eq.flags.contains(SymbolFlags::Alias) {
                    self.resolve_alias_base(eq)
                } else {
                    eq
                };
                if Arc::ptr_eq(&resolved, module_sym) {
                    Arc::clone(module_sym)
                } else {
                    resolved
                }
            }
            None => Arc::clone(module_sym),
        };
        let typ = self.get_type_of_symbol(&base);
        let has_signatures = typ.as_structured().is_some_and(|s| {
            !s.call_signatures().is_empty() || !s.construct_signatures().is_empty()
        });
        if !has_signatures && self.get_property_of_type(&typ, "default").is_none() {
            return typ;
        }
        let mut wrapper_members = SymbolTable::new();
        let mut wrapper_props: Vec<Arc<Symbol>> = Vec::new();
        if let Some(st) = typ.as_structured() {
            for p in &st.properties {
                wrapper_members.insert(p.name.clone(), Arc::clone(p));
                wrapper_props.push(Arc::clone(p));
            }
        }
        let mut default_alias = Symbol::new(SymbolFlags::Alias, "default");
        default_alias.set_parent(module_sym);
        let default_alias = Arc::new(default_alias);
        self.alias_symbol_links.insert(
            &default_alias,
            crate::checker::types::AliasSymbolLinks {
                alias_target: Some(Arc::clone(&base)),
                ..Default::default()
            },
        );
        wrapper_members.insert("default".to_string(), Arc::clone(&default_alias));
        wrapper_props.push(default_alias);
        let mut clone = Symbol::new(
            if base.flags.contains(SymbolFlags::Class) {
                base.flags
            } else {
                SymbolFlags::TypeLiteral
            },
            base.name.clone(),
        );
        clone.declarations = base.declarations.clone();
        clone.value_declaration = base.value_declaration.clone();
        clone.members = base.members.clone();
        clone.exports = base.exports.clone();
        if base.flags.contains(SymbolFlags::Class) && let Some(parent) = base.parent() {
            clone.set_parent(&parent);
        }
        let clone = Arc::new(clone);
        let construct_sigs = if base.flags.contains(SymbolFlags::Class) {
            typ
                .as_structured()
                .map(|s| s.construct_signatures().to_vec())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(&clone)),
            alias: None,
            data: crate::checker::types::TypeData::Object(
                crate::checker::types::ObjectTypeData {
                    structured: crate::checker::types::StructuredTypeData {
                        members: wrapper_members,
                        properties: wrapper_props,
                        signatures: construct_sigs,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
        });
        self.value_symbol_links.insert(
            &clone,
            crate::checker::types::ValueSymbolLinks {
                resolved_type: Some(Arc::clone(&result)),
                target: Some(Arc::clone(&base)),
                ..Default::default()
            },
        );
        result
    }

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
        // Go getExportsOfSymbol：限定名成员查找只看 exports 表，
        // 非导出局部（如 namespace 内无 export 的 import=）外部不可见

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

    pub(crate) fn namespace_full_path(&self, symbol: &Arc<Symbol>) -> String {
        // Go getFullyQualifiedName：沿符号 parent 链拼点分限定名；模块文件
        // 符号输出带引号 specifier（"mod".Ns），脚本文件无文件符号父级
        if let Some(parent) = symbol.parent() {
            if self.file_symbol_kind(&parent) != FileSymbolKind::Script {
                return format!("{}.{}", self.namespace_full_path(&parent), symbol.name);
            }
            return symbol.name.clone();
        }
        match self.file_symbol_kind(symbol) {
            FileSymbolKind::Module => format!(
                "\"{}\"",
                crate::checker::nodebuilder::module_specifier_of_name(&symbol.name)
            ),
            _ => symbol.name.clone(),
        }
    }

    fn file_symbol_kind(&self, symbol: &Arc<Symbol>) -> FileSymbolKind {
        let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
        else {
            return FileSymbolKind::Other;
        };
        let Some(sf) = self.get_source_file_of_node(decl) else {
            return FileSymbolKind::Other;
        };
        if sf.external_module_indicator.is_some() || sf.common_js_module_indicator.is_some() {
            FileSymbolKind::Module
        } else {
            FileSymbolKind::Script
        }
    }
}

#[derive(PartialEq)]
enum FileSymbolKind {
    Module,
    Script,
    Other,
}
