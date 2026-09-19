#![allow(unused_imports)]

use crate::checker::typenode_composites::*;

impl Checker {
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
            .parent()
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
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    type_name.loc,
                    tsox_core::diagnostics::messages_generated::GENERIC_TYPE_0_REQUIRES_1_TYPE_ARGUMENT_S,
                    vec![display, params.len().to_string()],
                ));
            }

            return false;
        }

        let param_names: Vec<String> = params
            .iter()
            .filter_map(|p| p.name().map(|n| n.text().to_string()))
            .collect();
        let arg_types: Vec<Arc<Type>> = provided
            .iter()
            .map(|a| self.get_type_from_type_node(a))
            .collect();

        for (i, arg_node) in provided.iter().enumerate() {
            let Some(param) = params.get(i) else { continue };
            let NodeData::TypeParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let Some(constraint_node) = &pd.constraint else {
                continue;
            };

            // Go checkTypeArgumentConstraints：约束经 newTypeMapper(typeParameters,
            // typeArguments) 实例化后再比较；裸标识符引用形参时直接取对应实参型
            let constraint_ref_name = match &constraint_node.data {
                NodeData::TypeReferenceNode(tr)
                    if tr.type_arguments.is_none() && tr.type_name.kind == SyntaxKind::Identifier =>
                {
                    Some(tr.type_name.text().to_string())
                }
                _ => {
                    if constraint_node.kind == SyntaxKind::Identifier {
                        Some(constraint_node.text().to_string())
                    } else {
                        None
                    }
                }
            };
            let constraint_type = if let Some(name) = constraint_ref_name {
                match param_names.iter().position(|n| *n == name) {
                    Some(j) if j < arg_types.len() => Arc::clone(&arg_types[j]),
                    _ => self.get_type_from_type_node(constraint_node),
                }
            } else {
                if type_node_references_names(constraint_node, &param_names) {
                    continue;
                }
                self.get_type_from_type_node(constraint_node)
            };

            let arg_type = Arc::clone(&arg_types[i]);

            if arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                || arg_type.is_type_parameter()
            {
                continue;
            }
            if constraint_type
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Never)
            {
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
            let clear_cut = (primitive_like(&arg_type)
                && (primitive_like(&constraint_type) || object_like(&constraint_type)))
                || (object_like(&arg_type) && object_like(&constraint_type));
            if !clear_cut {
                continue;
            }

            {
                if self.degraded_type_ptrs.contains(&arg_type.id)
                    || self.degraded_type_ptrs.contains(&constraint_type.id)
                {
                    continue;
                }
            }
            let arg_str = self.type_to_string(&arg_type);
            let constraint_str = self.type_to_string(&constraint_type);

            // Go reportRelationError：链条内层已是缺属性错误时头消息被抑制，
            // TS2741/TS2739 升为顶部错误；否则 TS2344 为顶并带首个不兼容属性链
            let missing = self.get_missing_required_properties(&arg_type, &constraint_type);
            let diag = if missing.len() == 1 {
                let prop_name = missing[0].clone();
                let mut d = tsox_frontend::ast::Diagnostic::new(
                    file.clone(),
                    arg_node.loc,
                    tsox_core::diagnostics::messages_generated::
                        PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![prop_name.clone(), arg_str.clone(), constraint_str.clone()],
                );
                if let Some(sym) = self.get_property_of_type(&constraint_type, &prop_name)
                    && let Some(decl) = sym.declarations.first()
                {
                    let related_file = self
                        .get_source_file_of_node(decl)
                        .or_else(|| file.clone());
                    d.related_information.push(tsox_frontend::ast::Diagnostic::new(
                        related_file,
                        decl.loc,
                        tsox_core::diagnostics::messages_generated::X_0_IS_DECLARED_HERE,
                        vec![prop_name],
                    ));
                }
                d
            } else if missing.len() > 1 {
                let names = missing.join(", ");
                if missing.len() > 5 {
                    tsox_frontend::ast::Diagnostic::new(
                        file.clone(),
                        arg_node.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                        vec![
                            arg_str.clone(),
                            constraint_str.clone(),
                            missing[..4].join(", "),
                            (missing.len() - 4).to_string(),
                        ],
                    )
                } else {
                    tsox_frontend::ast::Diagnostic::new(
                        file.clone(),
                        arg_node.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                        vec![arg_str.clone(), constraint_str.clone(), names],
                    )
                }
            } else {
                let mut d = tsox_frontend::ast::Diagnostic::new(
                    file.clone(),
                    arg_node.loc,
                    tsox_core::diagnostics::messages_generated::TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1,
                    vec![arg_str.clone(), constraint_str.clone()],
                );
                if let Some(prop) =
                    self.first_incompatible_property(&arg_type, &constraint_type)
                {
                    let (name, src_t, tgt_t) = prop;
                    d.message_chain.push(tsox_frontend::ast::Diagnostic::new(
                        None,
                        arg_node.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                        vec![name.clone()],
                    ));
                    let src_str = self.type_to_string(&src_t);
                    let tgt_str = self.type_to_string(&tgt_t);
                    d.message_chain[0].message_chain.push(
                        tsox_frontend::ast::Diagnostic::new(
                            None,
                            arg_node.loc,
                            tsox_core::diagnostics::messages_generated::
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                            vec![src_str, tgt_str],
                        ),
                    );
                }
                d
            };
            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.loc == arg_node.loc && (d.code == 2344 || d.code == 2741 || d.code == 2739 || d.code == 2740));
            if !already {
                self.diagnostics.add(diag);
            }
        }
        true
    }

    fn first_incompatible_property(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<(String, Arc<Type>, Arc<Type>)> {
        for prop in self.get_properties_of_type(target) {
            let Some(src_prop) = self.get_property_of_type(source, &prop.name) else {
                continue;
            };
            let src_t = self.get_type_of_symbol(&src_prop);
            let tgt_t = self.get_type_of_symbol(&prop);
            if !self.is_type_assignable_to(&src_t, &tgt_t) {
                return Some((prop.name.clone(), src_t, tgt_t));
            }
        }
        None
    }
}
