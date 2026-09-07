#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_accessor_signature_rules(
        &mut self,
        node: &Arc<Node>,
        body: &Option<Arc<Node>>,
        ambient: bool,
    ) {
        let name_loc = Self::class_member_name_node(node)
            .map(|n| n.loc)
            .unwrap_or(node.loc);
        fn first_param_is_this(params: &Arc<NodeList>) -> bool {
            params.iter().next().is_some_and(|p| {
                matches!(
                        &p.data,
                        crate::ast::NodeData::ParameterDeclaration(pd)
                    if pd.name.kind == SyntaxKind::Identifier
        && pd.name.text() == "this")
            })
        }
        let (has_type_params, params, set_has_return) = match &node.data {
            crate::ast::NodeData::GetAccessorDeclaration(d) => {
                (d.type_parameters.is_some(), Some(&d.parameters), false)
            }
            crate::ast::NodeData::SetAccessorDeclaration(d) => (
                d.type_parameters.is_some(),
                Some(&d.parameters),
                d.type_node.is_some(),
            ),
            _ => (false, None, false),
        };
        let param_count = params.map_or(0, |p| p.iter().count());
        let first_is_this = params.is_some_and(first_param_is_this);
        let expected = if node.kind == SyntaxKind::GetAccessor {
            0
        } else {
            1
        };
        let count_correct =
            param_count == expected || (first_is_this && param_count == expected + 1);
        let file = self.current_file.clone();
        if has_type_params {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_loc,
                crate::diagnostics::messages_generated::AN_ACCESSOR_CANNOT_HAVE_TYPE_PARAMETERS,
                vec![],
            ));
        } else if !count_correct {
            let message = if node.kind == SyntaxKind::GetAccessor {
                crate::diagnostics::messages_generated::A_GET_ACCESSOR_CANNOT_HAVE_PARAMETERS
            } else {
                crate::diagnostics::messages_generated::
                        A_SET_ACCESSOR_MUST_HAVE_EXACTLY_ONE_PARAMETER
            };
            self.diagnostics
                .add(crate::ast::Diagnostic::new(file, name_loc, message, vec![]));
        } else if node.kind == SyntaxKind::SetAccessor && set_has_return {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    name_loc,
                    crate::diagnostics::messages_generated::
                        A_SET_ACCESSOR_CANNOT_HAVE_A_RETURN_TYPE_ANNOTATION,
                    vec![],
                ));
        }

        if node.kind == SyntaxKind::GetAccessor
            && !ambient
            && let Some(body_node) = &body
            && !self.function_body_definitely_returns(body_node)
            && !Self::function_body_has_explicit_return(body_node)
        {
            let file = self.current_file.clone();
            self.diagnostics.add(crate::ast::Diagnostic::new(
                file,
                name_loc,
                crate::diagnostics::messages_generated::A_GET_ACCESSOR_MUST_RETURN_A_VALUE,
                vec![],
            ));
        }
    }
}
