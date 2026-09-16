#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

impl Checker {
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
                tsox_frontend::ast::NodeData::ConstructorDeclaration(data) => &data.parameters,
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
            let mut inherited: Option<(Arc<Node>, Arc<Node>, Arc<Node>)> = None;
            let mut cursor = Arc::clone(node);

            for _ in 0..1000 {
                let Some((base_node, _, heritage_expr)) = self.extends_base_class_node(&cursor)
                else {
                    break;
                };
                if Arc::ptr_eq(&base_node, &cursor) {
                    break;
                }
                if let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &base_node.data {
                    if let Some(ctor) = data.members.iter().find(|m| {
                        matches!(
                            m.data,
                            tsox_frontend::ast::NodeData::ConstructorDeclaration(_)
                        )
                    }) {
                        inherited = Some((Arc::clone(ctor), Arc::clone(&base_node), heritage_expr));
                        break;
                    }
                }
                cursor = base_node;
            }
            if let Some((ctor_decl, base_node, heritage_expr)) = inherited {
                if let tsox_frontend::ast::NodeData::ConstructorDeclaration(data) = &ctor_decl.data
                {
                    let params = Arc::clone(&data.parameters);
                    // 继承构造签名按 heritage 实参实例化（基类 ctor 参数里的
                    // 类型参数映射为派生类继承实参），映射压栈下构建
                    let base_tps: Vec<Arc<Symbol>> = match &base_node.data {
                        tsox_frontend::ast::NodeData::ClassDeclaration(cd) => {
                            match &cd.type_parameters {
                                Some(tps) => tps
                                    .iter()
                                    .filter_map(|tp| {
                                        self.program
                                            .symbol_map()
                                            .symbol_of(tp)
                                            .map(Arc::clone)
                                    })
                                    .collect(),
                                None => Vec::new(),
                            }
                        }
                        _ => Vec::new(),
                    };
                    let heritage_args: Option<Vec<Arc<Node>>> = match &heritage_expr.data {
                        tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ewa) => {
                            ewa.type_arguments.as_ref().map(|n| n.iter().cloned().collect())
                        }
                        _ => None,
                    };
                    let mut arg_types: Option<Vec<Arc<Type>>> = None;
                    if let Some(args) = &heritage_args
                        && !base_tps.is_empty()
                        && args.len() == base_tps.len()
                    {
                        let at: Vec<Arc<Type>> = args
                            .iter()
                            .map(|a| self.get_type_from_type_node(a))
                            .collect();
                        let mut mapping = HashMap::new();
                        let mut name_frame: Vec<(Arc<Symbol>, Arc<Type>)> = Vec::new();
                        for (i, tp_sym) in base_tps.iter().enumerate() {
                            mapping.insert(
                                Arc::as_ptr(tp_sym) as *const Symbol,
                                Arc::clone(&at[i]),
                            );
                            name_frame.push((Arc::clone(tp_sym), Arc::clone(&at[i])));
                        }
                        self.type_argument_stack.push(mapping);
                        self.type_argument_name_frames.push(name_frame);
                        arg_types = Some(at);
                    }
                    let sig = self.build_signature_from_function_like_type_node(
                        &params,
                        Arc::clone(&instance_type),
                        true,
                        None,
                        Some(Arc::clone(&ctor_decl)),
                    );
                    if arg_types.is_some() {
                        self.type_argument_stack.pop();
                        self.type_argument_name_frames.pop();
                    }
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

        // Go getSignatureFromDeclaration：构造签名类型参数取自类声明（构造器自身不可带）
        let class_tp_types: Vec<Arc<Type>> = self.class_type_parameter_types_of(node);
        if !class_tp_types.is_empty() {
            for sig in &construct_sigs {
                let sig_mut = Arc::as_ptr(sig) as *mut crate::checker::types::Signature;
                unsafe {
                    if (*sig_mut).type_parameters.is_empty() {
                        (*sig_mut).type_parameters = class_tp_types.clone();
                    }
                }
            }
        }

        if node.has_syntactic_modifier(ModifierFlags::Abstract) {
            construct_sigs = construct_sigs
                .into_iter()
                .map(|sig| {
                    let s = crate::checker::types::Signature {
                        id: sig.id,
                        flags: sig.flags | crate::checker::types::SignatureFlags::Abstract,
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
        let ctor_type = self.create_function_or_constructor_type(construct_sigs, true);

        self.attach_class_statics(&ctor_type, node);

        if let Some(class_sym) = self.program.symbol_map().symbol_of(node) {
            let t_mut = Arc::as_ptr(&ctor_type) as *mut crate::checker::types::Type;
            unsafe {
                (*t_mut).symbol = Some(Arc::clone(class_sym));
            }
        }
        ctor_type
    }


    /// extends_base_of 的限定名版：`class D extends React.Component<...>` 的
    /// 基类名是属性访问表达式，标识符-only 的 extends_base_of 解析不到；
    /// 同时返回 heritage 表达式（含类型实参，供继承构造签名实例化）
    pub(crate) fn extends_base_class_node(
        &mut self,
        class_node: &Arc<Node>,
    ) -> Option<(Arc<Node>, Arc<Symbol>, Arc<Node>)> {
        let heritage = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            tsox_frontend::ast::NodeData::ClassExpression(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        let extends_expr = heritage?.iter().find_map(|clause| {
            if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data {
                if hc.token == SyntaxKind::ExtendsKeyword {
                    return hc.types.iter().next().cloned();
                }
            }
            None
        })?;
        let base_expr = match &extends_expr.data {
            tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(data) => {
                Arc::clone(&data.expression)
            }
            _ => return None,
        };
        let symbol = match base_expr.kind {
            SyntaxKind::Identifier => self.resolve_identifier(&base_expr)?,
            SyntaxKind::QualifiedName | SyntaxKind::PropertyAccessExpression => {
                match self.resolve_qualified_symbol_traced(&base_expr) {
                    Ok(s) => s,
                    Err(_) => return None,
                }
            }
            _ => return None,
        };
        if !symbol.flags.contains(SymbolFlags::Class) {
            return None;
        }
        let base_node = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned()?;
        Some((base_node, symbol, extends_expr))
    }

    pub(crate) fn extends_base_of(
        &self,
        class_node: &Arc<Node>,
    ) -> Option<(Arc<Node>, Arc<Symbol>)> {
        let heritage = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => data.heritage_clauses.clone(),
            tsox_frontend::ast::NodeData::ClassExpression(data) => data.heritage_clauses.clone(),
            _ => return None,
        };
        let extends_expr = heritage?.iter().find_map(|clause| {
            if let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data {
                if hc.token == SyntaxKind::ExtendsKeyword {
                    return hc.types.iter().next().cloned();
                }
            }
            None
        })?;
        let base_expr = match &extends_expr.data {
            tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(data) => {
                Arc::clone(&data.expression)
            }
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
