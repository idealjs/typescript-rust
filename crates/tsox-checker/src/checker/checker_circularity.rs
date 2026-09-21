use std::sync::Arc;

use tsox_frontend::ast::{Node, Symbol};

use crate::checker::checker::*;

impl Checker {
    pub(crate) fn circular_constraint_type(&self) -> Arc<crate::checker::types::Type> {
        self.circular_constraint_type
            .get_or_init(|| {
                Arc::new(crate::checker::types::Type {
                    flags: crate::checker::types::TypeFlags::Object,
                    object_flags: crate::checker::types::ObjectFlags::Anonymous,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: None,
                    data: crate::checker::types::TypeData::Object(Default::default()),
                })
            })
            .clone()
    }

    fn cycle_crosses_rt_infer_boundary(&self, symbol: &Arc<Symbol>) -> bool {
        let target = Arc::as_ptr(symbol) as *const Symbol;
        let Some(idx) = self
            .type_resolution_stack
            .iter()
            .rposition(|e| e.target == target && e.property == TypeResolutionProperty::Type)
        else {
            return false;
        };
        self.rt_infer_boundary_marks.iter().any(|&m| m > idx)
    }

    pub(crate) fn report_circularity_error(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        if self.cycle_crosses_rt_infer_boundary(symbol) {
            return self.get_any_type();
        }
        let Some(decl) = symbol.value_declaration.clone() else {
            return self.get_any_type();
        };
        let (type_node, has_initializer, is_parameter, name_loc) = match &decl.data {
            tsox_frontend::ast::NodeData::VariableDeclaration(d) => {
                (d.type_node.clone(), d.initializer.is_some(), false, d.name.loc)
            }
            tsox_frontend::ast::NodeData::PropertyDeclaration(d) => (
                d.type_node.clone(),
                d.initializer.is_some(),
                false,
                d.name.loc,
            ),
            tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => (
                Some(Arc::clone(&d.type_node)),
                false,
                false,
                d.name.loc,
            ),
            tsox_frontend::ast::NodeData::ParameterDeclaration(d) => (
                d.type_node.clone(),
                d.initializer.is_some(),
                true,
                d.name.loc,
            ),
            tsox_frontend::ast::NodeData::BindingElement(d) => (
                None,
                d.initializer.is_some(),
                false,
                d.name.as_ref().map_or(decl.loc, |n| n.loc),
            ),
            _ => (None, false, false, decl.loc),
        };
        if type_node.is_some() {
            let file = self.get_source_file_of_node(&decl);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                name_loc,
                tsox_core::diagnostics::messages_generated::
                    X_0_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_TYPE_ANNOTATION,
                vec![symbol.name.clone()],
            ));
            return self.error_type();
        }
        if self.no_implicit_any && (!is_parameter || has_initializer) {
            let file = self.get_source_file_of_node(&decl);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                name_loc,
                tsox_core::diagnostics::messages_generated::
                    X_0_IMPLICITLY_HAS_TYPE_ANY_BECAUSE_IT_DOES_NOT_HAVE_A_TYPE_ANNOTATION_AND_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_INITIALIZER,
                vec![symbol.name.clone()],
            ));
        }
        self.get_any_type()
    }
}
