#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_index_constraint_for_property(
        &mut self,
        _t: &Arc<Type>,
        key_type: &Arc<Type>,
        prop_type: &Arc<Type>,
        name: &Arc<Node>,
        display: &str,
        local_name: Option<Arc<Node>>,
        local_index: Option<Arc<crate::checker::IndexInfo>>,
        interface_decl: Option<Arc<Node>>,
        index_infos: &[Arc<crate::checker::IndexInfo>],
    ) {
        for info in index_infos {
            let Some(info_key) = info.key_type.clone() else {
                continue;
            };
            if !self.is_applicable_index_type(key_type, &info_key) {
                continue;
            }
            let info_value = match info.value_type.clone() {
                Some(v) => v,
                None => continue,
            };
            if self.is_type_assignable_to(prop_type, &info_value) {
                continue;
            }

            let (error_loc, related_index_decl) = if let Some(n) = &local_name {
                (n.loc, None)
            } else if let Some(idx) = &local_index {
                (
                    idx.declaration.as_ref().map(|d| d.loc).unwrap_or(name.loc),
                    idx.declaration.clone(),
                )
            } else if let Some(idecl) = &interface_decl {
                (idecl.loc, None)
            } else {
                continue;
            };
            let file = self.current_file.clone();
            let mut diagnostic = crate::ast::Diagnostic::new(
                file,
                error_loc,
                crate::diagnostics::messages_generated::
                    PROPERTY_0_OF_TYPE_1_IS_NOT_ASSIGNABLE_TO_2_INDEX_TYPE_3,
                vec![
                    display.to_string(),
                    self.type_to_string(prop_type),
                    self.type_to_string(&info_key),
                    self.type_to_string(&info_value),
                ],
            );
            if let Some(idx_decl) = related_index_decl {
                diagnostic.related_information = vec![crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    idx_decl.loc,
                    crate::diagnostics::messages_generated::X_0_IS_DECLARED_HERE,
                    vec![display.to_string()],
                )];
            }
            self.diagnostics.add(diagnostic);
        }
    }

    pub(crate) fn explicit_type_argument_count(node: &Arc<Node>) -> usize {
        match &node.data {
            crate::ast::NodeData::CallExpression(d) => {
                d.type_arguments.as_ref().map(|t| t.len()).unwrap_or(0)
            }
            crate::ast::NodeData::NewExpression(d) => {
                d.type_arguments.as_ref().map(|t| t.len()).unwrap_or(0)
            }
            _ => 0,
        }
    }

    pub(crate) fn has_explicit_type_arguments(node: &Arc<Node>) -> bool {
        Self::explicit_type_argument_count(node) > 0
    }
}
