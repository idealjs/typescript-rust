#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_contextual_elements(
        &mut self,
        expr: &Arc<Node>,
        target: &Arc<Type>,
        missing_anchor: TextRange,
    ) {
        if target.flags.contains(TypeFlags::Any) {
            return;
        }
        if expr.kind == SyntaxKind::ArrayLiteralExpression {
            let crate::ast::NodeData::ArrayLiteralExpression(data) = &expr.data else {
                return;
            };

            let elem_t = if self.is_array_type(target)
                || matches!(target.data, TypeData::EvolvingArray(_))
            {
                self.get_array_element_type(target)
            } else {
                self.get_any_type()
            };
            if elem_t.flags.contains(TypeFlags::Any) {
                let index_source = target.symbol.as_ref().and_then(|sym| {
                    let args = target.as_object()?.type_arguments.clone();
                    if sym.flags.contains(SymbolFlags::Interface) && !args.is_empty() {
                        Some(self.resolve_interface_type_ex(sym, Some(args)))
                    } else {
                        None
                    }
                });
                let index_source = index_source.unwrap_or_else(|| Arc::clone(target));
                let indexed = index_source.as_structured().and_then(|s| {
                    s.index_infos
                        .iter()
                        .find(|info| {
                            info.key_type
                                .as_ref()
                                .is_some_and(|k| k.flags.contains(TypeFlags::Number))
                        })
                        .and_then(|info| info.value_type.clone())
                });
                let Some(elem_t) = indexed else {
                    return;
                };
                if elem_t.flags.contains(TypeFlags::Any) {
                    return;
                }
                let mut inner = Vec::new();
                for el in data.elements.iter() {
                    if el.kind == SyntaxKind::SpreadElement {
                        continue;
                    }
                    inner.push(Arc::clone(el));
                }
                for el in inner {
                    let loc = el.loc;
                    self.check_contextual_elements(&el, &elem_t, loc);
                }
                return;
            }
            for el in data.elements.iter() {
                if el.kind == SyntaxKind::SpreadElement {
                    continue;
                }
                self.check_contextual_elements(el, &elem_t, el.loc);
            }
            return;
        }

        if matches!(
            expr.kind,
            SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression
        ) {
            let target = Arc::clone(target);
            let anchor = expr.loc;
            let assertion_type = match &expr.data {
                crate::ast::NodeData::TypeAssertion(d) => {
                    self.get_type_from_type_node(&d.type_node)
                }
                crate::ast::NodeData::AsExpression(d) => self.get_type_from_type_node(&d.type_node),
                _ => return,
            };
            let missing = self.get_missing_required_properties(&assertion_type, &target);
            let file = self.current_file.clone();
            let src_str = self.type_to_string(&assertion_type);
            let tgt_str = self.type_to_string(&target);
            if missing.len() == 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    anchor,
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), src_str, tgt_str],
                ));
            } else if missing.len() > 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    anchor,
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![src_str, tgt_str, missing.join(", ")],
                ));
            }
            return;
        }
        let expr_type = self.get_type_of_node(expr);
        if expr.kind == SyntaxKind::ObjectLiteralExpression {
            if let Some(excess) = self.get_excess_property_name(&expr_type, target) {
                let loc = self
                    .find_object_literal_property_name_node(expr, &excess)
                    .unwrap_or(expr.loc);
                let tgt_str = self.type_to_string(target);
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    self.current_file.clone(),
                    loc,
                    OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                    vec![excess, tgt_str],
                ));
                return;
            }
            let missing = self.get_missing_required_properties(&expr_type, target);
            let file = self.current_file.clone();
            let src_str = self.type_to_string(&expr_type);
            let tgt_str = self.type_to_string(target);
            if missing.len() == 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    missing_anchor,
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                    vec![missing[0].clone(), src_str, tgt_str],
                ));
            } else if missing.len() > 1 {
                self.diagnostics.add(crate::ast::Diagnostic::new(
                    file,
                    missing_anchor,
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![src_str, tgt_str, missing.join(", ")],
                ));
            }
            return;
        }

        if matches!(
            expr.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
        ) && !self.is_type_assignable_to(&expr_type, target)
        {
            let display_type = if crate::checker::is_literal_type(&expr_type) {
                self.get_base_type_of_literal_type(&expr_type)
            } else {
                expr_type.clone()
            };
            let src_str = self.type_to_string(&display_type);
            let tgt_str = self.type_to_string(target);

            let already = self
                .diagnostics
                .get_all()
                .iter()
                .any(|d| d.code == 2322 && d.loc == expr.loc);
            if already {
                return;
            }
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                expr.loc,
                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                vec![src_str, tgt_str],
            ));
        }
    }

    pub(crate) fn unwrap_async_return_type(
        &self,
        declared: Arc<Type>,
        is_async: bool,
    ) -> Arc<Type> {
        if !is_async {
            return declared;
        }

        let is_promise = declared
            .symbol
            .as_ref()
            .is_some_and(|s| s.name == "Promise");
        if is_promise {
            if let crate::checker::TypeData::Object(obj) = &declared.data {
                if let Some(t) = obj.type_arguments.first() {
                    return Arc::clone(t);
                }
            }
            return self.get_any_type();
        }
        declared
    }

    pub fn get_awaited_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        self.get_awaited_type_with_depth(t, 0)
    }

    pub(crate) fn get_awaited_type_with_depth(
        &mut self,
        t: &Arc<Type>,
        depth: usize,
    ) -> Option<Arc<Type>> {
        if depth > 50 {
            return None;
        }

        if t.flags.contains(TypeFlags::Any) {
            return Some(Arc::clone(t));
        }

        if let crate::checker::TypeData::Union(u) = &t.data {
            let mut mapped: Vec<Arc<Type>> =
                Vec::with_capacity(u.union_or_intersection.types.len());
            for constituent in &u.union_or_intersection.types {
                let awaited = self
                    .get_awaited_type_with_depth(constituent, depth + 1)
                    .unwrap_or_else(|| Arc::clone(constituent));
                mapped.push(awaited);
            }
            return Some(self.get_union_type(mapped));
        }
        if let Some(promised) = self.get_promised_type_of_promise(t) {
            if Arc::ptr_eq(&promised, t) {
                return None;
            }
            return self.get_awaited_type_with_depth(&promised, depth + 1);
        }

        Some(Arc::clone(t))
    }

    pub(crate) fn get_promised_type_of_promise(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.symbol.as_ref().is_some_and(|s| s.name == "Promise") {
            if let crate::checker::TypeData::Object(obj) = &t.data {
                if let Some(first) = obj.type_arguments.first() {
                    return Some(Arc::clone(first));
                }
            }
            return None;
        }

        if !t.flags.contains(TypeFlags::Object) {
            return None;
        }
        let then_fn = self.get_property_of_type(t, "then")?;
        let then_type = self.get_type_of_symbol(&then_fn);
        if then_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let then_signatures = self.get_signatures_of_type(&then_type, SignatureKind::Call);
        let then_sig = then_signatures.first()?;
        let onfulfilled = then_sig.parameters.first()?;
        let callback_type = self.get_type_of_symbol(onfulfilled);
        if callback_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let callback_signatures = self.get_signatures_of_type(&callback_type, SignatureKind::Call);
        let callback_sig = callback_signatures.first()?;
        let value_param = callback_sig.parameters.first()?;
        Some(self.get_type_of_symbol(value_param))
    }

    pub(crate) fn declared_annotation_type_of(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        if node.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(node)?;
        let decl = sym.value_declaration.clone()?;
        if decl.kind != SyntaxKind::VariableDeclaration {
            return None;
        }
        let crate::ast::NodeData::VariableDeclaration(vd) = &decl.data else {
            return None;
        };
        let tn = vd.type_node.as_ref()?;
        Some(self.get_type_from_type_node(tn))
    }
}
