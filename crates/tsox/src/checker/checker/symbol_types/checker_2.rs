#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_symbol_declared_type_on_demand(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        use crate::ast::NodeData;
        let decl = symbol
            .value_declaration
            .clone()
            .or_else(|| symbol.declarations.first().cloned())?;
        let type_node_and_init: (Option<Arc<Node>>, Option<Arc<Node>>) = match &decl.data {
            NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertyDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertySignatureDeclaration(d) => (Some(Arc::clone(&d.type_node)), None),
            NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            _ => return None,
        };
        if type_node_and_init.0.is_none() && type_node_and_init.1.is_none() {
            if decl.kind == SyntaxKind::VariableDeclaration {
                let placeholder = self.get_any_type();
                let existing = self
                    .value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type
                    .replace(placeholder);
                let t = self.initial_type_of_declaration(&decl);
                match &t {
                    Some(t) => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type =
                            Some(Arc::clone(t));
                    }
                    None => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
                    }
                }
                return t;
            }
            return None;
        }

        let placeholder = self.get_any_type();
        let existing = self
            .value_symbol_links
            .get_or_default(symbol)
            .resolved_type
            .replace(placeholder);
        let result = self.with_declaring_file_context(&decl, |checker| {
            let (type_node, initializer) = match &decl.data {
                NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                NodeData::PropertyDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                NodeData::PropertySignatureDeclaration(d) => (Some(Arc::clone(&d.type_node)), None),
                NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                _ => (None, None),
            };
            if let Some(tn) = type_node {
                Some(checker.get_type_from_type_node(&tn))
            } else {
                let owner_class = match &decl.data {
                    NodeData::PropertyDeclaration(_) => decl
                        .parent
                        .as_ref()
                        .filter(|p| {
                            matches!(
                                p.kind,
                                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                            )
                        })
                        .cloned(),
                    _ => None,
                };
                if let Some(class) = owner_class.as_ref() {
                    let this_type = checker.build_class_instance_type_with_base(class);
                    checker.this_type_stack.push(this_type);
                }
                let t = initializer.map(|init| {
                    if !checker
                        .get_combined_node_flags(&decl)
                        .intersects(NodeFlags::Constant)
                        && matches!(
                            init.kind,
                            SyntaxKind::NullKeyword | SyntaxKind::UndefinedKeyword
                        )
                    {
                        return checker.auto_type();
                    }
                    if checker.is_empty_array_literal(&init) {
                        return checker.auto_array_type();
                    }
                    let raw = checker.get_type_of_node(&init);
                    let widened_literal =
                        checker.get_widened_literal_type_for_initializer(&decl, &raw);
                    let regularized = checker.get_regular_type_of_literal_type(&widened_literal);
                    checker.widen_initializer_type(&regularized)
                });
                if owner_class.is_some() {
                    checker.this_type_stack.pop();
                }
                t
            }
        });

        let result = match (&result, &decl.data) {
            (Some(t), NodeData::ParameterDeclaration(pd))
                if pd.question_token.is_some() && pd.initializer.is_none() =>
            {
                Some(self.add_optional_undefined(Arc::clone(t)))
            }

            (Some(t), NodeData::PropertySignatureDeclaration(psd))
                if psd
                    .postfix_token
                    .as_ref()
                    .is_some_and(|tk| tk.kind == SyntaxKind::QuestionToken) =>
            {
                Some(self.get_optional_type(Arc::clone(t)))
            }
            _ => result,
        };
        match &result {
            Some(t) => {
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(t));
            }
            None => {
                self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
            }
        }
        result
    }

    pub(crate) fn with_declaring_file_context<T>(
        &mut self,
        decl: &Arc<Node>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let saved_file = self.current_file.take();
        let saved_id = self.current_file_id;
        let saved_symbol = self.current_file_symbol.take();
        let mut pushed = 0usize;
        if let Some(file) = self.get_source_file_of_node(decl) {
            self.current_file = Some(Arc::clone(&file));
            self.current_file_id = file.node.id();
            self.current_file_symbol = self.program.symbol_map().symbol_of(&file.node).cloned();

            let mut chain: Vec<Arc<Node>> = Vec::new();
            let mut cur = decl.parent.clone();
            while let Some(n) = cur {
                if matches!(
                    n.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::ModuleDeclaration
                        | SyntaxKind::Block
                        | SyntaxKind::CatchClause
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::MethodSignature
                        | SyntaxKind::CallSignature
                        | SyntaxKind::ConstructSignature
                        | SyntaxKind::FunctionType
                        | SyntaxKind::ConstructorType
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::InterfaceDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                        | SyntaxKind::TypeAliasDeclaration
                        | SyntaxKind::MappedType
                        | SyntaxKind::EnumDeclaration
                ) {
                    chain.push(Arc::clone(&n));
                    if n.kind == SyntaxKind::SourceFile {
                        break;
                    }
                }
                cur = n.parent.clone();
            }
            for scope in chain.iter().rev() {
                self.push_scope(scope);
                pushed += 1;
            }
        }
        let result = f(self);
        for _ in 0..pushed {
            self.pop_scope();
        }
        self.current_file = saved_file;
        self.current_file_id = saved_id;
        self.current_file_symbol = saved_symbol;
        result
    }

    pub(crate) fn get_type_of_merged_namespace_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(cached) = self
            .declared_type_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let value_type = self.get_value_type_of_symbol(symbol);

        let ns_type = self.resolve_namespace_type(symbol);

        let (call_sigs, construct_sigs) = match &value_type.data {
            TypeData::Object(obj) => {
                let cs = obj.structured.call_signatures().to_vec();
                let xs = obj.structured.construct_signatures().to_vec();
                (cs, xs)
            }
            _ => (Vec::new(), Vec::new()),
        };
        let merged = if call_sigs.is_empty() && construct_sigs.is_empty() {
            ns_type
        } else {
            let ns_obj = match &ns_type.data {
                TypeData::Object(obj) => obj,
                _ => {
                    self.declared_type_links
                        .get_or_default(symbol)
                        .declared_type = Some(Arc::clone(&value_type));
                    return value_type;
                }
            };
            let ns_structured = &ns_obj.structured;
            let mut structured = StructuredTypeData::default();
            structured.members = ns_structured.members.clone();
            structured.properties = ns_structured.properties.clone();
            structured.index_infos = ns_structured.index_infos.clone();

            let existing_sigs = ns_structured.signatures.clone();
            let existing_call_count = ns_structured.call_signature_count;
            structured.call_signature_count = call_sigs.len() + existing_call_count;
            structured.signatures = call_sigs;
            structured
                .signatures
                .extend(existing_sigs[..existing_call_count].to_vec());
            structured.signatures.extend(construct_sigs);
            structured
                .signatures
                .extend(existing_sigs[existing_call_count..].to_vec());
            Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: ObjectFlags::Anonymous,
                id: crate::checker::types::next_type_id(),
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::Object(ObjectTypeData {
                    structured,
                    target: None,
                    mapper: None,
                    type_arguments: Vec::new(),
                }),
            })
        };

        self.declared_type_links
            .get_or_default(symbol)
            .declared_type = Some(Arc::clone(&merged));
        merged
    }
}
