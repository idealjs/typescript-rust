#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn resolve_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (type_name, type_arguments) = match &node.data {
            NodeData::TypeReferenceNode(data) => (&data.type_name, data.type_arguments.clone()),
            NodeData::ExpressionWithTypeArguments(data) => {
                (&data.expression, data.type_arguments.clone())
            }
            _ => return self.error_type(),
        };

        if type_name.kind == SyntaxKind::Identifier && type_name.text() == "intrinsic" {
            return self.error_type();
        }
        // Go checkGrammarForAtLeastOneTypeArgument：`C<>` 空实参列表报 1099，
        // 跨度覆盖 <>，文件带解析错误时不报（文法检查短路）
        if let Some(args) = &type_arguments
            && args.nodes.is_empty()
            && !self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 1099 && d.loc.pos() == args.loc.pos())
            && !self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone())
                .is_some_and(|f| f.has_parse_diagnostics)
        {
            let file = self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone());
            let loc = tsox_core::core::text::TextRange::new(args.loc.pos(), args.loc.end() + 1);
            let mut diag = tsox_frontend::ast::Diagnostic::new(
                file,
                loc,
                tsox_core::diagnostics::messages_generated::TYPE_ARGUMENT_LIST_CANNOT_BE_EMPTY,
                vec![],
            );
            self.diagnostics.add(diag);
        }
        let mut symbol = if type_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(type_name) {
                Some(s) => s,
                None => {
                    if self.ts2304_reporting_allowed_for(type_name) {
                        use tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                        let name_text = type_name.text();

                        let file = self
                            .get_source_file_of_node(type_name)
                            .or_else(|| self.current_file.clone());
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            CANNOT_FIND_NAME_0,
                            vec![name_text.to_string()],
                        ));
                    }
                    return self.error_type();
                }
            }
        } else if matches!(
            type_name.kind,
            SyntaxKind::Identifier
                | SyntaxKind::QualifiedName
                | SyntaxKind::PropertyAccessExpression
        ) {
            match self.resolve_qualified_symbol_traced(type_name) {
                Ok(s) => s,
                Err((segment, ns_path, member)) => {
                    self.report_qualified_name_resolution_failure(
                        type_name, &segment, ns_path, member,
                    );
                    return self.error_type();
                }
            }
        } else {
            return self.error_type();
        };

        // Go checkAndReportErrorForUsingNamespaceAsTypeOrValue：类型位解析到
        // 模块符号（不含类型含义）报 TS2709，优先于 TS2749/TS2304 兜底
        {
            let alias_decl_is_namespace_form = !symbol.flags.contains(SymbolFlags::Alias)
                || symbol
                    .value_declaration
                    .as_ref()
                    .is_some_and(|d| match &d.data {
                        NodeData::ImportEqualsDeclaration(_) => true,
                        NodeData::ImportDeclaration(id) => id
                            .import_clause
                            .as_ref()
                            .and_then(|c| match &c.data {
                                NodeData::ImportClause(ic) => ic.named_bindings.as_ref(),
                                _ => None,
                            })
                            .is_some_and(|nb| nb.kind == SyntaxKind::NamespaceImport),
                        _ => false,
                    });
            let effective = if symbol.flags.contains(SymbolFlags::Alias) {
                self.resolve_alias_base(Arc::clone(&symbol))
            } else {
                Arc::clone(&symbol)
            };
            if alias_decl_is_namespace_form
                && effective
                    .flags
                    .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule)
                && !effective.flags.intersects(SymbolFlags::TYPE)
                && type_name.kind == SyntaxKind::Identifier
                && self.ts2304_reporting_allowed_for(type_name)
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let file = self
                    .get_source_file_of_node(type_name)
                    .or_else(|| self.current_file.clone());
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    tsox_core::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_TYPE,
                    vec![type_name.text().to_string()],
                ));
                return self.error_type();
            }
        }

        if symbol.flags == SymbolFlags::Alias {
            let alias_name = type_name.text().to_string();
            if let Some(target) = self.resolve_import_alias_target_symbol(&symbol) {
                let target_has_type_meaning = target.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::ENUM
                        | SymbolFlags::TypeParameter,
                );
                if target_has_type_meaning {
                    symbol = target;
                } else {
                    if type_name.kind == SyntaxKind::Identifier
                        && self.ts2304_reporting_allowed_for(type_name)
                        && !self.has_same_named_type_symbol(&alias_name)
                        && self
                            .current_file
                            .as_ref()
                            .is_some_and(|f| !f.file_name.starts_with("bundled://"))
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            tsox_core::diagnostics::messages_generated::
                                X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                            vec![alias_name.clone(), alias_name],
                        ));
                    }
                    return self.error_type();
                }
            }
        }

        if !self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
            && symbol
                .flags
                .intersects(SymbolFlags::Interface | SymbolFlags::Class | SymbolFlags::TypeAlias)
            && !type_name_inside_conditional_branch(type_name)
            && !type_name_shadowed_by_type_parameter(type_name)
            && !self.check_type_reference_arguments(node, type_name, &symbol)
        {
            return self.error_type();
        }

        if std::env::var_os("TSOX_DEBUG_NS").is_some() && symbol.name == "B" {
            eprintln!(
                "[tr] B flags={:?} class={} iface={:?}",
                symbol.flags,
                symbol.flags.contains(SymbolFlags::Class),
                symbol.flags.contains(SymbolFlags::Interface)
            );
        }
        if symbol.flags.contains(SymbolFlags::TypeParameter) {
            return self.resolve_type_parameter_reference(&symbol, type_name);
        }
        // tsc getTypeReferenceType：class+interface 同名合并的符号走 class 引用路径，
        // interface 声明的成员与基类并入同一声明类型（getDeclaredTypeOfClassOrInterface）
        if symbol.flags.contains(SymbolFlags::Class)
            && symbol
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::ClassDeclaration)
        {
            let key = Arc::as_ptr(&symbol) as *const tsox_frontend::ast::Symbol;
            let merged_with_ns = symbol.flags.contains(SymbolFlags::ValueModule);
            // 声明缓存仅在无类型实参时可直接复用；带实参引用须实例化（attach），
            // 否则 C<number> 会被裸声明形态污染
            if !merged_with_ns && type_arguments.is_none() {
                if let Some(cached) = self
                    .type_alias_links
                    .get(&symbol)
                    .and_then(|l| l.declared_type.clone())
                {
                    return cached;
                }
            }
            if !self.resolving_type_aliases.insert(key) {
                return self.error_type();
            }
            let class_node = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                .cloned();
            let arg_types: Option<Vec<Arc<Type>>> = type_arguments.map(|nodes| {
                nodes
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect()
            });
            let class_tps: Vec<Arc<tsox_frontend::ast::Symbol>> = match &class_node {
                Some(node) => match &node.data {
                    tsox_frontend::ast::NodeData::ClassDeclaration(cd) => {
                        match &cd.type_parameters {
                            Some(tps) => tps
                                .iter()
                                .filter_map(|tp| {
                                    self.program
                                        .symbol_map()
                                        .symbol_of(tp)
                                        .map(Arc::clone)
                                })
                                .collect(),
                            None => Vec::new(),
                        }
                    }
                    _ => Vec::new(),
                },
                None => Vec::new(),
            };
            let args_match = arg_types
                .as_ref()
                .is_some_and(|a| !a.is_empty() && a.len() == class_tps.len());
            let instance_type = match class_node {
                Some(node) => {
                    let class_type = if args_match {
                        self.instantiate_class_instance_type(
                            &node,
                            &symbol,
                            &class_tps,
                            arg_types.as_ref().unwrap(),
                        )
                    } else {
                        self.build_class_instance_type_with_base(&node)
                    };
                    if symbol.flags.intersects(SymbolFlags::Interface) {
                        let iface_type =
                            self.resolve_interface_type_ex(&symbol, arg_types.clone());
                        self.merge_instance_types(&class_type, &iface_type)
                    } else {
                        class_type
                    }
                }
                None => self.error_type(),
            };
            self.resolving_type_aliases.remove(&key);
            if !merged_with_ns && arg_types.is_none() {
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&instance_type));
            }

            if let Some(arg_types) = arg_types {
                if args_match {
                    return instance_type;
                }
                let tps = self.declared_type_parameter_types(&symbol);
                if !tps.is_empty() && tps.len() == arg_types.len() {
                    return self.attach_explicit_type_arguments_cached(&instance_type, arg_types);
                }
            }
            return instance_type;
        }
        if symbol.flags.contains(SymbolFlags::Interface) {
            // tsc getTypeFromClassOrInterfaceReference：无实参引用且类型参数带默认值时按默认值实例化；
            // 部分实参（如 Iterator<T> 少于 TReturn/TNext）补声明默认值
            //（Go getTypeArguments 的默认填充语义）
            let declared_tp_count = self.declared_type_parameter_types(&symbol).len();
            match &type_arguments {
                None => {
                    let defaults = self.interface_default_type_arguments(&symbol);
                    if !defaults.is_empty() {
                        return self.resolve_interface_type_ex(&symbol, Some(defaults));
                    }
                }
                Some(nodes) if nodes.len() < declared_tp_count => {
                    let defaults = self.interface_default_type_arguments(&symbol);
                    if !defaults.is_empty() && nodes.len() < defaults.len() {
                        let mut args: Vec<Arc<Type>> = nodes
                            .iter()
                            .map(|a| self.get_type_from_type_node(a))
                            .collect();
                        args.extend(defaults[nodes.len()..].iter().cloned());
                        return self.resolve_interface_type_ex(&symbol, Some(args));
                    }
                }
                _ => {}
            }
            return self.resolve_interface_type(&symbol, type_arguments);
        }
        if symbol.flags.intersects(SymbolFlags::ENUM) {
            return self.resolve_enum_type(&symbol);
        }
        if !symbol.flags.contains(SymbolFlags::TypeAlias) {
            if matches!(&node.data, NodeData::ExpressionWithTypeArguments(_))
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::TypeParameter,
                )
            {
                let value_type = self.get_type_of_symbol(&symbol);
                if let Some(structured) = value_type.as_structured() {
                    for sig in structured.construct_signatures() {
                        if let Some(ret) = self.get_return_type_of_signature(sig) {
                            return ret;
                        }
                    }
                }
                return self.get_any_type();
            }

            if type_name.kind == SyntaxKind::Identifier
                && symbol.flags.intersects(
                    SymbolFlags::BlockScopedVariable
                        | SymbolFlags::FunctionScopedVariable
                        | SymbolFlags::Function,
                )
                && !symbol.flags.intersects(
                    SymbolFlags::Interface
                        | SymbolFlags::Class
                        | SymbolFlags::TypeParameter
                        | SymbolFlags::TypeAlias
                        | SymbolFlags::Alias,
                )
                && self.ts2304_reporting_allowed_for(type_name)
                && !self.has_same_named_type_symbol(type_name.text())
                && self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| !f.file_name.starts_with("bundled://"))
            {
                let name_text = type_name.text().to_string();
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    tsox_core::diagnostics::messages_generated::
                        X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                    vec![name_text.clone(), name_text],
                ));
            }

            return self.error_type();
        }

        self.resolve_type_alias_reference(&symbol, type_arguments)
    }

    pub(crate) fn interface_default_type_arguments(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Type>> {
        let decl = symbol.declarations.iter().find(|d| {
            matches!(d.data, NodeData::InterfaceDeclaration(_))
        });
        let Some(decl) = decl else {
            return Vec::new();
        };
        let NodeData::InterfaceDeclaration(data) = &decl.data else {
            return Vec::new();
        };
        let Some(tps) = &data.type_parameters else {
            return Vec::new();
        };
        let has_any_default = tps.iter().any(|tp| {
            matches!(&tp.data, NodeData::TypeParameterDeclaration(td) if td.default_type.is_some())
        });
        if !has_any_default {
            return Vec::new();
        }
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        self.push_scope(decl);
        let args: Vec<Arc<Type>> = tps
            .clone()
            .iter()
            .map(|tp| match &tp.data {
                NodeData::TypeParameterDeclaration(td) => match &td.default_type {
                    Some(default_node) => self.get_type_from_type_node(default_node),
                    None => self.get_unknown_type(),
                },
                _ => self.get_unknown_type(),
            })
            .collect();
        self.pop_scope();
        self.type_argument_stack = saved_stack;
        args
    }
}
