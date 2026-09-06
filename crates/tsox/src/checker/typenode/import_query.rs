use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    Node, Symbol,
    SyntaxKind,
};

use crate::checker::checker::Checker;


use super::*;


impl Checker {
    pub(crate) fn get_type_from_type_query_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_query(node);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn resolve_import_alias_target_symbol(
        &mut self,
        alias: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let (member_name, import_decl): (String, Arc<Node>) = {
            let decl = alias
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::ImportClause | SyntaxKind::ImportSpecifier
                    )
                })?
                .clone();
            match &decl.data {
                NodeData::ImportClause(_) => ("default".to_string(), decl),
                NodeData::ImportSpecifier(d) => (
                    d.property_name
                        .as_ref()
                        .map_or_else(|| d.name.text().to_string(), |p| p.text().to_string()),
                    decl,
                ),
                _ => return None,
            }
        };
        let mut import_decl = import_decl.parent.as_ref()?;
        while !matches!(import_decl.data, NodeData::ImportDeclaration(_)) {
            import_decl = import_decl.parent.as_ref()?;
        }
        let module_spec = match &import_decl.data {
            NodeData::ImportDeclaration(d) => d.module_specifier.text().to_string(),
            _ => return None,
        };
        let module_sym = self
            .resolve_module_file_symbol(&module_spec)
            .or_else(|| {
                let trimmed = module_spec.trim_matches(['"', '\'', '`']).to_string();
                let cur = self.current_file.clone()?;
                let path = self.program.resolve_external_module_path(
                    &trimmed,
                    &cur.file_name,
                    crate::core::compiler_options::ModuleKind::None,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()
            })?;
        let resolved = self
            .resolve_module_member_symbol(&module_sym, &member_name, 8)
            .or_else(|| {

                self.file_module_exported_member(&module_sym, &member_name)
            });
        let resolved = match resolved {
            Some(t)

                if !t.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                ) =>
            {
                let mut cur = Arc::clone(&t);
                for _ in 0..4 {
                    if cur.flags != crate::ast::SymbolFlags::Alias {
                        break;
                    }

                    let next = cur
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ExportAssignment)
                        .and_then(|d| match &d.data {
                            NodeData::ExportAssignment(ea)
                                if matches!(
                                    ea.expression.kind,
                                    SyntaxKind::Identifier | SyntaxKind::QualifiedName
                                ) =>
                            {
                                Some(ea.expression.text().to_string())
                            }
                            _ => None,
                        })
                        .and_then(|n| {
                            module_sym
                                .members
                                .get(&n)
                                .cloned()
                                .or_else(|| module_sym.exports.get(&n).cloned())
                        });
                    match next {
                        Some(n) => cur = n,
                        None => break,
                    }
                }
                let has_type_meaning = cur.flags.intersects(
                    crate::ast::SymbolFlags::Interface
                        | crate::ast::SymbolFlags::TypeAlias
                        | crate::ast::SymbolFlags::Class
                        | crate::ast::SymbolFlags::ENUM
                        | crate::ast::SymbolFlags::TypeParameter,
                );
                if has_type_meaning {
                    Some(cur)
                } else {
                    Some(t)
                }
            }
            other => other,
        };
        resolved
    }

    pub(crate) fn file_module_exported_member(
        &self,
        module_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if !module_sym
            .declarations
            .iter()
            .any(|d| d.kind == SyntaxKind::SourceFile)
        {
            return None;
        }
        if let Some(s) = module_sym.exports.get(name) {
            return Some(Arc::clone(s));
        }
        let sym_map = self.program.symbol_map();
        let mut found: Option<Arc<Symbol>> = None;
        self.for_each_module_statement(module_sym, |stmt| {
            match &stmt.data {
                NodeData::ExportAssignment(ea) => {
                    if !ea.is_export_equals && name == "default" && found.is_none() {

                        let by_name = match &ea.expression.kind {
                            SyntaxKind::Identifier => module_sym
                                .members
                                .get(ea.expression.text())
                                .cloned()
                                .or_else(|| module_sym.exports.get(ea.expression.text()).cloned()),
                            _ => None,
                        };
                        found = by_name.or_else(|| {
                            sym_map
                                .symbol_of(stmt)
                                .cloned()
                                .or_else(|| {
                                    stmt.expression()
                                        .and_then(|e| sym_map.symbol_of(e).cloned())
                                })
                        });
                    }
                }
                NodeData::VariableStatement(vs) => {
                    if let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data {
                        for decl in vdl.declarations.iter() {
                            if decl.name().is_some_and(|n| n.text() == name) {
                                found = sym_map.symbol_of(decl).cloned();
                            }
                        }
                    }
                }
                _ => {
                    if stmt.name().is_some_and(|n| n.text() == name)
                        && (stmt.has_syntactic_modifier(crate::ast::ModifierFlags::Export)
                            || stmt
                                .has_syntactic_modifier(crate::ast::ModifierFlags::Default))
                    {
                        found = sym_map.symbol_of(stmt).cloned();
                    }
                }
            }
            false
        });
        found
    }

    pub(crate) fn resolve_type_query(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let NodeData::TypeQueryNode(d) = &node.data else {
            return self.error_type();
        };

        fn report_unresolved(c: &mut Checker, seg: &Arc<Node>) {
            if c.ts2304_reporting_allowed_for(seg) {
                use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                let file = c
                    .get_source_file_of_node(seg)
                    .or_else(|| c.current_file.clone());
                c.diagnostics.add(crate::ast::Diagnostic::new(
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
                None => {
                    report_unresolved(self, &d.expr_name);
                    return self.error_type();
                }
            }
        };

        if symbol.flags == crate::ast::SymbolFlags::Alias {

            if d.expr_name.kind == SyntaxKind::Identifier {
                let base = self.resolve_alias_base(Arc::clone(&symbol));
                let is_true_namespace = base.declarations.iter().any(|dd| {
                    dd.kind == SyntaxKind::ModuleDeclaration
                        && dd.name().is_some_and(|n| {
                            !matches!(n.kind, SyntaxKind::StringLiteral)
                        })
                });
                if base.flags.contains(crate::ast::SymbolFlags::ValueModule)
                    && is_true_namespace
                    && !self.namespace_usable_as_value(&base)
                {
                    let file = self
                        .get_source_file_of_node(&d.expr_name)
                        .or_else(|| self.current_file.clone());
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        d.expr_name.loc,
                        crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
                        vec![d.expr_name.text().to_string()],
                    ));
                    return self.error_type();
                }
            }
            if let Some(t) = self.type_of_imported_symbol(&symbol) {
                return t;
            }
        }

        if symbol.flags.contains(crate::ast::SymbolFlags::ValueModule)
            && d.expr_name.kind == SyntaxKind::Identifier
            && symbol.declarations.iter().any(|dd| {
                dd.kind == SyntaxKind::ModuleDeclaration
                    && dd.name().is_some_and(|n| !matches!(n.kind, SyntaxKind::StringLiteral))
            })
            && !self.namespace_usable_as_value(&symbol)
        {
            let file = self
                .get_source_file_of_node(&d.expr_name)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                d.expr_name.loc,
                crate::diagnostics::messages_generated::CANNOT_USE_NAMESPACE_0_AS_A_VALUE,
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
            let tp_symbols: Vec<Arc<crate::ast::Symbol>> = match &decl.data {
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
                        Arc::as_ptr(tp) as *const crate::ast::Symbol,
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
                crate::checker::types::TypeData::Object(
                    match &built.data {
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
                    },
                ),
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

    pub(crate) fn instantiate_value_type_for_type_query(
        &mut self,
        base: &Arc<Type>,
        arg_types: &[Arc<Type>],
    ) -> Arc<Type> {
        if arg_types.is_empty() {
            return Arc::clone(base);
        }
        match &base.data {
            TypeData::Intersection(i) => {
                let parts: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_intersection_type(parts)
            }
            TypeData::Union(u) => {
                let parts: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|p| self.instantiate_value_type_for_type_query(p, arg_types))
                    .collect();
                self.get_union_type(parts)
            }
            TypeData::Object(o) => {
                let call_count = o.structured.call_signature_count;
                let sigs = &o.structured.signatures;
                let mut changed = false;
                let mut new_sigs: Vec<Arc<crate::checker::types::Signature>> =
                    Vec::with_capacity(sigs.len());
                for (idx, sig) in sigs.iter().enumerate() {
                    let is_construct = idx >= call_count;

                    let mut params: Vec<Arc<Type>> = sig.type_parameters.clone();
                    if params.is_empty() && is_construct {
                        if let Some(rt) = self.get_return_type_of_signature(sig)
                            && let Some(class_sym) = &rt.symbol
                        {
                            let class_tps = self.declared_type_parameter_types(class_sym);
                            if !class_tps.is_empty() {
                                params = class_tps;
                            }
                        }
                    }
                    if params.is_empty() || arg_types.len() > params.len() {
                        new_sigs.push(Arc::clone(sig));
                        continue;
                    }
                    let inst = self.get_signature_instantiation(sig, arg_types);

                    let rt0 = self.get_return_type_of_signature(&inst);
                    let inst = match rt0 {
                        Some(rt0) => {
                            let deep = self
                                .substitute_object_properties_deep(&rt0, &params, arg_types);
                            let mut rebuilt = crate::checker::types::Signature::new();
                            rebuilt.flags = inst.flags;
                            rebuilt.min_argument_count = inst.min_argument_count;
                            rebuilt.resolved_min_argument_count =
                                inst.resolved_min_argument_count;
                            rebuilt.declaration = inst.declaration.clone();
                            rebuilt.parameters = inst.parameters.clone();
                            rebuilt.this_parameter = inst.this_parameter.clone();
                            rebuilt.resolved_type_predicate =
                                inst.resolved_type_predicate.clone();
                            rebuilt.target = inst.target.clone();
                            rebuilt.mapper = inst.mapper.clone();
                            rebuilt.instantiated_parameter_types =
                                inst.instantiated_parameter_types.clone();
                            if let Some(it) = inst.isolated_signature_type.get() {
                                let _ = rebuilt.isolated_signature_type.set(it.clone());
                            }
                            let _ = rebuilt.resolved_return_type.set(deep);
                            Arc::new(rebuilt)
                        }
                        None => inst,
                    };
                    changed = true;
                    new_sigs.push(inst);
                }
                if !changed {
                    return Arc::clone(base);
                }
                let shell = Arc::new(Type::new(
                    base.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData {
                            members: o.structured.members.clone(),
                            properties: o.structured.properties.clone(),
                            signatures: new_sigs,
                            call_signature_count: call_count,
                            index_infos: o.structured.index_infos.clone(),
                            ..Default::default()
                        },
                        target: o.target.clone(),
                        mapper: o.mapper.clone(),
                        type_arguments: o.type_arguments.clone(),
                    }),
                ));
                {
                    let t_mut = Arc::as_ptr(&shell) as *mut Type;
                    unsafe {
                        (*t_mut).object_flags =
                            base.object_flags | crate::checker::types::ObjectFlags::Instantiated;
                        (*t_mut).symbol = base.symbol.clone();
                    }
                }
                shell
            }
            _ => Arc::clone(base),
        }
    }
}
