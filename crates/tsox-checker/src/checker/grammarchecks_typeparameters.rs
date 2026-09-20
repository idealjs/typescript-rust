use std::sync::Arc;

use crate::checker::checker::Checker;
use tsox_frontend::ast::{Node, NodeData, NodeList, SyntaxKind};

impl Checker {
    pub(crate) fn check_type_parameters_on_node(&mut self, node: &Arc<Node>) {
        let Some(params) = node_type_parameters(node) else {
            return;
        };
        let params: Vec<Arc<Node>> = params.iter().cloned().collect();
        let mut seen_default = false;
        for (i, param) in params.iter().enumerate() {
            let NodeData::TypeParameterDeclaration(pd) = &param.data else {
                continue;
            };
            match &pd.default_type {
                Some(default_type) => {
                    seen_default = true;
                    self.check_type_parameter_default_references(default_type, &params, i);
                }
                None if seen_default => {
                    let file = self.get_source_file_of_node(param);
                    let already = self.diagnostics.get_all().iter().any(|d| {
                        d.code == 2706 && d.loc == param.loc
                    });
                    if !already {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            param.loc,
                            tsox_core::diagnostics::messages_generated::
                                REQUIRED_TYPE_PARAMETERS_MAY_NOT_FOLLOW_OPTIONAL_TYPE_PARAMETERS,
                            Vec::new(),
                        ));
                    }
                }
                None => {}
            }
        }
    }

    fn check_type_parameter_default_references(
        &mut self,
        default_type: &Arc<Node>,
        params: &[Arc<Node>],
        index: usize,
    ) {
        let mut refs: Vec<Arc<Node>> = Vec::new();
        collect_type_references(default_type, &mut refs);
        for reference in refs {
            let type_ = self.get_type_from_type_reference(&reference);
            if !type_.is_type_parameter() {
                continue;
            }
            let Some(sym) = &type_.symbol else { continue };
            let later = params[index..]
                .iter()
                .any(|p| self.program.symbol_map().symbol_of(p).is_some_and(|s| Arc::ptr_eq(s, sym)));
            if !later {
                continue;
            }
            let file = self.get_source_file_of_node(&reference);
            let already = self.diagnostics.get_all().iter().any(|d| {
                d.code == 2744 && d.loc == reference.loc
            });
            if !already {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    reference.loc,
                    tsox_core::diagnostics::messages_generated::
                        TYPE_PARAMETER_DEFAULTS_CAN_ONLY_REFERENCE_PREVIOUSLY_DECLARED_TYPE_PARAMETERS,
                    Vec::new(),
                ));
            }
        }
    }
}

fn collect_type_references(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
    if node.kind == SyntaxKind::TypeReference {
        out.push(Arc::clone(node));
    }
    tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
        collect_type_references(child, out);
        false
    });
}

pub(crate) fn node_type_parameters(node: &Arc<Node>) -> Option<Arc<NodeList>> {
    match &node.data {
        NodeData::ClassDeclaration(d) => d.type_parameters.clone(),
        NodeData::ClassExpression(d) => d.type_parameters.clone(),
        NodeData::InterfaceDeclaration(d) => d.type_parameters.clone(),
        NodeData::TypeAliasDeclaration(d) => d.type_parameters.clone(),
        NodeData::FunctionDeclaration(d) => d.type_parameters.clone(),
        NodeData::MethodDeclaration(d) => d.type_parameters.clone(),
        NodeData::MethodSignatureDeclaration(d) => d.type_parameters.clone(),
        NodeData::CallSignatureDeclaration(d) => d.type_parameters.clone(),
        NodeData::ConstructSignatureDeclaration(d) => d.type_parameters.clone(),
        NodeData::ConstructorDeclaration(d) => d.type_parameters.clone(),
        NodeData::GetAccessorDeclaration(d) => d.type_parameters.clone(),
        NodeData::SetAccessorDeclaration(d) => d.type_parameters.clone(),
        NodeData::ArrowFunction(d) => d.type_parameters.clone(),
        NodeData::FunctionExpression(d) => d.type_parameters.clone(),
        NodeData::FunctionTypeNode(d) => d.type_parameters.clone(),
        NodeData::ConstructorTypeNode(d) => d.type_parameters.clone(),
        _ => None,
    }
}
