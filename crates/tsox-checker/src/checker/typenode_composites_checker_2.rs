#![allow(unused_imports)]

use crate::checker::typenode_composites::*;

impl Checker {
    pub(crate) fn get_type_from_type_literal_or_function_or_constructor_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_type_literal_or_function_or_constructor_type_node"); 
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        if matches!(&node.data, NodeData::TypeLiteralNode(_)) {
            self.check_type_literal_duplicate_declarations(node);
            // Go checkSignatureDeclaration：类型字面量成员的索引签名文法
            if let NodeData::TypeLiteralNode(data) = &node.data {
                for member in data.members.iter() {
                    if member.kind == SyntaxKind::IndexSignature {
                        self.check_grammar_index_signature(member);
                    }
                }
            }
        }
        self.cache_type(node, self.error_type());
        let result = match &node.data {
            NodeData::TypeLiteralNode(data) => {
                let symbol = self.program.symbol_map().symbol_of(node).map(Arc::clone);
                // Go getTypeFromTypeLiteralOrFunctionOrConstructorTypeNode
                //（checker.go:24277-24284）：符号缺失或成员空且非别名体的类型
                // 字面量取 emptyTypeLiteralType 单例——X & {} 交集的空对象
                // 归约（addTypeToIntersection 空对象臂 + removeRedundantSupertypes）
                // 以该形态为锚；别名附加只在成员构造臂（else 分支 t.alias = alias），
                // 单例共享对象不得写入别名
                let has_alias = self.alias_symbol_for_type_node(node).is_some();
                // 成员空判据取节点成员表（Go binder 建符号时把成员记入
                // symbol.Members，本侧 binder 不落 members、成员由节点惰性
                // 构建——build_interface_type_from_members_with_symbol 的同源）
                let members_empty = data.members.is_empty();
                if symbol.is_none() || (members_empty && !has_alias) {
                    self.empty_type_literal_type()
                } else {
                    let t = self.build_interface_type_from_members_with_symbol(&data.members, symbol.clone(), symbol.as_ref());
                    self.attach_alias_for_type_node(node, &t);
                    t
                }
            }
            NodeData::FunctionTypeNode(_) => {
                let t = self.get_type_from_function_type_node(node);
                self.attach_alias_for_type_node(node, &t);
                t
            }
            NodeData::ConstructorTypeNode(_) => {
                let t = self.get_type_from_constructor_type_node(node);
                self.attach_alias_for_type_node(node, &t);
                t
            }
            _ => self.error_type(),
        };
        self.cache_type_overwrite_error(node, result.clone());
        result
    }

    #[allow(dead_code)]
    pub(crate) fn get_type_from_type_literal_members(
        &mut self,
        members: &Arc<NodeList>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_type_literal_members"); 
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
                    let optional = data.postfix_token.is_some();
                    let mut flags = SymbolFlags::Property;
                    if optional {
                        flags |= SymbolFlags::Optional;
                    }
                    let symbol = Arc::new(Symbol::new(flags, name.clone()));
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
                    // Go resolveStructuredTypeMembers：同键索引签名去重（重
                    // 复声明由 2374 检查报错，类型只保留一个）
                    let key_id = key_type.as_ref().map(|k| k.id);
                    if key_id.is_some_and(|id| {
                        index_infos
                            .iter()
                            .any(|info| info.key_type.as_ref().is_some_and(|k| k.id == id))
                    }) {
                        continue;
                    }
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
            data: TypeData::Object(ObjectTypeData { node: None,
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

    pub(crate) fn get_type_from_function_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_function_type_node"); 
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
                let symbol = self.anonymous_type_literal_symbol(node);
                self.create_function_or_constructor_type_ex(vec![sig], false, symbol)
            }
            _ => self.error_type(),
        }
    }

    pub(crate) fn get_type_from_constructor_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_constructor_type_node"); 
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
                let symbol = self.anonymous_type_literal_symbol(node);
                self.create_function_or_constructor_type_ex(vec![sig], true, symbol)
            }
            _ => self.error_type(),
        }
    }

    fn anonymous_type_literal_symbol(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("anonymous_type_literal_symbol"); 
        if let Some(existing) = self.program.symbol_map().symbol_of(node)
            && existing
                .flags
                .contains(tsox_frontend::ast::SymbolFlags::TypeLiteral)
        {
            return Some(Arc::clone(existing));
        }
        let mut symbol = Symbol::new(
            tsox_frontend::ast::SymbolFlags::TypeLiteral,
            tsox_frontend::ast::INTERNAL_SYMBOL_NAME_TYPE,
        );
        symbol.declarations = vec![Arc::clone(node)];
        Some(Arc::new(symbol))
    }

    pub(crate) fn get_array_element_type_node(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_array_element_type_node"); 
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

    pub(crate) fn get_tuple_element_flags(&self, node: &Arc<Node>) -> ElementFlags { ::tsox_core::fntrace::enter("get_tuple_element_flags"); 
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

    pub(crate) fn get_tuple_element_info(&self, node: &Arc<Node>) -> TupleElementInfo { ::tsox_core::fntrace::enter("get_tuple_element_info"); 
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
        element_infos: Vec<TupleElementInfo>,
        readonly: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("create_tuple_type_ex"); 
        // Go getTupleTargetType（checker.go:26218-26224）：单 Rest 元素即
        // [...X[]] ≡ X[]，取数组
        if element_infos.len() == 1 && element_infos[0].flags.contains(ElementFlags::Rest) {
            let elem = element_types.into_iter().next().unwrap_or_else(|| self.any_type());
            return self.create_array_type(elem);
        }
        // Go createTupleTypeEx（checker.go:26204-26214）→
        // createNormalizedTypeReference（24656-24662）→
        // createNormalizedTupleTypeEx（24664-24711）：target 带 NonRequired 位时
        // TupleNormalizer.normalize 归一，Variadic 非泛型/非元组元素按数组处理取
        // number 索引型改标 Rest（24755-24764），归一后的 infos 重走
        // getTupleTargetType（24703），单 Rest 收敛为数组
        let mut combined_flags = ElementFlags::None;
        let mut variadic_position_holds_union = false;
        for (i, e) in element_infos.iter().enumerate() {
            combined_flags |= e.flags;
            if e.flags.contains(ElementFlags::Variadic)
                && element_types
                    .get(i)
                    .is_some_and(|t| t.flags.intersects(TypeFlags::Never | TypeFlags::Union))
            {
                variadic_position_holds_union = true;
            }
        }
        let mut built_types = element_types;
        let mut built_infos = element_infos;
        if combined_flags.intersects(crate::checker::types_type_flags_instantiable_non_primitive::ELEMENT_FLAGS_NON_REQUIRED)
            && !variadic_position_holds_union
        {
            let mut n = crate::checker::mig::m2b_2::TupleNormalizer {
                types: Vec::new(),
                infos: Vec::new(),
                last_required_index: -1,
                first_rest_index: -1,
                last_optional_or_rest_index: -1,
            };
            let infos_len = built_infos.len();
            if !n.normalize(self, &built_types[..infos_len], &built_infos) {
                return self.error_type();
            }
            if built_types.len() > infos_len {
                n.types.push(built_types[infos_len].clone());
            }
            if n.infos.len() == 1 && n.infos[0].flags.contains(ElementFlags::Rest) {
                return self.create_array_type_ex(n.types[0].clone(), readonly);
            }
            built_types = n.types;
            built_infos = n.infos;
        }
        for (i, info) in built_infos.iter_mut().enumerate() {
            info.type_ = built_types.get(i).cloned();
        }
        let min_length = built_infos
            .iter()
            .filter(|e| e.flags.intersects(ElementFlags::Required | ElementFlags::Variadic))
            .count();
        let fixed_length = built_infos
            .iter()
            .filter(|e| e.flags.intersects(ElementFlags::Required | ElementFlags::Optional))
            .count();
        let combined_flags = built_infos
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
                element_infos: built_infos,
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
    ) -> bool { ::tsox_core::fntrace::enter("iife_with_too_few_arguments"); 
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
