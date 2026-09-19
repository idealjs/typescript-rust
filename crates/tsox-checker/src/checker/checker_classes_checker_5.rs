#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn check_indexed_access_index_type(&mut self, node: &Arc<Node>) {
        use crate::checker::types::{TypeData, TypeFlags};
        let t = self.get_type_from_type_node(node);

        if !self.type_argument_stack.is_empty() {
            return;
        }

        if self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"))
        {
            return;
        }
        let (object_type, index_type) = match &t.data {
            TypeData::IndexedAccess(d) => match (&d.object_type, &d.index_type) {
                (Some(o), Some(i)) => (Arc::clone(o), Arc::clone(i)),
                _ => return,
            },
            _ => return,
        };

        if object_type
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown)
        {
            return;
        }

        if self.type_flags_is_generic_object_type(&object_type) {
            return;
        }

        let object_index_type = self.get_index_type(&object_type);
        let has_number_index_info = self
            .get_index_info_of_type(&object_type, &self.number_type())
            .is_some();

        let constituents: Vec<Arc<Type>> = if index_type.flags.contains(TypeFlags::Union) {
            match &index_type.data {
                TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => vec![Arc::clone(&index_type)],
            }
        } else {
            vec![Arc::clone(&index_type)]
        };
        for c in &constituents {
            let mut ok = self.is_type_assignable_to(c, &object_index_type);
            if !ok && has_number_index_info {
                ok = self.is_type_assignable_to(c, &self.number_type());
            }
            if ok {
                continue;
            }
            if object_type
                .object_flags
                .intersects(crate::checker::types::ObjectFlags::IsGenericObjectType)
            {
                if let Some(name) = self.property_name_from_index(c) {
                    if let Some(sym) = self.get_constituent_property(&object_type, &name) {
                        let non_public = sym
                            .value_declaration
                            .as_ref()
                            .map(|d| {
                                self.get_combined_modifier_flags(d).intersects(
                                    tsox_frontend::ast::ModifierFlags::NonPublicAccessibilityModifier,
                                )
                            })
                            .unwrap_or(false);
                        if non_public {
                            let file = self.current_file.clone();
                            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                file,
                                node.loc,
                                tsox_core::diagnostics::messages_generated::
                                    PRIVATE_OR_PROTECTED_MEMBER_0_CANNOT_BE_ACCESSED_ON_A_TYPE_PARAMETER,
                                vec![name],
                            ));
                            return;
                        }
                    }
                }
            }
            let index_display = self.type_to_string(&index_type);
            let object_display = self.type_to_string(&object_type);
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::TYPE_0_CANNOT_BE_USED_TO_INDEX_TYPE_1,
                vec![index_display, object_display],
            ));
            return;
        }
    }

    pub(crate) fn property_name_from_index(&mut self, t: &Arc<Type>) -> Option<String> {
        use crate::checker::types::{TypeData, TypeFlags};
        if t.flags
            .intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral)
        {
            if let TypeData::Literal(l) = &t.data {
                return match &l.value {
                    crate::checker::types::LiteralValue::String(s) => Some(s.clone()),
                    crate::checker::types::LiteralValue::Number(n) => Some(n.to_string()),
                    _ => None,
                };
            }
        }
        None
    }
    pub(crate) fn check_heritage_clause(&mut self, node: &Arc<Node>) {
        let data = match &node.data {
            tsox_frontend::ast::NodeData::HeritageClause(d) => d,
            _ => return,
        };
        if data.token == SyntaxKind::ExtendsKeyword {
            if data.types.len() > 1 {
                for type_ref in data.types.iter().skip(1) {
                    let file = self.current_file.clone();
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        type_ref.loc,
                        tsox_core::diagnostics::messages_generated::
                            CLASSES_CAN_ONLY_EXTEND_A_SINGLE_CLASS,
                        Vec::new(),
                    ));
                }
            }

            for type_ref in data.types.iter() {
                if let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ewa) =
                    &type_ref.data
                {
                    if ewa.expression.kind == SyntaxKind::Identifier {
                        if let Some(sym) = self.resolve_identifier(&ewa.expression) {
                            // Go checkResolvedBlockScopedVariable：基类在声明前使用
                            //（isBlockScopedNameDeclaredBeforeUse 判定）
                            let name = ewa.expression.text().to_string();
                            self.check_block_scoped_variable_used_before_declaration(
                                &ewa.expression,
                                &sym,
                                &name,
                            );
                            if sym.flags == SymbolFlags::Interface {
                                let name = ewa.expression.text().to_string();
                                let file = self.current_file.clone();
                                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                                    file,
                                    ewa.expression.loc,
                                    tsox_core::diagnostics::messages_generated::
                                        CANNOT_EXTEND_AN_INTERFACE_0_DID_YOU_MEAN_IMPLEMENTS,
                                    vec![name],
                                ));
                            }
                        }
                    }

                    self.push_ts2304_suppression();
                    let base_type = self.get_type_from_heritage_type_reference(type_ref);
                    self.pop_ts2304_suppression();

                    // Go checkClassDeclaration：派生实例型须可赋给基类实例型，
                    // 否则 TS2415（成员级错误链随附，含私有分离说明）
                    if !base_type.flags.contains(TypeFlags::Any) {
                        let class_node = node.parent();
                        if let Some(class_node) = class_node
                            && let tsox_frontend::ast::NodeData::ClassDeclaration(cd) =
                                &class_node.data
                        {
                            let instance_type =
                                self.build_class_instance_type_with_base(&class_node);
                            let saved_chain = std::mem::take(&mut self.relater_error_chain);
                            let was_active = self.relater_chain_active;
                            self.relater_chain_active = true;
                            let overall_ok =
                                self.is_type_assignable_to(&instance_type, &base_type);
                            let entries =
                                std::mem::replace(&mut self.relater_error_chain, saved_chain);
                            self.relater_chain_active = was_active;
                            if !overall_ok {
                                // Go issueMemberSpecificError：优先成员级 TS2416
                                let mut issued_member_error = false;
                                let instance_str = self.type_to_string(&instance_type);
                                let base_str = self.type_to_string(&base_type);
                                for member in cd.members.iter() {
                                    if member.has_syntactic_modifier(ModifierFlags::Static) {
                                        continue;
                                    }
                                    let Some(name_node) = member.name() else {
                                        continue;
                                    };
                                    let prop_name = name_node.text().to_string();
                                    if prop_name.is_empty() {
                                        continue;
                                    }
                                    let Some(prop) =
                                        self.get_property_of_type(&instance_type, &prop_name)
                                    else {
                                        continue;
                                    };
                                    let Some(base_prop) =
                                        self.get_property_of_type(&base_type, &prop_name)
                                    else {
                                        continue;
                                    };
                                    let prop_t = self.get_type_of_symbol(&prop);
                                    let base_t = self.get_type_of_symbol(&base_prop);
                                    let saved_chain2 =
                                        std::mem::take(&mut self.relater_error_chain);
                                    let was_active2 = self.relater_chain_active;
                                    self.relater_chain_active = true;
                                    let member_ok =
                                        self.is_type_assignable_to(&prop_t, &base_t);
                                    let member_entries = std::mem::replace(
                                        &mut self.relater_error_chain,
                                        saved_chain2,
                                    );
                                    self.relater_chain_active = was_active2;
                                    if !member_ok {
                                        let _ = &member_entries;
                                        // heritage 二次求值防护：同位同码已有 TS2416 不再发
                                        let already = self.diagnostics.get_all().iter().any(
                                            |d| d.code == 2416 && d.loc == name_node.loc,
                                        );
                                        if already {
                                            continue;
                                        }
                                        let mut diag = tsox_frontend::ast::Diagnostic::new(
                                            self.current_file.clone(),
                                            name_node.loc,
                                            tsox_core::diagnostics::messages_generated::
                                                PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                                            vec![prop_name.clone(), instance_str.clone(), base_str.clone()],
                                        );
                                        let dt_str = self.type_to_string(&prop_t);
                                        let bt_str = self.type_to_string(&base_t);
                                        self.relater_error_chain = member_entries;
                                        self.relater_chain_active = true;
                                        self.push_relation_head_with_tp_note(
                                            &prop_t,
                                            &base_t,
                                            tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                            vec![dt_str, bt_str],
                                        );
                                        let final_entries =
                                            std::mem::take(&mut self.relater_error_chain);
                                        self.relater_chain_active = was_active2;
                                        let mut child: Option<
                                            tsox_frontend::ast::Diagnostic,
                                        > = None;
                                        for entry in final_entries
                                            .iter()
                                            .filter(|e| !e.message.elided_in_compatibility_pyramid)
                                        {
                                            let mut d = tsox_frontend::ast::Diagnostic::new(
                                                None,
                                                name_node.loc,
                                                entry.message,
                                                entry.args.clone(),
                                            );
                                            if let Some(c) = child.take() {
                                                d.message_chain = vec![c];
                                            }
                                            child = Some(d);
                                        }
                                        if let Some(c) = child {
                                            diag.message_chain = vec![c];
                                        }
                                        self.diagnostics.add(diag);
                                        issued_member_error = true;
                                    }
                                }
                                if !issued_member_error {
                                    let error_loc = cd
                                        .name
                                        .as_ref()
                                        .map(|n| n.loc)
                                        .unwrap_or(class_node.loc);
                                    let mut diag = tsox_frontend::ast::Diagnostic::new(
                                        self.current_file.clone(),
                                        error_loc,
                                        tsox_core::diagnostics::messages_generated::
                                            CLASS_0_INCORRECTLY_EXTENDS_BASE_CLASS_1,
                                        vec![instance_str.clone(), base_str.clone()],
                                    );
                                    let mut child: Option<tsox_frontend::ast::Diagnostic> =
                                        None;
                                    for entry in entries
                                        .iter()
                                        .filter(|e| !e.message.elided_in_compatibility_pyramid)
                                    {
                                        let mut d = tsox_frontend::ast::Diagnostic::new(
                                            None,
                                            error_loc,
                                            entry.message,
                                            entry.args.clone(),
                                        );
                                        if let Some(c) = child.take() {
                                            d.message_chain = vec![c];
                                        }
                                        child = Some(d);
                                    }
                                    if let Some(c) = child {
                                        diag.message_chain = vec![c];
                                    }
                                    self.diagnostics.add(diag);
                                }
                            }

                            // Go：实例可赋时才查静态侧（TS2417）
                            if overall_ok {
                                let base_node = base_type.symbol.as_ref().and_then(|sym| {
                                    sym.declarations.iter().find(|d| {
                                        matches!(
                                            d.data,
                                            tsox_frontend::ast::NodeData::ClassDeclaration(_)
                                        )
                                    }).cloned()
                                });
                                if let Some(base_node) = base_node {
                                    let mut static_bad: Option<(
                                        Arc<Node>,
                                        String,
                                        bool,
                                    )> = None;
                                    for member in cd.members.iter() {
                                        if !member.has_syntactic_modifier(ModifierFlags::Static) {
                                            continue;
                                        }
                                        let Some(name_node) = member.name() else {
                                            continue;
                                        };
                                        let prop_name = name_node.text().to_string();
                                        if prop_name.is_empty() {
                                            continue;
                                        }
                                        let Some(own_sym) = self
                                            .program
                                            .symbol_map()
                                            .symbol_of(member)
                                            .cloned()
                                        else {
                                            continue;
                                        };
                                        let base_member_opt =
                                            if let tsox_frontend::ast::NodeData::ClassDeclaration(
                                                bd,
                                            ) = &base_node.data
                                            {
                                                bd.members.iter().find(|m| {
                                                    m.has_syntactic_modifier(
                                                        ModifierFlags::Static,
                                                    ) && m.name().is_some_and(|n| {
                                                        n.kind == SyntaxKind::Identifier
                                                            && n.text() == prop_name
                                                    })
                                                })
                                                .cloned()
                                            } else {
                                                None
                                            };
                                        let Some(base_member) = base_member_opt else {
                                            continue;
                                        };
                                        let own_t = self.get_type_of_symbol(&own_sym);
                                        let base_sym = self
                                            .program
                                            .symbol_map()
                                            .symbol_of(&base_member)
                                            .cloned();
                                        let Some(base_sym) = base_sym else {
                                            continue;
                                        };
                                        let base_t = self.get_type_of_symbol(&base_sym);
                                        let both_private = member
                                            .has_syntactic_modifier(ModifierFlags::Private)
                                            && base_member
                                                .has_syntactic_modifier(ModifierFlags::Private);
                                        // 静态可见性收窄：protected 覆盖 public 同报
                                        let visibility_narrowed = member
                                            .has_syntactic_modifier(ModifierFlags::Protected)
                                            && !base_member
                                                .has_syntactic_modifier(ModifierFlags::Protected);
                                        let related =
                                            self.is_type_assignable_to(&own_t, &base_t);
                                        let identical_ok = if !both_private {
                                            true
                                        } else {
                                            Arc::ptr_eq(&own_t, &base_t) || {
                                                let a = self.type_to_string(&own_t);
                                                let b = self.type_to_string(&base_t);
                                                a == b
                                            }
                                        };
                                        if !related || !identical_ok || visibility_narrowed {
                                            static_bad = Some((
                                                Arc::clone(&name_node),
                                                prop_name,
                                                visibility_narrowed,
                                            ));
                                            break;
                                        }
                                    }
                                    if let Some((name_node, prop_name, narrowed)) =
                                        static_bad
                                    {
                                        let class_name = cd
                                            .name
                                            .as_ref()
                                            .map(|n| n.text().to_string())
                                            .unwrap_or_default();
                                        let mut diag =
                                            tsox_frontend::ast::Diagnostic::new(
                                                self.current_file.clone(),
                                                cd.name
                                                    .as_ref()
                                                    .map(|n| n.loc)
                                                    .unwrap_or(class_node.loc),
                                                tsox_core::diagnostics::messages_generated::
                                                    CLASS_STATIC_SIDE_0_INCORRECTLY_EXTENDS_BASE_CLASS_STATIC_SIDE_1,
                                                vec![
                                                    format!("typeof {class_name}"),
                                                    format!("typeof {}", self.type_to_string(&base_type)),
                                                ],
                                            );
                                        let chain_msg = if narrowed {
                                            tsox_core::diagnostics::messages_generated::
                                                PROPERTY_0_IS_PROTECTED_IN_TYPE_1_BUT_PUBLIC_IN_TYPE_2
                                        } else {
                                            tsox_core::diagnostics::messages_generated::
                                                TYPES_HAVE_SEPARATE_DECLARATIONS_OF_A_PRIVATE_PROPERTY_0
                                        };
                                        let chain_args = if narrowed {
                                            vec![
                                                prop_name,
                                                format!("typeof {class_name}"),
                                                format!(
                                                    "typeof {}",
                                                    self.type_to_string(&base_type)
                                                ),
                                            ]
                                        } else {
                                            vec![prop_name]
                                        };
                                        diag.message_chain = vec![
                                            tsox_frontend::ast::Diagnostic::new(
                                                None,
                                                name_node.loc,
                                                chain_msg,
                                                chain_args,
                                            ),
                                        ];
                                        self.diagnostics.add(diag);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return;
        }
        if data.token != SyntaxKind::ImplementsKeyword {
            return;
        }

        let class_node = match node.parent() {
            Some(p) => p,
            None => return,
        };
        let class_data = match &class_node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(d) => d,
            _ => return,
        };

        let instance_type = self.build_class_instance_type_with_base(&class_node);
        let class_name = self.type_to_string(&instance_type);

        for type_ref in data.types.iter() {
            let interface_type = self.get_type_from_heritage_type_reference(type_ref);
            if interface_type.flags.contains(TypeFlags::Any) {
                continue;
            }
            if !self.is_type_assignable_to(&instance_type, &interface_type) {
                let mut issued_member_error = false;
                for member in class_data.members.iter() {
                    if member.has_syntactic_modifier(ModifierFlags::Static) {
                        continue;
                    }
                    let name_node = match &member.data {
                        tsox_frontend::ast::NodeData::PropertyDeclaration(d) => &d.name,
                        tsox_frontend::ast::NodeData::MethodDeclaration(d) => &d.name,
                        tsox_frontend::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                        tsox_frontend::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
                        _ => continue,
                    };
                    let prop_name = name_node.text().to_string();
                    if prop_name.is_empty() {
                        continue;
                    }
                    let Some(prop) = self.get_property_of_type(&instance_type, &prop_name) else {
                        continue;
                    };
                    let Some(base_prop) = self.get_property_of_type(&interface_type, &prop_name)
                    else {
                        continue;
                    };
                    let prop_type = self.get_type_of_symbol(&prop);
                    let base_type = self.get_type_of_symbol(&base_prop);

                    let saved_chain = std::mem::take(&mut self.relater_error_chain);
                    let was_active = self.relater_chain_active;
                    self.relater_chain_active = true;
                    let assignable = self.is_type_assignable_to(&prop_type, &base_type);
                    let captured = std::mem::replace(&mut self.relater_error_chain, saved_chain);
                    self.relater_chain_active = was_active;
                    if assignable {
                        continue;
                    }
                    let class_str = self.type_to_string(&instance_type);
                    let iface_str = self.type_to_string(&interface_type);
                    let file = self.current_file.clone();
                    let mut diag = tsox_frontend::ast::Diagnostic::new(
                        file,
                        name_node.loc,
                        tsox_core::diagnostics::messages_generated::
                            PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                        vec![prop_name, class_str, iface_str],
                    );

                    let dt_str = self.type_to_string(&prop_type);
                    let bt_str = self.type_to_string(&base_type);
                    self.relater_error_chain = captured;
                    self.relater_chain_active = true;
                    self.push_relation_head_with_tp_note(
                        &prop_type,
                        &base_type,
                        tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                        vec![dt_str, bt_str],
                    );
                    let entries = std::mem::take(&mut self.relater_error_chain);
                    self.relater_chain_active = was_active;

                    let mut child: Option<tsox_frontend::ast::Diagnostic> = None;
                    for entry in entries
                        .iter()
                        .filter(|e| !e.message.elided_in_compatibility_pyramid)
                    {
                        let mut d = tsox_frontend::ast::Diagnostic::new(
                            None,
                            name_node.loc,
                            entry.message,
                            entry.args.clone(),
                        );
                        if let Some(c) = child.take() {
                            d.message_chain = vec![c];
                        }
                        child = Some(d);
                    }
                    if let Some(c) = child {
                        diag.message_chain = vec![c];
                    }
                    self.diagnostics.add(diag);
                    issued_member_error = true;
                }
                if !issued_member_error {
                    let iface_name = self.type_to_string(&interface_type);
                    let saved_chain = std::mem::take(&mut self.relater_error_chain);
                    let was_active = self.relater_chain_active;
                    self.relater_chain_active = true;
                    let _ = self.is_type_assignable_to(&instance_type, &interface_type);
                    let entries = std::mem::replace(&mut self.relater_error_chain, saved_chain);
                    self.relater_chain_active = was_active;

                    let error_loc = class_data
                        .name
                        .as_ref()
                        .map(|n| n.loc)
                        .unwrap_or(class_node.loc);
                    let file = self.current_file.clone();
                    let mut diag = tsox_frontend::ast::Diagnostic::new(
                        file,
                        error_loc,
                        tsox_core::diagnostics::messages_generated::
                            CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1,
                        vec![class_name.clone(), iface_name],
                    );
                    let mut child: Option<tsox_frontend::ast::Diagnostic> = None;
                    for entry in entries
                        .iter()
                        .filter(|e| !e.message.elided_in_compatibility_pyramid)
                    {
                        let mut d = tsox_frontend::ast::Diagnostic::new(
                            None,
                            error_loc,
                            entry.message,
                            entry.args.clone(),
                        );
                        if let Some(c) = child.take() {
                            d.message_chain = vec![c];
                        }
                        child = Some(d);
                    }
                    if let Some(c) = child {
                        diag.message_chain = vec![c];
                    }
                    self.diagnostics.add(diag);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn build_class_instance_type(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        self.build_interface_type_from_members(members)
    }
}
