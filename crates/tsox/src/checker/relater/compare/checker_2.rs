#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn elaborate_array_literal(
        &mut self,
        node: &Arc<crate::ast::Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        if target.flags.intersects(
            TypeFlags::String
                | TypeFlags::Number
                | TypeFlags::Boolean
                | TypeFlags::BigInt
                | TypeFlags::ESSymbol
                | TypeFlags::Void
                | TypeFlags::Undefined
                | TypeFlags::Null
                | TypeFlags::Never
                | TypeFlags::Enum
                | TypeFlags::StringLiteral
                | TypeFlags::NumberLiteral
                | TypeFlags::BooleanLiteral,
        ) {
            return false;
        }
        let elements = match &node.data {
            crate::ast::NodeData::ArrayLiteralExpression(d) => &d.elements,
            _ => return false,
        };
        let _ = source;
        let mut reported = false;
        for (i, element) in elements.iter().enumerate() {
            if element.kind == crate::ast::SyntaxKind::OmittedExpression
                || element.kind == crate::ast::SyntaxKind::SpreadElement
            {
                continue;
            }

            let target_elem = if self.is_array_type(target) {
                self.get_array_element_type(target)
            } else if self.is_tuple_type(target) {
                match self.get_tuple_element_type(target, i) {
                    Some(t) => t,
                    None => continue,
                }
            } else {
                let index_source = match target.symbol.as_ref() {
                    Some(sym)
                        if sym.flags.contains(SymbolFlags::Interface)
                            && target
                                .as_object()
                                .is_some_and(|o| !o.type_arguments.is_empty()) =>
                    {
                        let args = target.as_object().unwrap().type_arguments.clone();
                        Some(self.resolve_interface_type_ex(sym, Some(args)))
                    }
                    _ => None,
                }
                .unwrap_or_else(|| Arc::clone(target));
                let indexed = index_source.as_structured().and_then(|st| {
                    st.index_infos.iter().find_map(|info| {
                        info.key_type
                            .as_ref()
                            .filter(|k| k.flags.contains(TypeFlags::Number))
                            .and_then(|_| info.value_type.clone())
                    })
                });
                match indexed {
                    Some(t) => t,
                    None => continue,
                }
            };
            let source_elem = self.get_type_of_node(element);
            if self.is_type_related_to(&source_elem, &target_elem, relation) {
                continue;
            }
            if self.elaborate_error(
                element,
                &source_elem,
                &target_elem,
                relation,
                out.as_deref_mut(),
            ) {
                reported = true;
                continue;
            }

            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2322 && d.loc == element.loc);
            if !already {
                match out.as_deref_mut() {
                    Some(o) => {
                        self.check_type_related_to_and_optionally_elaborate(
                            &source_elem,
                            &target_elem,
                            relation,
                            Some(element),
                            None,
                            None,
                            Some(o),
                        );
                    }
                    None => {
                        self.check_type_related_to_and_optionally_elaborate(
                            &source_elem,
                            &target_elem,
                            relation,
                            Some(element),
                            None,
                            None,
                            None,
                        );
                    }
                }
            }
            reported = true;
        }
        reported
    }

    pub fn check_type_assignable_to_and_optionally_elaborate(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        error_node: Option<&Arc<crate::ast::Node>>,
        _expr: Option<&Arc<crate::ast::Node>>,
        _head_message: Option<&crate::diagnostics::Message>,
        _diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        self.check_type_related_to_and_optionally_elaborate(
            source,
            target,
            RelationKind::Assignable,
            error_node,
            _expr,
            _head_message,
            _diagnostic_output,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn check_type_related_to_and_elaborate_display(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        error_node: Option<&Arc<crate::ast::Node>>,
        expr: Option<&Arc<crate::ast::Node>>,
        head_message: Option<&crate::diagnostics::Message>,
        diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
        display_target: Option<&Arc<Type>>,
    ) -> bool {
        let saved_display = self.display_target_override.take();
        self.display_target_override = display_target.cloned();
        let r = self.check_type_related_to_and_optionally_elaborate(
            source,
            target,
            relation,
            error_node,
            expr,
            head_message,
            diagnostic_output,
        );
        self.display_target_override = saved_display;
        r
    }
}
