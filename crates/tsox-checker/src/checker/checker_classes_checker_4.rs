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
            let Some(modifiers) = &pd.modifiers else {
                continue;
            };
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

    pub(crate) fn check_parameter_implicit_any(
        &mut self,
        node: &Arc<Node>,
        params: &NodeList,
        contextual_param_count: usize,
    ) {
        if !self.no_implicit_any {
            return;
        }
        for (i, param) in params.iter().enumerate() {
            let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.type_node.is_some() || pd.initializer.is_some() {
                continue;
            }
            let name = &pd.name;
            if name.kind != SyntaxKind::Identifier || name.text() == "this" {
                continue;
            }

            if i < contextual_param_count {
                continue;
            }

            if self.param_has_typed_jsdoc_tag(node, name.text()) {
                continue;
            }
            if self.declaration_belongs_to_private_ambient_member(param) {
                continue;
            }
            if self.contextual_type_of_parameter(param).is_some() {
                continue;
            }
            if matches!(
                node.kind,
                SyntaxKind::FunctionType | SyntaxKind::MethodSignature | SyntaxKind::CallSignature
            ) && self.parameter_name_resolves_as_type(name)
            {
                let name_text = name.text().to_string();
                let arg_name = format!("arg{i}");
                let type_name = format!(
                    "{name_text}{}",
                    if pd.dot_dot_dot_token.is_some() { "[]" } else { "" }
                );
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    param.loc,
                    tsox_core::diagnostics::messages_generated::
                        PARAMETER_HAS_A_NAME_BUT_NO_TYPE_DID_YOU_MEAN_0_COLON_1,
                    vec![arg_name, type_name],
                ));
                continue;
            }
            let file = self.current_file.clone();
            let name_text = name.text().to_string();
            let diagnostic = if pd.dot_dot_dot_token.is_some() {
                tsox_frontend::ast::Diagnostic::new(
                    file,
                    param.loc,
                    tsox_core::diagnostics::messages_generated::
                        REST_PARAMETER_0_IMPLICITLY_HAS_AN_ANY_TYPE,
                    vec![name_text],
                )
            } else {
                tsox_frontend::ast::Diagnostic::new(
                    file,
                    param.loc,
                    tsox_core::diagnostics::messages_generated::PARAMETER_0_IMPLICITLY_HAS_AN_1_TYPE,
                    vec![name_text, "any".to_string()],
                )
            };
            self.diagnostics.add(diagnostic);
        }
    }

    pub(crate) fn declaration_belongs_to_private_ambient_member(&self, decl: &Arc<Node>) -> bool {
        let mut member = Arc::clone(decl);
        loop {
            match member.kind {
                SyntaxKind::BindingElement | SyntaxKind::VariableDeclaration => {
                    let Some(parent) = member.parent() else { break };
                    member = parent;
                }
                _ => break,
            }
        }
        if member.kind == SyntaxKind::Parameter
            && let Some(parent) = member.parent()
        {
            member = parent;
        }
        let is_private = member.has_syntactic_modifier(ModifierFlags::Private)
            || member
                .name()
                .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier);
        let is_ambient = self.ambient_context_depth > 0
            || member.has_syntactic_modifier(ModifierFlags::Ambient)
            || {
                let mut anc = member.parent();
                let mut found = false;
                while let Some(a) = anc {
                    if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                        found = true;
                        break;
                    }
                    anc = a.parent();
                }
                found
            }
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
        is_private && is_ambient
    }

    fn parameter_name_resolves_as_type(&self, name: &Arc<Node>) -> bool {
        const TYPE_KEYWORD_NAMES: &[&str] = &[
            "any", "unknown", "never", "void", "undefined", "string", "number", "boolean",
            "bigint", "object", "symbol",
        ];
        let text = name.text();
        if TYPE_KEYWORD_NAMES.contains(&text) {
            return true;
        }
        self.resolve_identifier_with_meaning(name, SymbolFlags::TYPE)
            .is_some()
    }

    pub(crate) fn param_has_typed_jsdoc_tag(&self, node: &Arc<Node>, param_name: &str) -> bool {
        let Some(file) = &self.current_file else {
            return false;
        };
        for jsdoc in file.resolve_jsdoc(node) {
            let tsox_frontend::ast::NodeData::JSDoc(d) = &jsdoc.data else {
                continue;
            };
            let Some(tags) = &d.tags else { continue };
            for tag in tags.iter() {
                if let tsox_frontend::ast::NodeData::JSDocParameterOrPropertyTag(td) = &tag.data
                    && td.name.kind == SyntaxKind::Identifier
                    && td.name.text() == param_name
                    && td.type_expression.is_some()
                {
                    return true;
                }
            }
        }
        false
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
                }
            }
            _ => {}
        }
    }
}
