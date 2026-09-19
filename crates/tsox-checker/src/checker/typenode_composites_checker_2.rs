#![allow(unused_imports)]

use crate::checker::typenode_composites::*;

impl Checker {
    pub(crate) fn get_type_from_type_literal_or_function_or_constructor_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        if matches!(&node.data, NodeData::TypeLiteralNode(_)) {
            self.check_type_literal_duplicate_declarations(node);
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
        self.attach_alias_for_type_node(node, &result);
        self.cache_type_overwrite_error(node, result.clone());
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

    pub(crate) fn get_array_element_type_node(node: &Arc<Node>) -> Option<Arc<Node>> {
        match &node.data {
            NodeData::ParenthesizedTypeNode(d) => Self::get_array_element_type_node(&d.type_node),
            NodeData::TupleTypeNode(d) => {
                if d.elements.len() == 1 {
                    let elem = d.elements.iter().next();
                    if let Some(NodeData::RestTypeNode(rd)) = elem.map(|e| &e.data) {
                        return Self::get_array_element_type_node(&rd.type_node);
                    }
                    if let Some(NodeData::NamedTupleMember(nd)) = elem.map(|e| &e.data) {
                        if nd.dot_dot_dot_token.is_some() {
                            return Self::get_array_element_type_node(&nd.type_node);
                        }
                    }
                }
                None
            }
            NodeData::ArrayTypeNode(d) => Some(Arc::clone(&d.element_type)),
            _ => None,
        }
    }

    pub(crate) fn get_tuple_element_flags(&self, node: &Arc<Node>) -> ElementFlags {
        match &node.data {
            NodeData::OptionalTypeNode(_) => ElementFlags::Optional,
            NodeData::RestTypeNode(rd) => {
                if Self::get_array_element_type_node(&rd.type_node).is_some() {
                    ElementFlags::Rest
                } else {
                    ElementFlags::Variadic
                }
            }
            NodeData::NamedTupleMember(nd) => {
                if nd.question_token.is_some() {
                    ElementFlags::Optional
                } else if nd.dot_dot_dot_token.is_some() {
                    if Self::get_array_element_type_node(&nd.type_node).is_some() {
                        ElementFlags::Rest
                    } else {
                        ElementFlags::Variadic
                    }
                } else {
                    ElementFlags::Required
                }
            }
            _ => ElementFlags::Required,
        }
    }

    pub(crate) fn get_tuple_element_info(&self, node: &Arc<Node>) -> TupleElementInfo {
        let label = match &node.data {
            NodeData::NamedTupleMember(nd) => Some(nd.name.text().to_string()),
            _ => None,
        };
        let labeled_declaration = match &node.data {
            NodeData::NamedTupleMember(_) => Some(Arc::clone(node)),
            _ => node
                .parent()
                .as_ref()
                .filter(|_| node.kind == SyntaxKind::Parameter)
                .cloned(),
        };
        TupleElementInfo {
            label,
            flags: self.get_tuple_element_flags(node),
            labeled_declaration,
            type_: None,
        }
    }

    pub(crate) fn create_tuple_type_ex(
        &mut self,
        element_types: Vec<Arc<Type>>,
        mut element_infos: Vec<TupleElementInfo>,
        readonly: bool,
    ) -> Arc<Type> {
        if element_infos.len() == 1 && element_infos[0].flags.contains(ElementFlags::Rest) {
            let elem = element_types.into_iter().next().unwrap_or_else(|| self.any_type());
            return self.create_array_type(elem);
        }
        for (i, info) in element_infos.iter_mut().enumerate() {
            info.type_ = element_types.get(i).cloned();
        }
        let min_length = element_infos
            .iter()
            .filter(|e| e.flags.intersects(ElementFlags::Required | ElementFlags::Variadic))
            .count();
        let fixed_length = element_infos
            .iter()
            .filter(|e| e.flags.intersects(ElementFlags::Required | ElementFlags::Optional))
            .count();
        let combined_flags = element_infos
            .iter()
            .fold(ElementFlags::None, |acc, e| acc | e.flags);
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Tuple,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Tuple(TupleTypeData {
                interface_data: Default::default(),
                element_infos,
                min_length,
                fixed_length,
                combined_flags,
                readonly,
            }),
        })
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
        let mut parent: Option<Arc<Node>> = decl.parent();
        while matches!(
            parent.as_ref().map(|p| p.kind),
            Some(SyntaxKind::ParenthesizedExpression)
        ) {
            prev = parent.clone().expect("checked Some above");
            parent = prev.parent();
        }
        let Some(parent) = parent else {
            return false;
        };
        if parent.kind != SyntaxKind::CallExpression {
            return false;
        }
        let tsox_frontend::ast::NodeData::CallExpression(call) = &parent.data else {
            return false;
        };

        Arc::ptr_eq(&call.expression, &prev) && parameter_count > call.arguments.nodes.len()
    }
}
