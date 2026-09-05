use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{
    ModifierFlags, Node, NodeList, Symbol, SymbolFlags, SymbolTable,
    SyntaxKind,
};

use crate::checker::checker::Checker;


use super::*;


impl Checker {
    pub(crate) fn get_type_from_array_or_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        match &node.data {
            NodeData::ArrayTypeNode(d) => {
                let elem_type = self.get_type_from_type_node(&d.element_type);
                self.create_array_type(elem_type)
            }
            NodeData::TupleTypeNode(d) => {
                let mut element_types = Vec::new();

                let mut variadic_types: Vec<Arc<Type>> = Vec::new();
                let mut has_variadic_union = false;
                for elem in d.elements.iter() {
                    if let NodeData::RestTypeNode(rd) = &elem.data {
                        let inner = Arc::clone(&rd.type_node);
                        let inner_t = self.get_type_from_type_node(&inner);
                        has_variadic_union |= inner_t.flags.contains(TypeFlags::Never)
                            || matches!(&inner_t.data, TypeData::Union(_));
                        variadic_types.push(inner_t);
                    }
                    element_types.push(self.get_type_from_type_node(elem));
                }
                if has_variadic_union
                    && !self.check_cross_product_union(node, &variadic_types)
                {
                    return self.error_type();
                }
                self.create_tuple_type(element_types)
            }
            _ => self.error_type(),
        }
    }

    pub(crate) fn check_type_reference_arguments(
        &mut self,
        _node: &Arc<Node>,
        type_name: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {

        let params: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .find_map(|d| {
                let tps = match &d.data {
                    NodeData::InterfaceDeclaration(i) => i.type_parameters.as_ref(),
                    NodeData::ClassDeclaration(c) => c.type_parameters.as_ref(),
                    NodeData::TypeAliasDeclaration(t) => t.type_parameters.as_ref(),
                    _ => None,
                }?;
                Some(tps.iter().cloned().collect())
            })
            .unwrap_or_default();
        if params.is_empty() {
            return true;
        }
        let provided: Vec<Arc<Node>> = type_name
            .parent
            .as_ref()
            .and_then(|p| match &p.data {
                NodeData::TypeReferenceNode(tr) => tr.type_arguments.clone(),

                NodeData::ExpressionWithTypeArguments(e) => e.type_arguments.clone(),
                _ => None,
            })
            .map(|list| list.iter().cloned().collect())
            .unwrap_or_default();

        let required = params
            .iter()
            .rposition(|p| {
                !matches!(&p.data, NodeData::TypeParameterDeclaration(d) if d.default_type.is_some())
            })
            .map_or(0, |i| i + 1);
        let file = self
            .get_source_file_of_node(type_name)
            .or_else(|| self.current_file.clone());
        if provided.len() < required || provided.len() > params.len() {
            let display = format!(
                "{}<{}>",
                symbol.name,
                params
                    .iter()
                    .filter_map(|p| p.name().map(|n| n.text().to_string()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2314 && d.loc == type_name.loc);
            if !already {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    crate::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_1_TYPE_ARGUMENT_S,
                    vec![display, params.len().to_string()],
                ));
            }

            return false;
        }

        for (i, arg_node) in provided.iter().enumerate() {
            let Some(param) = params.get(i) else { continue };
            let NodeData::TypeParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let Some(constraint_node) = &pd.constraint else {
                continue;
            };

            let param_names: Vec<String> = params
                .iter()
                .filter_map(|p| p.name().map(|n| n.text().to_string()))
                .collect();
            if type_node_references_names(constraint_node, &param_names) {
                continue;
            }
            let arg_type = self.get_type_from_type_node(arg_node);

            if arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                || arg_type.is_type_parameter()
            {
                continue;
            }
            let constraint_type = self.get_type_from_type_node(constraint_node);
            if constraint_type.flags.intersects(TypeFlags::Any | TypeFlags::Never) {
                continue;
            }
            if self.is_type_assignable_to(&arg_type, &constraint_type) {
                continue;
            }

            let primitive_like = |t: &Arc<Type>| {
                t.flags.intersects(
                    TypeFlags::String
                        | TypeFlags::Number
                        | TypeFlags::Boolean
                        | TypeFlags::BigInt
                        | TypeFlags::ESSymbol
                        | TypeFlags::Enum
                        | TypeFlags::StringLiteral
                        | TypeFlags::NumberLiteral
                        | TypeFlags::BooleanLiteral
                        | TypeFlags::EnumLiteral
                        | TypeFlags::Null
                        | TypeFlags::Undefined,
                )
            };
            let object_like = |t: &Arc<Type>| {
                t.flags.contains(TypeFlags::Object)
                    && t.as_structured().is_some()
                    && !t.object_flags.contains(ObjectFlags::Tuple)
                    && !t.object_flags.contains(ObjectFlags::Reference)
                    && t.as_structured()
                        .is_some_and(|s| s.call_signature_count == 0)
            };
            let clear_cut = (primitive_like(&arg_type) && (primitive_like(&constraint_type) || object_like(&constraint_type)))
                || (object_like(&arg_type) && object_like(&constraint_type));
            if !clear_cut {
                continue;
            }

            {
                let ap = Arc::as_ptr(&arg_type) as *const Type as usize;
                let cp = Arc::as_ptr(&constraint_type) as *const Type as usize;
                if self.degraded_type_ptrs.contains(&ap)
                    || self.degraded_type_ptrs.contains(&cp)
                {
                    continue;
                }
            }
            let arg_str = self.type_to_string(&arg_type);
            let constraint_str = self.type_to_string(&constraint_type);
            let mut diag = crate::ast::Diagnostic::new(
                file.clone(),
                arg_node.loc,
                crate::diagnostics::messages_generated::TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1,
                vec![arg_str.clone(), constraint_str.clone()],
            );

            let missing = self.get_missing_required_properties(&arg_type, &constraint_type);
            if missing.len() == 1 {
                diag.message_chain.push(crate::ast::Diagnostic::new(
                    None,
                    arg_node.loc,
                    crate::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), arg_str.clone(), constraint_str.clone()],
                ));
            }
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2344 && d.loc == arg_node.loc);
            if !already {
                self.diagnostics.add(diag);
            }
        }
        true
    }

    pub(crate) fn get_type_from_optional_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("OptionalType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.add_optionality(&t)
    }

    pub(crate) fn get_type_from_union_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::UnionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }
        let result = self.get_union_type(member_types);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_intersection_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let types = match &node.data {
            NodeData::IntersectionTypeNode(data) => &data.types,
            _ => return self.error_type(),
        };
        let mut member_types = Vec::new();
        for member in types.iter() {
            member_types.push(self.get_type_from_type_node(member));
        }

        let has_union = member_types
            .iter()
            .any(|t| matches!(&t.data, TypeData::Union(_)));
        if has_union {
            let all_undefined = member_types.iter().all(|t| {
                let TypeData::Union(u) = &t.data else {
                    return false;
                };
                u.union_or_intersection
                    .types
                    .first()
                    .is_some_and(|f| f.flags.contains(TypeFlags::Undefined))
            });
            let all_null = member_types.iter().all(|t| {
                let TypeData::Union(u) = &t.data else {
                    return false;
                };
                u.union_or_intersection
                    .types
                    .iter()
                    .take(2)
                    .any(|f| f.flags.contains(TypeFlags::Null))
            });
            if !all_undefined
                && !all_null
                && !self.check_cross_product_union(node, &member_types)
            {
                let e = self.error_type();
                self.cache_type(node, e.clone());
                return e;
            }
        }
        let result = self.get_intersection_type(member_types);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn get_type_from_named_tuple_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("NamedTupleMember has type").clone();
        self.get_type_from_type_node(&inner)
    }

    pub(crate) fn get_type_from_rest_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let inner = node.type_node().expect("RestType has type").clone();
        let t = self.get_type_from_type_node(&inner);
        self.create_array_type(t)
    }

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
    pub(crate) fn get_type_from_type_literal_members(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
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
            id: 0,
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
