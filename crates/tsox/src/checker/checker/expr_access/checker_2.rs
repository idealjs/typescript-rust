#![allow(unused_imports)]

use super::*;

impl Checker {
    #[allow(dead_code)]
    pub(crate) fn property_type_includes_undefined(
        &mut self,
        data: &crate::ast::node_data_generated::PropertyDeclarationData,
    ) -> bool {
        let Some(tn) = &data.type_node else {
            return false;
        };
        let t = self.get_type_from_type_node(tn);
        if t.flags.contains(TypeFlags::Undefined) {
            return true;
        }
        if let Some(u) = t.as_union_or_intersection() {
            return u
                .types
                .iter()
                .any(|m| m.flags.contains(TypeFlags::Undefined));
        }
        false
    }

    #[allow(dead_code)]
    pub(crate) fn class_constructor_assigns_property(&self, name: &str) -> bool {
        let Some(class) = self.enclosing_class_stack.last() else {
            return false;
        };
        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return false;
        };
        cd.members.iter().any(|member| {
            if member.kind != SyntaxKind::Constructor {
                return false;
            }
            let crate::ast::NodeData::ConstructorDeclaration(ctor) = &member.data else {
                return false;
            };
            ctor.body
                .as_ref()
                .is_some_and(|body| body_assigns_this_property(body, name))
        })
    }

    pub(crate) fn check_call_arg_with_context(
        &mut self,
        callee_expr: &Arc<Node>,
        arg_index: usize,
        arg: &Arc<Node>,
    ) {
        let is_function_arg = matches!(
            arg.kind,
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression
        );
        if is_function_arg {
            let ctx = self.contextual_param_count_for_arg(callee_expr, arg_index);
            if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
                eprintln!("[ctx-arg] pushed ctx={ctx}");
            }
            self.call_arg_arrow_context.push(ctx);
        }
        self.check_expression(arg);
        if is_function_arg {
            self.call_arg_arrow_context.pop();
        }
    }

    pub(crate) fn contextual_signature_of_arrow(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Signature>> {
        if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
            eprintln!(
                "[arrow-ctx] entered parent={:?}",
                node.parent.as_ref().map(|p| p.kind)
            );
        }
        let t = self.get_contextual_type(node, ContextFlags::None)?;
        if let TypeData::IndexedAccess(ia) = &t.data
            && let (Some(o), Some(i)) = (&ia.object_type, &ia.index_type)
            && o.flags.contains(TypeFlags::TypeParameter)
        {
            let resolved = self.get_indexed_access_type(o, i);
            if !matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
                return self.first_call_signature(&resolved);
            }
        }
        self.first_call_signature(&t)
    }

    pub(crate) fn first_call_signature(&mut self, t: &Arc<Type>) -> Option<Arc<Signature>> {
        if let TypeData::Union(u) = &t.data {
            for constituent in &u.union_or_intersection.types {
                if constituent
                    .flags
                    .intersects(TypeFlags::Undefined | TypeFlags::Null)
                {
                    continue;
                }
                if let Some(sig) = self.first_call_signature(constituent) {
                    return Some(sig);
                }
            }
            return None;
        }
        let structured = t.as_structured()?;
        structured.call_signatures().first().cloned()
    }

    pub(crate) fn contextual_param_count_for_arg(
        &mut self,
        callee_expr: &Arc<Node>,
        arg_index: usize,
    ) -> usize {
        let t = self.get_type_of_node(callee_expr);
        if std::env::var_os("TSOX_DEBUG_SYMBOL").is_some() {
            eprintln!(
                "[ctx-arg] callee={:?} intr={:?} union={} structured={}",
                callee_expr.kind,
                t.intrinsic_name(),
                matches!(&t.data, TypeData::Union(_)),
                t.as_structured()
                    .map(|s| s.call_signatures().len())
                    .unwrap_or(usize::MAX),
            );
        }
        if t.flags.contains(TypeFlags::Any) {
            if let crate::ast::NodeData::PropertyAccessExpression(data) = &callee_expr.data {
                let method = data.name.text().to_string();
                const ARRAY_CALLBACK_SIGS: &[(&str, usize)] = &[
                    ("map", 3),
                    ("filter", 3),
                    ("forEach", 3),
                    ("every", 3),
                    ("some", 3),
                    ("find", 3),
                    ("findIndex", 3),
                    ("findLast", 3),
                    ("findLastIndex", 3),
                    ("flatMap", 3),
                    ("reduce", 4),
                    ("reduceRight", 4),
                    ("sort", 2),
                ];
                if let Some((_, count)) = ARRAY_CALLBACK_SIGS.iter().find(|(m, _)| *m == method) {
                    let recv_type = self.get_type_of_node(&data.expression);
                    if self.is_array_type(&recv_type) {
                        return *count;
                    }
                }
            }
            return 0;
        }

        let t = if let TypeData::Union(u) = &t.data {
            match u.union_or_intersection.types.iter().find(|c| {
                !c.flags.intersects(TypeFlags::Undefined | TypeFlags::Null)
                    && c.as_structured()
                        .is_some_and(|s| !s.call_signatures().is_empty())
            }) {
                Some(c) => Arc::clone(c),
                None => return 0,
            }
        } else {
            t
        };
        let Some(structured) = t.as_structured() else {
            return 0;
        };

        let Some(sig) = structured
            .call_signatures()
            .iter()
            .find(|s| s.parameters.len() > arg_index)
            .or_else(|| structured.call_signatures().first())
        else {
            return 0;
        };
        let Some(param) = sig.parameters.get(arg_index) else {
            return 0;
        };
        let param_type = self.get_type_of_symbol(param);
        if param_type.flags.contains(TypeFlags::Any) {
            return 0;
        }
        let Some(param_structured) = param_type.as_structured() else {
            return 0;
        };
        param_structured
            .call_signatures()
            .first()
            .map_or(0, |callback_sig| callback_sig.parameters.len())
    }

    pub(crate) fn symbol_is_abstract_class(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if decl.kind == SyntaxKind::ClassDeclaration
                && decl.has_syntactic_modifier(ModifierFlags::Abstract)
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn type_includes_abstract_constructor(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Any) {
            return false;
        }
        if let Some(u) = t.as_union_or_intersection() {
            return u
                .types
                .iter()
                .any(|m| self.type_includes_abstract_constructor(m));
        }

        if t.flags.contains(TypeFlags::Object) {
            if let Some(s) = t.as_structured()
                && s.construct_signatures().iter().any(|sig| {
                    sig.flags
                        .contains(crate::checker::types::SignatureFlags::Abstract)
                })
            {
                return true;
            }
        }
        if let Some(symbol) = &t.symbol {
            return self.symbol_is_abstract_class(symbol);
        }
        false
    }

    pub(crate) fn declaring_class_of_member(
        &self,
        member_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        self.declaring_class_of_private_member(member_symbol)
            .or_else(|| {
                for decl in &member_symbol.declarations {
                    if matches!(
                        decl.kind,
                        SyntaxKind::PropertyDeclaration | SyntaxKind::MethodDeclaration
                    ) {
                        if let Some(parent) = &decl.parent {
                            if parent.kind == SyntaxKind::ClassDeclaration {
                                return Some(Arc::clone(parent));
                            }
                        }
                    }
                }
                None
            })
    }

    pub(crate) fn declaring_class_of_private_member(
        &self,
        member_symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        for decl in &member_symbol.declarations {
            if matches!(
                decl.kind,
                SyntaxKind::PropertyDeclaration
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
            ) {
                if let Some(parent) = &decl.parent {
                    if matches!(
                        parent.kind,
                        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                    ) {
                        return Some(Arc::clone(parent));
                    }
                }
            }
        }
        None
    }
}
