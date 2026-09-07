#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_property_access(&mut self, node: &Arc<Node>) {
        let (obj_expr, question_dot, name) = match &node.data {
            crate::ast::NodeData::PropertyAccessExpression(data) => (
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

        if name.kind == SyntaxKind::PrivateIdentifier
            && self.check_private_identifier_access(node, name, name_text, &obj_type)
        {
            return;
        }

        if let Some(structured) = obj_type.as_structured() {
            if let Some(member_symbol) = structured.members.get(name_text) {
                let in_ctor = self.in_ctor_body_stack.last() == Some(&true);
                let in_prop_init = !in_ctor && self.access_in_property_initializer(node);
                if obj_expr.kind == SyntaxKind::ThisKeyword
                    && (in_ctor || in_prop_init)
                    && let Some(abstract_decl) = member_symbol.declarations.iter().find(|d| {
                        d.kind == SyntaxKind::PropertyDeclaration
                            && d.has_syntactic_modifier(ModifierFlags::Abstract)
                    })
                    && let Some(parent) = &abstract_decl.parent
                    && parent.kind == SyntaxKind::ClassDeclaration
                    && let Some(class_name) = class_declaration_name(parent)
                {
                    let file = self.current_file.clone();
                    let diagnostic = crate::ast::Diagnostic::new(
                        file,
                        name.loc,
                        crate::diagnostics::messages_generated::
                            ABSTRACT_PROPERTY_0_IN_CLASS_1_CANNOT_BE_ACCESSED_IN_THE_CONSTRUCTOR,
                        vec![name_text.to_string(), class_name],
                    );
                    self.diagnostics.add(diagnostic);
                }

                if in_prop_init
                    && obj_expr.kind == SyntaxKind::ThisKeyword
                    && get_assignment_target_kind(node) == AssignmentKind::None
                    && let Some(prop_decl) = member_symbol.declarations.iter().find(|d| {
                        d.kind == SyntaxKind::PropertyDeclaration
                            && !d.has_syntactic_modifier(ModifierFlags::Static)
                    })
                {
                    let asserted = matches!(
                        &prop_decl.data,
                        crate::ast::NodeData::PropertyDeclaration(d) if d.postfix_token.is_some()
                    );
                    let uninitialized = !prop_decl_has_initializer(prop_decl) && !asserted;
                    let later = later_sibling_property(node, prop_decl);
                    if uninitialized || later {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_IS_USED_BEFORE_ITS_INITIALIZATION,
                            vec![name_text.to_string()],
                        ));
                    }
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
                            crate::ast::NodeData::ClassDeclaration(d) => d
                                .name
                                .as_ref()
                                .map(|n| n.text().to_string())
                                .unwrap_or_default(),
                            _ => String::new(),
                        };
                        self.diagnostics.add(crate::ast::Diagnostic::new(
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

        let display_type = if obj_type.flags.contains(TypeFlags::IndexedAccess) {
            self.constraint_of_indexed_access(&obj_type)
                .unwrap_or_else(|| Arc::clone(&obj_type))
        } else {
            Arc::clone(&obj_type)
        };
        let type_str = self.type_to_string(&display_type);

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
        if let Some(sugg) = suggestion {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                crate::diagnostics::messages_generated::
                    PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2,
                vec![name_text.to_string(), type_str, sugg],
            ));
        } else {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name.loc,
                PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![name_text.to_string(), type_str],
            ));
        }
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
        let (message, args): (crate::diagnostics::Message, Vec<String>) = if invoke_form {
            (
                if possibly_undefined {
                    if possibly_null {
                        crate::diagnostics::messages_generated::
                            CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL_OR_UNDEFINED
                    } else {
                        crate::diagnostics::messages_generated::
                            CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_UNDEFINED
                    }
                } else {
                    crate::diagnostics::messages_generated::
                        CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL
                },
                Vec::new(),
            )
        } else if let Some(text) = entity_text {
            if possibly_undefined {
                if possibly_null {
                    (
                        crate::diagnostics::messages_generated::X_0_IS_POSSIBLY_NULL_OR_UNDEFINED,
                        vec![text],
                    )
                } else {
                    (
                        crate::diagnostics::messages_generated::X_0_IS_POSSIBLY_UNDEFINED,
                        vec![text],
                    )
                }
            } else {
                (
                    crate::diagnostics::messages_generated::X_0_IS_POSSIBLY_NULL,
                    vec![text],
                )
            }
        } else if possibly_undefined {
            if possibly_null {
                (
                    crate::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_NULL_OR_UNDEFINED,
                    Vec::new(),
                )
            } else {
                (
                    crate::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_UNDEFINED,
                    Vec::new(),
                )
            }
        } else {
            (
                crate::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_NULL,
                Vec::new(),
            )
        };
        self.diagnostics.add(crate::ast::Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            message,
            args,
        ));
        true
    }
}
