#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_private_name_conflicts(&mut self, node: &Arc<Node>) {
        let class_node = node.parent.clone();
        if let Some(cls) = &class_node
            && matches!(
                cls.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            )
            && let crate::ast::NodeData::ClassDeclaration(cd) = &cls.data
        {
            let (my_name, my_loc) = match &node.data {
                crate::ast::NodeData::PropertyDeclaration(d) => {
                    (d.name.text().to_string(), d.name.loc)
                }
                crate::ast::NodeData::MethodDeclaration(d) => {
                    (d.name.text().to_string(), d.name.loc)
                }
                _ => (String::new(), node.loc),
            };
            if !my_name.is_empty() && my_name.starts_with('#') {
                let i_am_static = node.has_syntactic_modifier(ModifierFlags::Static);
                let conflict = cd.members.iter().any(|m| {
                    if m.loc.pos() >= node.loc.pos() {
                        return false;
                    }
                    let Some(mn) = m.name() else { return false };
                    mn.kind == SyntaxKind::PrivateIdentifier
                        && mn.text() == my_name
                        && m.has_syntactic_modifier(ModifierFlags::Static) != i_am_static
                });
                if conflict {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        my_loc,
                        crate::diagnostics::messages_generated::
                            DUPLICATE_IDENTIFIER_0_STATIC_AND_INSTANCE_ELEMENTS_CANNOT_SHARE_THE_SAME_PRIVATE_NAME,
                        vec![my_name.clone()],
                    ));
                }
            }
            if !my_name.is_empty() && !my_name.starts_with('#') {
                let kind_of = |m: &Arc<Node>| match &m.data {
                    crate::ast::NodeData::PropertyDeclaration(_) => "prop",
                    crate::ast::NodeData::MethodDeclaration(d) => {
                        if d.body.is_some() {
                            "method-body"
                        } else {
                            "method-sig"
                        }
                    }
                    crate::ast::NodeData::GetAccessorDeclaration(_) => "get",
                    crate::ast::NodeData::SetAccessorDeclaration(_) => "set",
                    _ => "",
                };
                let mine = kind_of(node);
                let mut theirs_all_prop = true;
                let dup = cd.members.iter().any(|m| {
                    if Arc::ptr_eq(m, node) || m.loc.pos() >= node.loc.pos() {
                        return false;
                    }
                    let name_match = match &m.data {
                        crate::ast::NodeData::PropertyDeclaration(d) => d.name.text() == my_name,
                        crate::ast::NodeData::MethodDeclaration(d) => d.name.text() == my_name,
                        _ => false,
                    };
                    if !name_match {
                        return false;
                    }
                    let theirs = kind_of(m);
                    if theirs != "prop" {
                        theirs_all_prop = false;
                    }
                    match (mine, theirs) {
                        ("prop", "prop") => true,
                        ("prop", "method-body") | ("prop", "method-sig") => true,
                        ("method-body", "prop") | ("method-sig", "prop") => true,

                        ("method-body", "method-body") => true,
                        _ => false,
                    }
                });

                let earlier_has_prop = cd.members.iter().any(|m| {
                    m.loc.pos() < node.loc.pos()
                        && matches!(&m.data, crate::ast::NodeData::PropertyDeclaration(d) if d.name.text() == my_name)
                });

                let earlier_has_method = cd.members.iter().any(|m| {
                    m.loc.pos() < node.loc.pos()
                        && matches!(&m.data, crate::ast::NodeData::MethodDeclaration(d) if d.name.text() == my_name)
                });
                let report_here = match mine {
                    "prop" => true,
                    "method-body" | "method-sig" => earlier_has_prop,
                    _ => false,
                };
                let _ = earlier_has_method;
                if dup && report_here {
                    let file = self.current_file.clone();
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        file,
                        my_loc,
                        crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                        vec![my_name.clone()],
                    ));
                }

                if dup && mine == "prop" && !report_here {
                    self.diagnostics.add(crate::ast::Diagnostic::new(
                        self.current_file.clone(),
                        my_loc,
                        crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                        vec![my_name.clone()],
                    ));
                }

                if dup && matches!(mine, "method-body" | "method-sig") && theirs_all_prop {
                    if let Some(earlier) = cd.members.iter().find(|m| {
                        m.loc.pos() < node.loc.pos()
                            && matches!(&m.data, crate::ast::NodeData::PropertyDeclaration(d) if d.name.text() == my_name)
                    }) {
                        let earlier_loc = earlier
                            .name()
                            .map(|n| n.loc)
                            .unwrap_or(earlier.loc);
                        let already = self
                            .diagnostics
                            .get_all()
                            .iter()
                            .any(|d| d.code == 2300 && d.loc == earlier_loc);
                        if !already {
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                earlier_loc,
                                crate::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0,
                                vec![my_name.clone()],
                            ));
                        }
                    }
                }

                if dup && mine == "prop" {
                    let first = cd.members.iter().find(|m| {
                        m.loc.pos() < node.loc.pos()
                            && match &m.data {
                                crate::ast::NodeData::PropertyDeclaration(d) => {
                                    d.name.text() == my_name
                                }
                                crate::ast::NodeData::MethodDeclaration(d) => {
                                    d.name.text() == my_name
                                }
                                _ => false,
                            }
                    });
                    if let Some(first) = first {
                        let first_type = match &first.data {
                            crate::ast::NodeData::PropertyDeclaration(d) => {
                                let tn = d.type_node.clone();
                                tn.map(|tn| {
                                    let t = self.get_type_from_type_node(&tn);
                                    self.type_to_string(&t)
                                })
                            }
                            _ => None,
                        };
                        let first_sig = match &first.data {
                            crate::ast::NodeData::MethodDeclaration(d) => {
                                let ret = d
                                    .type_node
                                    .as_ref()
                                    .map(|tn| self.get_type_from_type_node(tn))
                                    .unwrap_or_else(|| self.any_type());
                                let sig = self.build_signature_from_function_like_type_node(
                                    &d.parameters,
                                    ret,
                                    false,
                                    None,
                                    Some(Arc::clone(first)),
                                );
                                Some(self.type_to_string(
                                    &self.create_function_or_constructor_type(vec![sig], false),
                                ))
                            }
                            _ => None,
                        };
                        let later_type = match &node.data {
                            crate::ast::NodeData::PropertyDeclaration(d) => {
                                let tn = d.type_node.clone();
                                tn.map(|tn| {
                                    let t = self.get_type_from_type_node(&tn);
                                    self.type_to_string(&t)
                                })
                            }
                            _ => None,
                        };
                        if let (Some(f), Some(l)) = (first_type.or(first_sig), later_type) {
                            if f != l {
                                self.diagnostics.add(crate::ast::Diagnostic::new(
                                    self.current_file.clone(),
                                    my_loc,
                                    crate::diagnostics::messages_generated::
                                        SUBSEQUENT_PROPERTY_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_PROPERTY_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2,
                                    vec![my_name.clone(), f, l],
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}
