#![allow(unused_imports)]

use crate::checker::checker_impl_chunk_5::*;

impl Checker {
    pub(crate) fn widen_object_literal_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let structured = match t.as_structured() {
            Some(s) => s,
            None => return Arc::clone(t),
        };

        let mut widened_pairs: Vec<(String, Arc<Type>, Arc<Symbol>)> = Vec::new();
        for prop in &structured.properties {
            let prop_type = self.get_type_of_symbol(prop);
            let widened = self.widen_initializer_type(&prop_type);
            widened_pairs.push((prop.name.clone(), widened, Arc::clone(prop)));
        }

        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(widened_pairs.len());
        for (name, t, src_prop) in widened_pairs {
            let symbol = Arc::new(Symbol::new(SymbolFlags::Property, name.clone()));

            {
                let sym_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*sym_mut).flags |= src_prop.flags & SymbolFlags::Optional;
                    (*sym_mut).check_flags |=
                        src_prop.check_flags & tsox_frontend::ast::CheckFlags::Readonly;

                    (*sym_mut).declarations = src_prop.declarations.clone();
                }
            }
            members.insert(name, Arc::clone(&symbol));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous | ObjectFlags::ObjectLiteral,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn build_union_from_types(&self, types: Vec<Arc<Type>>) -> Arc<Type> {
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }

        let mut seen: Vec<Arc<Type>> = Vec::new();
        for t in types {
            if let TypeData::Union(u) = &t.data {
                for inner in &u.union_or_intersection.types {
                    if !seen.iter().any(|s| Arc::ptr_eq(s, inner)) {
                        seen.push(Arc::clone(inner));
                    }
                }
            } else if !seen.iter().any(|s| Arc::ptr_eq(s, &t)) {
                seen.push(t);
            }
        }
        if seen.len() == 1 {
            return seen.into_iter().next().expect("exactly one");
        }

        seen.sort_by_key(|t| {
            if t.flags.intersects(TypeFlags::EnumLiteral | TypeFlags::Enum) {
                return TypeFlags::Enum.bits();
            }
            let b = t.flags.bits();
            b & b.wrapping_neg()
        });
        let enum_symbol = crate::checker::typenode_constructors_checker::uniform_enum_symbol(&seen);
        let mut union = Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: seen,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        );
        union.symbol = enum_symbol;
        Arc::new(union)
    }

    pub fn get_constraint_of_type_parameter(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::TypeParameter(tp) = &t.data {
            return tp.constraint.clone();
        }
        None
    }

    pub fn get_default_from_type_parameter(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::TypeParameter(tp) = &t.data {
            return tp.resolved_default_type.get().cloned();
        }
        None
    }

    pub fn get_resolved_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            if let Some(rt) = ct.resolved_true_type.get() {
                return Some(rt.clone());
            }
            if let Some(rt) = ct.resolved_false_type.get() {
                return Some(rt.clone());
            }
        }
        None
    }

    pub fn get_constraint_of_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(mt) = &t.data {
            return mt.constraint_type.clone();
        }
        if let TypeData::ReverseMapped(rm) = &t.data {
            return rm.constraint_type.clone();
        }
        None
    }

    pub fn get_true_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_true_type.get().cloned();
        }
        None
    }

    pub fn get_false_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_false_type.get().cloned();
        }
        None
    }

    pub fn get_return_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        sig.resolved_return_type.get().cloned()
    }

    pub fn get_type_predicate_of_signature<'a>(
        &self,
        sig: &'a Arc<Signature>,
    ) -> Option<&'a TypePredicate> {
        sig.resolved_type_predicate.as_deref()
    }

    pub fn compute_type_predicate_of_signature(
        &mut self,
        sig: &Arc<Signature>,
    ) -> Option<TypePredicate> {
        if let Some(pred) = sig.resolved_type_predicate.as_deref() {
            if pred.parameter_name == "<<unresolved>>" {
                return None;
            }
            return Some(pred.clone());
        }

        let Some(decl) = sig.declaration.as_ref() else {
            return None;
        };
        // TS5.5 body 谓词推断：唯一 return 表达式是参数方法的 this 谓词调用
        if decl.type_node().is_none() {
            if let Some(pred) = self.infer_type_predicate_from_body(decl, sig) {
                return Some(pred);
            }
        }
        let Some(type_node) = decl.type_node() else {
            return None;
        };
        if type_node.kind != SyntaxKind::TypePredicate {
            return None;
        }
        let NodeData::TypePredicateNode(pred_data) = &type_node.data else {
            return None;
        };
        let t = pred_data
            .type_node
            .as_ref()
            .map(|tn| self.get_type_from_type_node(tn));
        let is_this = pred_data.parameter_name.kind == SyntaxKind::ThisKeyword
            || pred_data.parameter_name.kind == SyntaxKind::ThisType
            || (pred_data.parameter_name.kind == SyntaxKind::Identifier
                && pred_data.parameter_name.text() == "this");
        let kind = if pred_data.asserts_modifier.is_some() {
            if is_this {
                TypePredicateKind::AssertsThis
            } else {
                TypePredicateKind::AssertsIdentifier
            }
        } else {
            if is_this {
                TypePredicateKind::This
            } else {
                TypePredicateKind::Identifier
            }
        };
        let parameter_name = if is_this {
            String::new()
        } else {
            match &pred_data.parameter_name.data {
                NodeData::Identifier(id) => id.text.clone(),
                _ => String::new(),
            }
        };
        let parameter_index = if kind == TypePredicateKind::Identifier
            || kind == TypePredicateKind::AssertsIdentifier
        {
            sig.parameters
                .iter()
                .position(|p| p.name == parameter_name)
                .map(|i| i as i32)
                .unwrap_or(-1)
        } else {
            0
        };
        Some(TypePredicate {
            kind,
            parameter_index,
            parameter_name,
            t,
        })
    }

    pub fn get_base_constraint_of_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        match &t.data {
            TypeData::TypeParameter(tp) => tp.constrained.resolved_base_constraint.get().cloned(),
            TypeData::Conditional(ct) => ct.constrained.resolved_base_constraint.get().cloned(),
            TypeData::IndexedAccess(ia) => ia.constrained.resolved_base_constraint.get().cloned(),
            TypeData::Index(it) => it.constrained.resolved_base_constraint.get().cloned(),
            _ => None,
        }
    }

    pub fn get_type_arguments(&self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        if let TypeData::Object(obj) = &t.data {
            return obj.type_arguments.clone();
        }
        Vec::new()
    }

    pub fn get_unique_symbol_type(&self, _name: &str) -> Option<Arc<Type>> {
        None
    }

    pub fn was_canceled(&self) -> bool {
        false
    }
}

impl Checker {
    /// Go checkIfExpressionRefinesParameter（简化移植）：
    /// `return g.isLeader()` 且 isLeader 的签名是 `this is T` 谓词 → 推断 `g is T`
    fn infer_type_predicate_from_body(
        &mut self,
        decl: &Arc<Node>,
        sig: &Arc<Signature>,
    ) -> Option<TypePredicate> {
        let body = match &decl.data {
            NodeData::FunctionDeclaration(d) => d.body.as_ref(),
            _ => None,
        }?;
        let NodeData::Block(block) = &body.data else {
            return None;
        };
        let rets: Vec<&Arc<Node>> = block
            .statements
            .iter()
            .filter(|s| s.kind == SyntaxKind::ReturnStatement)
            .collect();
        if rets.len() != 1 {
            return None;
        }
        let NodeData::ReturnStatement(ret) = &rets[0].data else {
            return None;
        };
        let call = ret.expression.as_ref()?;
        if call.kind != SyntaxKind::CallExpression {
            return None;
        }
        let NodeData::CallExpression(cd) = &call.data else {
            return None;
        };
        // 返回类型应为 boolean
        let ret_type = self.get_return_type_of_signature(sig)?;
        if !ret_type.flags.contains(TypeFlags::Boolean) {
            return None;
        }
        let callee = match &cd.expression.data {
            NodeData::PropertyAccessExpression(p) => p,
            _ => return None,
        };
        let NodeData::Identifier(recv) = &callee.expression.data else {
            return None;
        };
        // 接收者须是本签名的参数
        let param = sig
            .parameters
            .iter()
            .find(|p| p.name == recv.text)?;
        let callee_type = self.get_type_of_node(&cd.expression);
        if !callee_type.flags.contains(TypeFlags::Object) {
            return None;
        }
        let callee_sig = self
            .get_signatures_of_type(&callee_type, crate::checker::SignatureKind::Call)
            .into_iter()
            .next()?;
        let pred = self.compute_type_predicate_of_signature(&callee_sig)?;
        if !matches!(pred.kind, TypePredicateKind::This) {
            return None;
        }
        let t = pred.t?;
        Some(TypePredicate {
            kind: TypePredicateKind::Identifier,
            parameter_name: param.name.clone(),
            parameter_index: -1,
            t: Some(t),
        })
    }
}
