use crate::checker::checker_classes::*;

fn class_like_declarations(symbol: &Arc<Symbol>) -> Vec<Arc<Node>> {
    symbol
        .declarations
        .iter()
        .filter(|d| {
            matches!(
                d.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            )
        })
        .cloned()
        .collect()
}

fn class_display_name(decl: &Arc<Node>) -> String {
    let (name, type_parameters) = match &decl.data {
        tsox_frontend::ast::NodeData::ClassDeclaration(d) => (
            d.name.as_ref().map(|n| n.text().to_string()),
            d.type_parameters.clone(),
        ),
        tsox_frontend::ast::NodeData::ClassExpression(d) => (
            d.name.as_ref().map(|n| n.text().to_string()),
            d.type_parameters.clone(),
        ),
        _ => (None, None),
    };
    let name = name.unwrap_or_default();
    match type_parameters {
        Some(tps) if !tps.nodes.is_empty() => {
            let params: Vec<String> = tps
                .iter()
                .filter_map(|tp| match &tp.data {
                    tsox_frontend::ast::NodeData::TypeParameterDeclaration(td) => {
                        Some(td.name.text().to_string())
                    }
                    _ => None,
                })
                .collect();
            if params.is_empty() {
                name
            } else {
                format!("{name}<{}>", params.join(", "))
            }
        }
        _ => name,
    }
}

fn node_within_class(node: &Arc<Node>, class_decl: &Arc<Node>) -> bool {
    let mut current = node.parent();
    while let Some(c) = current {
        if Arc::ptr_eq(&c, class_decl) {
            return true;
        }
        current = c.parent();
    }
    false
}

impl Checker {
    pub(crate) fn check_new_expression_ctor_accessibility(&mut self, node: &Arc<Node>) -> bool {
        let tsox_frontend::ast::NodeData::NewExpression(data) = &node.data else {
            return false;
        };
        let Some(symbol) = self.symbol_of_entity_expression(&data.expression) else {
            return false;
        };
        let base = self.resolve_alias_base(symbol);
        for decl in class_like_declarations(&base) {
            let members: Vec<Arc<Node>> = match &decl.data {
                tsox_frontend::ast::NodeData::ClassDeclaration(d) => {
                    d.members.iter().cloned().collect()
                }
                tsox_frontend::ast::NodeData::ClassExpression(d) => {
                    d.members.iter().cloned().collect()
                }
                _ => continue,
            };
            for member in members {
                let tsox_frontend::ast::NodeData::ConstructorDeclaration(_) = &member.data else {
                    continue;
                };
                let is_private = member.has_syntactic_modifier(ModifierFlags::Private);
                let is_protected = member.has_syntactic_modifier(ModifierFlags::Protected);
                if !is_private && !is_protected {
                    continue;
                }
                if node_within_class(node, &decl) {
                    continue;
                }
                if is_protected && self.node_class_chain_contains(node, &decl) {
                    continue;
                }
                let message = if is_private {
                    tsox_core::diagnostics::messages_generated::
                        CONSTRUCTOR_OF_CLASS_0_IS_PRIVATE_AND_ONLY_ACCESSIBLE_WITHIN_THE_CLASS_DECLARATION
                } else {
                    tsox_core::diagnostics::messages_generated::
                        CONSTRUCTOR_OF_CLASS_0_IS_PROTECTED_AND_ONLY_ACCESSIBLE_WITHIN_THE_CLASS_DECLARATION
                };
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    node.loc,
                    message,
                    vec![class_display_name(&decl)],
                ));
                return true;
            }
        }
        false
    }

    pub(crate) fn check_extends_private_ctor(&mut self, type_ref: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(ewa) = &type_ref.data else {
            return;
        };
        let Some(symbol) = self.symbol_of_entity_expression(&ewa.expression) else {
            return;
        };
        let base = self.resolve_alias_base(symbol);
        if !base.flags.contains(SymbolFlags::Class) {
            return;
        }
        for decl in class_like_declarations(&base) {
            let has_private_ctor = match &decl.data {
                tsox_frontend::ast::NodeData::ClassDeclaration(d) => d.members.iter().any(|m| {
                    m.kind == SyntaxKind::Constructor
                        && m.has_syntactic_modifier(ModifierFlags::Private)
                }),
                tsox_frontend::ast::NodeData::ClassExpression(d) => d.members.iter().any(|m| {
                    m.kind == SyntaxKind::Constructor
                        && m.has_syntactic_modifier(ModifierFlags::Private)
                }),
                _ => false,
            };
            if has_private_ctor && !node_within_class(type_ref, &decl) {
                let qualified = self.qualified_symbol_name(&base);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    ewa.expression.loc,
                    tsox_core::diagnostics::messages_generated::
                        CANNOT_EXTEND_A_CLASS_0_CLASS_CONSTRUCTOR_IS_MARKED_AS_PRIVATE,
                    vec![qualified],
                ));
                return;
            }
        }
    }

    fn node_class_chain_contains(&self, node: &Arc<Node>, target_decl: &Arc<Node>) -> bool {
        let mut ancestor = node.parent();
        while let Some(a) = ancestor {
            if matches!(
                a.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            ) {
                let mut current = Arc::clone(&a);
                let mut guard = 0;
                loop {
                    guard += 1;
                    if guard > 64 {
                        return false;
                    }
                    if Arc::ptr_eq(&current, target_decl) {
                        return true;
                    }
                    match self.extends_base_of(&current).map(|(n, _)| n) {
                        Some(next) => current = next,
                        None => return false,
                    }
                }
            }
            ancestor = a.parent();
        }
        false
    }

    fn symbol_of_entity_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        match node.kind {
            SyntaxKind::Identifier => self.resolve_identifier(node),
            SyntaxKind::PropertyAccessExpression | SyntaxKind::QualifiedName => {
                self.resolve_qualified_symbol(node)
            }
            _ => None,
        }
    }
}
