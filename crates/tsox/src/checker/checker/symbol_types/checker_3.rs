#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_value_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        if let Some(decl) = &symbol.value_declaration {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }

        for decl in &symbol.declarations {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }
        self.get_any_type()
    }

    pub(crate) fn resolve_enum_value_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        let _ = self.resolve_enum_type(symbol);

        let members: Vec<(String, Arc<Symbol>)> = symbol
            .members
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in &members {
            if name.starts_with("\u{FE}") {
                continue;
            }

            let _ = self.get_type_of_symbol(member_sym);
            symbol_table.insert(name.clone(), Arc::clone(member_sym));
            props.push(Arc::clone(member_sym));
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    constrained: ConstrainedTypeData::default(),
                    members: symbol_table,
                    properties: props,
                    signatures: Vec::new(),
                    call_signature_count: 0,
                    index_infos: Vec::new(),
                    object_type_without_abstract_construct_signatures: std::sync::OnceLock::new(),
                },
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        });
        self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&result));
        result
    }

    pub(crate) fn get_type_of_function_like(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (parameters, body, type_node) = match &node.data {
            crate::ast::NodeData::FunctionExpression(data) => {
                (&data.parameters, Some(&data.body), data.type_node.as_ref())
            }
            crate::ast::NodeData::ArrowFunction(data) => {
                (&data.parameters, Some(&data.body), data.type_node.as_ref())
            }
            crate::ast::NodeData::FunctionDeclaration(data) => (
                &data.parameters,
                data.body.as_ref(),
                data.type_node.as_ref(),
            ),
            _ => return self.get_any_type(),
        };

        let contextual_signature: Option<Arc<Signature>> = self
            .get_contextual_signature(node)
            .or_else(|| self.iife_contextual_signature(node));
        let contextual_signature = contextual_signature.as_ref();

        let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
        if is_arrow {
            self.push_arrow_function_scope(node);
        } else {
            self.push_function_scope(node);
        }

        let placeholder = self.get_any_type();
        let _primed = self.build_signature_from_function_like_type_node(
            parameters,
            placeholder,
            false,
            contextual_signature,
            None,
        );

        let return_type = self.infer_function_return_type(body, type_node);
        if is_arrow {
            self.pop_arrow_function_scope();
        } else {
            self.pop_function_scope();
        }

        let sig = self.build_signature_from_function_like_type_node(
            parameters,
            return_type,
            false,
            contextual_signature,
            Some(Arc::clone(node)),
        );

        if !sig.type_parameters.is_empty()
            && let Some(contextual) = contextual_signature
        {
            if contextual.type_parameters.is_empty() {
                let inst = self.instantiate_signature_in_context_of(&sig, contextual);
                return self.create_function_or_constructor_type(vec![inst], false);
            }
        }
        self.create_function_or_constructor_type(vec![sig], false)
    }

    pub(crate) fn build_overload_function_type(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        let fn_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::FunctionDeclaration)
            .cloned()
            .collect();
        if fn_decls.len() <= 1 {
            return None;
        }

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        for decl in &fn_decls {
            let has_body = match &decl.data {
                crate::ast::NodeData::FunctionDeclaration(data) => data.body.is_some(),
                _ => false,
            };
            if has_body {
                continue;
            }
            let (parameters, type_node) = match &decl.data {
                crate::ast::NodeData::FunctionDeclaration(data) => {
                    (&data.parameters, data.type_node.as_ref())
                }
                _ => continue,
            };

            self.push_scope(decl);
            let return_type = match type_node {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            };
            let sig = self.build_signature_from_function_like_type_node(
                parameters,
                return_type,
                false,
                None,
                Some(Arc::clone(decl)),
            );
            self.pop_scope();
            signatures.push(sig);
        }
        if signatures.is_empty() {
            return None;
        }
        Some(self.create_function_or_constructor_type(signatures, false))
    }

    pub(crate) fn get_type_of_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let members = match &node.data {
            crate::ast::NodeData::ClassDeclaration(data) => Arc::clone(&data.members),
            _ => return self.get_any_type(),
        };

        if let Some(links) = self.type_node_links.get(node) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        let node_id = node.id();
        if self.class_type_resolution_stack.contains(&node_id) {
            return self.get_any_type();
        }
        self.class_type_resolution_stack.push(node_id);
        let result = self.build_type_of_class_declaration(node, &members);
        self.class_type_resolution_stack.pop();
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }
}
