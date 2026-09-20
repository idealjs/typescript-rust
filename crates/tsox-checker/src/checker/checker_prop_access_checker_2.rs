#![allow(unused_imports)]

use crate::checker::checker_prop_access::*;

impl Checker {
    pub(crate) fn check_property_access(&mut self, node: &Arc<Node>) {
        let (obj_expr, question_dot, name) = match &node.data {
            tsox_frontend::ast::NodeData::PropertyAccessExpression(data) => (
                &data.expression,
                data.question_dot_token.is_some(),
                &data.name,
            ),
            _ => return,
        };
        let obj_type = self.get_type_of_node(obj_expr);
        let name_text = name.text();

        let lookup_type;
        if !question_dot && self.strict_null_checks && type_is_possibly_undefined(&obj_type) {
            self.report_possibly_null_or_undefined(obj_expr, &obj_type, false);
            lookup_type = self.get_non_nullable_type_of(&obj_type);
        } else {
            lookup_type = obj_type;
        }
        let obj_type = lookup_type;

        if name_text.is_empty() {
            return;
        }

        if name.kind == SyntaxKind::PrivateIdentifier
            && self.check_private_identifier_access(node, name, name_text, &obj_type)
        {
            return;
        }

        if let Some(prop) = self.get_property_of_type(&obj_type, name_text) {
            self.check_property_not_used_before_declaration(&prop, node, name);
        }

        if let Some(structured) = obj_type.as_structured() {
            if let Some(member_symbol) = structured.members.get(name_text) {
                // TS2855：super 访问基类实例字段（字段在实例上而非原型）不可达，
                // 优先于 private/protected 可访问性错误
                if obj_expr.kind == SyntaxKind::SuperKeyword
                    && member_symbol
                        .declarations
                        .iter()
                        .any(|d| d.kind == SyntaxKind::PropertyDeclaration)
                {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        name.loc,
                        tsox_core::diagnostics::messages_generated::
                            CLASS_FIELD_0_DEFINED_BY_THE_PARENT_CLASS_IS_NOT_ACCESSIBLE_IN_THE_CHILD_CLASS_VIA_SUPER,
                        vec![name_text.to_string()],
                    ));
                    return;
                }
                let in_ctor = self.in_ctor_body_stack.last() == Some(&true);
                let in_prop_init = !in_ctor && self.access_in_property_initializer(node);
                if obj_expr.kind == SyntaxKind::ThisKeyword
                    && (in_ctor || in_prop_init)
                    && let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
                        d.kind == SyntaxKind::PropertyDeclaration
                            && d.has_syntactic_modifier(ModifierFlags::Abstract)
                    })
                    && let Some(parent) = &abstract_decl.parent()
                    && parent.kind == SyntaxKind::ClassDeclaration
                    && let Some(class_name) = class_declaration_name(parent)
                {
                    let file = self.current_file.clone();
                    let diagnostic = tsox_frontend::ast::Diagnostic::new(
                        file,
                        name.loc,
                        tsox_core::diagnostics::messages_generated::
                            ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
                        vec![name_text.to_string(), class_name],
                    );
                    self.diagnostics.add(diagnostic);
                }

                if let Some(declaring_class) = self.declaring_class_of_member(member_symbol) {
                    let is_private =
                        crate::checker::exports::get_declaration_modifier_flags_from_symbol_ex(
                            member_symbol,
                            false,
                        )
                        .contains(ModifierFlags::Private);
                    if is_private && !self.is_within_declaring_class(&declaring_class) {
                        let class_name = match &declaring_class.data {
                            tsox_frontend::ast::NodeData::ClassDeclaration(d) => d
                                .name
                                .as_ref()
                                .map(|n| n.text().to_string())
                                .unwrap_or_default(),
                            _ => String::new(),
                        };
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            name.loc,
                            PROPERTY_0_IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_CLASS_1,
                            vec![name_text.to_string(), class_name],
                        ));
                        return;
                    }
                }
            }
        }

        if !obj_type.flags.contains(TypeFlags::Never)
            && self.has_property_of_type(&obj_type, name_text)
        {
            return;
        }

        if self.global_constructor_value_has_property(obj_expr, name_text) {
            return;
        }

        // Go resolveAccessExpression：typeof globalThis 上的属性访问无索引签名，
        // 命中全局符号且为 block-scoped 才报 2339，否则静默返回 any
        if self.global_this_property_access_error(&obj_type, name_text, name) {
            return;
        }

        if obj_expr.kind == SyntaxKind::Identifier
            && let Some(sym) = self.resolve_identifier(obj_expr)
        {
            let base = self.resolve_alias_base(sym);
            if base.flags.contains(SymbolFlags::ValueModule) {
                let found = base.exports.entries.contains_key(name_text)
                    || base.members.entries.contains_key(name_text)
                    || self.ambient_namespace_local(&base, name_text).is_some();
                if found {
                    return;
                }
            }
        }
        let file = self.current_file.clone();

        if obj_type
            .symbol
            .as_ref()
            .is_some_and(|s| self.global_this_symbol.as_ref().is_some_and(|g| Arc::ptr_eq(g, s)))
        {
            let block_scoped = self.globals.get(name_text).is_some_and(|sym| {
                sym.flags
                    .intersects(tsox_frontend::ast::SymbolFlags::BLOCK_SCOPED)
            });
            if !block_scoped {
                if self.no_implicit_any {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        name.loc,
                        tsox_core::diagnostics::messages_generated::
                            ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_TYPE_0_HAS_NO_INDEX_SIGNATURE,
                        vec!["typeof globalThis".to_string()],
                    ));
                }
                return;
            }
        }

        let display_type = if obj_type.flags.contains(TypeFlags::IndexedAccess) {
            self.constraint_of_indexed_access(&obj_type)
                .unwrap_or_else(|| Arc::clone(&obj_type))
        } else if crate::checker::mapper::is_this_type_parameter(&obj_type)
            && let crate::checker::types::TypeData::TypeParameter(tp) = &obj_type.data
            && let Some(constraint) = &tp.constraint
        {
            Arc::clone(constraint)
        } else {
            Arc::clone(&obj_type)
        };
        let mut type_str = self.type_to_string(&display_type);
        if display_type
            .symbol
            .as_ref()
            .is_some_and(|s| s.flags.intersects(tsox_frontend::ast::SymbolFlags::ENUM))
            && matches!(&display_type.data, crate::checker::types::TypeData::Object(_))
        {
            type_str = format!(
                "typeof {}",
                self.namespace_qualified_name(display_type.symbol.as_ref().unwrap())
            );
        }

        let suggestion = display_type.as_structured().and_then(|st| {
            let rune_len = name_text.chars().count();
            let maximum_length_difference = 2.max((rune_len as f64 * 0.34) as usize);
            let mut best_distance = (rune_len as f64 * 0.4).floor() + 0.9;
            let mut best: Option<String> = None;
            let mut members: Vec<&String> = st.members.entries.keys().collect();
            members.sort();
            for cand in members {
                let cand = cand.as_str();
                if cand.is_empty()
                    || cand.starts_with('"')
                    || cand.starts_with('\'')
                    || cand.starts_with('`')
                    || cand.starts_with('\u{FE}')
                {
                    continue;
                }
                let cand_len = cand.chars().count();

                if cand_len < 3 && !cand.eq_ignore_ascii_case(name_text) {
                    continue;
                }
                if rune_len.max(cand_len) - rune_len.min(cand_len) > maximum_length_difference {
                    continue;
                }
                if cand == name_text {
                    continue;
                }
                let Some(d) = levenshtein_with_max(name_text, cand, best_distance) else {
                    continue;
                };
                if d < best_distance {
                    best_distance = d;
                    best = Some(cand.to_string());
                }
            }
            best
        });
        let mut chain: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
        if obj_type.is_union() && !obj_type.flags.intersects(TYPE_FLAGS_PRIMITIVE) {
            let name_literal = self.get_string_literal_type(name_text);
            for subtype in self.constituent_types(&obj_type) {
                if self.get_property_of_type(&subtype, name_text).is_none()
                    && self.get_applicable_index_info(&subtype, &name_literal).is_none()
                {
                    chain.push(tsox_frontend::ast::Diagnostic::new(
                        file.clone(),
                        name.loc,
                        PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                        vec![
                            name_text.to_string(),
                            self.type_to_string(&subtype),
                        ],
                    ));
                    break;
                }
            }
        }
        let static_hit = self.type_has_static_property(name_text, &display_type);
        if static_hit {
            let mut diag = tsox_frontend::ast::Diagnostic::new(
                file,
                name.loc,
                tsox_core::diagnostics::messages_generated::
                    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_TO_ACCESS_THE_STATIC_MEMBER_2_INSTEAD,
                vec![
                    name_text.to_string(),
                    type_str.clone(),
                    format!("{type_str}.{name_text}"),
                ],
            );
            diag.message_chain = chain;
            self.diagnostics.add(diag);
            return;
        }
        // Go reportNonexistentProperty：属性名命中 lib 特性表先报 TS2550
        //（容器取 containingType 的 apparent 符号名，primitive 映射到包装接口）
        if let Some(lib) = self
            .lib_suggestion_container_name(&obj_type)
            .and_then(|container| {
                crate::checker::checker_lib_feature_map::suggested_lib_for_property(
                    &container,
                    name_text,
                )
            })
        {
            let mut diag = tsox_frontend::ast::Diagnostic::new(
                file,
                name.loc,
                tsox_core::diagnostics::messages_generated::
                    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_2_OR_LATER,
                vec![name_text.to_string(), type_str, lib.to_string()],
            );
            diag.message_chain = chain;
            self.diagnostics.add(diag);
            return;
        }
        let mut diag = if let Some(sugg) = suggestion {
            tsox_frontend::ast::Diagnostic::new(
                file,
                name.loc,
                tsox_core::diagnostics::messages_generated::
                    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2,
                vec![name_text.to_string(), type_str, sugg],
            )
        } else {
            tsox_frontend::ast::Diagnostic::new(
                file,
                name.loc,
                PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![name_text.to_string(), type_str],
            )
        };
        diag.message_chain = chain;
        self.diagnostics.add(diag);
    }

    pub(crate) fn global_this_property_access_error(
        &mut self,
        obj_type: &Arc<Type>,
        name_text: &str,
        name: &Arc<Node>,
    ) -> bool {
        if self
            .global_this_symbol
            .as_ref()
            .is_some_and(|gt| obj_type.symbol.as_ref().is_some_and(|s| Arc::ptr_eq(s, gt)))
        {
            let block_scoped_hit = self.globals.get(name_text).is_some_and(|sym| {
                sym.flags.intersects(
                    tsox_frontend::ast::SymbolFlags::BlockScopedVariable
                        | tsox_frontend::ast::SymbolFlags::Class
                        | tsox_frontend::ast::SymbolFlags::ENUM,
                )
            });
            if block_scoped_hit {
                let file = self.current_file.clone();
                let type_str = self.type_to_string(obj_type);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    name.loc,
                    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                    vec![name_text.to_string(), type_str],
                ));
            }
            return true;
        }
        false
    }

    pub(crate) fn report_possibly_null_or_undefined(
        &mut self,
        node: &Arc<Node>,
        t: &Arc<Type>,
        invoke_form: bool,
    ) -> bool {
        if !self.strict_null_checks || !type_is_possibly_undefined(t) {
            return false;
        }
        let possibly_undefined = type_includes_undefined_only(t);
        let possibly_null = type_includes_null_only(t);
        let entity_text = if is_entity_name_expression(node) {
            let text = if node.kind == SyntaxKind::Identifier {
                node.text().to_string()
            } else {
                self.node_source_text(node).unwrap_or_default()
            };
            if !text.is_empty() && text.len() < 100 {
                Some(text)
            } else {
                None
            }
        } else {
            None
        };
        let (message, args): (tsox_core::diagnostics::Message, Vec<String>) = if invoke_form {
            (
                if possibly_undefined {
                    if possibly_null {
                        tsox_core::diagnostics::messages_generated::
                            CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL_OR_UNDEFINED
                    } else {
                        tsox_core::diagnostics::messages_generated::
                            CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_UNDEFINED
                    }
                } else {
                    tsox_core::diagnostics::messages_generated::
                        CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL
                },
                Vec::new(),
            )
        } else if let Some(text) = entity_text {
            if possibly_undefined {
                if possibly_null {
                    (
                        tsox_core::diagnostics::messages_generated::X_0_IS_POSSIBLY_NULL_OR_UNDEFINED,
                        vec![text],
                    )
                } else {
                    (
                        tsox_core::diagnostics::messages_generated::X_0_IS_POSSIBLY_UNDEFINED,
                        vec![text],
                    )
                }
            } else {
                (
                    tsox_core::diagnostics::messages_generated::X_0_IS_POSSIBLY_NULL,
                    vec![text],
                )
            }
        } else if possibly_undefined {
            if possibly_null {
                (
                    tsox_core::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_NULL_OR_UNDEFINED,
                    Vec::new(),
                )
            } else {
                (
                    tsox_core::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_UNDEFINED,
                    Vec::new(),
                )
            }
        } else {
            (
                tsox_core::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_NULL,
                Vec::new(),
            )
        };
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            message,
            args,
        ));
        true
    }
}
