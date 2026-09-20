#![allow(unused_imports)]

use crate::checker::checker::*;

impl Checker {
    /// Go checkObjectLiteral 的 contextualTypeHasPattern 分支：绑定模式名声明
    /// 的对象字面量初始化式按模式隐含型做多余属性检查（TS2353）
    pub(crate) fn check_binding_pattern_initializer_excess(
        &mut self,
        pattern: &Arc<Node>,
        init: &Arc<Node>,
    ) {
        let NodeData::ObjectLiteralExpression(ol) = &init.data else {
            return;
        };
        let NodeData::BindingPattern(bp) = &pattern.data else {
            return;
        };

        struct ImpliedMember {
            name: String,
            optional: bool,
            default_type: Option<Arc<Type>>,
        }
        let mut implied: Vec<ImpliedMember> = Vec::new();
        let mut has_rest = false;
        let mut pattern_has_computed = false;
        for e in bp.elements.iter() {
            let NodeData::BindingElement(be) = &e.data else {
                continue;
            };
            if be.dot_dot_dot_token.is_some() {
                has_rest = true;
                continue;
            }
            let name_node = be.property_name.as_ref().or(be.name.as_ref());
            let Some(name_node) = name_node else { continue };
            let usable = match &name_node.data {
                NodeData::ComputedPropertyName(cd) => {
                    let t = self.get_type_of_node(&cd.expression);
                    if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&t) {
                        true
                    } else {
                        pattern_has_computed = true;
                        false
                    }
                }
                _ => true,
            };
            if !usable {
                continue;
            }
            let name = name_node.text().trim_matches(|c| c == '"' || c == '\'').to_string();
            let default_type = match &be.initializer {
                Some(d) => {
                    let raw = self.get_type_of_node(d);
                    Some(self.get_widened_type(&raw))
                }
                None => None,
            };
            implied.push(ImpliedMember {
                name,
                optional: be.initializer.is_some(),
                default_type,
            });
        }
        if has_rest || pattern_has_computed {
            return;
        }

        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for m in &implied {
            let mut flags = SymbolFlags::Property;
            if m.optional {
                flags |= SymbolFlags::Optional;
            }
            let symbol = Arc::new(Symbol::new(flags, m.name.clone()));
            self.value_symbol_links.insert(
                &symbol,
                crate::checker::types::ValueSymbolLinks {
                    resolved_type: Some(Arc::clone(m.default_type.as_ref().unwrap_or(&{
                        let any = self.any_type();
                        any
                    }))),
                    ..Default::default()
                },
            );
            members.insert(m.name.clone(), Arc::clone(&symbol));
            props.push(symbol);
        }
        let display = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous | ObjectFlags::ObjectLiteral,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        });
        let type_str = self.type_to_string(&display);

        for elem in ol.properties.iter() {
            let name_node = match &elem.data {
                NodeData::PropertyAssignment(d) => &d.name,
                NodeData::ShorthandPropertyAssignment(d) => &d.name,
                NodeData::MethodDeclaration(d) => &d.name,
                _ => continue,
            };
            let member_name = match &name_node.data {
                NodeData::ComputedPropertyName(cd) => {
                    let t = self.get_type_of_node(&cd.expression);
                    if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&t) {
                        crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(&t)
                    } else {
                        String::new()
                    }
                }
                _ => name_node.text().to_string(),
            };
            if !member_name.is_empty()
                && implied.iter().any(|m| m.name == member_name)
            {
                continue;
            }
            let display_name = crate::checker::property_name_for_display(
                &if member_name.is_empty() {
                    let NodeData::ComputedPropertyName(cd) = &name_node.data else {
                        continue;
                    };
                    format!(
                        "[{}]",
                        crate::checker::checker_attach_explicit_type_arguments::qualified_name_text(
                            &cd.expression
                        )
                    )
                } else {
                    member_name.clone()
                },
            );
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                name_node.loc,
                tsox_core::diagnostics::messages_generated::
                    OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                vec![display_name, type_str.clone()],
            ));
        }
    }
}
