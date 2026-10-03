#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    pub fn check_class_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_class_declaration"); 
        self.check_grammar_modifiers(node);
        self.check_grammar_class_declaration_heritage_clauses(node);
        self.check_exports_on_merged_declarations(node);
        self.check_type_parameters_on_node(node);

        if node.name().is_none() && !node.has_syntactic_modifier(ModifierFlags::Default) {
            self.grammar_error_on_first_token(
                node,
                &tsox_core::diagnostics::messages_generated::
                    A_CLASS_DECLARATION_WITHOUT_THE_DEFAULT_MODIFIER_MUST_HAVE_A_NAME,
            );
        }

        if let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                self.check_reserved_type_name(
                    name,
                    &tsox_core::diagnostics::messages_generated::CLASS_NAME_CANNOT_BE_0,
                );

                self.check_cjs_reserved_top_level_name(node, name);
            }
        }

        self.push_scope(node);
        self.check_node_decorators(node);

        let this_type = self.build_class_instance_type_with_base(node);
        self.this_type_stack.push(this_type);

        self.enclosing_class_stack.push(Arc::clone(node));

        self.check_class_type_for_duplicate_declarations(node);

        if let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(heritage) = &data.heritage_clauses {
                let mut seen_extends = false;
                let mut seen_implements = false;
                for clause in heritage.iter() {
                    let duplicate_kind = match &clause.data {
                        tsox_frontend::ast::NodeData::HeritageClause(h) => match h.token {
                            SyntaxKind::ExtendsKeyword if seen_extends => true,
                            SyntaxKind::ImplementsKeyword if seen_implements => true,
                            SyntaxKind::ExtendsKeyword => {
                                seen_extends = true;
                                false
                            }
                            SyntaxKind::ImplementsKeyword => {
                                seen_implements = true;
                                false
                            }
                            _ => false,
                        },
                        _ => false,
                    };
                    if !duplicate_kind {
                        self.check_heritage_clause(clause);
                    }
                }
            }

            if !node.has_syntactic_modifier(ModifierFlags::Ambient)
                && self.ambient_context_depth == 0
                && !self
                    .current_file
                    .as_ref()
                    .is_some_and(|f| f.is_declaration_file)
            {
                self.check_class_member_overloads(&data.members);
            }

            for member in data.members.iter() {
                self.check_class_member(member);
            }

            if let Some(this_type) = self.this_type_stack.last().cloned() {
                self.check_index_constraints(&this_type, node);
                // Go checkClassDeclaration：static 索引签名同样过 number⊑string
                // 约束（构造侧类型）
                let static_type = self.get_type_of_class_declaration(node);
                self.check_index_constraints(&static_type, node);
            }
            self.check_class_heritage_members(node);

            self.check_property_accessor_override_kinds(node);

            self.check_members_for_override_modifier(node);

            self.check_property_initialization(node);
        }
        self.pop_scope();
        self.this_type_stack.pop();
        self.enclosing_class_stack.pop();

        let class_type = self.get_type_of_class_declaration(node);
        self.type_node_links.get_or_default(node).resolved_type = Some(class_type.clone());
        if let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &node.data {
            if let Some(name) = &data.name {
                if let Some(symbol) = self.resolve_identifier(name) {
                    self.value_symbol_links
                        .get_or_default(&symbol)
                        .resolved_type = Some(class_type);
                }
            }
        }
    }

    pub fn check_enum_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_enum_declaration"); 
        self.check_grammar_modifiers(node);
        self.check_exports_on_merged_declarations(node);
        // Go checkEnumDeclaration：非 ambient 枚举在 erasableSyntaxOnly 下报
        // TS1294（span 取名字，GetErrorRangeForNode 声明类规则）
        if !self.declaration_is_ambient(node)
            && let tsox_frontend::ast::NodeData::EnumDeclaration(data) = &node.data
        {
            self.erasable_syntax_error(node, data.name.loc);
        }

        if let tsox_frontend::ast::NodeData::EnumDeclaration(data) = &node.data {
            self.check_cjs_reserved_top_level_name(node, &data.name);
            self.check_reserved_type_name(
                &data.name,
                &tsox_core::diagnostics::messages_generated::ENUM_NAME_CANNOT_BE_0,
            );

            if let Some(sym) = self.program.symbol_map().symbol_of(node) {
                let enum_decls: Vec<&Arc<Node>> = sym
                    .declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::EnumDeclaration)
                    .collect();
                if enum_decls.len() > 1 {
                    let is_first_decl = enum_decls.first().is_some_and(|d| Arc::ptr_eq(d, &node));

                    let first_decl_starts_uninit = enum_decls.first().and_then(|d| {
                            let NodeData::EnumDeclaration(ed) = &d.data else {
                                return None;
                            };
                            ed.members.iter().next().and_then(|m| {
                                matches!(&m.data, tsox_frontend::ast::NodeData::EnumMember(em) if em.initializer.is_none())
                                    .then_some(())
                            })
                        }) == Some(());
                    if !is_first_decl && first_decl_starts_uninit {
                        let first_member = data.members.iter().next();
                        let uninit = first_member.is_some_and(|m| {
                            matches!(
                                &m.data,
                                tsox_frontend::ast::NodeData::EnumMember(em)
                                    if em.initializer.is_none()
                            )
                        });
                        if uninit {
                            let loc = first_member
                                .and_then(|m| m.name())
                                .map(|n| n.loc)
                                .unwrap_or(node.loc);
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    loc,
                                    tsox_core::diagnostics::messages_generated::
                                        IN_AN_ENUM_WITH_MULTIPLE_DECLARATIONS_ONLY_ONE_DECLARATION_CAN_OMIT_AN_INITIALIZER_FOR_ITS_FIRST_ENUM_ELEMENT,
                                    Vec::new(),
                                ));
                        }
                    }
                }
            }
        }

        self.push_scope(node);
        if let tsox_frontend::ast::NodeData::EnumDeclaration(data) = &node.data {
            for member in data.members.iter() {
                self.check_enum_member(member);
            }
        }
        self.pop_scope();
        self.compute_enum_member_values(node);
    }

}
