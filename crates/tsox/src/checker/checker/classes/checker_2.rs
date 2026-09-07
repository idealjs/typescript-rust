#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_base_class_constructor_type(&mut self) -> Option<Arc<Type>> {
        let (base_node, symbol) = self.base_class_node_of_enclosing_class()?;

        let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
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

    pub(crate) fn resolve_base_class_instance_type(&mut self, type_ref: &Arc<Node>) -> Arc<Type> {
        if let crate::ast::NodeData::ExpressionWithTypeArguments(data) = &type_ref.data {
            if data.expression.kind == SyntaxKind::Identifier {
                if let Some(symbol) = self.resolve_identifier(&data.expression) {
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
                            let key = Arc::as_ptr(&symbol) as *const crate::ast::Symbol;
                            if !self.push_type_resolution(
                                key,
                                TypeResolutionProperty::ResolvedBaseTypes,
                            ) {
                                return self.get_any_type();
                            }

                            let heritage_args = data.type_arguments.clone();
                            let base_tps: Vec<Arc<crate::ast::Symbol>> = match &class_node.data {
                                crate::ast::NodeData::ClassDeclaration(cd) => {
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
                            let pushed = if let Some(args) = &heritage_args
                                && !base_tps.is_empty()
                            {
                                let arg_types: Vec<Arc<Type>> = args
                                    .iter()
                                    .map(|a| self.get_type_from_type_node(a))
                                    .collect();
                                let mut mapping = HashMap::new();
                                let mut name_frame: Vec<(Arc<Symbol>, Arc<Type>)> = Vec::new();
                                for (i, tp_sym) in base_tps.iter().enumerate() {
                                    if i < arg_types.len() {
                                        mapping.insert(
                                            Arc::as_ptr(tp_sym) as *const crate::ast::Symbol,
                                            Arc::clone(&arg_types[i]),
                                        );
                                        name_frame
                                            .push((Arc::clone(tp_sym), Arc::clone(&arg_types[i])));
                                    }
                                }
                                self.type_argument_stack.push(mapping);
                                self.type_argument_name_frames.push(name_frame);
                                true
                            } else {
                                false
                            };
                            let instance = {
                                self.push_scope(&class_node);
                                let i = self.build_class_instance_type_with_base(&class_node);
                                self.pop_scope();
                                i
                            };
                            if pushed {
                                self.type_argument_stack.pop();
                                self.type_argument_name_frames.pop();
                            }
                            self.pop_type_resolution();
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
            crate::ast::NodeData::ClassDeclaration(d) => &d.members,
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
            let crate::ast::NodeData::PropertyDeclaration(pd) = &member.data else {
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
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_node.loc,
                PROPERTY_0_HAS_NO_INITIALIZER_AND_IS_NOT_DEFINITELY_ASSIGNED_IN_THE_CONSTRUCTOR,
                vec![name_text],
            ));
        }
    }

    pub(crate) fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            crate::ast::NodeData::Identifier(d) => d.text.clone(),
            crate::ast::NodeData::PrivateIdentifier(d) => d.text.clone(),
            crate::ast::NodeData::ComputedPropertyName(_) => {
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
