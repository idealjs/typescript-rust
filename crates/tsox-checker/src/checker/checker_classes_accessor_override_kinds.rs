use crate::checker::checker_classes::*;

#[derive(PartialEq)]
enum PropMemberKind {
    Property,
    Accessor,
}

fn member_override_kind(member: &Arc<Node>) -> Option<PropMemberKind> {
    match member.kind {
        SyntaxKind::PropertyDeclaration => Some(PropMemberKind::Property),
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => Some(PropMemberKind::Accessor),
        _ => None,
    }
}

fn member_name_text(member: &Arc<Node>) -> Option<String> {
    let name = member.name()?;
    match name.kind {
        SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
            Some(name.text().to_string())
        }
        _ => None,
    }
}

fn member_is_private(member: &Arc<Node>) -> bool {
    member.has_syntactic_modifier(ModifierFlags::Private)
        || member
            .name()
            .is_some_and(|n| n.kind == SyntaxKind::PrivateIdentifier)
}

fn member_is_abstract(member: &Arc<Node>) -> bool {
    member.has_syntactic_modifier(ModifierFlags::Abstract)
}

impl Checker {
    pub(crate) fn check_property_accessor_override_kinds(&self, class_node: &Arc<Node>) {
        let Some((base_node, _)) = self.extends_base_of(class_node) else {
            return;
        };
        let derived_name = Self::class_name_text(class_node);
        let base_name = Self::class_name_text(&base_node);
        let tsox_frontend::ast::NodeData::ClassDeclaration(derived_data) = &class_node.data else {
            return;
        };
        let mut members: Vec<(&Arc<Node>, PropMemberKind)> = Vec::new();
        for member in derived_data.members.iter() {
            match member.kind {
                SyntaxKind::PropertyDeclaration => {
                    if !member.has_syntactic_modifier(ModifierFlags::Static) {
                        members.push((member, PropMemberKind::Property));
                    }
                }
                SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                    if !member.has_syntactic_modifier(ModifierFlags::Static) {
                        members.push((member, PropMemberKind::Accessor));
                    }
                }
                SyntaxKind::Constructor => {
                    if let tsox_frontend::ast::NodeData::ConstructorDeclaration(cd) = &member.data {
                        for p in cd.parameters.iter() {
                            if let tsox_frontend::ast::NodeData::ParameterDeclaration(pdd) = &p.data
                                && pdd
                                    .modifiers
                                    .as_ref()
                                    .is_some_and(|m| !m.list.nodes.is_empty())
                            {
                                members.push((p, PropMemberKind::Property));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        for (member, kind) in members {
            let Some(name) = member_name_text(member) else {
                continue;
            };
            if member_is_private(member) {
                continue;
            }
            let Some(base_kind) = self.base_member_kind(&base_node, &name) else {
                continue;
            };
            if base_kind == kind {
                continue;
            }
            let message = if base_kind == PropMemberKind::Accessor {
                tsox_core::diagnostics::messages_generated::
                    X_0_IS_DEFINED_AS_AN_ACCESSOR_IN_CLASS_1_BUT_IS_OVERRIDDEN_HERE_IN_2_AS_AN_INSTANCE_PROPERTY
            } else {
                tsox_core::diagnostics::messages_generated::
                    X_0_IS_DEFINED_AS_A_PROPERTY_IN_CLASS_1_BUT_IS_OVERRIDDEN_HERE_IN_2_AS_AN_ACCESSOR
            };
            let name_loc = member
                .name()
                .map(|n| n.loc)
                .unwrap_or(member.loc);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                name_loc,
                message,
                vec![name, base_name.clone(), derived_name.clone()],
            ));
        }
    }

    fn base_member_kind(
        &self,
        base_node: &Arc<Node>,
        name: &str,
    ) -> Option<PropMemberKind> {
        let mut current = Arc::clone(base_node);
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 64 {
                return None;
            }
            let tsox_frontend::ast::NodeData::ClassDeclaration(data) = &current.data else {
                return None;
            };
            for member in data.members.iter() {
                let Some(kind) = member_override_kind(member) else {
                    continue;
                };
                if member_is_private(member) || member_is_abstract(member) {
                    continue;
                }
                if member_name_text(member).as_deref() == Some(name) {
                    return Some(kind);
                }
            }
            let next = self.extends_base_of(&current).map(|(n, _)| n);
            match next {
                Some(n) => current = n,
                None => return None,
            }
        }
    }
}
