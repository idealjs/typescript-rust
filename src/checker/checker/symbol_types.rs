use std::sync::Arc;

use crate::ast::{
    ModifierFlags, Node,
    NodeFlags, NodeList, Symbol, SymbolFlags, SymbolTable, SyntaxKind,
};







use super::*;


impl Checker {
    pub fn get_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

        if symbol.flags.contains(SymbolFlags::Alias) {
            let target = self.follow_alias(symbol);
            if let Some(target) = target
                && !Arc::ptr_eq(&target, symbol)
            {
                let t = self.get_type_of_symbol(&target);
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&t));
                return t;
            }
            return self.get_any_type();
        }

        if symbol.flags.contains(SymbolFlags::ValueModule)
            && (symbol.flags.contains(SymbolFlags::Function)
                || symbol.flags.contains(SymbolFlags::Class)
                || symbol.flags.contains(SymbolFlags::RegularEnum)
                || symbol.flags.contains(SymbolFlags::ConstEnum))
        {
            return self.get_type_of_merged_namespace_symbol(symbol);
        }

        if symbol.flags.contains(SymbolFlags::Prototype) {
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
            let result = self.get_type_of_prototype_property(symbol);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(result.clone());
            return result;
        }

        if symbol.flags.intersects(SymbolFlags::Method)
            && let Some(decl) = symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::MethodDeclaration)
            && let crate::ast::NodeData::MethodDeclaration(data) = &decl.data
        {
            if let Some(links) = self.value_symbol_links.get(symbol)
                && let Some(ref t) = links.resolved_type
            {
                return Arc::clone(t);
            }
            self.push_scope(decl);
            let return_type = match data.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            };
            let sig = self.build_signature_from_function_like_type_node(
                &data.parameters,
                return_type,
                 false,
                 None,
                 Some(Arc::clone(decl)),
            );
            self.pop_scope();
            let t = self.create_function_or_constructor_type(vec![sig], false);
            self.value_symbol_links
                .get_or_default(symbol)
                .resolved_type = Some(Arc::clone(&t));
            return t;
        }

        if symbol.flags.contains(SymbolFlags::BlockScopedVariable)
            || symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            || symbol.flags.contains(SymbolFlags::Function)
            || symbol.flags.contains(SymbolFlags::Class)
            || symbol.flags.contains(SymbolFlags::Property)
            || symbol.flags.contains(SymbolFlags::EnumMember)
        {

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

            if let Some(t) = self.resolve_symbol_declared_type_on_demand(symbol) {
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&t));
                return t;
            }
            self.get_any_type()
        } else if symbol.flags.contains(SymbolFlags::ValueModule) {

            self.resolve_namespace_type(symbol)
        } else if symbol.flags.intersects(SymbolFlags::ENUM) {

            self.resolve_enum_value_type(symbol)
        } else {
            self.get_any_type()
        }
    }

    pub(crate) fn attach_function_expando_type(
        &mut self,
        symbol: &Arc<crate::ast::Symbol>,
        base: Arc<Type>,
    ) -> Arc<Type> {
        let mut entries: Vec<(String, Arc<Node>)> = Vec::new();
        for (name, sym) in symbol.exports.iter() {
            if name == crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT {
                for d in &sym.declarations {

                    let mname = match &d.data {
                        crate::ast::NodeData::BinaryExpression(b) => match &b.left.data {
                            crate::ast::NodeData::ElementAccessExpression(eae) => self
                                .node_source_text(&eae.argument_expression)
                                .map(|t| format!("[{t}]"))
                                .unwrap_or_default(),
                            _ => String::new(),
                        },
                        _ => String::new(),
                    };
                    entries.push((mname, Arc::clone(d)));
                }
            } else if sym.flags.contains(SymbolFlags::Property)
                && !sym.declarations.is_empty()
                && sym
                    .declarations
                    .iter()
                    .all(|d| d.kind == SyntaxKind::BinaryExpression)
            {
                for d in &sym.declarations {
                    entries.push((name.clone(), Arc::clone(d)));
                }
            }
        }
        if entries.is_empty() {
            return base;
        }
        let mut table = crate::ast::SymbolTable::new();
        let mut props: Vec<Arc<crate::ast::Symbol>> = Vec::new();
        for (name, node) in entries {
            if table.entries.contains_key(&name) {
                continue;
            }
            let crate::ast::NodeData::BinaryExpression(bin) = &node.data else {
                continue;
            };
            let rhs_type = self.with_declaring_file_context(&node, |c| {
                let t = c.get_type_of_node(&bin.right);
                c.get_widened_type(&t)
            });
            let prop = Arc::new(crate::ast::Symbol::new(
                SymbolFlags::Property,
                name.clone(),
            ));
            self.value_symbol_links.insert(
                &prop,
                ValueSymbolLinks {
                    resolved_type: Some(rhs_type),
                    ..Default::default()
                },
            );
            table.insert(name.clone(), Arc::clone(&prop));
            props.push(prop);
        }
        if props.is_empty() {
            return base;
        }
        let face = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: 0,
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        Arc::new(Type {
            flags: TypeFlags::Intersection,
            object_flags: ObjectFlags::None,
            id: 0,
            symbol: None,
            alias: None,
            data: TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: vec![base, face],
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn add_optional_undefined(&mut self, t: Arc<Type>) -> Arc<Type> {
        if !self.strict_null_checks {
            return t;
        }

        if t.flags.contains(TypeFlags::Any) && t.intrinsic_name() == Some("error") {
            return t;
        }
        let already = t.flags.contains(TypeFlags::Undefined)
            || (t.flags.contains(TypeFlags::Union)
                && t.types()
                    .is_some_and(|ts| ts.iter().any(|c| c.flags.contains(TypeFlags::Undefined))));
        if already {
            return t;
        }
        self.get_union_type(vec![t, self.undefined_type()])
    }

    pub(crate) fn strip_optional_undefined(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Union)
            && let Some(ts) = t.types()
        {
            let kept: Vec<Arc<Type>> = ts
                .iter()
                .filter(|c| !c.flags.contains(TypeFlags::Undefined))
                .cloned()
                .collect();
            if !kept.is_empty() && kept.len() != ts.len() {
                return if kept.len() == 1 {
                    kept.into_iter().next().expect("nonempty")
                } else {
                    self.get_union_type(kept)
                };
            }
        }
        Arc::clone(t)
    }

    pub(crate) fn resolve_symbol_declared_type_on_demand(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
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
                        self.value_symbol_links
                            .get_or_default(symbol)
                            .resolved_type = Some(Arc::clone(t));
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
                NodeData::VariableDeclaration(d) => {
                    (d.type_node.clone(), d.initializer.clone())
                }
                NodeData::PropertyDeclaration(d) => {
                    (d.type_node.clone(), d.initializer.clone())
                }
                NodeData::PropertySignatureDeclaration(d) => {
                    (Some(Arc::clone(&d.type_node)), None)
                }
                NodeData::ParameterDeclaration(d) => {
                    (d.type_node.clone(), d.initializer.clone())
                }
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
                    let regularized =
                        checker.get_regular_type_of_literal_type(&widened_literal);
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
                if psd.postfix_token.as_ref().is_some_and(|tk| {
                    tk.kind == SyntaxKind::QuestionToken
                }) =>
            {
                Some(self.get_optional_type(Arc::clone(t)))
            }
            _ => result,
        };
        match &result {
            Some(t) => {
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(t));
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

    pub(crate) fn get_type_of_merged_namespace_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {

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

                    self.declared_type_links.get_or_default(symbol).declared_type =
                        Some(Arc::clone(&value_type));
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
                id: 0,
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

        self.declared_type_links.get_or_default(symbol).declared_type = Some(Arc::clone(&merged));
        merged
    }

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
            id: 0,
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

        if !sig.type_parameters.is_empty() && let Some(contextual) = contextual_signature {
            if contextual.type_parameters.is_empty() {
                let inst = self.instantiate_signature_in_context_of(&sig, contextual);
                return self.create_function_or_constructor_type(vec![inst], false);
            }
        }
        self.create_function_or_constructor_type(vec![sig], false)
    }

    pub(crate) fn build_overload_function_type(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {

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

    pub(crate) fn build_type_of_class_declaration(
        &mut self,
        node: &Arc<Node>,
        members: &Arc<NodeList>,
    ) -> Arc<Type> {

        self.push_scope(node);

        let instance_type = self.build_class_instance_type_with_base(node);
        let mut construct_sigs: Vec<Arc<Signature>> = Vec::new();
        for member in members.iter() {
            if member.kind != SyntaxKind::Constructor {
                continue;
            }
            let params = match &member.data {
                crate::ast::NodeData::ConstructorDeclaration(data) => &data.parameters,
                _ => continue,
            };
            let sig = self.build_signature_from_function_like_type_node(
                params,
                Arc::clone(&instance_type),
                 true,
                 None,
                 Some(Arc::clone(member)),
            );
            construct_sigs.push(sig);
        }
        self.pop_scope();
        if construct_sigs.is_empty() {

            let mut inherited: Option<(Arc<Node>, Arc<Node>)> = None;
            let mut cursor = Arc::clone(node);

            for _ in 0..1000 {
                let Some((base_node, _)) = self.extends_base_of(&cursor) else {
                    break;
                };
                if Arc::ptr_eq(&base_node, &cursor) {
                    break;
                }
                if let crate::ast::NodeData::ClassDeclaration(data) = &base_node.data {
                    if let Some(ctor) = data.members.iter().find(|m| {
                        matches!(m.data, crate::ast::NodeData::ConstructorDeclaration(_))
                    }) {
                        inherited = Some((Arc::clone(ctor), Arc::clone(&base_node)));
                        break;
                    }
                }
                cursor = base_node;
            }
            if let Some((ctor_decl, _)) = inherited {
                if let crate::ast::NodeData::ConstructorDeclaration(data) = &ctor_decl.data {
                    let params = Arc::clone(&data.parameters);
                    let sig = self.build_signature_from_function_like_type_node(
                        &params,
                        Arc::clone(&instance_type),
                         true,
                        None,
                        Some(ctor_decl),
                    );
                    construct_sigs.push(sig);
                }
            }
        }
        if construct_sigs.is_empty() {
            let sig = self.build_signature_from_function_like_type_node(
                &Arc::new(NodeList::default()),
                Arc::clone(&instance_type),
                 true,
                None,
                None,
            );
            construct_sigs.push(sig);
        }

        if node.has_syntactic_modifier(ModifierFlags::Abstract) {
            construct_sigs = construct_sigs
                .into_iter()
                .map(|sig| {
                    let s = crate::checker::types::Signature {
                        id: sig.id,
                        flags: sig.flags
                            | crate::checker::types::SignatureFlags::Abstract,
                        min_argument_count: sig.min_argument_count,
                        resolved_min_argument_count: sig.resolved_min_argument_count,
                        declaration: sig.declaration.clone(),
                        type_parameters: sig.type_parameters.clone(),
                        parameters: sig.parameters.clone(),
                        this_parameter: sig.this_parameter.clone(),
                        resolved_return_type: std::sync::OnceLock::new(),
                        resolved_type_predicate: sig.resolved_type_predicate.clone(),
                        target: None,
                        mapper: sig.mapper.clone(),
                        isolated_signature_type: std::sync::OnceLock::new(),
                        instantiated_parameter_types: sig.instantiated_parameter_types.clone(),
                    };
                    if let Some(rt) = sig.resolved_return_type.get() {
                        let _ = s.resolved_return_type.set(rt.clone());
                    }
                    if let Some(it) = sig.isolated_signature_type.get() {
                        let _ = s.isolated_signature_type.set(it.clone());
                    }
                    Arc::new(s)
                })
                .collect();
        }
        let ctor_type = self.create_function_or_constructor_type(construct_sigs,  true);

        self.attach_class_statics(&ctor_type, node);

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let t_mut = Arc::as_ptr(&ctor_type) as *mut crate::checker::types::Type;
            unsafe {
                (*t_mut).symbol = Some(Arc::clone(class_sym));
            }
        }
        ctor_type
    }

    pub(crate) fn attach_class_statics(&mut self, ctor_type: &Arc<Type>, node: &Arc<Node>) {

        let node_id = node.id();
        if self.class_statics_resolution_stack.contains(&node_id)
            || self.class_statics_resolution_stack.len() >= 200
        {
            return;
        }
        self.class_statics_resolution_stack.push(node_id);
        let mut members = SymbolTable::new();
        let mut properties: Vec<Arc<Symbol>> = Vec::new();

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let mut statics: Vec<(String, Arc<Symbol>)> = Vec::new();
            for sym in class_sym.members.entries.values() {
                if sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for sym in class_sym.exports.entries.values() {

                if (sym
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Static))
                    || sym.flags.contains(SymbolFlags::Prototype))
                    && !statics.iter().any(|(n, _)| *n == sym.name)
                {
                    statics.push((sym.name.clone(), Arc::clone(sym)));
                }
            }
            for (name, sym) in statics {
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        let class_members: Option<Arc<NodeList>> = match &node.data {
            crate::ast::NodeData::ClassDeclaration(d) => Some(Arc::clone(&d.members)),
            crate::ast::NodeData::ClassExpression(d) => Some(Arc::clone(&d.members)),
            _ => None,
        };
        if let Some(member_list) = class_members {
            for member in member_list.iter() {
                if !member.has_syntactic_modifier(ModifierFlags::Static) {
                    continue;
                }
                let Some(name_node) = member.name() else { continue };
                let name = name_node.text().to_string();
                if name.is_empty() || members.get(&name).is_some() {
                    continue;
                }
                let flags = match member.kind {
                    SyntaxKind::MethodDeclaration => SymbolFlags::Method,
                    SyntaxKind::GetAccessor => SymbolFlags::GetAccessor,
                    SyntaxKind::SetAccessor => SymbolFlags::SetAccessor,
                    _ => SymbolFlags::Property,
                };
                let mut sym = Symbol::new(flags, name.clone());
                sym.declarations.push(Arc::clone(member));
                let sym = Arc::new(sym);

                if let crate::ast::NodeData::PropertyDeclaration(pd) = &member.data
                    && let Some(tn) = &pd.type_node
                {
                    let t = self.get_type_from_type_node(tn);
                    self.value_symbol_links.insert(
                        &sym,
                        crate::checker::types::ValueSymbolLinks {
                            resolved_type: Some(t),
                            ..Default::default()
                        },
                    );
                }
                properties.push(Arc::clone(&sym));
                members.insert(name, sym);
            }
        }

        if let Some((base_node, _)) = self.extends_base_of(node) {
            let base_ctor = self.get_type_of_class_declaration(&base_node);
            if let Some(base_structured) = base_ctor.as_structured() {
                for (name, sym) in base_structured.members.iter() {
                    if members.get(name).is_none() {
                        members.insert(name.clone(), Arc::clone(sym));
                    }
                }
                for prop in &base_structured.properties {
                    let name = prop.name.clone();
                    if members.get(&name).is_some() && !properties.iter().any(|p| Arc::ptr_eq(p, prop)) {
                        properties.push(Arc::clone(prop));
                    }
                }
            }
        }
        self.class_statics_resolution_stack.pop();
        if members.is_empty() {
            return;
        }
        let t_mut = Arc::as_ptr(ctor_type) as *mut crate::checker::types::Type;
        unsafe {
            if let TypeData::Object(obj) = &mut (*t_mut).data {
                obj.structured.members = members;
                obj.structured.properties = properties;
            }
        }
    }

    pub(crate) fn extends_base_of(&self, class_node: &Arc<Node>) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let heritage = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            crate::ast::NodeData::ClassExpression(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        let extends_expr = heritage?.iter().find_map(|clause| {
            if let crate::ast::NodeData::HeritageClause(hc) = &clause.data {
                if hc.token == SyntaxKind::ExtendsKeyword {
                    return hc.types.iter().next().cloned();
                }
            }
            None
        })?;
        let base_expr = match &extends_expr.data {
            crate::ast::NodeData::ExpressionWithTypeArguments(data) => Arc::clone(&data.expression),
            _ => return None,
        };
        if base_expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let symbol = self.resolve_identifier(&base_expr)?;
        if !symbol.flags.contains(SymbolFlags::Class) {
            return None;
        }
        symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned()
            .map(|n| (n, symbol))
    }
}
