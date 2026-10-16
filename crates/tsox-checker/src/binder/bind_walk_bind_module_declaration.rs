#![allow(unused_imports)]

use crate::binder::bind_walk::*;

impl Binder {
    pub(crate) fn bind_module_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("bind_module_declaration"); 
        self.set_export_context_flag(node);
        self.bind_module_declaration_inner(node)
    }

    /// Go setExportContextFlag：ambient 模块且无 export 声明时是隐式导出语境
    /// （declare namespace 内未加 export 的声明自动入 exports）
    pub fn set_export_context_flag(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("set_export_context_flag"); 
        let is_ambient = node.has_syntactic_modifier(ModifierFlags::Ambient)
            || node.flags.contains(NodeFlags::Ambient)
            || self.in_ambient_context(node);
        if is_ambient && !Self::has_export_declarations(node) {
            let ptr = Arc::as_ptr(node) as *mut tsox_frontend::ast::Node;
            unsafe {
                (*ptr).flags |= NodeFlags::ExportContext;
            }
        }
    }

    /// Go parser 的 NodeFlagsAmbient 由解析器上下文注入（d.ts 整文件、declare
    /// 声明子树），Rust parser 未承载该位，此处按容器链等价重建
    fn in_ambient_context(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("in_ambient_context"); 
        if self
            .current_source_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file)
        {
            return true;
        }
        let mut parent = node.parent();
        while let Some(p) = parent {
            if p.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }
            parent = p.parent();
        }
        false
    }

    fn bind_module_declaration_inner(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("bind_module_declaration_inner"); 
        let dotted_name = match &node.data {
            tsox_frontend::ast::NodeData::ModuleDeclaration(md) => match md.name.kind {
                SyntaxKind::Identifier => md.name.text().to_string(),
                SyntaxKind::QualifiedName => {
                    fn qualified_text(n: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("qualified_text"); 
                        match &n.data {
                            tsox_frontend::ast::NodeData::QualifiedName(q) => {
                                format!("{}.{}", qualified_text(&q.left), q.right.text())
                            }
                            _ => n.text().to_string(),
                        }
                    }
                    qualified_text(&md.name)
                }
                _ => String::new(),
            },
            _ => String::new(),
        };
        if dotted_name.contains('.') {
            let parts: Vec<&str> = dotted_name.split('.').collect();

            let container = self.container.clone();
            let parent_sym = self.parent_symbol.clone();
            let mut table: Option<Arc<Symbol>> = None;
            let mut locals_key: Option<u64> = None;
            if let Some(ps) = &parent_sym {
                table = Some(Arc::clone(ps));
            } else if let Some(c) = &container {
                locals_key = Some(c.id());
            }
            let mut current: Option<Arc<Symbol>> = None;
            for part in &parts[..parts.len() - 1] {
                let existing = current.as_ref().map_or_else(
                    || {
                        table
                            .as_ref()
                            .and_then(|t| {
                                t.members
                                    .get(*part)
                                    .cloned()
                                    .or_else(|| t.exports.get(*part).cloned())
                            })
                            .or_else(|| {
                                locals_key
                                    .and_then(|k| self.symbol_map.locals.get(&k))
                                    .and_then(|l| l.get(*part).cloned())
                            })
                    },
                    |cur| cur.exports.get(*part).cloned(),
                );
                let sym = match existing {
                    Some(s) if s.flags.contains(SymbolFlags::ValueModule) => s,
                    _ => {
                        let fresh =
                            Arc::new(Symbol::new(SymbolFlags::ValueModule, part.to_string()));
                        if let Some(cur) = &current {
                            let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                            unsafe {
                                (*cur_mut)
                                    .exports
                                    .insert(part.to_string(), Arc::clone(&fresh));
                            }
                        } else if let Some(t) = &table {
                            let t_mut = Arc::as_ptr(t) as *mut Symbol;
                            unsafe {
                                (*t_mut)
                                    .members
                                    .insert(part.to_string(), Arc::clone(&fresh));
                            }
                        } else if let Some(k) = locals_key {
                            self.symbol_map
                                .locals
                                .entry(k)
                                .or_default()
                                .insert(part.to_string(), Arc::clone(&fresh));
                        }
                        fresh
                    }
                };
                current = Some(sym);
            }

            let last = parts[parts.len() - 1];
            let (includes, _excludes) = Self::module_symbol_flags(node);
            let symbol = Arc::new(Symbol::new(includes, last.to_string()));
            {
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*symbol_mut).declarations.push(Arc::clone(node));
                }
            }
            match &current {
                Some(cur) => {
                    let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                    unsafe {
                        (*cur_mut)
                            .exports
                            .insert(last.to_string(), Arc::clone(&symbol));
                    }
                }
                None => {
                    if let Some(t) = &table {
                        let t_mut = Arc::as_ptr(t) as *mut Symbol;
                        unsafe {
                            (*t_mut)
                                .members
                                .insert(last.to_string(), Arc::clone(&symbol));
                        }
                    } else if let Some(k) = locals_key {
                        self.symbol_map
                            .locals
                            .entry(k)
                            .or_default()
                            .insert(last.to_string(), Arc::clone(&symbol));
                    }
                }
            }
            self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        } else {
            let name_is_string_literal = match &node.data {
                tsox_frontend::ast::NodeData::ModuleDeclaration(md) => {
                    md.name.kind == SyntaxKind::StringLiteral
                }
                _ => false,
            };
            if name_is_string_literal {
                if tsox_frontend::ast::mig::m3g::is_module_augmentation_external(node) {
                    let (includes, excludes) = Self::module_symbol_flags(node);
                    self.declare_symbol(node, includes, excludes);
                } else {
                    let symbol = self.declare_symbol(
                        node,
                        SymbolFlags::ValueModule,
                        SymbolFlags::ValueModuleExcludes,
                    );
                    // Go binder bindModuleDeclaration（binder.go:820-833）：字符串名
                    // ambient 模块按 TryParsePattern 分派——多通配符 TS5061（binder
                    // 无诊断通道，记交接）；StarIndex >= 0 收进 file 级 pattern
                    // ambient 模块表（checker 初始化 mergePatternAmbientModules
                    // 消费）；无通配符带 with 属性 TS1454（同前，记交接）
                    let (pattern, has_attributes) = match &node.data {
                        tsox_frontend::ast::NodeData::ModuleDeclaration(md) => (
                            // Go Node.Text() 对 StringLiteral 取去引号字面值，
                            // 本仓 text() 含引号，先剥离再 TryParsePattern
                            Some(tsox_core::core::core::try_parse_pattern(
                                md.name.text().trim_matches(['"', '\'']),
                            )),
                            md.attributes.is_some(),
                        ),
                        _ => (None, false),
                    };
                    if let Some(pattern) = pattern {
                        if !pattern.is_valid() {
                            // TS5061：binder 诊断通道缺失，见交接记录
                        } else if pattern.star_index >= 0 {
                            crate::checker::mig::m2b::r22k6_defs::push_pattern_ambient_module(
                                crate::checker::mig::m2a::r20k6_defs::PatternAmbientModule {
                                    pattern,
                                    symbol,
                                },
                            );
                        } else if has_attributes {
                            // TS1454：binder 诊断通道缺失，见交接记录
                        }
                    }
                }
            } else {
                // Go declareModuleSymbol：按模块实例化状态取 ValueModule/NamespaceModule
                let state = get_module_instance_state(node);
                let (includes, excludes) = Self::module_symbol_flags(node);
                let symbol = self.declare_symbol(node, includes, excludes);
                if state != ModuleInstanceState::NonInstantiated {
                    let const_enum_only = !symbol
                        .flags
                        .intersects(SymbolFlags::Function | SymbolFlags::Class | SymbolFlags::RegularEnum)
                        && state == ModuleInstanceState::ConstEnumOnly
                        && !self.not_const_enum_only_modules.contains(&symbol.id());
                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    unsafe {
                        if const_enum_only {
                            (*symbol_mut).flags |= SymbolFlags::ConstEnumOnlyModule;
                        } else {
                            (*symbol_mut).flags &= !SymbolFlags::ConstEnumOnlyModule;
                        }
                    }
                    if !const_enum_only {
                        self.not_const_enum_only_modules.insert(symbol.id());
                    }
                }
            }
        }
    }

    fn module_symbol_flags(node: &Arc<Node>) -> (SymbolFlags, SymbolFlags) { ::tsox_core::fntrace::enter("module_symbol_flags"); 
        let state = get_module_instance_state(node);
        if state != ModuleInstanceState::NonInstantiated {
            (SymbolFlags::ValueModule, SymbolFlags::ValueModuleExcludes)
        } else {
            (SymbolFlags::NamespaceModule, SymbolFlags::NamespaceModuleExcludes)
        }
    }
}
