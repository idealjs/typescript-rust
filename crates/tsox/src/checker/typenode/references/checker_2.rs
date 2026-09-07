#![allow(unused_imports)]

use super::*;

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
        let mut symbol = if type_name.kind == SyntaxKind::Identifier {
            match self.resolve_identifier(type_name) {
                Some(s) => s,
                None => {
                    if self.ts2304_reporting_allowed_for(type_name) {
                        use crate::diagnostics::messages_generated::CANNOT_FIND_NAME_0;
                        let name_text = type_name.text();

                        let file = self
                            .get_source_file_of_node(type_name)
                            .or_else(|| self.current_file.clone());
                        self.diagnostics.add(crate::ast::Diagnostic::new(
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
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            type_name.loc,
                            crate::diagnostics::messages_generated::
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

        if symbol.flags.contains(SymbolFlags::TypeParameter) {
            return self.resolve_type_parameter_reference(&symbol, type_name);
        }
        if symbol.flags.contains(SymbolFlags::Interface) {
            return self.resolve_interface_type(&symbol, type_arguments);
        }
        if symbol.flags.intersects(SymbolFlags::ENUM) {
            return self.resolve_enum_type(&symbol);
        }
        if symbol.flags.contains(SymbolFlags::Class) {
            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
            let merged_with_ns = symbol.flags.contains(SymbolFlags::ValueModule);
            if !merged_with_ns {
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
            let instance_type = match class_node {
                Some(node) => self.build_class_instance_type_with_base(&node),
                None => self.error_type(),
            };
            self.resolving_type_aliases.remove(&key);
            if !merged_with_ns {
                self.type_alias_links.get_or_default(&symbol).declared_type =
                    Some(Arc::clone(&instance_type));
            }

            let arg_types: Option<Vec<Arc<Type>>> = type_arguments.map(|nodes| {
                nodes
                    .iter()
                    .map(|a| self.get_type_from_type_node(a))
                    .collect()
            });
            if let Some(arg_types) = arg_types {
                let tps = self.declared_type_parameter_types(&symbol);
                if !tps.is_empty() && tps.len() == arg_types.len() {
                    return self.attach_explicit_type_arguments_cached(&instance_type, arg_types);
                }
            }
            return instance_type;
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
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    crate::diagnostics::messages_generated::
                        X_0_REFERS_TO_A_VALUE_BUT_IS_BEING_USED_AS_A_TYPE_HERE_DID_YOU_MEAN_TYPEOF_0,
                    vec![name_text.clone(), name_text],
                ));
            }

            return self.error_type();
        }

        self.resolve_type_alias_reference(&symbol, type_arguments)
    }
}
