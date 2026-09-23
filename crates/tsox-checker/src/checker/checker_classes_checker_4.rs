#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn check_parameter_property_modifiers(
        &mut self,
        params: &NodeList,
        is_ctor_impl: bool,
    ) {
        for param in params.iter() {
            let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };

            if pd.modifiers.is_some() {
                self.check_grammar_modifiers(param);
            }
            self.check_node_decorators(param);
            let Some(modifiers) = &pd.modifiers else {
                continue;
            };
            // Go checkParameter：参数属性在 erasableSyntaxOnly 下报 TS1294
            //（整参数 span，先于构造器实现位置检查）
            if modifiers.modifier_flags.intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            ) {
                self.erasable_syntax_error(param, param.loc);
            }
            if is_ctor_impl {
                continue;
            }
            if modifiers.modifier_flags.intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            ) {
                let file = self.current_file.clone();
                let diagnostic = tsox_frontend::ast::Diagnostic::new(
                    file,
                    param.loc,
                    tsox_core::diagnostics::messages_generated::
                        A_PARAMETER_PROPERTY_IS_ONLY_ALLOWED_IN_A_CONSTRUCTOR_IMPLEMENTATION,
                    Vec::new(),
                );
                self.diagnostics.add(diagnostic);
            }
        }
    }

    pub(crate) fn check_type_annotation(&mut self, tn: &Arc<Node>) {
        self.with_declaring_file_context(tn, |c| c.check_type_annotation_inner(tn));
    }

    pub(crate) fn check_type_annotation_inner(&mut self, tn: &Arc<Node>) {
        match tn.kind {
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType => {
                let (params, return_type): (&NodeList, Option<&Arc<Node>>) = match &tn.data {
                    tsox_frontend::ast::NodeData::FunctionTypeNode(d) => {
                        (&d.parameters, d.type_node.as_ref())
                    }
                    tsox_frontend::ast::NodeData::ConstructorTypeNode(d) => {
                        (&d.parameters, d.type_node.as_ref())
                    }
                    _ => return,
                };
                self.check_parameter_property_modifiers(params, false);
                self.check_parameter_implicit_any(tn, params, 0);
                for p in params.iter() {
                    if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &p.data
                        && let Some(pt) = &pd.type_node
                    {
                        self.check_type_annotation(pt);
                    }
                }
                if let Some(rt) = return_type {
                    self.check_type_annotation(rt);
                }
            }
            SyntaxKind::TypeReference => {
                if let tsox_frontend::ast::NodeData::TypeReferenceNode(d) = &tn.data
                    && let Some(args) = &d.type_arguments
                {
                    for a in args.iter() {
                        self.check_type_annotation(a);
                    }
                }
            }
            SyntaxKind::UnionType | SyntaxKind::IntersectionType => {
                if let tsox_frontend::ast::NodeData::UnionTypeNode(d) = &tn.data {
                    for t in d.types.iter() {
                        self.check_type_annotation(t);
                    }
                }
                if let tsox_frontend::ast::NodeData::IntersectionTypeNode(d) = &tn.data {
                    for t in d.types.iter() {
                        self.check_type_annotation(t);
                    }
                }
            }
            SyntaxKind::ParenthesizedType => {
                if let tsox_frontend::ast::NodeData::ParenthesizedTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.type_node);
                }
            }
            SyntaxKind::ArrayType | SyntaxKind::TypeOperator => {
                if let tsox_frontend::ast::NodeData::ArrayTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.element_type);
                }
                if let tsox_frontend::ast::NodeData::TypeOperatorNode(d) = &tn.data {
                    self.check_type_annotation(&d.type_node);
                }
            }
            SyntaxKind::TupleType => {
                if let tsox_frontend::ast::NodeData::TupleTypeNode(d) = &tn.data {
                    for t in d.elements.iter() {
                        self.check_type_annotation(t);
                    }
                }
            }
            SyntaxKind::IndexedAccessType => {
                if let tsox_frontend::ast::NodeData::IndexedAccessTypeNode(d) = &tn.data {
                    self.check_type_annotation(&d.object_type);
                    self.check_type_annotation(&d.index_type);

                    self.check_indexed_access_index_type(tn);
                }
            }
            SyntaxKind::TypeLiteral => {
                if let tsox_frontend::ast::NodeData::TypeLiteralNode(d) = &tn.data {
                    for member in d.members.iter() {
                        if matches!(
                            member.kind,
                            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
                        ) {
                            self.check_accessor_in_type_context(member);
                        }
                        // Go checkTypeLiteral → checkSourceElements：成员计算名
                        // 须解析表达式（TS2304/TS2464）
                        if let Some(name) = member.name()
                            && name.kind == SyntaxKind::ComputedPropertyName
                        {
                            self.check_computed_property_name(&name);
                        }
                        if let tsox_frontend::ast::NodeData::PropertySignatureDeclaration(psd) =
                            &member.data
                            && psd.type_node.kind == SyntaxKind::MissingDeclaration
                            && self.no_implicit_any
                        {
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                self.current_file.clone(),
                                member.loc,
                                tsox_core::diagnostics::messages_generated::
                                    MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE,
                                vec![
                                    member
                                        .name()
                                        .map(|n| n.text().to_string())
                                        .unwrap_or_default(),
                                    "any".to_string(),
                                ],
                            ));
                        }
                        let member_params: Option<&NodeList> = match &member.data {
                            tsox_frontend::ast::NodeData::MethodSignatureDeclaration(md) => {
                                Some(&md.parameters)
                            }
                            tsox_frontend::ast::NodeData::CallSignatureDeclaration(cd) => {
                                Some(&cd.parameters)
                            }
                            tsox_frontend::ast::NodeData::ConstructSignatureDeclaration(cd) => {
                                Some(&cd.parameters)
                            }
                            _ => None,
                        };
                        let member_return: Option<&Arc<Node>> = match &member.data {
                            tsox_frontend::ast::NodeData::CallSignatureDeclaration(cd) => {
                                cd.type_node.as_ref()
                            }
                            tsox_frontend::ast::NodeData::ConstructSignatureDeclaration(cd) => {
                                cd.type_node.as_ref()
                            }
                            _ => None,
                        };
                        if let Some(params) = member_params {
                            self.check_parameter_property_modifiers(params, false);
                            self.check_parameter_implicit_any(member, params, 0);
                            for p in params.iter() {
                                if let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) =
                                    &p.data
                                    && let Some(pt) = &pd.type_node
                                {
                                    self.check_type_annotation(pt);
                                }
                            }
                            if self.no_implicit_any
                                && member_return.is_none()
                                && matches!(
                                    member.kind,
                                    SyntaxKind::CallSignature | SyntaxKind::ConstructSignature
                                )
                            {
                                let message = if member.kind == SyntaxKind::ConstructSignature {
                                    tsox_core::diagnostics::messages_generated::
                                        CONSTRUCT_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                                } else {
                                    tsox_core::diagnostics::messages_generated::
                                        CALL_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE
                                };
                                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    member.loc,
                                    message,
                                    vec![],
                                ));
                            }
                        }
                    }
                    // Go checkTypeForDuplicateIndexSignatures：同键索引签名
                    // 按参数型分组，重复组逐声明报 2374
                    let mut index_groups: Vec<(u32, Arc<Type>, Vec<Arc<Node>>)> = Vec::new();
                    for member in d.members.iter() {
                        let tsox_frontend::ast::NodeData::IndexSignatureDeclaration(sd) =
                            &member.data
                        else {
                            continue;
                        };
                        let Some(param) = sd.parameters.iter().next() else {
                            continue;
                        };
                        let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data
                        else {
                            continue;
                        };
                        let Some(pt) = &pd.type_node else {
                            continue;
                        };
                        let key_type = self.get_type_from_type_node(pt);
                        match index_groups
                            .iter_mut()
                            .find(|(id, _, _)| *id == key_type.id)
                        {
                            Some((_, _, decls)) => decls.push(Arc::clone(member)),
                            None => {
                                index_groups.push((key_type.id, key_type, vec![Arc::clone(member)]))
                            }
                        }
                    }
                    for (_, key_type, decls) in &index_groups {
                        if decls.len() > 1 {
                            let key_str = self.type_to_string(key_type);
                            for decl in decls {
                                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    decl.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        DUPLICATE_INDEX_SIGNATURE_FOR_TYPE_0,
                                    vec![key_str.clone()],
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
