#![allow(unused_imports)]

use crate::checker::services::*;

impl Checker {
    pub fn is_type_invalid_due_to_union_discriminant(
        &mut self,
        contextual_type: &Arc<Type>,
        obj: &Arc<Node>,
    ) -> bool {
        let properties: Vec<Arc<Node>> = match &obj.data {
            NodeData::ObjectLiteralExpression(data) => data.properties.nodes.clone(),
            NodeData::JsxAttributes(data) => data.properties.iter().cloned().collect(),
            _ => Vec::new(),
        };
        for property in &properties {
            let mut name_type = None;
            if let Some(property_name) = property.name() {
                if property_name.kind == SyntaxKind::JsxNamespacedName {
                    name_type = Some(self.get_string_literal_type(property_name.text()));
                } else {
                    name_type = self.get_literal_type_from_property_name(property_name);
                }
            }
            let mut name = String::new();
            if let Some(ref nt) = name_type {
                if is_type_usable_as_property_name(nt) {
                    name = get_property_name_from_type(nt);
                }
            }
            let mut expected = None;
            if !name.is_empty() {
                expected = self.get_type_of_property_of_type(contextual_type, &name);
            }
            if let Some(ref exp) = expected {
                if is_literal_type(exp) {
                    let prop_type = self.discriminant_property_value_type(property);
                    if !self.is_type_assignable_to(&prop_type, exp) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 判别式属性的取值类型：JsxAttribute 走初始化式字面量（不经符号拓宽），
    /// 其余按节点类型（Go getTypeOfNode）
    fn discriminant_property_value_type(&mut self, property: &Arc<Node>) -> Arc<Type> {
        if property.kind == SyntaxKind::JsxAttribute {
            if let NodeData::JsxAttribute(d) = &property.data
                && let Some(value) = &d.initializer
            {
                let expr = match &value.data {
                    NodeData::JsxExpression(je) => je.expression.clone(),
                    _ => Some(Arc::clone(value)),
                };
                if let Some(expr) = expr {
                    return self.get_type_of_node(&expr);
                }
            }
            return self.get_any_type();
        }
        self.get_type_of_node(property)
    }

    pub fn get_exports_and_properties_of_module(
        &mut self,
        module_symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Symbol>> {
        let mut exports = self.get_exports_of_module_as_array(module_symbol);
        let export_equals = self.resolve_external_module_symbol(module_symbol, false);
        if !Arc::ptr_eq(&export_equals, module_symbol) {
            let t = self.get_type_of_symbol(&export_equals);
            if self.should_treat_properties_of_external_module_as_exports(&t) {
                exports.extend(self.get_properties_of_type(&t));
            }
        }
        exports
    }

    pub fn get_exports_of_module_as_array(&self, module_symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        symbols_to_array(&self.get_exports_of_module_table(module_symbol))
    }

    pub fn get_jsx_intrinsic_tag_names_at(&mut self, location: &Arc<Node>) -> Vec<Arc<Symbol>> {
        let intrinsics = self.get_jsx_type_symbol("IntrinsicElements", location);
        if let Some(intrinsics) = intrinsics {
            return self.get_properties_of_type(&intrinsics);
        }
        Vec::new()
    }

    pub fn get_constant_value_for_services(&mut self, node: &Arc<Node>) -> Option<EvalValue> {
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value(node).value;
        }

        self.check_expression(node);

        let symbol = self
            .symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone());

        if let Some(ref sym) = symbol {
            if sym.flags.contains(SymbolFlags::EnumMember) {
                if let Some(ref member) = sym.value_declaration {
                    if let Some(ref parent) = member.parent {
                        if parent.flags.contains(tsox_frontend::ast::NodeFlags::Const) {
                            return self.get_enum_member_value(member).value;
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_resolved_signature_worker(
        &mut self,
        _node: &Arc<Node>,
        _check_mode: CheckMode,
        _argument_count: i32,
    ) -> (Option<Arc<Signature>>, Vec<Arc<Signature>>) {
        (None, Vec::new())
    }

    pub fn get_candidate_signatures_for_string_literal_completions(
        &mut self,
        _call: &Arc<Node>,
        _editing_argument: &Arc<Node>,
    ) -> Vec<Arc<Signature>> {
        Vec::new()
    }

    pub fn get_type_at_position_for_services(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        self.get_type_at_position(sig, pos)
    }

    pub fn get_type_parameter_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Arc<Type> {
        let t = self.get_type_at_position(sig, pos);

        if t.flags.contains(TypeFlags::Index) {
            if let TypeData::Index(index_data) = &t.data {
                if let Some(ref target) = index_data.target {
                    if self.is_this_type_parameter(target) {
                        if let Some(constraint) = self.get_base_constraint_of_type(target) {
                            return self.get_index_type(&constraint);
                        }
                    }
                }
            }
        }
        t
    }

    pub fn get_contextual_type_for_array_literal_at_position(
        &mut self,
        contextual_array_type: Option<&Arc<Type>>,
        array_literal: &Arc<Node>,
        position: usize,
    ) -> Option<Arc<Type>> {
        let contextual_array_type = contextual_array_type?;
        let mut first_spread_index = -1i32;
        let mut last_spread_index = -1i32;
        let mut element_index = 0i32;

        let elements: Vec<Arc<Node>> = match &array_literal.data {
            NodeData::ArrayLiteralExpression(data) => data.elements.nodes.clone(),
            _ => return None,
        };

        for (i, elem) in elements.iter().enumerate() {
            if elem.pos() < position {
                element_index += 1;
            }
            if elem.kind == SyntaxKind::SpreadElement {
                if first_spread_index == -1 {
                    first_spread_index = i as i32;
                }
                last_spread_index = i as i32;
            }
        }

        self.get_contextual_type_for_element_expression(
            contextual_array_type,
            element_index as usize,
            None,
            first_spread_index,
            last_spread_index,
        )
    }

    pub fn get_first_type_argument_from_known_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.object_flags.contains(ObjectFlags::Reference) {
            if let Some(ref symbol) = t.symbol {
                if is_known_generic_name(&symbol.name) {
                    let type_args = self.get_type_arguments(t);
                    return type_args.into_iter().next();
                }
            }
        }
        if let Some(ref alias) = t.alias {
            if let Some(ref alias_symbol) = alias.symbol {
                if is_known_generic_name(&alias_symbol.name) {
                    return alias.type_arguments.first().cloned();
                }
            }
        }
        None
    }

    pub fn get_property_symbols_from_contextual_type(
        &mut self,
        node: &Arc<Node>,
        contextual_type: &Arc<Type>,
        union_symbol_ok: bool,
    ) -> Vec<Arc<Symbol>> {
        let name = node.name().map(|n| n.text()).unwrap_or("");
        if name.is_empty() {
            return Vec::new();
        }

        if !contextual_type.flags.contains(TypeFlags::Union) {
            if let Some(symbol) = self.get_property_of_type(contextual_type, name) {
                return vec![symbol];
            }
            return Vec::new();
        }

        let mut filtered_types: Vec<Arc<Type>> = contextual_type
            .types()
            .unwrap_or(&[])
            .iter()
            .cloned()
            .collect();

        if let Some(ref parent) = node.parent {
            if parent.kind == SyntaxKind::ObjectLiteralExpression
                || parent.kind == SyntaxKind::JsxAttributes
            {
                filtered_types
                    .retain(|t| !self.is_type_invalid_due_to_union_discriminant(t, parent));
            }
        }

        let mut discriminated_property_symbols: Vec<Arc<Symbol>> = filtered_types
            .iter()
            .filter_map(|t| self.get_property_of_type(t, name))
            .collect();

        let constituent_count = contextual_type.types().map(|t| t.len()).unwrap_or(0);

        if union_symbol_ok
            && (discriminated_property_symbols.is_empty()
                || discriminated_property_symbols.len() == constituent_count)
        {
            if let Some(symbol) = self.get_property_of_type(contextual_type, name) {
                return vec![symbol];
            }
        }

        if filtered_types.is_empty() && discriminated_property_symbols.is_empty() {
            return contextual_type
                .types()
                .unwrap_or(&[])
                .iter()
                .filter_map(|t| self.get_property_of_type(t, name))
                .collect();
        }

        let mut seen = std::collections::HashSet::new();
        discriminated_property_symbols.retain(|s| seen.insert(s.id()));
        discriminated_property_symbols
    }

    pub fn get_property_symbol_of_destructuring_assignment(
        &mut self,
        location: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let parent = location.parent.as_ref()?;
        let grandparent = parent.parent.as_ref()?;

        if is_array_literal_or_object_literal_destructuring_pattern(grandparent) {
            if let Some(type_of_object_literal) = self.get_type_of_assignment_pattern(grandparent) {
                return self.get_property_of_type(&type_of_object_literal, location.text());
            }
        }
        None
    }

    pub fn get_type_of_assignment_pattern(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {
        use crate::checker::types::StructuredTypeData;
        use tsox_frontend::ast::SymbolTable;
        if expr.kind != SyntaxKind::ObjectBindingPattern {
            return None;
        }
        let NodeData::BindingPattern(pattern) = &expr.data else {
            return None;
        };
        let mut table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for el in &pattern.elements.nodes {
            let NodeData::BindingElement(be) = &el.data else {
                continue;
            };
            let prop_name = be
                .property_name
                .as_ref()
                .map(|p| p.text())
                .or_else(|| be.name.as_ref().map(|n| n.text()));
            let Some(prop_name) = prop_name else {
                continue;
            };
            if prop_name.is_empty() {
                continue;
            }
            let elem_type: Arc<Type> = match &be.name {
                Some(name_node) if name_node.kind == SyntaxKind::ObjectBindingPattern => {
                    self.get_type_of_assignment_pattern(name_node)
                        .unwrap_or_else(|| self.get_any_type())
                }
                _ => self.get_any_type(),
            };
            let symbol = Arc::new(Symbol::new(SymbolFlags::Property, prop_name.clone()));
            self.value_symbol_links
                .get_or_default(&symbol)
                .resolved_type = Some(Arc::clone(&elem_type));
            table.entries.insert(prop_name.to_string(), Arc::clone(&symbol));
            props.push(symbol);
        }
        Some(Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        }))
    }

    pub fn get_signature_from_declaration(&mut self, _node: &Arc<Node>) -> Option<Arc<Signature>> {
        None
    }
}
