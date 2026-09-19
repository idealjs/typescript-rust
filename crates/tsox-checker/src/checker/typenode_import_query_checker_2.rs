#![allow(unused_imports)]

use crate::checker::typenode_import_query::*;

impl Checker {
    pub(crate) fn resolve_type_query(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let NodeData::TypeQueryNode(d) = &node.data else {
            return self.error_type();
        };

        // Go checkExpressionWithTypeArguments：`typeof this`/`typeof this.x` 的
        // this 根部按 this 表达式求值（静态块/静态成员=构造侧类型）
        {
            let mut leftmost = &d.expr_name;
            let mut segments: Vec<Arc<Node>> = Vec::new();
            loop {
                match &leftmost.data {
                    NodeData::QualifiedName(q) => {
                        segments.push(Arc::clone(&q.right));
                        leftmost = &q.left;
                    }
                    _ => break,
                }
            }
            if leftmost.kind == SyntaxKind::Identifier && leftmost.text() == "this" {
                let mut t = self.this_expression_type(&d.expr_name);
                for seg in segments.iter().rev() {
                    let name = self.node_text(seg);
                    match self.get_property_of_type(&t, &name) {
                        Some(prop) => t = self.get_type_of_symbol(&prop),
                        None => {
                            report_unresolved(self, seg);
                            return self.error_type();
                        }
                    }
                }
                let widened = self.get_widened_type(&t);
                return self.get_regular_type_of_literal_type(&widened);
            }
        }

        fn report_unresolved(c: &mut Checker, seg: &Arc<Node>) {
            if c.ts2304_reporting_allowed_for(seg) {
                use tsox_core::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                let file = c
                    .get_source_file_of_node(seg)
                    .or_else(|| c.current_file.clone());
                c.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    seg.loc,
                    CANNOT_FIND_NAME_0,
                    vec![seg.text().to_string()],
                ));
            }
        }
        let symbol = if d.expr_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(&d.expr_name) {
                Some(s) => s,
                None => {
                    report_unresolved(self, &d.expr_name);
                    return self.error_type();
                }
            }
        } else {
            match self.resolve_qualified_symbol(&d.expr_name) {
                Some(s) => s,
                // Go checkExpressionWithTypeArguments：限定名按值位解析最左实体后
                // 逐段取属性类型（`typeof Symbol.obs` 全局扩充成员）
                None => match self.resolve_qualified_via_property_chain(&d.expr_name) {
                    Some(t) => {
                        let widened = self.get_widened_type(&t);
                        return self.get_regular_type_of_literal_type(&widened);
                    }
                    None => {
                        report_unresolved(self, &d.expr_name);
                        return self.error_type();
                    }
                },
            }
        };

        if symbol.flags == tsox_frontend::ast::SymbolFlags::Alias {
            if d.expr_name.kind == SyntaxKind::Identifier {
                let base = self.resolve_alias_base(Arc::clone(&symbol));
                let is_true_namespace = base.declarations.iter().any(|dd| {
                    dd.kind == SyntaxKind::ModuleDeclaration
                        && dd
                            .name()
                            .is_some_and(|n| !matches!(n.kind, SyntaxKind::StringLiteral))
                });
                if base
                    .flags
                    .contains(tsox_frontend::ast::SymbolFlags::ValueModule)
                    && is_true_namespace
                    && !self.namespace_usable_as_value(&base)
                {
                    let file = self
                        .get_source_file_of_node(&d.expr_name)
                        .or_else(|| self.current_file.clone());
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        d.expr_name.loc,
                        tsox_core::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                        vec![d.expr_name.text().to_string()],
                    ));
                    return self.error_type();
                }
            }
            if let Some(t) = self.type_of_imported_symbol(&symbol) {
                return t;
            }
        }

        if symbol
            .flags
            .contains(tsox_frontend::ast::SymbolFlags::ValueModule)
            && d.expr_name.kind == SyntaxKind::Identifier
            && symbol.declarations.iter().any(|dd| {
                dd.kind == SyntaxKind::ModuleDeclaration
                    && dd
                        .name()
                        .is_some_and(|n| !matches!(n.kind, SyntaxKind::StringLiteral))
            })
            && !self.namespace_usable_as_value(&symbol)
        {
            let file = self
                .get_source_file_of_node(&d.expr_name)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                d.expr_name.loc,
                tsox_core::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                vec![d.expr_name.text().to_string()],
            ));
            return self.error_type();
        }

        let class_decl = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned();
        if let Some(decl) = class_decl {
            if d.type_arguments.is_none() {
                return self.get_type_of_class_declaration(&decl);
            }
            let sym_map = self.program.symbol_map();
            let tp_symbols: Vec<Arc<tsox_frontend::ast::Symbol>> = match &decl.data {
                NodeData::ClassDeclaration(cd) => match &cd.type_parameters {
                    Some(tps) => tps
                        .iter()
                        .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                        .collect(),
                    None => Vec::new(),
                },
                _ => Vec::new(),
            };
            let arg_types: Vec<Arc<Type>> = d
                .type_arguments
                .as_ref()
                .unwrap()
                .iter()
                .map(|a| self.get_type_from_type_node(a))
                .collect();
            if tp_symbols.is_empty() || arg_types.len() > tp_symbols.len() {
                return self.get_type_of_class_declaration(&decl);
            }
            let mut key = Vec::with_capacity(arg_types.len() + 1);
            key.push(decl.id() as usize);
            key.extend(
                arg_types
                    .iter()
                    .map(|t| Arc::as_ptr(t) as *const Type as usize),
            );
            if let Some(cached) = self.typequery_instantiation_cache.get(&key) {
                return Arc::clone(&cached.1);
            }
            let mut mapping = HashMap::new();
            for (i, tp) in tp_symbols.iter().enumerate() {
                if i < arg_types.len() {
                    mapping.insert(
                        Arc::as_ptr(tp) as *const tsox_frontend::ast::Symbol,
                        Arc::clone(&arg_types[i]),
                    );
                }
            }
            self.type_argument_stack.push(mapping);
            let members = match &decl.data {
                NodeData::ClassDeclaration(data) => Arc::clone(&data.members),
                _ => unreachable!("class decl checked above"),
            };

            let built = if self.class_type_resolution_stack.contains(&decl.id()) {
                self.get_type_of_class_declaration(&decl)
            } else {
                self.class_type_resolution_stack.push(decl.id());
                let built = self.build_type_of_class_declaration(&decl, &members);
                self.class_type_resolution_stack.pop();
                built
            };
            self.type_argument_stack.pop();

            let stripped = Arc::new(Type::new(
                built.flags,
                crate::checker::types::TypeData::Object(match &built.data {
                    crate::checker::types::TypeData::Object(o) => ObjectTypeData {
                        structured: StructuredTypeData {
                            members: o.structured.members.clone(),
                            properties: o.structured.properties.clone(),
                            signatures: o.structured.signatures.clone(),
                            call_signature_count: o.structured.call_signature_count,
                            index_infos: o.structured.index_infos.clone(),
                            ..Default::default()
                        },
                        target: o.target.clone(),
                        mapper: o.mapper.clone(),
                        type_arguments: o.type_arguments.clone(),
                    },
                    _ => unreachable!("class build yields an object type"),
                }),
            ));
            {
                let t_mut = Arc::as_ptr(&stripped) as *mut Type;
                unsafe {
                    (*t_mut).object_flags =
                        built.object_flags | crate::checker::types::ObjectFlags::Instantiated;
                }
            }

            self.typequery_instantiation_cache
                .insert(key, (arg_types, Arc::clone(&stripped)));
            return stripped;
        }

        if self.is_undefined_symbol(&symbol) {
            return self.undefined_type();
        }

        let value_type = self
            .value_symbol_links
            .get(&symbol)
            .and_then(|l| l.resolved_type.clone());
        if let Some(t) = value_type {
            if d.type_arguments.is_some() {
                let arg_types: Vec<Arc<Type>> = d
                    .type_arguments
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect();
                return self.instantiate_value_type_for_type_query(&t, &arg_types);
            }
            return t;
        }
        self.error_type()
    }

    /// Go checkExpressionWithTypeArguments 的限定名回退：最左 Identifier 按
    /// 值位解析，其余段作为属性逐段取类型（`typeof Symbol.obs` 全局扩充成员）
    pub(crate) fn resolve_qualified_via_property_chain(
        &mut self,
        entity: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let mut segments: Vec<Arc<Node>> = Vec::new();
        let mut leftmost = entity;
        loop {
            match &leftmost.data {
                NodeData::QualifiedName(q) => {
                    segments.push(Arc::clone(&q.right));
                    leftmost = &q.left;
                }
                _ => break,
            }
        }
        if leftmost.kind != SyntaxKind::Identifier {
            return None;
        }
        let base = self.resolve_identifier(leftmost)?;
        let base_type = self
            .value_symbol_links
            .get(&base)
            .and_then(|l| l.resolved_type.clone())
            .or_else(|| {
                if base
                    .flags
                    .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
                {
                    Some(self.get_type_of_symbol(&base))
                } else {
                    None
                }
            })?;
        let mut t = base_type;
        for seg in segments.iter().rev() {
            let name = self.get_property_name_from_node(seg);
            let prop = self.get_property_of_type(&t, &name)?;
            t = self.get_type_of_symbol(&prop);
        }
        Some(t)
    }
}
