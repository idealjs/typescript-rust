#![allow(unused_imports)]

use crate::checker::inference::*;

impl Checker {
    pub fn get_type_of_property_of_contextual_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
        use crate::checker::types::TypeData;

        if t.flags.contains(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_of_type_parameter(t)?;
            return self.get_type_of_property_of_contextual_type(&constraint, name);
        }

        if t.flags.contains(TypeFlags::Union)
            && let TypeData::Union(u) = &t.data
        {
            let found: Vec<Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .filter_map(|c| self.get_type_of_property_of_contextual_type(c, name))
                .collect();
            return match found.len() {
                0 => None,
                1 => Some(found.into_iter().next().unwrap()),
                _ => {
                    let types = found;
                    if types.iter().all(|x| Arc::ptr_eq(x, &types[0])) {
                        Some(Arc::clone(&types[0]))
                    } else {
                        Some(self.get_union_type(types))
                    }
                }
            };
        }

        if matches!(&t.data, TypeData::Mapped(_))
            && let TypeData::Mapped(m) = &t.data
            && m.type_parameter.is_some()
            && m.template_type.is_some()
            && m.name_type.is_none()
        {
            let constraint = m.constraint_type.clone()?;
            let name_literal = self.get_string_literal_type(name);

            let gate_target = self.reduced_keyof_for_contextual_gate(&constraint);
            let gate_ok = match &gate_target {
                Some(g) => self.is_type_assignable_to(&name_literal, g),
                None => {
                    if matches!(&constraint.data, TypeData::Index(idx)
                    if idx.target.as_ref().is_some_and(|tgt| {
                        tgt.flags.contains(TypeFlags::TypeParameter)
                            && self
                                .get_constraint_of_type_parameter(tgt)
                                .is_none()
                    })) {
                        true
                    } else {
                        self.is_type_assignable_to(&name_literal, &constraint)
                    }
                }
            };
            if !gate_ok {
                return None;
            }
            let tp = m.type_parameter.clone().unwrap();
            let template = m.template_type.clone().unwrap();
            let substituted =
                self.substitute_infer_type_parameters(&template, &[tp], &[name_literal]);

            if let TypeData::IndexedAccess(ia) = &substituted.data
                && let (Some(obj), Some(idx)) = (&ia.object_type, &ia.index_type)
            {
                let resolved = self.get_indexed_access_type(obj, idx);
                if !matches!(resolved.intrinsic_name(), Some("any") | Some("error")) {
                    return Some(resolved);
                }
            }
            return Some(substituted);
        }

        if let TypeData::Conditional(c) = &t.data {
            let mut branches: Vec<Arc<Type>> = Vec::new();
            if let Some(root) = &c.root
                && let Some(node) = &root.node
                && let tsox_frontend::ast::NodeData::ConditionalTypeNode(cd) = &node.data
            {
                let true_t = self.get_type_from_type_node(&cd.true_type);
                let false_t = self.get_type_from_type_node(&cd.false_type);
                for branch in [true_t, false_t] {
                    if let Some(found) = self.get_type_of_property_of_contextual_type(&branch, name)
                    {
                        branches.push(found);
                    }
                }
            }
            return match branches.len() {
                0 => None,
                1 => Some(branches.pop().unwrap()),
                _ => {
                    if branches.iter().all(|b| Arc::ptr_eq(b, &branches[0])) {
                        Some(Arc::clone(&branches[0]))
                    } else {
                        Some(self.get_union_type(branches))
                    }
                }
            };
        }

        if t.flags.contains(TypeFlags::Intersection)
            && let TypeData::Intersection(i) = &t.data
        {
            for c in &i.union_or_intersection.types {
                if let Some(found) = self.get_type_of_property_of_contextual_type(c, name) {
                    return Some(found);
                }
            }
            return None;
        }

        if let Some(prop) = self.get_property_of_type(t, name) {
            let prop_type = self.get_type_of_symbol(&prop);
            // 接口实例的成员类型在退化窗口期可能按裸类型参数驻留（成员符号
            // 链接先建先用）；仍含实例类型参数时按 [tp→实参] 按需重代入
            //（Go 实例成员经引用 mapper 实例化，成员类型不携带裸类型参数）
            if t.symbol.as_ref().is_some_and(|s| s.flags.contains(SymbolFlags::Interface))
                && let Some(obj) = t.as_object()
                && !obj.type_arguments.is_empty()
                && crate::checker::type_contains_type_parameter(&prop_type)
            {
                let iface_sym = t.symbol.clone().unwrap();
                let mut tp_nodes: Vec<Arc<tsox_frontend::ast::Node>> = Vec::new();
                for d in &iface_sym.declarations {
                    if let tsox_frontend::ast::NodeData::InterfaceDeclaration(idata) = &d.data
                        && let Some(tps) = &idata.type_parameters
                    {
                        tp_nodes.extend(tps.iter().cloned());
                    }
                }
                let mut tp_syms: Vec<Arc<tsox_frontend::ast::Symbol>> = Vec::new();
                for tp in &tp_nodes {
                    if let Some(sym) = self.program.symbol_map().symbol_of(tp) {
                        tp_syms.push(Arc::clone(sym));
                    }
                }
                let tp_types: Vec<Arc<Type>> = tp_syms
                    .iter()
                    .map(|sym| self.get_type_parameter_from_symbol(sym))
                    .collect();
                if tp_types.len() == obj.type_arguments.len() && !tp_types.is_empty() {
                    // 符号实例漂移容错：params 取 prop_type 内实际引用的 tp 实例
                    //（ptr 必中），按名字对位接口声明序取实参
                    let mut params: Vec<Arc<Type>> = Vec::new();
                    let mut subs: Vec<Arc<Type>> = Vec::new();
                    for (idx, tp_t) in tp_types.iter().enumerate() {
                        let tp_name = tp_t
                            .symbol
                            .as_ref()
                            .map(|s| s.name.clone())
                            .unwrap_or_default();
                        if let (Some(arg), Some(instance)) = (
                            obj.type_arguments.get(idx),
                            Self::find_tp_instance_by_name(&prop_type, &tp_name),
                        ) {
                            params.push(instance);
                            subs.push(Arc::clone(arg));
                        }
                    }
                    if !params.is_empty() {
                        let reinstance =
                            self.substitute_infer_type_parameters(&prop_type, &params, &subs);
                        // mapped 结果自身含其类型参数 K 属正常，只要求解发生替换
                        if !Arc::ptr_eq(&reinstance, &prop_type) {
                            return Some(reinstance);
                        }
                    }
                }
            }
            return Some(prop_type);
        }

        let name_literal = self.get_string_literal_type(name);
        if let Some(info) = self.get_applicable_index_info(t, &name_literal) {
            return info.value_type.clone();
        }
        None
    }

    pub(crate) fn reduced_keyof_for_contextual_gate(
        &mut self,
        constraint: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        use crate::checker::types::TypeData;
        let TypeData::Index(idx) = &constraint.data else {
            return None;
        };
        let target = idx.target.as_ref()?;
        if !target.flags.contains(TypeFlags::TypeParameter) {
            return None;
        }
        let target_constraint = self.get_constraint_of_type_parameter(target)?;
        Some(self.get_index_type(&target_constraint))
    }

    pub fn get_contextual_type(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
        _context_flags: ContextFlags,
    ) -> Option<Arc<Type>> {
        let parent = match node.parent() {
            Some(p) => Arc::clone(&p),
            None => return None,
        };

        match parent.kind {
            SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::BindingElement => {
                self.get_contextual_type_for_initializer_expression(node, _context_flags)
            }
            SyntaxKind::ArrowFunction | SyntaxKind::ReturnStatement => {
                self.get_contextual_type_for_return_expression(node, _context_flags)
            }
            SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
                self.get_contextual_type_for_argument_ex(&parent, node, _context_flags)
            }

            SyntaxKind::TypeAssertionExpression => {
                if let tsox_frontend::ast::NodeData::TypeAssertion(d) = &parent.data {
                    Some(self.get_type_from_type_node(&d.type_node))
                } else {
                    None
                }
            }
            SyntaxKind::AsExpression => {
                if let tsox_frontend::ast::NodeData::AsExpression(d) = &parent.data {
                    Some(self.get_type_from_type_node(&d.type_node))
                } else {
                    None
                }
            }
            SyntaxKind::SatisfiesExpression => {
                if let tsox_frontend::ast::NodeData::SatisfiesExpression(d) = &parent.data {
                    Some(self.get_type_from_type_node(&d.type_node))
                } else {
                    None
                }
            }
            SyntaxKind::BinaryExpression => {
                self.get_contextual_type_for_binary_operand(node, _context_flags)
            }
            SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
                self.get_contextual_type_for_object_literal_element(&parent, _context_flags)
            }
            SyntaxKind::JsxExpression => self.get_contextual_type_for_jsx_expression(node, _context_flags),
            SyntaxKind::ArrayLiteralExpression => {
                self.get_contextual_type_for_array_literal_element(node, &parent, _context_flags)
            }

            SyntaxKind::ParenthesizedExpression | SyntaxKind::NonNullExpression => {
                self.get_contextual_type(&parent, _context_flags)
            }
            _ => None,
        }
    }

    pub fn get_contextual_signature(
        &mut self,
        node: &Arc<tsox_frontend::ast::Node>,
    ) -> Option<Arc<Signature>> {
        let t = self.get_contextual_type(node, ContextFlags::Signature)?;
        if let TypeData::Union(u) = &t.data {
            let mut first: Option<Arc<Signature>> = None;
            for current in &u.union_or_intersection.types {
                let Some(signature) = self.get_contextual_call_signature(current, node) else {
                    continue;
                };
                match &first {
                    None => first = Some(signature),
                    Some(f) => {
                        if f.parameters.len() != signature.parameters.len() {
                            return None;
                        }
                    }
                }
            }
            return first;
        }
        self.get_contextual_call_signature(&t, node)
    }

    pub(crate) fn get_contextual_call_signature(
        &mut self,
        t: &Arc<Type>,
        node: &Arc<tsox_frontend::ast::Node>,
    ) -> Option<Arc<Signature>> {
        let signatures = self.get_signatures_of_type(t, SignatureKind::Call);
        signatures
            .into_iter()
            .find(|s| !self.is_arity_smaller(s, node))
    }

    pub(crate) fn is_arity_smaller(
        &self,
        signature: &Arc<Signature>,
        target: &Arc<tsox_frontend::ast::Node>,
    ) -> bool {
        let Some(parameters) = function_like_parameters(target) else {
            return false;
        };
        let mut target_parameter_count: i32 = 0;
        for param in parameters.iter() {
            let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.initializer.is_some()
                || pd.question_token.is_some()
                || pd.dot_dot_dot_token.is_some()
            {
                break;
            }
            target_parameter_count += 1;
        }
        if let Some(first) = parameters.iter().next() {
            if is_this_parameter_node(first) {
                target_parameter_count -= 1;
            }
        }
        let has_effective_rest = signature.flags.contains(SignatureFlags::HasRestParameter);
        let parameter_count =
            signature.parameters.len() as i32 - if has_effective_rest { 1 } else { 0 };
        !has_effective_rest && parameter_count < target_parameter_count
    }

    /// 在类型中找指定名字的类型参数实例（多副本符号按名容错）
    pub(crate) fn find_tp_instance_by_name(t: &Arc<Type>, name: &str) -> Option<Arc<Type>> {
        if let TypeData::TypeParameter(_) = &t.data
            && t.symbol.as_ref().is_some_and(|s| s.name == name)
        {
            return Some(Arc::clone(t));
        }
        match &t.data {
            TypeData::Union(_) | TypeData::Intersection(_) => t.types().and_then(|ms| {
                ms.iter().find_map(|m| Self::find_tp_instance_by_name(m, name))
            }),
            TypeData::Tuple(tup) => tup
                .element_infos
                .iter()
                .filter_map(|e| e.type_.as_ref())
                .find_map(|e| Self::find_tp_instance_by_name(e, name)),
            TypeData::IndexedAccess(ia) => ia
                .object_type
                .as_ref()
                .and_then(|o| Self::find_tp_instance_by_name(o, name))
                .or_else(|| {
                    ia.index_type
                        .as_ref()
                        .and_then(|i| Self::find_tp_instance_by_name(i, name))
                }),
            TypeData::Conditional(c) => c
                .check_type
                .as_ref()
                .and_then(|c| Self::find_tp_instance_by_name(c, name))
                .or_else(|| {
                    c.extends_type
                        .as_ref()
                        .and_then(|e| Self::find_tp_instance_by_name(e, name))
                }),
            TypeData::Mapped(m) => m
                .constraint_type
                .as_ref()
                .and_then(|c| Self::find_tp_instance_by_name(c, name))
                .or_else(|| {
                    m.template_type
                        .as_ref()
                        .and_then(|t| Self::find_tp_instance_by_name(t, name))
                }),
            TypeData::Object(o) => o
                .type_arguments
                .iter()
                .find_map(|a| Self::find_tp_instance_by_name(a, name)),
            TypeData::Substitution(sub) => sub
                .base_type
                .as_ref()
                .and_then(|b| Self::find_tp_instance_by_name(b, name)),
            _ => None,
        }
    }
}
