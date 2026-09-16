#![allow(unused_imports)]

use crate::checker::typenode_import_query::*;

use tsox_frontend::ast::SymbolFlags;

fn import_type_argument_text(argument: &Arc<Node>) -> Option<String> {
    let NodeData::LiteralTypeNode(lt) = &argument.data else {
        return None;
    };
    if lt.literal.kind != SyntaxKind::StringLiteral {
        return None;
    }
    Some(lt.literal.text().trim_matches(['"', '\'']).to_string())
}

fn identifier_chain(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::Identifier(_) => vec![Arc::clone(node)],
        NodeData::QualifiedName(d) => {
            let mut chain = identifier_chain(&d.left);
            chain.push(Arc::clone(&d.right));
            chain
        }
        _ => Vec::new(),
    }
}

fn chain_contains(chain: &[Arc<Node>], target: &Arc<Node>) -> bool {
    chain.iter().any(|n| Arc::ptr_eq(n, target))
}

impl Checker {
    /// Go resolveExternalModuleSymbol：模块符号的 export= 经 resolveSymbol
    /// 解出目标（export = Foo 时为 Foo 的符号）
    pub(crate) fn resolve_external_module_symbol_go(
        &self,
        module_symbol: &Arc<Symbol>,
    ) -> Arc<Symbol> {
        if let Some(ee) = module_symbol
            .exports
            .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
        {
            if let Some(target) = self.resolve_export_assignment_target(ee) {
                return self.get_merged_symbol(&target);
            }
            // js 的 export= 声明是 `module.exports = X` BinaryExpression，其
            // parent 链可能未接：目标类/typedef 直接在模块 exports 表
            // （Go resolveAlias 对 js export= alias 解析到右侧符号）
            if let Some(right) = ee.declarations.iter().find_map(|d| match &d.data {
                NodeData::BinaryExpression(be) => Some(Arc::clone(&be.right)),
                _ => None,
            }) && let Some(target) = self.resolve_entity_symbol_in(&right, module_symbol) {
                return self.get_merged_symbol(&target);
            }
        }
        Arc::clone(module_symbol)
    }

    fn resolve_entity_symbol_in(&self, expr: &Arc<Node>, module_symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let name = expr.text().to_string();
        module_symbol
            .exports
            .entries
            .get(&name)
            .cloned()
            .or_else(|| module_symbol.members.entries.get(&name).cloned())
    }

    fn resolve_export_assignment_target(&self, ee: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let decl = ee
            .declarations
            .iter()
            .find(|d| matches!(d.data, NodeData::ExportAssignment(_)))?;
        let NodeData::ExportAssignment(ea) = &decl.data else {
            return None;
        };
        if !matches!(
            ea.expression.kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName
        ) {
            return Some(Arc::clone(ee));
        }
        let sym_map = self.program.symbol_map();
        if let Some(target) = sym_map.symbol_of(&ea.expression) {
            return Some(Arc::clone(target));
        }
        let name = ea.expression.text().to_string();
        decl.parent()
            .as_ref()
            .and_then(|sf| sym_map.locals.get(&sf.id()))
            .and_then(|l| l.get(&name).cloned())
            .or_else(|| {
                decl.parent().and_then(|sf| {
                    let sf_sym = sym_map.symbol_of(&sf)?;
                    sf_sym
                        .members
                        .get(&name)
                        .cloned()
                        .or_else(|| sf_sym.exports.get(&name).cloned())
                })
            })
    }

    /// Go getTypeFromImportTypeNode：import("./m") / import("./m").Q 的类型解析
    pub(crate) fn get_type_from_import_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let placeholder = self.error_type();
        self.cache_type(node, placeholder.clone());

        let NodeData::ImportTypeNode(d) = &node.data else {
            return placeholder;
        };
        let resolution_mode = if let Some(attrs) = &d.attributes {
            let attrs = Arc::clone(attrs);
            self.get_resolution_mode_override(&attrs, true)
        } else {
            None
        }
        .unwrap_or(tsox_core::core::compiler_options::ModuleKind::None);
        let Some(argument_text) = import_type_argument_text(&d.argument) else {
            let file = self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                d.argument.loc,
                tsox_core::diagnostics::messages_generated::STRING_LITERAL_EXPECTED,
                Vec::new(),
            ));
            return placeholder;
        };
        let target_meaning = if d.is_type_of {
            SymbolFlags::VALUE
        } else {
            SymbolFlags::TYPE
        };
        let inner_module = self.resolve_import_type_module(&argument_text, node, resolution_mode);
        let Some(inner_module) = inner_module else {
            let file = self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone());
            let (message, args) =
                tsox_frontend::parser::cannot_resolve_module_error(&self.compiler_options, &argument_text);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                d.argument.loc,
                *message,
                args,
            ));
            return placeholder;
        };
        let module_symbol = self.resolve_external_module_symbol_go(&inner_module);
        let qualifier = d.qualifier.clone();
        let is_type_of = d.is_type_of;

        // Go NodeIsMissing(Qualifier)：尾部点的零宽 missing qualifier 视同无
        // qualifier（不参与链解析）
        let resolved: Option<Arc<Type>> = if let Some(qualifier) = qualifier.as_ref().filter(|q| q.pos() < q.end()) {
            let chain = identifier_chain(qualifier);
            match self.resolve_import_type_chain(
                &inner_module,
                &module_symbol,
                &chain,
                is_type_of,
                target_meaning,
                None,
            ) {
                Some(current) => self.resolve_import_symbol_type(&current, target_meaning),
                None => return placeholder,
            }
        } else {
            if self.get_symbol_flags(&module_symbol).intersects(target_meaning) {
                self.resolve_import_symbol_type(&module_symbol, target_meaning)
            } else {
                let file = self
                    .get_source_file_of_node(node)
                    .or_else(|| self.current_file.clone());
                let msg = if target_meaning == SymbolFlags::VALUE {
                    tsox_core::diagnostics::messages_generated::
                        MODULE_0_DOES_NOT_REFER_TO_A_VALUE_BUT_IS_USED_AS_A_VALUE_HERE
                } else {
                    tsox_core::diagnostics::messages_generated::
                        MODULE_0_DOES_NOT_REFER_TO_A_TYPE_BUT_IS_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_IMPORT_0
                };
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    msg,
                    vec![argument_text],
                ));
                None
            }
        };
        match resolved {
            Some(t) => {
                self.cache_type_overwrite_error(node, t.clone());
                t
            }
            None => placeholder,
        }
    }

    /// Go resolveImportSymbolType：Value 位给符号值类型（typeof 侧，
    /// class 为含静态成员的构造器侧），Type 位给类型引用语义的类型
    fn resolve_import_symbol_type(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Type>> {
        if meaning == SymbolFlags::VALUE {
            if symbol.flags.intersects(SymbolFlags::Class)
                && let Some(decl) = symbol
                    .declarations
                    .iter()
                    .find(|d| matches!(d.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression))
            {
                return Some(self.get_type_of_class_declaration(decl));
            }
            return Some(self.get_type_of_symbol(symbol));
        }
        let resolved = self.resolve_symbol(symbol);
        Some(self.get_type_of_symbol(&resolved))
    }

    /// 模块说明符解析：常规检查流程用 current_file，补全路径（current_file
    /// 为空）按 location 节点所属文件算相对目录，非相对说明符走 ambient 枚举。
    /// 非相对说明符先经 program 的 node_modules 解析（package.json exports/条件），
    /// 未命中再走 ambient/索引回退
    fn resolve_import_type_module(
        &self,
        specifier: &str,
        location: &Arc<Node>,
        resolution_mode: tsox_core::core::compiler_options::ModuleKind,
    ) -> Option<Arc<Symbol>> {
        let program_symbol = |containing: &str| -> Option<Arc<Symbol>> {
            self.program
                .resolve_external_module_path(specifier, containing, resolution_mode)
                .and_then(|path| {
                    let sf = self.program.get_source_file(&path)?;
                    self.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        if self.current_file.is_some() {
            return program_symbol(&self.current_file.as_ref().unwrap().file_name)
                .or_else(|| self.resolve_module_file_symbol(specifier));
        }
        let file = self.get_source_file_of_node(location);
        file.and_then(|f| {
            let dir = match f.file_name.rfind('/') {
                Some(i) => f.file_name[..i].to_string(),
                None => String::new(),
            };
            program_symbol(&f.file_name)
                .or_else(|| self.resolve_module_file_symbol_in(&dir, specifier))
        })
        .or_else(|| self.resolve_module_file_symbol(specifier))
    }

    /// qualifier 链逐段解析（Go getTypeFromImportTypeNode 链循环）。
    /// upto 为 Some 时解析到该段（含）即停（成员补全的点左段），None 走完整链
    fn resolve_import_type_chain(
        &mut self,
        _inner_module: &Arc<Symbol>,
        module_symbol: &Arc<Symbol>,
        chain: &[Arc<Node>],
        is_type_of: bool,
        target_meaning: SymbolFlags,
        upto: Option<&Arc<Node>>,
    ) -> Option<Arc<Symbol>> {
        let mut current_namespace = Arc::clone(module_symbol);
        let n = chain.len();
        for (i, segment) in chain.iter().enumerate() {
            let meaning = if i == n - 1 {
                target_meaning
            } else {
                SymbolFlags::NAMESPACE
            };
            let merged = self.get_merged_symbol(&self.resolve_symbol(&current_namespace));
            let name = segment.text().to_string();
            // is_type_of 臂走值属性（Go getPropertyOfTypeEx），否则 exports 按
            // meaning 查（Go getSymbol(getExportsOfSymbol, ...)）
            let next = if is_type_of {
                let t = self.get_type_of_symbol(&merged);
                self.get_property_of_type(&t, &name)
                    .filter(|s| s.flags.intersects(SymbolFlags::VALUE | SymbolFlags::NAMESPACE))
                    .or_else(|| {
                        self.get_exports_of_symbol(&merged)
                            .get(&name)
                            .filter(|s| s.flags.intersects(SymbolFlags::VALUE))
                            .cloned()
                    })
            } else {
                // Go getSymbol：取到符号后先 resolveSymbol 再验 meaning
                //（re-export 的 alias 符号需 follow 到 namespace/类型终点）
                self.get_exports_of_symbol(&merged)
                    .get(&name)
                    .map(|s| {
                        // Go resolveSymbol：alias 链循环展开到 meaning 命中
                        //（re-export 的 namespace import 解到模块符号）
                        let mut cur = self
                            .follow_alias(s)
                            .unwrap_or_else(|| Arc::clone(s));
                        for _ in 0..10 {
                            if cur.flags.intersects(meaning) {
                                break;
                            }
                            let next = self.resolve_alias_base(Arc::clone(&cur));
                            if std::sync::Arc::ptr_eq(&next, &cur) {
                                break;
                            }
                            cur = next;
                        }
                        cur
                    })
                    .filter(|s| s.flags.intersects(meaning))
                    .or_else(|| {
                        self.get_exports_of_symbol(&merged)
                            .get(&name)
                            .filter(|s| s.flags.intersects(SymbolFlags::NAMESPACE))
                            .cloned()
                    })
            };
            if std::env::var_os("TSOX_DEBUG_QN").is_some() {
                let raw = self
                    .get_exports_of_symbol(&merged)
                    .get(&name)
                    .cloned();
                eprintln!("[it-chain] seg={} raw={:?} next={:?}",
                    name,
                    raw.as_ref().map(|s| (s.flags, s.export_symbol.as_ref().map(|e| e.name.clone()))),
                    next.as_ref().map(|s| s.name.clone()));
            }
            let Some(next) = next else {
                let file = self
                    .get_source_file_of_node(segment)
                    .or_else(|| self.current_file.clone());
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    segment.loc,
                    tsox_core::diagnostics::messages_generated::
                        NAMESPACE_0_HAS_NO_EXPORTED_MEMBER_1,
                    vec![current_namespace.name.clone(), name],
                ));
                return None;
            };
            current_namespace = next;
            if let Some(stop) = upto
                && Arc::ptr_eq(stop, segment)
            {
                return Some(current_namespace);
            }
        }
        Some(current_namespace)
    }

    /// ImportType 语境成员补全基点：receiver 为 ImportTypeNode 时给经 export=
    /// 解析后的模块符号；为 qualifier 链上标识符时给解析到该段的符号。
    /// 第二个返回值是 is_type_of（值位成员过滤用）
    pub fn resolve_import_type_member_base(
        &mut self,
        receiver: &Arc<Node>,
    ) -> Option<(Arc<Symbol>, bool)> {
        let (import_node, upto, is_qualifier_segment) = match &receiver.data {
            NodeData::ImportTypeNode(_) => (Arc::clone(receiver), None, false),
            NodeData::Identifier(_) | NodeData::QualifiedName(_) => {
                let mut cur = receiver.parent()?;
                while matches!(cur.data, NodeData::QualifiedName(_)) {
                    cur = cur.parent()?;
                }
                let NodeData::ImportTypeNode(d) = &cur.data else {
                    return None;
                };
                let qualifier = d.qualifier.as_ref()?;
                // 限定名 receiver 的解析终点是其链尾段（点左最后一段）
                let stop = match &receiver.data {
                    NodeData::QualifiedName(q) => Arc::clone(&q.right),
                    _ => Arc::clone(receiver),
                };
                if !chain_contains(&identifier_chain(qualifier), &stop) {
                    return None;
                }
                (cur, Some(stop), true)
            }
            _ => return None,
        };
        let NodeData::ImportTypeNode(d) = &import_node.data else {
            return None;
        };
        let argument_text = import_type_argument_text(&d.argument)?;
        let is_type_of = d.is_type_of;
        let target_meaning = if is_type_of {
            SymbolFlags::VALUE
        } else {
            SymbolFlags::TYPE
        };
        // 补全路径下 current_file 为空：相对说明符按 ImportType 所属文件算目录
        let resolution_mode = d
            .attributes
            .as_ref()
            .and_then(|attrs| self.get_resolution_mode_override(attrs, false))
            .unwrap_or(tsox_core::core::compiler_options::ModuleKind::None);
        let inner_module =
            self.resolve_import_type_module(&argument_text, &import_node, resolution_mode);
        let Some(inner_module) = inner_module else {
            return None;
        };
        let module_symbol = self.resolve_external_module_symbol_go(&inner_module);
        if !is_qualifier_segment {
            return Some((module_symbol, is_type_of));
        }
        let Some(qualifier) = &d.qualifier else {
            return None;
        };
        let upto = upto?;
        let chain = identifier_chain(qualifier);
        let tail = self.resolve_import_type_chain(
            &inner_module,
            &module_symbol,
            &chain,
            is_type_of,
            target_meaning,
            Some(&upto),
        )?;
        Some((tail, is_type_of))
    }
}
