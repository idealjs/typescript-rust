use crate::checker::checker_unused_diagnostics::*;

fn containing_parameter(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = node.parent();
    while let Some(c) = current {
        match c.kind {
            SyntaxKind::Parameter => return Some(Arc::clone(&c)),
            SyntaxKind::BindingElement
            | SyntaxKind::ObjectBindingPattern
            | SyntaxKind::ArrayBindingPattern => {
                current = c.parent();
            }
            _ => return None,
        }
    }
    None
}

fn containing_function_body_missing(node: &Arc<Node>) -> bool {
    let mut current = node.parent();
    while let Some(c) = current {
        let body_missing = match &c.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.body.is_none(),
            tsox_frontend::ast::NodeData::FunctionTypeNode(_) => true,
            tsox_frontend::ast::NodeData::MethodSignatureDeclaration(_) => true,
            tsox_frontend::ast::NodeData::ConstructorDeclaration(d) => d.body.is_none(),
            tsox_frontend::ast::NodeData::FunctionExpression(_) => false,
            tsox_frontend::ast::NodeData::ArrowFunction(_) => false,
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => d.body.is_none(),
            _ => false,
        };
        if body_missing {
            return true;
        }
        if matches!(
            c.kind,
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionType
                | SyntaxKind::MethodSignature
                | SyntaxKind::Constructor
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
        ) {
            return false;
        }
        current = c.parent();
    }
    false
}

impl Checker {
    pub(crate) fn check_unused_renamed_binding_elements(&mut self, file_node: &Arc<Node>) {
        let mut worklist = vec![Arc::clone(file_node)];
        while let Some(n) = worklist.pop() {
            if n.kind == SyntaxKind::BindingElement
                && let tsox_frontend::ast::NodeData::BindingElement(be) = &n.data
                && let Some(prop_name) = &be.property_name
                && prop_name.kind == SyntaxKind::Identifier
                && let Some(name) = &be.name
                && name.kind == SyntaxKind::Identifier
                && containing_parameter(&n).is_some()
                && containing_function_body_missing(&n)
            {
                self.report_unused_renamed_binding_element(&n, name, prop_name);
            }
            tsox_frontend::ast::node_data_generated::for_each_child(&n, |child| {
                worklist.push(Arc::clone(child));
                false
            });
        }
    }

    fn report_unused_renamed_binding_element(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
        prop_name: &Arc<Node>,
    ) {
        let Some(sym) = self.program.symbol_map().symbol_of(node) else {
            return;
        };
        if self
            .symbol_reference_kinds
            .get(&sym.id())
            .is_some_and(|k| !k.is_empty())
        {
            return;
        }
        let mut wrapping = Arc::clone(node);
        while let Some(parent) = wrapping.parent() {
            if matches!(
                parent.kind,
                SyntaxKind::BindingElement | SyntaxKind::ObjectBindingPattern
            ) {
                wrapping = Arc::clone(&parent);
            } else {
                break;
            }
        }
        let param = containing_parameter(&wrapping);
        let has_type_annotation = param
            .as_ref()
            .and_then(|p| match &p.data {
                tsox_frontend::ast::NodeData::ParameterDeclaration(d) => {
                    d.type_node.as_ref().map(|_| ())
                }
                _ => None,
            })
            .is_some();
        let file = self.current_file.clone();
        let mut diag = tsox_frontend::ast::Diagnostic::new(
            file,
            name.loc,
            tsox_core::diagnostics::messages_generated::
                X_0_IS_AN_UNUSED_RENAMING_OF_1_DID_YOU_INTEND_TO_USE_IT_AS_A_TYPE_ANNOTATION,
            vec![name.text().to_string(), prop_name.text().to_string()],
        );
        if !has_type_annotation
            && let Some(p) = param
        {
            let end_loc = tsox_core::core::text::TextRange::new(p.loc.end(), p.loc.end());
            diag.related_information
                .push(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    end_loc,
                    tsox_core::diagnostics::messages_generated::
                        WE_CAN_ONLY_WRITE_A_TYPE_FOR_0_BY_ADDING_A_TYPE_FOR_THE_ENTIRE_PARAMETER_HERE,
                    vec![prop_name.text().to_string()],
                ));
        }
        self.diagnostics.add(diag);
    }
}
