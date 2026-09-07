#![allow(unused_imports)]

use super::*;

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
                                    crate::ast::ModifierFlags::NonPublicAccessibilityModifier,
                                )
                            })
                            .unwrap_or(false);
                        if non_public {
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                node.loc,
                                crate::diagnostics::messages_generated::
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
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                node.loc,
                crate::diagnostics::messages_generated::TYPE_0_CANNOT_BE_USED_TO_INDEX_TYPE_1,
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
            crate::ast::NodeData::HeritageClause(d) => d,
            _ => return,
        };
        if data.token == SyntaxKind::ExtendsKeyword {
            if data.types.len() > 1 {
                for type_ref in data.types.iter().skip(1) {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        type_ref.loc,
                        crate::diagnostics::messages_generated::
                            CLASSES_CAN_ONLY_EXTEND_A_SINGLE_CLASS,
                        Vec::new(),
                    ));
                }
            }

            for type_ref in data.types.iter() {
                if let crate::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data {
                    if ewa.expression.kind == SyntaxKind::Identifier {
                        if let Some(sym) = self.resolve_identifier(&ewa.expression)
                            && sym.flags == SymbolFlags::Interface
                        {
                            let name = ewa.expression.text().to_string();
                            let file = self.current_file.clone();
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                file,
                                ewa.expression.loc,
                                crate::diagnostics::messages_generated::
                                    CANNOT_EXTEND_AN_INTERFACE_0_DID_YOU_MEAN_IMPLEMENTS,
                                vec![name],
                            ));
                        }
                    }

                    self.push_ts2304_suppression();
                    let _ = self.get_type_from_type_node(&ewa.expression);
                    self.pop_ts2304_suppression();
                }
            }
            return;
        }
        if data.token != SyntaxKind::ImplementsKeyword {
            return;
        }

        let class_node = match node.parent.as_ref() {
            Some(p) => p,
            None => return,
        };
        let class_data = match &class_node.data {
            crate::ast::NodeData::ClassDeclaration(d) => d,
            _ => return,
        };
        let class_name = class_data
            .name
            .as_ref()
            .map(|n| n.text().to_string())
            .unwrap_or_default();

        let instance_type = self.build_class_instance_type_with_base(class_node);

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
                        crate::ast::NodeData::PropertyDeclaration(d) => &d.name,
                        crate::ast::NodeData::MethodDeclaration(d) => &d.name,
                        crate::ast::NodeData::GetAccessorDeclaration(d) => &d.name,
                        crate::ast::NodeData::SetAccessorDeclaration(d) => &d.name,
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
                    if !self.is_type_assignable_to(&prop_type, &base_type) {
                        let class_str = self.type_to_string(&instance_type);
                        let iface_str = self.type_to_string(&interface_type);
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            name_node.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                            vec![prop_name, class_str, iface_str],
                        ));
                        issued_member_error = true;
                        break;
                    }
                }
                if !issued_member_error {
                    let iface_name = self.type_to_string(&interface_type);
                    self.grammar_error_on_node_with_args(
                        class_node,
                        &crate::diagnostics::messages_generated::CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1,
                        &[class_name.clone(), iface_name],
                    );
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn build_class_instance_type(&mut self, members: &Arc<NodeList>) -> Arc<Type> {
        self.build_interface_type_from_members(members)
    }
}
