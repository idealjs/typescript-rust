#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn build_template_literal_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (head, spans) = match &node.data {
            NodeData::TemplateLiteralTypeNode(data) => {
                (Arc::clone(&data.head), Arc::clone(&data.template_spans))
            }
            _ => return self.error_type(),
        };
        let head_text = template_token_text(&head);

        let mut span_types: Vec<Arc<Type>> = Vec::new();
        let mut span_texts: Vec<String> = Vec::new();
        for span_node in spans.iter() {
            let (type_node, literal_node) = match &span_node.data {
                NodeData::TemplateLiteralTypeSpan(data) => {
                    (Arc::clone(&data.type_node), Arc::clone(&data.literal))
                }
                _ => return self.error_type(),
            };
            span_types.push(self.get_type_from_type_node(&type_node));
            span_texts.push(template_token_text(&literal_node));
        }

        if span_types
            .iter()
            .any(|t| t.flags.contains(TypeFlags::Never) || matches!(&t.data, TypeData::Union(_)))
            && !self.check_cross_product_union(node, &span_types)
        {
            return self.error_type();
        }

        let all_literal = span_types.iter().all(|t| {
            t.flags
                .intersects(TYPE_FLAGS_LITERAL | TypeFlags::Null | TypeFlags::Undefined)
        });
        if all_literal {
            let mut sb = String::new();
            sb.push_str(&head_text);
            for (t, text) in span_types.iter().zip(span_texts.iter()) {
                sb.push_str(&self.template_string_for_type(t));
                sb.push_str(text);
            }
            return self.get_string_literal_type(&sb);
        }

        let mut texts = Vec::with_capacity(span_types.len() + 1);
        texts.push(head_text);
        for t in span_texts {
            texts.push(t);
        }
        Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(TemplateLiteralTypeData {
                constrained: ConstrainedTypeData::default(),
                texts,
                types: span_types,
            }),
        ))
    }

    pub(crate) fn template_string_for_type(&self, t: &Arc<Type>) -> String {
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return s.clone();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Number(n) = &lit.value {
                    return n.to_string();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Boolean(b) = &lit.value {
                    return if *b { "true".into() } else { "false".into() };
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::Null) {
            return "null".into();
        }
        if t.flags.contains(TypeFlags::Undefined) {
            return "undefined".into();
        }
        String::new()
    }

    pub(crate) fn build_mapped_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.push_scope(node);
        let result = self.build_mapped_type_inner(node);
        self.pop_scope();
        result
    }

    pub(crate) fn build_mapped_type_inner(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let data = match &node.data {
            NodeData::MappedTypeNode(data) => data,
            _ => return self.error_type(),
        };

        let constraint_node = match &data.type_parameter.data {
            NodeData::TypeParameterDeclaration(tp) => match &tp.constraint {
                Some(c) => Arc::clone(c),
                None => return self.error_type(),
            },
            _ => return self.error_type(),
        };
        let constraint_type = self.get_type_from_type_node(&constraint_node);

        if data.type_node.is_none()
            && self.no_implicit_any
            && self
                .current_file
                .as_ref()
                .is_some_and(|f| !f.file_name.starts_with("bundled://"))
        {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::
                    MAPPED_OBJECT_TYPE_IMPLICITLY_HAS_AN_ANY_TEMPLATE_TYPE,
                Vec::new(),
            ));
        }

        let keys = self.string_literal_values(&constraint_type);

        let constraint_all_literals = self.union_is_all_string_literals(&constraint_type);
        if keys.is_empty() || !constraint_all_literals {
            let tp_type = self.get_type_from_type_node(&data.type_parameter);
            let template_type = match &data.type_node {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
            };
            let name_type = data
                .name_type
                .as_ref()
                .map(|n| self.get_type_from_type_node(n));
            return Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: crate::checker::types::ObjectFlags::Mapped,
                id: crate::checker::types::next_type_id(),
                symbol: None,
                alias: None,
                data: TypeData::Mapped(MappedTypeData {
                    object: ObjectTypeData {
                        structured: StructuredTypeData::default(),
                        ..Default::default()
                    },
                    declaration: Some(Arc::clone(node)),
                    type_parameter: Some(tp_type),
                    constraint_type: Some(constraint_type),
                    name_type,
                    template_type: Some(template_type),
                    modifiers_type: None,
                    resolved_apparent_type: OnceLock::new(),
                    contains_error: false,
                }),
            });
        }

        let tp_symbol = self
            .program
            .symbol_map()
            .symbol_of(&data.type_parameter)
            .map(Arc::clone);
        let tp_key = tp_symbol
            .as_ref()
            .map(|s| Arc::as_ptr(s) as *const crate::ast::Symbol);

        let is_optional = data
            .question_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);

        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for key in &keys {
            let mut prop_type = match &data.type_node {
                Some(tn) => {
                    if let Some(k) = tp_key {
                        let mut mapping = HashMap::new();
                        mapping.insert(k, self.get_string_literal_type(key));
                        self.type_argument_stack.push(mapping);
                    }
                    let t = self.get_type_from_type_node(tn);
                    if tp_key.is_some() {
                        self.type_argument_stack.pop();
                    }
                    t
                }
                None => self.get_any_type(),
            };
            if is_optional {
                prop_type = self.get_optional_type(prop_type);
            }
            let mut flags = SymbolFlags::Property;
            if is_optional {
                flags |= SymbolFlags::Optional;
            }
            let symbol = Arc::new(Symbol::new(flags, key.clone()));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(key.clone(), Arc::clone(&symbol));
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos: Vec::new(),
                    signatures: Vec::new(),
                    call_signature_count: 0,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn string_literal_values(&self, t: &Arc<Type>) -> Vec<String> {
        if t.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return vec![s.clone()];
                }
            }
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .flat_map(|c| self.string_literal_values(c))
                    .collect();
            }
        }
        Vec::new()
    }

    pub(crate) fn union_is_all_string_literals(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::StringLiteral) {
            return true;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .all(|c| self.union_is_all_string_literals(c));
            }
        }
        false
    }

    pub(crate) fn add_optionality(&self, t: &Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            self.make_union_two(Arc::clone(t), self.undefined_type())
        } else {
            Arc::clone(t)
        }
    }
}
