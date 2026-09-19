#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn resolve_base_class_constructor_type(&mut self) -> Option<Arc<Type>> {
        let (base_node, symbol) = self.base_class_node_of_enclosing_class()?;

        let key = Arc::as_ptr(&symbol) as *const tsox_frontend::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return None;
        }
        let ctor_type = self.get_type_of_class_declaration(&base_node);
        self.resolving_type_aliases.remove(&key);
        Some(ctor_type)
    }

    pub(crate) fn base_class_node_of_enclosing_class(&self) -> Option<(Arc<Node>, Arc<Symbol>)> {
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
    ) -> Arc<Type> {
        let node_id = class_node.id();
        let saved = self.class_instance_type_cache.remove(&node_id);

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

    pub(crate) fn resolve_entity_name_class_symbol(&mut self, expr: &Arc<Node>) -> Option<Arc<Symbol>> {
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

    pub(crate) fn emit_ts2506(&mut self, class_node: &Arc<Node>, symbol: &Arc<Symbol>) {
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

    pub(crate) fn resolve_base_class_instance_type(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {
        if let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(data) = &type_ref.data {
            let entity_symbol = match data.expression.kind {
                SyntaxKind::Identifier => self.resolve_identifier(&data.expression),
                SyntaxKind::PropertyAccessExpression => {
                    self.resolve_entity_name_class_symbol(&data.expression)
                }
                _ => None,
            };
            if let Some(symbol) = entity_symbol {
                {
                    if symbol.flags.contains(SymbolFlags::Class) {
                        if self.type_resolution_stack.len() >= 200 {
                            return self.get_any_type();
                        }

                        if let Some(class_node) = symbol
                            .declarations
                            .iter()
                            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
                            .cloned()
                        {
                            let key = Arc::as_ptr(&symbol) as *const tsox_frontend::ast::Symbol;
                            if self.is_resolving(key, TypeResolutionProperty::ResolvedBaseTypes) {
                                self.mark_type_resolution_cycle(
                                    key,
                                    TypeResolutionProperty::ResolvedBaseTypes,
                                );
                                return self.get_any_type();
                            }

                            let heritage_args = data.type_arguments.clone();
                            let base_tps: Vec<Arc<tsox_frontend::ast::Symbol>> =
                                match &class_node.data {
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
                            let mut pushed_args: Option<Vec<Arc<Type>>> = None;
                            let _pushed = if let Some(args) = &heritage_args
                                && !base_tps.is_empty()
                            {
                                let arg_types: Vec<Arc<Type>> = args
                                    .iter()
                                    .map(|a| self.get_type_from_type_node(a))
                                    .collect();
                                if arg_types.len() == base_tps.len() {
                                    pushed_args = Some(arg_types);
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            };
                            let instance = if let Some(arg_types) = pushed_args {
                                self.instantiate_class_instance_type(
                                    &class_node,
                                    &symbol,
                                    &base_tps,
                                    &arg_types,
                                )
                            } else {
                                self.push_scope(&class_node);
                                let i = self.build_class_instance_type_with_base(&class_node);
                                self.pop_scope();
                                i
                            };
                            return instance;
                        }
                    }
                }
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

    pub(crate) fn merge_instance_types(
        &mut self,
        derived: &Arc<Type>,
        base: &Arc<Type>,
    ) -> Arc<Type> {
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
            data: TypeData::Object(ObjectTypeData {
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
    ) -> Arc<Type> {
        self.get_type_from_type_node(type_ref)
    }

    pub(crate) fn check_property_initialization(&mut self, class_node: &Arc<Node>) {
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

            let Some(type_node) = &pd.type_node else {
                continue;
            };
            let prop_type = self.get_type_from_type_node(type_node);
            if prop_type
                .flags
                .intersects(TYPE_FLAGS_ANY_OR_UNKNOWN | TypeFlags::Undefined)
                || type_contains_undefined(&prop_type)
            {
                continue;
            }

            if let Some(ctor) = constructor {
                if self.is_property_assigned_in_constructor(name_node, ctor) {
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

    pub(crate) fn node_text(&self, node: &Arc<Node>) -> String {
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
    ) -> Option<Arc<Symbol>> {
        self.resolve_identifier(name)
    }
}
