#![allow(unused_imports)]

use crate::checker::checker_impl_chunk_5::*;
use crate::checker::mig::wc3_3::is_generic_tuple_type;

impl Checker {
    pub(crate) fn widen_object_literal_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("widen_object_literal_type"); 
        let structured = match t.as_structured() {
            Some(s) => s,
            None => return Arc::clone(t),
        };

        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(structured.properties.len());
        for prop in &structured.properties {
            // Go getWidenedProperty（checker.go:19519-19525）：非 Property 位
            //（Accessor 型）属性原样返回，getter 返回值已在体推断通道加宽
            if !prop.flags.contains(SymbolFlags::Property) {
                members.insert(prop.name.clone(), Arc::clone(prop));
                props.push(Arc::clone(prop));
                continue;
            }
            // Go getWidenedProperty（checker.go:19519-19531）：成员加宽走
            // getWidenedTypeWithContext——REQUIRES_WIDENING 门控，字面量型
            // object flags 恒空（newLiteralType ObjectFlagsNone，
            // checker.go:26530-26533），fresh/regular 字面量成员在此结构性
            // 不加宽；保真由 check 期 checkExpressionForMutableLocation 的
            // as-assertion/const 语境豁免决定（as const 属性成员呈字面量型）
            let original = self.get_type_of_symbol(prop);
            let widened = self.get_widened_type_with_context(&original, None);
            let symbol = if Arc::ptr_eq(&widened, &original) {
                // Go getWidenedProperty：widened == original 时原符号返回
                Arc::clone(prop)
            } else {
                let symbol = self.create_symbol_with_type(prop, &widened);
                // Go createSymbolWithType（checker.go:22977-22988）：widen 复制符号
                // 随型传播 nameType 与 target（target 指回源符号），late-bound 计算
                // 名属性（[E1.A] 等）在 widened 对象型上保持 nameType 分支显示
                let (name_type, target) = {
                    let src_links = self.value_symbol_links.get(prop);
                    (
                        src_links.and_then(|l| l.name_type.clone()),
                        src_links.and_then(|l| l.target.clone()),
                    )
                };
                if name_type.is_some() || target.is_some() {
                    let links = self.value_symbol_links.get_or_default(&symbol);
                    links.name_type = name_type;
                    links.target = target;
                }
                symbol
            };
            members.insert(prop.name.clone(), Arc::clone(&symbol));
            props.push(symbol);
        }
        // Go getWidenedTypeOfObjectLiteral：newAnonymousType(t.symbol, ...) 保留原符号，
        // objectFlags 仅 Anonymous + 原 JSLiteral/NonInferrableType（checker.go:19508-19509）
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous
                | (t.object_flags & (ObjectFlags::JSLiteral | ObjectFlags::NonInferrableType)),
            id: crate::checker::types::next_type_id(),
            symbol: t.symbol.clone(),
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn build_union_from_types(&self, types: Vec<Arc<Type>>) -> Arc<Type> { ::tsox_core::fntrace::enter("build_union_from_types"); 
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
                    // Go addTypesToUnion→insertType 以 CompareTypes 结构化判等
                    //（utilities.go:464 排序键；符号跨 checker 实例共享，结构同
                    // 即同型）——Arc::ptr_eq 在多 checker 实例（check/emit 分离）
                    // 下对同一结构型判重失效
                    if !seen.iter().any(|s| {
                        crate::checker::utilities_has_only_expression_initialization::compare_types(s, inner)
                            == std::cmp::Ordering::Equal
                    }) {
                        seen.push(Arc::clone(inner));
                    }
                }
            } else if !seen.iter().any(|s| {
                crate::checker::utilities_has_only_expression_initialization::compare_types(s, &t)
                    == std::cmp::Ordering::Equal
            }) {
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
        // Go getUnionTypeFromSortedListing（checker.go:27277-27279）：双成员均为
        // boolean 字面量的联合置 TypeFlagsBoolean（如 a && b 对 boolean 操作数
        // 归并出的 false|true），显示走 boolean 关键字（判定须在 seen 被
        // TypeData move 前完成）
        let is_boolean_union = seen.len() == 2
            && seen[0].flags.contains(TypeFlags::BooleanLiteral)
            && seen[1].flags.contains(TypeFlags::BooleanLiteral);
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
        if is_boolean_union {
            union.flags.insert(TypeFlags::Boolean);
        }
        union.symbol = enum_symbol;
        Arc::new(union)
    }

    pub fn get_constraint_of_type_parameter(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_constraint_of_type_parameter"); 
        if let TypeData::TypeParameter(tp) = &t.data {
            return tp.constraint.clone();
        }
        None
    }

    pub fn get_default_from_type_parameter(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_default_from_type_parameter");
        if let TypeData::TypeParameter(_) = &t.data {
            let default_type = self.get_resolved_type_parameter_default(t);
            if default_type.id != self.no_constraint_type().id
                && default_type.id != self.circular_constraint_type().id
            {
                return Some(default_type);
            }
        }
        None
    }

    pub fn get_resolved_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_resolved_type_of_conditional_type"); 
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

    pub fn get_constraint_of_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_constraint_of_mapped_type"); 
        if let TypeData::Mapped(mt) = &t.data {
            return mt.constraint_type.clone();
        }
        if let TypeData::ReverseMapped(rm) = &t.data {
            return rm.constraint_type.clone();
        }
        None
    }

    pub fn get_true_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_true_type_of_conditional_type"); 
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_true_type.get().cloned();
        }
        None
    }

    pub fn get_false_type_of_conditional_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_false_type_of_conditional_type"); 
        if let TypeData::Conditional(ct) = &t.data {
            return ct.resolved_false_type.get().cloned();
        }
        None
    }

    pub fn get_return_type_of_signature(&self, sig: &Arc<Signature>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_return_type_of_signature");
        sig.resolved_return_type.get().cloned()
    }

    pub fn get_return_type_of_signature_lazy(&mut self, sig: &Arc<Signature>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_return_type_of_signature_lazy");
        // Go getReturnTypeOfSignature（checker.go:21190）：缓存 miss 时按
        // 注解 → 体推断 → any 懒求值并回填；重入（返回类型环）落 errorType
        if let Some(rt) = sig.resolved_return_type.get() {
            return rt.clone();
        }
        if !self.push_type_resolution(
            Arc::as_ptr(sig) as *const tsox_frontend::ast::Symbol,
            crate::checker::TypeResolutionProperty::ResolvedReturnType,
        ) {
            return self.error_type();
        }
        let t = match sig.target.clone() {
            Some(target) => {
                let rt = self.get_return_type_of_signature_lazy(&target);
                self.instantiate_type(&rt, sig.mapper.as_ref())
            }
            None => {
                let annotation = sig.declaration.as_ref().and_then(|d| d.type_node());
                match annotation {
                    Some(tn) => self.get_type_from_type_node(tn),
                    None => match sig.declaration.as_ref().and_then(|d| d.body()) {
                        Some(body) => self.infer_function_return_type(None, Some(body), None),
                        None => self.any_type(),
                    },
                }
            }
        };
        // Go getReturnTypeOfSignature（checker.go:21214-21218）：call chain 签名
        // 返回型挂 optional 标记/undefined 并集
        let t = if sig.flags.contains(SignatureFlags::IsInnerCallChain) {
            self.add_optional_type_marker(&t)
        } else if sig.flags.contains(SignatureFlags::IsOuterCallChain) {
            self.get_optional_type(t, false)
        } else {
            t
        };
        let resolved = if !self.pop_type_resolution() {
            self.any_type()
        } else {
            t
        };
        let _ = sig.resolved_return_type.set(resolved.clone());
        resolved
    }

    pub fn get_type_predicate_of_signature<'a>(
        &mut self,
        sig: &'a Arc<Signature>,
    ) -> Option<&'a TypePredicate> { ::tsox_core::fntrace::enter("get_type_predicate_of_signature");
        if sig.resolved_type_predicate.get().is_none() {
            let computed: Option<Box<TypePredicate>> = if let Some(target) = sig.target.clone() {
                let target_predicate = self.get_type_predicate_of_signature(&target).cloned();
                match target_predicate {
                    Some(target_predicate) => match sig.mapper.clone() {
                        Some(mapper) => self.instantiate_type_predicate(&target_predicate, &mapper),
                        None => Some(Box::new(target_predicate)),
                    },
                    None => None,
                }
            } else if let Some(decl) = sig.declaration.clone() {
                match decl.type_node() {
                    Some(type_node) => {
                        if type_node.kind == SyntaxKind::TypePredicate {
                            self.create_type_predicate_from_type_predicate_node(&type_node, sig)
                        } else {
                            None
                        }
                    }
                    None => {
                        let return_type_boolean_or_unresolved = match sig.resolved_return_type.get() {
                            Some(rt) => rt.flags.contains(TypeFlags::Boolean),
                            None => true,
                        };
                        if tsox_frontend::ast::is_function_like_declaration(&decl)
                            && return_type_boolean_or_unresolved
                            && self.get_parameter_count(sig) > 0
                        {
                            self.infer_type_predicate_from_body(&decl, sig).map(Box::new)
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            };
            let _ = sig.resolved_type_predicate.set(computed);
        }
        sig.resolved_type_predicate.get().and_then(|o| o.as_deref())
    }

    pub fn compute_type_predicate_of_signature(
        &mut self,
        sig: &Arc<Signature>,
    ) -> Option<TypePredicate> { ::tsox_core::fntrace::enter("compute_type_predicate_of_signature"); 
        if let Some(pred) = sig.resolved_type_predicate.get().and_then(|o| o.as_deref()) {
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

    pub fn get_base_constraint_of_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_base_constraint_of_type");
        if t.flags.intersects(
            crate::checker::types_type_flags_instantiable_non_primitive::TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE
                | crate::checker::types_type_id::TYPE_FLAGS_UNION_OR_INTERSECTION
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping
                | TypeFlags::Index,
        ) || is_generic_tuple_type(t)
        {
            // Go AsConstrainedType（types.go:904 TypeBase 默认 nil）仅型参/条件/
            // 索引访问/索引/替换/模板/串映射内嵌 ConstrainedType；并/交/元组
            // 不被接受 → getResolvedBaseConstraint 原样返回 t 本身
            //（checker.go:29078-29080）
            if t.flags.intersects(crate::checker::types_type_id::TYPE_FLAGS_UNION_OR_INTERSECTION)
                || is_generic_tuple_type(t)
            {
                return Some(Arc::clone(t));
            }
            let constraint = self.get_resolved_base_constraint(t, &[]);
            let no_constraint = self.no_constraint_type();
            let circular = self.circular_constraint_type();
            if !Arc::ptr_eq(&constraint, &no_constraint) && !Arc::ptr_eq(&constraint, &circular) {
                return Some(constraint);
            }
            return None;
        }
        None
    }

    pub fn get_type_arguments(&self, t: &Arc<Type>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_arguments"); 
        if let TypeData::Object(obj) = &t.data {
            return obj.type_arguments.clone();
        }
        Vec::new()
    }

    pub fn get_unique_symbol_type(&self, _name: &str) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_unique_symbol_type"); 
        None
    }

    pub fn was_canceled(&self) -> bool { ::tsox_core::fntrace::enter("was_canceled"); 
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
    ) -> Option<TypePredicate> { ::tsox_core::fntrace::enter("infer_type_predicate_from_body"); 
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
