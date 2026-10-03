#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn resolve_base_class_constructor_type(&mut self) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("resolve_base_class_constructor_type"); 
        let (base_node, symbol) = self.base_class_node_of_enclosing_class()?;

        let key = Arc::as_ptr(&symbol) as *const tsox_frontend::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return None;
        }
        let ctor_type = self.get_type_of_class_declaration(&base_node);
        self.resolving_type_aliases.remove(&key);
        Some(ctor_type)
    }

    pub(crate) fn base_class_node_of_enclosing_class(&self) -> Option<(Arc<Node>, Arc<Symbol>)> { ::tsox_core::fntrace::enter("base_class_node_of_enclosing_class"); 
        let class_node = self.enclosing_class_stack.last().cloned()?;
        self.extends_base_of(&class_node)
    }

    /// 泛型类引用/继承的实例化：类实例缓存按节点驻留的是裸声明形态，
    /// 带实参构建须绕开缓存——取出裸缓存、类型实参映射压栈下重建成员，
    /// 再还原裸缓存（Go 声明类型与 instantiation 分离的等价实现）
    pub(crate) fn instantiate_class_instance_type(
        &mut self,
        class_node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        class_tps: &[Arc<tsox_frontend::ast::Symbol>],
        arg_types: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_class_instance_type"); 
        let node_id = class_node.id();
        let saved = self.class_instance_type_cache.remove(&node_id);
        self.this_type_cache.remove(&node_id);

        let mut mapping = HashMap::new();
        let mut name_frame: Vec<(Arc<tsox_frontend::ast::Symbol>, Arc<Type>)> = Vec::new();
        for (i, tp_sym) in class_tps.iter().enumerate() {
            if let Some(arg) = arg_types.get(i) {
                mapping.insert(
                    Arc::as_ptr(tp_sym) as *const tsox_frontend::ast::Symbol,
                    Arc::clone(arg),
                );
                name_frame.push((Arc::clone(tp_sym), Arc::clone(arg)));
            }
        }
        self.type_argument_stack.push(mapping);
        self.type_argument_name_frames.push(name_frame);
        self.push_scope(class_node);
        let instance = self.build_class_instance_type_with_base(class_node);
        self.pop_scope();
        self.type_argument_stack.pop();
        self.type_argument_name_frames.pop();

        self.class_instance_type_cache.remove(&node_id);
        self.this_type_cache.remove(&node_id);
        match saved {
            Some(raw) => {
                self.class_instance_type_cache.insert(node_id, raw);
            }
            None => {
                let _ = self.build_class_instance_type_with_base(class_node);
            }
        }

        let tp_types: Vec<Arc<Type>> = class_tps
            .iter()
            .map(|s| self.get_type_parameter_from_symbol(s))
            .collect();
        if !arg_types.is_empty() {
            let instance_mut = Arc::as_ptr(&instance) as *mut crate::checker::types::Type;
            unsafe {
                if let crate::checker::types::TypeData::Object(o) = &mut (*instance_mut).data {
                    o.type_arguments = arg_types.to_vec();
                }
            }
        }
        self.mark_structured_members_instantiated(&instance, symbol, class_tps, &tp_types, arg_types);
        // 实例化类实例型记录显式实参（Go 类实例引用携带 target+args，
        // 消息显示按带参形态）
        {
            let ptr = Arc::as_ptr(&instance) as *mut crate::checker::types::Type;
            unsafe {
                if let crate::checker::types::TypeData::Object(obj) = &mut (*ptr).data
                    && obj.type_arguments.is_empty()
                {
                    obj.type_arguments = arg_types.to_vec();
                }
            }
        }
        instance
    }

    pub(crate) fn resolve_entity_name_class_symbol(&mut self, expr: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_entity_name_class_symbol"); 
        match expr.kind {
            SyntaxKind::Identifier => self.resolve_identifier(expr),
            SyntaxKind::PropertyAccessExpression => {
                let tsox_frontend::ast::NodeData::PropertyAccessExpression(d) = &expr.data else {
                    return None;
                };
                let base = self.resolve_entity_name_class_symbol(&d.expression)?;
                let name = d.name.text();
                base.members
                    .get(name)
                    .cloned()
                    .or_else(|| base.exports.get(name).cloned())
            }
            _ => None,
        }
    }

    /// Go resolveQualifiedName：heritage 左端以 Namespace 含义预解析成功时
    /// 返回其 node id（仅 namespace 无 Value 含义者会触发 TS2708）
    pub(crate) fn namespace_leftmost_suppress_id(&mut self, expr: &Arc<Node>) -> Option<u64> { ::tsox_core::fntrace::enter("namespace_leftmost_suppress_id"); 
        let mut current: &Arc<Node> = expr;
        while let tsox_frontend::ast::NodeData::PropertyAccessExpression(d) = &current.data {
            current = &d.expression;
        }
        if current.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier_with_meaning(current, SymbolFlags::NAMESPACE)?;
        let module_without_value_meaning = sym.flags.contains(SymbolFlags::NamespaceModule)
            && !sym.flags.intersects(SymbolFlags::VALUE);
        if module_without_value_meaning {
            Some(current.id())
        } else {
            None
        }
    }

    pub(crate) fn emit_ts2506(&mut self, class_node: &Arc<Node>, symbol: &Arc<Symbol>) { ::tsox_core::fntrace::enter("emit_ts2506"); 
        let class_name_loc = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(cd) => cd
                .name
                .as_ref()
                .map(|n| n.loc)
                .unwrap_or(class_node.loc),
            _ => class_node.loc,
        };
        let file = self.get_source_file_of_node(class_node);
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            class_name_loc,
            tsox_core::diagnostics::messages_generated::
                X_0_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_BASE_EXPRESSION,
            vec![symbol.name.clone()],
        ));
    }

    pub(crate) fn resolve_base_class_instance_type(&mut self, type_ref: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_base_class_instance_type"); 
        if let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(data) = &type_ref.data {
            // Go resolveBaseTypesOfClass 非 class 符号分支（mixin 形态）：
            // extends 表达式是调用时，实例基型 = 基构造类型首个构造签名的
            // 返回型；经 get_base_constructor_type_of_class 解析，保持其
            // TS2507 校验与首解析记忆化
            if data.expression.kind == SyntaxKind::CallExpression {
                return self.resolve_mixin_base_instance_type(type_ref, &data.expression);
            }
            // Go getBaseConstructorTypeOfClass：heritage 表达式按值位求值，
            // var+interface 合并名（如 lib 的 var Iterator: IteratorConstructor）
            // 取 var 侧构造类型，而非把节点当类型引用解析
            let own_type = type_ref
                .parent()
                .and_then(|clause| clause.parent())
                .and_then(|class| {
                    if matches!(
                        class.kind,
                        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                    ) {
                        self.class_instance_type_cache.get(&class.id()).cloned()
                    } else {
                        None
                    }
                });
            let base_constructor_type = match own_type {
                Some(ref own) => self.get_base_constructor_type_of_class(own),
                None => Some(self.check_expression_ex(&data.expression, crate::checker::checker::CheckMode::Normal)),
            };
            let Some(base_constructor_type) = base_constructor_type else {
                return self.get_any_type();
            };
            if self.is_error_type(&base_constructor_type) {
                return self.get_any_type();
            }
            let apparent = self.get_apparent_type(&base_constructor_type);
            if !apparent
                .flags
                .intersects(TypeFlags::Object | TypeFlags::Intersection | TypeFlags::Any)
            {
                return self.get_any_type();
            }
            // Go resolveBaseTypesOfClass：符号是类且声明的裸泛型未被外层实参
            // 捕获时按类型引用取基型（默认实参在此填充），否则按构造函数分支
            // 实例化构造签名取返回型
            let original_base_type = apparent
                .symbol
                .as_ref()
                .map(|s| self.get_declared_type_of_symbol(s));
            let is_class_reference = apparent
                .symbol
                .as_ref()
                .is_some_and(|s| s.flags.contains(SymbolFlags::Class))
                && original_base_type
                    .is_some_and(|t| self.are_all_outer_type_parameters_applied(&t));
            if is_class_reference {
                let Some(symbol) = apparent.symbol.clone() else {
                    return self.get_any_type();
                };
                if self.type_resolution_stack.len() >= 200 {
                    return self.get_any_type();
                }
                let Some(class_node) = symbol
                    .declarations
                    .iter()
                    .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                    .cloned()
                else {
                    return self.get_type_from_type_node(type_ref);
                };
                let key = Arc::as_ptr(&symbol) as *const tsox_frontend::ast::Symbol;
                if self.is_resolving(key, TypeResolutionProperty::ResolvedBaseTypes) {
                    self.mark_type_resolution_cycle(key, TypeResolutionProperty::ResolvedBaseTypes);
                    return self.get_any_type();
                }
                let heritage_args = data.type_arguments.clone();
                let base_tps: Vec<Arc<tsox_frontend::ast::Symbol>> = match &class_node.data {
                    tsox_frontend::ast::NodeData::ClassDeclaration(cd) => match &cd.type_parameters
                    {
                        Some(tps) => tps
                            .iter()
                            .filter_map(|tp| {
                                self.program.symbol_map().symbol_of(tp).map(Arc::clone)
                            })
                            .collect(),
                        None => Vec::new(),
                    },
                    _ => Vec::new(),
                };
                let mut pushed_args: Option<Vec<Arc<Type>>> = None;
                if let Some(args) = &heritage_args
                    && !base_tps.is_empty()
                {
                    let mut arg_types: Vec<Arc<Type>> =
                        args.iter().map(|a| self.get_type_from_type_node(a)).collect();
                    if arg_types.len() < base_tps.len() {
                        // Go createTypeReference：实参不足时按类型参数默认值填充
                        let tp_types = self.declared_type_parameter_types(&symbol);
                        let min = self.get_min_type_argument_count(&tp_types);
                        arg_types = self.fill_missing_type_arguments(
                            &arg_types,
                            &tp_types,
                            min,
                            false,
                        );
                    }
                    if arg_types.len() == base_tps.len() {
                        pushed_args = Some(arg_types);
                    }
                }
                if let Some(arg_types) = pushed_args {
                    return self.instantiate_class_instance_type(
                        &class_node,
                        &symbol,
                        &base_tps,
                        &arg_types,
                    );
                }
                self.push_scope(&class_node);
                let instance = self.build_class_instance_type_with_base(&class_node);
                self.pop_scope();
                return instance;
            } else if apparent.flags.contains(TypeFlags::Any) {
                return apparent;
            } else if !apparent.flags.intersects(TypeFlags::Object | TypeFlags::Intersection) {
                return self.get_any_type();
            } else {
                let type_arg_nodes: Vec<Arc<Node>> = data
                    .type_arguments
                    .as_ref()
                    .map(|tl| tl.nodes.clone())
                    .unwrap_or_default();
                let constructors = self.get_instantiated_constructors_for_type_arguments(
                    &apparent,
                    &type_arg_nodes,
                    Some(type_ref),
                );
                if constructors.is_empty() {
                    self.error_message(
                        &data.expression,
                        tsox_core::diagnostics::messages_generated::NO_BASE_CONSTRUCTOR_HAS_THE_SPECIFIED_NUMBER_OF_TYPE_ARGUMENTS,
                        &[],
                    );
                    return self.get_any_type();
                }
                return self
                    .get_return_type_of_signature(&constructors[0])
                    .unwrap_or_else(|| self.get_any_type());
            }
        }

        let t = self.get_type_from_type_node(type_ref);
        if t.flags.contains(TypeFlags::Any) {
            return self.get_any_type();
        }

        if t.flags.contains(TypeFlags::Object) {
            return t;
        }
        self.get_any_type()
    }

    fn resolve_mixin_base_instance_type(
        &mut self,
        type_ref: &Arc<Node>,
        expr: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_mixin_base_instance_type"); 
        let own_type = type_ref
            .parent()
            .and_then(|clause| clause.parent())
            .and_then(|class| {
                if matches!(
                    class.kind,
                    SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                ) {
                    self.class_instance_type_cache.get(&class.id()).cloned()
                } else {
                    None
                }
            });
        let base_constructor_type = match own_type {
            Some(ref own) => self.get_base_constructor_type_of_class(own),
            None => Some(self.check_expression_ex(expr, crate::checker::checker::CheckMode::Normal)),
        };
        let Some(base_constructor_type) = base_constructor_type else {
            return self.get_any_type();
        };
        if !base_constructor_type
            .flags
            .intersects(TypeFlags::Object | TypeFlags::Intersection | TypeFlags::Any)
        {
            return self.get_any_type();
        }
        let constructors =
            self.get_signatures_of_type(&base_constructor_type, SignatureKind::Construct);
        for constructor in &constructors {
            if let Some(return_type) = self.get_return_type_of_signature(constructor) {
                if return_type.flags.contains(TypeFlags::Any) {
                    return self.get_any_type();
                }
                return return_type;
            }
        }
        self.get_any_type()
    }

    pub(crate) fn merge_instance_types(
        &mut self,
        derived: &Arc<Type>,
        base: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("merge_instance_types"); 
        if base.flags.contains(TypeFlags::Any) {
            return Arc::clone(derived);
        }
        let derived_data = match &derived.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };
        let base_data = match &base.data {
            TypeData::Object(o) => &o.structured,
            _ => return Arc::clone(derived),
        };

        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();

        for prop in &derived_data.properties {
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }

        for prop in &base_data.properties {
            if symbol_table.get(&prop.name).is_some() {
                continue;
            }
            symbol_table.insert(prop.name.clone(), Arc::clone(prop));
            props.push(Arc::clone(prop));
        }

        let mut index_infos = derived_data.index_infos.clone();
        index_infos.extend(base_data.index_infos.iter().cloned());

        let mut call_signatures: Vec<Arc<Signature>> = derived_data.call_signatures().to_vec();
        let derived_call_count = call_signatures.len();
        call_signatures.extend(base_data.call_signatures().iter().cloned());
        let mut signatures = call_signatures;
        signatures.extend(derived_data.construct_signatures().iter().cloned());
        signatures.extend(base_data.construct_signatures().iter().cloned());
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),

            symbol: derived.symbol.clone(),
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos,
                    signatures,
                    call_signature_count: derived_call_count + base_data.call_signatures().len(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn get_type_from_heritage_type_reference(
        &mut self,
        type_ref: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_heritage_type_reference"); 
        self.get_type_from_type_node(type_ref)
    }

    pub(crate) fn check_property_initialization(&mut self, class_node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_property_initialization"); 
        if !self.strict_null_checks || !self.strict_property_initialization {
            return;
        }

        if class_node.has_syntactic_modifier(ModifierFlags::Ambient)
            || self.ambient_context_depth > 0
            || self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file)
        {
            return;
        }
        let members = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => &d.members,
            tsox_frontend::ast::NodeData::ClassExpression(d) => &d.members,
            _ => return,
        };

        let constructor = members.iter().find(|m| m.kind == SyntaxKind::Constructor);
        for member in members.iter() {
            if member.kind != SyntaxKind::PropertyDeclaration {
                continue;
            }

            let mods = self.get_combined_modifier_flags(member);
            if mods.contains(ModifierFlags::Ambient) || mods.contains(ModifierFlags::Static) {
                continue;
            }

            if mods.contains(ModifierFlags::Abstract) {
                continue;
            }
            let tsox_frontend::ast::NodeData::PropertyDeclaration(pd) = &member.data else {
                continue;
            };

            if pd.initializer.is_some() || pd.postfix_token.is_some() {
                continue;
            }

            let name_node = &pd.name;
            if !matches!(
                name_node.kind,
                SyntaxKind::Identifier
                    | SyntaxKind::PrivateIdentifier
                    | SyntaxKind::ComputedPropertyName
            ) {
                continue;
            }

            let prop_type = if let Some(sym) = self
                .program
                .symbol_map()
                .symbol_of(member)
                .cloned()
            {
                self.get_type_of_symbol(&sym)
            } else {
                match &pd.type_node {
                    Some(type_node) => self.get_type_from_type_node(type_node),
                    None => continue,
                }
            };
            if prop_type
                .flags
                .intersects(TYPE_FLAGS_ANY_OR_UNKNOWN | TypeFlags::Undefined)
                || type_contains_undefined(&prop_type)
            {
                continue;
            }

            if let Some(ctor) = constructor {
                if self.is_property_assigned_in_constructor(name_node, &prop_type, ctor) {
                    continue;
                }
            }

            let name_text = self.node_text(name_node);
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                name_node.loc,
                PROPERTY_0_HAS_NO_INITIALIZER_AND_IS_NOT_DEFINITELY_ASSIGNED_IN_THE_CONSTRUCTOR,
                vec![name_text],
            ));
        }
    }

    pub(crate) fn node_text(&self, node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("node_text"); 
        match &node.data {
            tsox_frontend::ast::NodeData::Identifier(d) => d.text.clone(),
            tsox_frontend::ast::NodeData::PrivateIdentifier(d) => d.text.clone(),
            tsox_frontend::ast::NodeData::ComputedPropertyName(_) => {
                let Some(file) = &self.current_file else {
                    return String::new();
                };
                let pos = node.loc.pos();
                let end = node.loc.end();
                if pos < end && end <= file.text.len() {
                    file.text[pos..end].to_string()
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_property_name(
        &mut self,
        _member: &Arc<Node>,
        name: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_property_name"); 
        self.resolve_identifier(name)
    }
}
