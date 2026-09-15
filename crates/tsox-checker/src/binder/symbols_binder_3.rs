#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    pub(crate) fn declare_symbol_into(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
        target: DeclareTarget,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        let existing: Option<Arc<Symbol>> = match &target {
            DeclareTarget::Exports(parent_sym) => parent_sym.exports.get(&name).cloned(),
            DeclareTarget::Locals(container) => {
                let locals_hit = || {
                    self.symbol_map
                        .locals
                        .get(&container.id())
                        .and_then(|locals| locals.get(&name).cloned())
                };
                // 文件顶层的别名（import）与普通声明（typedef/class 等，经
                // declare_symbol 进容器符号 members）必须互相可见才能合并
                if container.kind == SyntaxKind::SourceFile {
                    locals_hit().or_else(|| {
                        self.symbol_map
                            .symbol_of(container)
                            .and_then(|sym| sym.members.get(&name).cloned())
                    })
                } else {
                    locals_hit()
                }
            }
        };

        if let Some(existing) = existing {
            if self.can_merge_symbols(existing.flags, includes) {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(node));
                    (*existing_mut).flags |= includes;
                    if (*existing_mut).value_declaration.is_none()
                        && includes.intersects(SymbolFlags::VALUE)
                    {
                        (*existing_mut).value_declaration = Some(Arc::clone(node));
                    }
                }
                self.symbol_map.set_symbol(node, Arc::clone(&existing));
                return existing;
            }
        }

        let symbol = self.new_symbol(includes, name.clone());
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        match &target {
            DeclareTarget::Exports(parent_sym) => {
                let parent_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_mut)
                        .exports
                        .insert(name.clone(), Arc::clone(&symbol));

                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    (*symbol_mut).set_parent(parent_sym);
                }
            }
            DeclareTarget::Locals(container) => {
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
                if container.kind == SyntaxKind::SourceFile
                    && let Some(container_sym) = self.symbol_map.symbol_of(container)
                {
                    let container_sym_mut = Arc::as_ptr(&container_sym) as *mut Symbol;
                    unsafe {
                        (*container_sym_mut)
                            .members
                            .insert(name.clone(), Arc::clone(&symbol));
                    }
                }
            }
        }

        self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        symbol
    }

    pub(crate) fn can_merge_symbols(
        &self,
        existing_flags: SymbolFlags,
        new_flags: SymbolFlags,
    ) -> bool {
        // Go bindExportDeclaration 以 excludes=None 声明 __export：多个
        // export * 恒合并进同一符号的 declarations
        if existing_flags.contains(SymbolFlags::ExportStar)
            && new_flags.contains(SymbolFlags::ExportStar)
        {
            return true;
        }
        let existing_alias = existing_flags.contains(SymbolFlags::Alias);
        let new_alias = new_flags.contains(SymbolFlags::Alias);
        if existing_alias || new_alias {
            return !(existing_alias && new_alias);
        }

        // Go NamespaceModuleExcludes = None：非实例化 namespace 声明与任何既有符号合并且不冲突
        if new_flags.contains(SymbolFlags::NamespaceModule) {
            return true;
        }
        // 镜像：既有非实例化 namespace（纯类型导出）与后续值意义声明
        //（const/let/var/function/class）合并（namespace N + const N；
        // Go 变量 excludes=Value 与 NamespaceModule 不相交，declareSymbol
        // 无条件合并；type alias 的 excludes=Type 含 namespace 位仍冲突）
        if existing_flags.contains(SymbolFlags::NamespaceModule)
            && !existing_flags.contains(SymbolFlags::ValueModule)
            && new_flags.intersects(SymbolFlags::VALUE)
        {
            return true;
        }

        if existing_flags.contains(SymbolFlags::Interface)
            && new_flags.contains(SymbolFlags::Interface)
        {
            return true;
        }

        let existing_interface = existing_flags.contains(SymbolFlags::Interface);
        let new_interface = new_flags.contains(SymbolFlags::Interface);
        let existing_type_alias = existing_flags.contains(SymbolFlags::TypeAlias);
        let new_type_alias = new_flags.contains(SymbolFlags::TypeAlias);
        let class_side = SymbolFlags::Class;

        // Go TypeAliasExcludes = SymbolFlagsType（仅类型意义 class/interface/enum）：
        // type alias 与 class/interface/enum 相遇是 TS2300；与纯值声明（var/let/const/
        // function）合法合并（lib 形态：type NodeFilter + declare var NodeFilter）
        if (existing_type_alias
            && new_flags
                .intersects(SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::ENUM))
            || (new_type_alias
                && existing_flags
                    .intersects(SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::ENUM))
        {
            return false;
        }
        // type alias 与纯值声明合并
        if (existing_type_alias && new_flags.intersects(SymbolFlags::VALUE))
            || (new_type_alias && existing_flags.intersects(SymbolFlags::VALUE))
        {
            return true;
        }

        let enum_side = SymbolFlags::ENUM;
        if (existing_flags.intersects(enum_side) && new_interface)
            || (new_flags.intersects(enum_side) && existing_interface)
        {
            return false;
        }
        // Go InterfaceExcludes = Type & ^(Interface|Class)：interface 与 class/值意义
        // （var/function 等）合法合并；type alias 冲突已由上方分支处理
        if (existing_interface && new_flags.intersects(SymbolFlags::VALUE | SymbolFlags::Class))
            || (new_interface && existing_flags.intersects(SymbolFlags::VALUE | SymbolFlags::Class))
        {
            return true;
        }

        let existing_class = existing_flags.contains(SymbolFlags::Class);
        let new_class = new_flags.contains(SymbolFlags::Class);
        let existing_fn = existing_flags.contains(SymbolFlags::Function);
        let new_fn = new_flags.contains(SymbolFlags::Function);
        if (existing_class && new_fn) || (existing_fn && new_class) {
            return true;
        }
        // Go ClassExcludes 含 Class：同名 class 相交冲突（TS2300，含 ambient），
        // 冲突符号另建、不合并 declarations
        if existing_class && new_class {
            return false;
        }

        let existing_ns = existing_flags.contains(SymbolFlags::ValueModule);
        let new_ns = new_flags.contains(SymbolFlags::ValueModule);
        if existing_ns || new_ns {
            let other_existing = if existing_ns {
                new_flags
            } else {
                existing_flags
            };
            let _other_new = if existing_ns {
                existing_flags
            } else {
                new_flags
            };

            let can_merge_with_ns = other_existing.contains(SymbolFlags::ValueModule)
                || other_existing.contains(SymbolFlags::Function)
                || other_existing.contains(SymbolFlags::Class)
                || other_existing.contains(SymbolFlags::RegularEnum)
                || other_existing.contains(SymbolFlags::ConstEnum)
                || other_existing.contains(SymbolFlags::Interface);
            if can_merge_with_ns {
                return true;
            }
        }

        if existing_flags.contains(SymbolFlags::Function)
            && new_flags.contains(SymbolFlags::Function)
        {
            return true;
        }

        if (existing_flags.contains(SymbolFlags::RegularEnum)
            || existing_flags.contains(SymbolFlags::ConstEnum))
            && (new_flags.contains(SymbolFlags::RegularEnum)
                || new_flags.contains(SymbolFlags::ConstEnum))
        {
            return true;
        }

        let type_param_existing = existing_flags.contains(SymbolFlags::TypeParameter);
        let type_param_new = new_flags.contains(SymbolFlags::TypeParameter);
        if (type_param_existing && !new_flags.intersects(SymbolFlags::TYPE))
            || (type_param_new && !existing_flags.intersects(SymbolFlags::TYPE))
        {
            return true;
        }
        false
    }

    pub(crate) fn is_let_or_const_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent().as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        true
    }

    pub(crate) fn has_export_declarations(container: &Arc<Node>) -> bool {
        let statements: &[Arc<Node>] = match &container.data {
            tsox_frontend::ast::NodeData::SourceFile(sf) => &sf.statements.nodes,
            tsox_frontend::ast::NodeData::ModuleDeclaration(md) => {
                if let Some(body) = &md.body
                    && body.kind == SyntaxKind::ModuleBlock
                    && let tsox_frontend::ast::NodeData::ModuleBlock(block) = &body.data
                {
                    &block.statements.nodes
                } else {
                    &[]
                }
            }
            _ => &[],
        };
        statements.iter().any(|s| {
            s.kind == SyntaxKind::ExportDeclaration || s.kind == SyntaxKind::ExportAssignment
        })
    }

    #[allow(dead_code)]
    pub(crate) fn is_var_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent().as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        false
    }

    pub(crate) fn declaration_is_var(node: &Arc<Node>) -> bool {
        let mut current = Arc::clone(node);
        loop {
            match current.kind {
                SyntaxKind::VariableDeclaration => {
                    return current.parent().is_some_and(|parent| {
                        parent.kind == SyntaxKind::VariableDeclarationList
                            && !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const)
                    });
                }
                SyntaxKind::BindingElement
                | SyntaxKind::ObjectBindingPattern
                | SyntaxKind::ArrayBindingPattern => match current.parent() {
                    Some(parent) => current = parent,
                    None => return false,
                },
                _ => return false,
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn symbol_is_var_declaration(symbol: &Arc<Symbol>) -> bool {
        let decl: Option<&Arc<Node>> = symbol
            .value_declaration
            .as_ref()
            .or_else(|| symbol.declarations.first());
        match decl {
            Some(node) => Self::is_var_declaration(node),
            None => false,
        }
    }

    pub(crate) fn get_combined_modifier_flags(&self, node: &Arc<Node>) -> ModifierFlags {
        let mut flags = node.syntactic_modifier_flags();
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    flags |= parent.syntactic_modifier_flags();
                    if let Some(gp) = parent.parent() {
                        if gp.kind == SyntaxKind::VariableStatement {
                            flags |= gp.syntactic_modifier_flags();
                        }
                    }
                }
            }
        }
        flags
    }
}
