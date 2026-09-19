use crate::checker::checker_expressions::*;

impl Checker {
    pub(crate) fn check_object_literal_spread_overrides(&mut self, node: &Arc<Node>) {
        if !self.strict_null_checks {
            return;
        }
        let tsox_frontend::ast::NodeData::ObjectLiteralExpression(data) = &node.data else {
            return;
        };
        let mut own_props: Vec<(String, Arc<Node>)> = Vec::new();
        let mut reported: std::collections::HashSet<String> = std::collections::HashSet::new();
        for prop in data.properties.iter() {
            match prop.kind {
                SyntaxKind::PropertyAssignment
                | SyntaxKind::ShorthandPropertyAssignment
                | SyntaxKind::MethodDeclaration => {
                    if let Some(name) = Self::literal_property_name(prop) {
                        own_props.push((name, Arc::clone(prop)));
                    }
                }
                SyntaxKind::SpreadElement | SyntaxKind::SpreadAssignment => {
                    let Some(spread_expr) = Self::spread_expression(prop) else {
                        continue;
                    };
                    let spread_type = self.get_type_of_node(spread_expr);
                    self.check_spread_prop_overrides(&spread_type, &own_props, &mut reported, prop);
                }
                _ => {}
            }
        }
    }

    fn spread_expression(prop: &Arc<Node>) -> Option<&Arc<Node>> {
        match &prop.data {
            tsox_frontend::ast::NodeData::SpreadElement(d) => Some(&d.expression),
            tsox_frontend::ast::NodeData::SpreadAssignment(d) => Some(&d.expression),
            _ => None,
        }
    }

    pub(crate) fn check_jsx_attributes_spread_overrides(&mut self, attrs: &Arc<Node>) {
        if !self.strict_null_checks {
            return;
        }
        let tsox_frontend::ast::NodeData::JsxAttributes(data) = &attrs.data else {
            return;
        };
        let mut own_props: Vec<(String, Arc<Node>)> = Vec::new();
        let mut reported: std::collections::HashSet<String> = std::collections::HashSet::new();
        for attr in data.properties.iter() {
            match &attr.data {
                tsox_frontend::ast::NodeData::JsxAttribute(_) => {
                    if let Some(name) = attr.name().map(|n| n.text().to_string()) {
                        own_props.push((name, Arc::clone(attr)));
                    }
                }
                tsox_frontend::ast::NodeData::JsxSpreadAttribute(spread) => {
                    let spread_type = self.get_type_of_node(&spread.expression);
                    self.check_spread_prop_overrides(
                        &spread_type,
                        &own_props,
                        &mut reported,
                        attr,
                    );
                }
                _ => {}
            }
        }
    }

    fn check_spread_prop_overrides(
        &mut self,
        spread_type: &Arc<Type>,
        own_props: &[(String, Arc<Node>)],
        reported: &mut std::collections::HashSet<String>,
        spread_node: &Arc<Node>,
    ) {
        for right in self.get_properties_of_type(spread_type) {
            if right.flags.contains(SymbolFlags::Optional) || reported.contains(&right.name) {
                continue;
            }
            if let Some((name, own_node)) = own_props.iter().find(|(n, _)| *n == right.name) {
                reported.insert(name.clone());
                self.report_spread_overwrites_property(name, own_node, spread_node);
            }
        }
    }

    fn literal_property_name(prop: &Arc<Node>) -> Option<String> {
        let name = prop.name()?;
        match name.kind {
            SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                Some(name.text().to_string())
            }
            _ => None,
        }
    }

    fn report_spread_overwrites_property(
        &mut self,
        name: &str,
        own_node: &Arc<Node>,
        spread_node: &Arc<Node>,
    ) {
        let file = self
            .get_source_file_of_node(own_node)
            .or_else(|| self.current_file.clone());
        let mut diag = tsox_frontend::ast::Diagnostic::new(
            file,
            own_node.loc,
            tsox_core::diagnostics::messages_generated::
                X_0_IS_SPECIFIED_MORE_THAN_ONCE_SO_THIS_USAGE_WILL_BE_OVERWRITTEN,
            vec![name.to_string()],
        );
        let spread_file = self
            .get_source_file_of_node(spread_node)
            .or_else(|| self.current_file.clone());
        diag.related_information
            .push(tsox_frontend::ast::Diagnostic::new(
                spread_file,
                spread_node.loc,
                tsox_core::diagnostics::messages_generated::THIS_SPREAD_ALWAYS_OVERWRITES_THIS_PROPERTY,
                vec![],
            ));
        self.diagnostics.add(diag);
    }
}
