#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_type_from_type_literal_or_function_or_constructor_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        self.cache_type(node, self.error_type());
        let result = match &node.data {
            NodeData::TypeLiteralNode(data) => {
                self.build_interface_type_from_members(&data.members)
            }
            NodeData::FunctionTypeNode(_) => self.get_type_from_function_type_node(node),
            NodeData::ConstructorTypeNode(_) => self.get_type_from_constructor_type_node(node),
            _ => self.error_type(),
        };
        self.cache_type(node, result.clone());
        result
    }

    #[allow(dead_code)]
    pub(crate) fn get_type_from_type_literal_members(
        &mut self,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        let mut index_infos: Vec<Arc<crate::checker::IndexInfo>> = Vec::new();
        for member in members.iter() {
            match &member.data {
                NodeData::PropertySignatureDeclaration(data) => {
                    let name = data.name.text().to_string();
                    if name.is_empty() {
                        continue;
                    }
                    let prop_type = self.get_type_from_type_node(&data.type_node);
                    let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));
                    self.value_symbol_links.insert(
                        &symbol,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                    symbol_table.insert(name, Arc::clone(&symbol));
                    props.push(symbol);
                }
                NodeData::IndexSignatureDeclaration(data) => {
                    let mut key_type = None;
                    let value_type;
                    if let Some(param) = data.parameters.iter().next() {
                        if let NodeData::ParameterDeclaration(pd) = &param.data {
                            key_type = pd
                                .type_node
                                .as_ref()
                                .map(|t| self.get_type_from_type_node(t));
                        }
                    }
                    value_type = Some(self.get_type_from_type_node(&data.type_node));
                    let is_readonly = member
                        .modifiers()
                        .as_ref()
                        .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
                    index_infos.push(Arc::new(crate::checker::IndexInfo {
                        key_type,
                        value_type,
                        is_readonly,
                        declaration: Some(Arc::clone(member)),
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
                _ => {}
            }
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn get_type_from_function_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::FunctionTypeNode(data) => {
                self.push_scope(node);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    false,
                    None,
                    Some(Arc::clone(node)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], false)
            }
            _ => self.error_type(),
        }
    }

    pub(crate) fn get_type_from_constructor_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ConstructorTypeNode(data) => {
                self.push_scope(node);
                let return_type = match data.type_node.as_ref() {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => self.get_any_type(),
                };
                let sig = self.build_signature_from_function_like_type_node(
                    &data.parameters,
                    return_type,
                    true,
                    None,
                    Some(Arc::clone(node)),
                );
                self.pop_scope();
                self.create_function_or_constructor_type(vec![sig], true)
            }
            _ => self.error_type(),
        }
    }

    pub(crate) fn iife_with_too_few_arguments(
        declaration: &Option<Arc<Node>>,
        parameter_count: usize,
    ) -> bool {
        let Some(decl) = declaration else {
            return false;
        };
        if !matches!(
            decl.kind,
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
        ) {
            return false;
        }
        let mut prev: Arc<Node> = Arc::clone(decl);
        let mut parent: Option<Arc<Node>> = decl.parent.clone();
        while matches!(
            parent.as_ref().map(|p| p.kind),
            Some(SyntaxKind::ParenthesizedExpression)
        ) {
            prev = parent.clone().expect("checked Some above");
            parent = prev.parent.clone();
        }
        let Some(parent) = parent else {
            return false;
        };
        if parent.kind != SyntaxKind::CallExpression {
            return false;
        }
        let crate::ast::NodeData::CallExpression(call) = &parent.data else {
            return false;
        };

        Arc::ptr_eq(&call.expression, &prev) && parameter_count > call.arguments.nodes.len()
    }
}
