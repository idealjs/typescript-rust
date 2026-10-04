#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

impl Checker {
    pub fn get_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_symbol"); 
        // CommonJS require 符号类型为 any（Go getTypeOfVariableOrParameterOrPropertyWorker）
        if self
            .require_symbol
            .as_ref()
            .is_some_and(|s| Arc::ptr_eq(s, symbol))
        {
            return self.get_any_type();
        }
        // 声明型解析进行中不可重入取同符号类型（Go 该状态不可达：接口成员
        // 重建等求值路径不会出现在 getDeclaredType 之下）；静默返回占位，
        // 不开 Type 帧也不报环
        if self.is_resolving(
            Arc::as_ptr(symbol) as *const Symbol,
            crate::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return self.error_type();
        }
        // Go getTypeOfInstantiatedSymbol：实例化符号类型 = 目标符号类型经
        // mapper 实例化（惰性求值并驻留）
        if symbol
            .check_flags
            .intersects(tsox_frontend::ast::CheckFlags::Instantiated)
        {
            if let Some(t) = self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
            {
                return t;
            }
            let (target, mapper) = match self.value_symbol_links.get(symbol) {
                Some(l) => (l.target.clone(), l.mapper.clone()),
                None => (None, None),
            };
            if let Some(target) = target {
                let t = self.get_type_of_symbol(&target);
                let t = self.instantiate_type(&t, mapper.as_ref());
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                return t;
            }
        }
        // js export=（`module.exports = <表达式>`，BinaryExpression 声明）：
        // 符号类型 = 右侧表达式类型（具名导入经 tryGetMemberInModuleExports
        // AndProperties 取该类型的属性）
        if let Some(right) = symbol.declarations.iter().find_map(|d| match &d.data {
            tsox_frontend::ast::NodeData::BinaryExpression(be)
                if matches!(&be.left.data, tsox_frontend::ast::NodeData::PropertyAccessExpression(pa)
                    if pa.expression.text() == "module" && pa.name.text() == "exports") =>
            {
                Some(Arc::clone(&be.right))
            }
            _ => None,
        }) {
            if let Some(existing) = self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
            {
                return existing;
            }
            let t = self.get_type_of_node(&right);
            self.value_symbol_links
                .get_or_default(symbol)
                .resolved_type = Some(Arc::clone(&t));
            return t;
        }
        // Go getTypeOfReverseMappedSymbol：反向映射符号类型经
        // inferReverseMappedType(propertyType, mappedType, constraintType) 惰性求值
        if let Some(links) = self.reverse_mapped_symbol_links.get(symbol)
            && links.property_type.is_some()
        {
            if let Some(t) = self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
            {
                return t;
            }
            let (Some(prop_t), Some(mapped), Some(constraint)) = (
                links.property_type.clone(),
                links.mapped_type.clone(),
                links.constraint_type.clone(),
            ) else {
                return self.get_unknown_type();
            };
            let inferred = self.infer_reverse_mapped_type(&prop_t, &mapped, &constraint);
            if self.template_resolution_letway {
                // 模板让位（外层解析在途）：不驻留 unknown，待外层完成后重取
                return self.get_unknown_type();
            }
            let t = inferred.unwrap_or_else(|| self.get_unknown_type());
            self.value_symbol_links
                .get_or_default(symbol)
                .resolved_type = Some(Arc::clone(&t));
            return t;
        }
        if symbol.flags.contains(SymbolFlags::Alias) {
            // 环守卫：`export import A = require(./b)` 与 b 的 `export import
            // B = require(./a)` 互指时，别名类型解析经模块命名空间成员枚举
            // 形成无限递归；in-flight 符号重入返回 any
            if self.alias_type_resolution_stack.contains(&symbol.id()) {
                return self.get_any_type();
            }
            self.alias_type_resolution_stack.push(symbol.id());
            let result = self.get_type_of_symbol_alias_inner(symbol);
            self.alias_type_resolution_stack.pop();
            if let Some(t) = result {
                return t;
            }
        }

        if symbol.flags.contains(SymbolFlags::ValueModule)
            && symbol.flags.contains(SymbolFlags::Class)
        {
            // Go getTypeOfSymbol：Function|Method|Class|Enum|ValueModule 任一位
            // 即 getTypeOfFuncClassEnumModule（checker.go:17482-17484），无
            // namespace 合并特判；返回匿名静态侧对象（typeof m 的实体）
            return self.get_type_of_func_class_enum_module(symbol);
        }

        if (symbol.flags.contains(SymbolFlags::ValueModule)
            && (symbol.flags.contains(SymbolFlags::Function)
                || symbol.flags.contains(SymbolFlags::RegularEnum)
                || symbol.flags.contains(SymbolFlags::ConstEnum)))
            || (symbol.flags.contains(SymbolFlags::NamespaceModule)
                && symbol.flags.contains(SymbolFlags::Function))
        {
            return self.get_type_of_merged_namespace_symbol(symbol);
        }

        if symbol.flags.contains(SymbolFlags::Prototype) {
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
            let result = self.get_type_of_prototype_property(symbol);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(result.clone());
            return result;
        }

        if symbol.flags.intersects(SymbolFlags::ACCESSOR) {
            if let Some(t) = self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
            {
                return t;
            }
            let Some(decl) = symbol
                .declarations
                .iter()
                .find(|d| matches!(d.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor))
                .cloned()
            else {
                return self.get_any_type();
            };
            let key = Arc::as_ptr(symbol) as *const Symbol;
            if !self.push_type_resolution(key, crate::checker::TypeResolutionProperty::Type) {
                return self.error_type();
            }
            let t = self.resolve_accessor_pair_type(&decl);
            if !self.pop_type_resolution() {
                let annotated = symbol.declarations.iter().find_map(|d| match &d.data {
                    tsox_frontend::ast::NodeData::GetAccessorDeclaration(g) => {
                        g.type_node.as_ref().map(|_| Arc::clone(&g.name))
                    }
                    tsox_frontend::ast::NodeData::SetAccessorDeclaration(s) => s
                        .parameters
                        .iter()
                        .find_map(|p| match &p.data {
                            tsox_frontend::ast::NodeData::ParameterDeclaration(pd)
                                if pd.type_node.is_some() =>
                            {
                                Some(Arc::clone(&pd.name))
                            }
                            _ => None,
                        }),
                    _ => None,
                });
                if let Some(name) = annotated {
                    let file = self
                        .get_source_file_of_node(&name)
                        .or_else(|| self.current_file.clone());
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        name.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_0_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ITS_OWN_TYPE_ANNOTATION,
                        vec![symbol.name.clone()],
                    ));
                } else if decl.kind == SyntaxKind::GetAccessor && self.no_implicit_any {
                    let tsox_frontend::ast::NodeData::GetAccessorDeclaration(g) = &decl.data else {
                        return self.get_any_type();
                    };
                    let file = self
                        .get_source_file_of_node(&decl)
                        .or_else(|| self.current_file.clone());
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        g.name.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_0_IMPLICITLY_HAS_RETURN_TYPE_ANY_BECAUSE_IT_DOES_NOT_HAVE_A_RETURN_TYPE_ANNOTATION_AND_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_ONE_OF_ITS_RETURN_EXPRESSIONS,
                        vec![symbol.name.clone()],
                    ));
                }
                // Go getTypeOfAccessors 尾段：环失败驻留 any，后续求值不重报
                let any = self.get_any_type();
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&any));
                return any;
            }
            self.value_symbol_links
                .get_or_default(symbol)
                .resolved_type = Some(Arc::clone(&t));
            return t;
        }

        if symbol.flags.intersects(SymbolFlags::Method)
            && let Some(decl) = symbol
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature
                    )
                })
        {
            if let Some(links) = self.value_symbol_links.get(symbol)
                && let Some(ref t) = links.resolved_type
            {
                return Arc::clone(t);
            }
            // Go getTypeOfFuncClassEnumModule：符号有多声明（过载）时类型是
            // 各声明函数类型的联合；方法侧等价为收集全部无实现体签名
            //（接口/类型字面量方法是 MethodSignature，一律无实现体）
            let method_decls: Vec<Arc<Node>> = symbol
                .declarations
                .iter()
                .filter(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature
                    )
                })
                .cloned()
                .collect();
            let overload_sigs: Vec<Arc<Signature>> = {
                let mut sigs: Vec<Arc<Signature>> = Vec::new();
                for d in &method_decls {
                    let (parameters, type_node, has_body) = match &d.data {
                        tsox_frontend::ast::NodeData::MethodDeclaration(md) => (
                            &md.parameters,
                            md.type_node.as_ref(),
                            md.body.is_some(),
                        ),
                        tsox_frontend::ast::NodeData::MethodSignatureDeclaration(md) => {
                            (&md.parameters, md.type_node.as_ref(), false)
                        }
                        _ => continue,
                    };
                    if has_body {
                        continue;
                    }
                    self.push_scope(d);
                    let saved_stack = std::mem::take(&mut self.type_argument_stack);
                    let saved_frames = std::mem::take(&mut self.type_argument_name_frames);
                    let return_type = match type_node {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    let sig = self.build_signature_from_function_like_type_node(
                        parameters,
                        return_type,
                        false,
                        None,
                        Some(Arc::clone(d)),
                    );
                    self.type_argument_name_frames = saved_frames;
                    self.type_argument_stack = saved_stack;
                    self.pop_scope();
                    sigs.push(sig);
                }
                sigs
            };
            if overload_sigs.len() > 1 {
                let t = self.create_function_or_constructor_type(overload_sigs, false);
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                return t;
            }
            if decl.kind == SyntaxKind::MethodSignature {
                let t = self.create_function_or_constructor_type(overload_sigs, false);
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                return t;
            }
            let tsox_frontend::ast::NodeData::MethodDeclaration(data) = &decl.data else {
                return self.get_any_type();
            };
            self.push_scope(decl);
            // 符号类型是声明形式：重建时与进行中的实例化映射隔离，防止类型参数被外部绑定污染缓存
            let saved_stack = std::mem::take(&mut self.type_argument_stack);
            let saved_frames = std::mem::take(&mut self.type_argument_name_frames);
            let return_type = match data.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                // 无注解方法：从函数体推断返回类型（无 return 的 body 是 void，非 any）
                None => self.infer_method_return_type(&decl, &data.body),
            };
            let sig = self.build_signature_from_function_like_type_node(
                &data.parameters,
                return_type,
                false,
                None,
                Some(Arc::clone(decl)),
            );
            self.type_argument_name_frames = saved_frames;
            self.type_argument_stack = saved_stack;
            self.pop_scope();
            let t = self.create_function_or_constructor_type(vec![sig], false);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return t;
        }

        if symbol.flags.contains(SymbolFlags::BlockScopedVariable)
            || symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            || symbol.flags.contains(SymbolFlags::Function)
            || symbol.flags.contains(SymbolFlags::Class)
            || symbol.flags.contains(SymbolFlags::Property)
            || symbol.flags.contains(SymbolFlags::EnumMember)
        {
            if symbol.flags.contains(SymbolFlags::Function)
                && symbol.flags.contains(SymbolFlags::Class)
                && let Some(t) = self.merged_class_function_symbol_type(symbol)
            {
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&t));
                return t;
            }
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref t) = links.resolved_type {
                    // Go getTypeOfVariableOrParameterOrProperty：重入经
                    // pushTypeResolution 失败标记环窗口内全部帧（含外层互引
                    // 符号），再 reportCircularityError
                    if crate::checker::utilities::is_type_error(t)
                        && self.is_resolving(
                            Arc::as_ptr(symbol) as *const Symbol,
                            TypeResolutionProperty::Type,
                        )
                    {
                        // Go checkNodeDeferred（函数表达式/对象字面量成员箭头
                        // 体检查延迟）：rt_infer 边界下的重入不构成环，不标记
                        // 外层帧（标记会让帧出栈时迟报 TS7022）
                        if !self.cycle_crosses_rt_infer_boundary(symbol) {
                            self.mark_type_resolution_cycle(
                                Arc::as_ptr(symbol) as *const Symbol,
                                TypeResolutionProperty::Type,
                            );
                        }
                        return self.report_circularity_error(symbol);
                    }
                    return Arc::clone(t);
                }
            }

            // 无注解参数的 any 可能是节点级缓存的占位（get_type_of_node 污染），
            // 须放行到 on-demand 上下文定型
            let param_any_placeholder = |decl: &Arc<Node>, t: &Arc<Type>| {
                decl.kind == SyntaxKind::Parameter && t.flags.contains(TypeFlags::Any)
            };

            // 环断路器 in-flight error（递归接口构建窗口）不作为声明型返回，
            // 落到 on-demand 重解析
            if let Some(decl) = &symbol.value_declaration {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        if !param_any_placeholder(decl, t)
                            && !crate::checker::utilities::is_type_error(t)
                        {
                            return Arc::clone(t);
                        }
                    }
                }
            }

            for decl in &symbol.declarations {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        if !param_any_placeholder(decl, t)
                            && !crate::checker::utilities::is_type_error(t)
                        {
                            return Arc::clone(t);
                        }
                    }
                }
            }

            if let Some(t) = self.resolve_symbol_declared_type_on_demand(symbol) {
                // 同 resolve_symbol_declared_type_on_demand：error（环断路器
                // in-flight 产物）不驻留
                if !crate::checker::utilities::is_type_error(&t) {
                    self.value_symbol_links.get_or_default(symbol).resolved_type =
                        Some(Arc::clone(&t));
                }
                return t;
            }
            self.get_any_type()
        } else if symbol.flags.intersects(SymbolFlags::ENUM)
            && symbol
                .flags
                .intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule)
        {
            // Go getTypeOfSymbol：Enum 位优先于 module 位，合并符号统一走
            // getTypeOfFuncClassEnumModule（枚举成员与导出共构匿名对象）
            self.get_type_of_merged_namespace_symbol(symbol)
        } else if symbol.flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule) {
            // 脚本级 namespace 声明在 binder 中为 NamespaceModule，但值位
            // 引用（new multiM.c()）同样取模块成员型（Go NamespaceModule
            // 与 ValueModule 的 getTypeOfFuncClassEnumModule 同路）
            self.resolve_namespace_type(symbol)
        } else if symbol.flags.intersects(SymbolFlags::ENUM) {
            self.resolve_enum_value_type(symbol)
        } else {
            self.get_any_type()
        }
    }

    // 方法体返回类型推断：走通用 infer_function_return_type（无 return 的 body 推断为 void）；
    // 生成器方法按 Go getReturnTypeFromBody 分派 infer_generator_return_type
    pub(crate) fn infer_method_return_type(
        &mut self,
        decl: &Arc<Node>,
        body: &Option<Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("infer_method_return_type"); 
        // Go getReturnTypeOfSignature：方法帧整体即返回型解析窗口,重入计环
        if body.is_some() && !self.push_signature_return_resolution(Arc::as_ptr(decl)) {
            return self.get_any_type();
        }
        if body.is_some() {
            self.set_signature_return_inference_phase(Arc::as_ptr(decl), true);
        }
        let is_generator = match &decl.data {
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => d.asterisk_token.is_some(),
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.asterisk_token.is_some(),
            tsox_frontend::ast::NodeData::FunctionExpression(d) => d.asterisk_token.is_some(),
            _ => false,
        };
        let is_async_fn = decl.has_syntactic_modifier(ModifierFlags::Async);
        let result = if is_generator && let Some(body) = body.as_ref() {
            self.infer_generator_return_type(body, is_async_fn)
        } else {
            self.infer_function_return_type(Some(decl), body.as_ref(), None)
        };
        if body.is_some() {
            self.set_signature_return_inference_phase(Arc::as_ptr(decl), false);
        }
        if body.is_some() && !self.pop_signature_return_resolution() {
            self.report_signature_return_circularity(decl);
            return self.get_any_type();
        }
        result
    }

    pub(crate) fn attach_function_expando_type(
        &mut self,
        symbol: &Arc<tsox_frontend::ast::Symbol>,
        base: Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("attach_function_expando_type"); 
        let mut entries: Vec<(String, Arc<Node>)> = Vec::new();
        for (name, sym) in symbol.exports.iter() {
            if name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT {
                for d in &sym.declarations {
                    let mname = match &d.data {
                        tsox_frontend::ast::NodeData::BinaryExpression(b) => match &b.left.data {
                            tsox_frontend::ast::NodeData::ElementAccessExpression(eae) => {
                                let arg = &eae.argument_expression;
                                if !tsox_frontend::ast::is_entity_name_expression(arg) {
                                    String::new()
                                } else {
                                    let t = self.with_declaring_file_context(d, |c| {
                                        c.get_type_of_node(arg)
                                    });
                                    if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&t) {
                                        crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(&t)
                                    } else {
                                        String::new()
                                    }
                                }
                            }
                            _ => String::new(),
                        },
                        _ => String::new(),
                    };
                    entries.push((mname, Arc::clone(d)));
                }
            } else if sym.flags.contains(SymbolFlags::Property)
                && !sym.declarations.is_empty()
                && sym
                    .declarations
                    .iter()
                    .all(|d| d.kind == SyntaxKind::BinaryExpression)
            {
                for d in &sym.declarations {
                    entries.push((name.clone(), Arc::clone(d)));
                }
            }
        }
        if entries.is_empty() {
            return base;
        }
        entries.sort_by_key(|(_, node)| node.loc.pos());
        let mut table = tsox_frontend::ast::SymbolTable::new();
        let mut props: Vec<Arc<tsox_frontend::ast::Symbol>> = Vec::new();
        for (name, node) in entries {
            if table.entries.contains_key(&name) {
                continue;
            }
            let tsox_frontend::ast::NodeData::BinaryExpression(bin) = &node.data else {
                continue;
            };
            let member_name = name.clone();
            let rhs_type = self.with_declaring_file_context(&node, |c| {
                let t = c.get_type_of_node(&bin.right);
                let ctx = c.get_contextual_type(&bin.right, crate::checker::types::ContextFlags::None);
                let literal_of_ctx =
                    ctx.as_ref().is_some_and(|cx| c.is_literal_of_contextual_type(&t, cx));
                let t = if literal_of_ctx {
                    t
                } else {
                    c.get_widened_literal_type(&t)
                };
                let t = c.get_regular_type_of_literal_type(&t);
                let empty_array_elem = c
                    .get_element_type_of_array_type(&t)
                    .is_some_and(|el| c.is_empty_literal_type(&el));
                if empty_array_elem && !c.expando_parent_has_type_annotation(symbol) {
                    c.report_expando_implicit_any_array(&node, &member_name);
                    c.create_array_type(c.get_any_type())
                } else {
                    c.get_widened_type(&t)
                }
            });
            let prop = Arc::new(tsox_frontend::ast::Symbol::new(
                SymbolFlags::Property,
                name.clone(),
            ));
            let prop_mut = Arc::as_ptr(&prop) as *mut tsox_frontend::ast::Symbol;
            unsafe {
                (*prop_mut).declarations.push(Arc::clone(&node));
            }
            self.value_symbol_links.insert(
                &prop,
                ValueSymbolLinks {
                    resolved_type: Some(rhs_type),
                    ..Default::default()
                },
            );
            table.insert(name.clone(), Arc::clone(&prop));
            props.push(prop);
        }
        if props.is_empty() {
            return base;
        }
        let face_structured = |members, props, sigs, count, index_infos| StructuredTypeData {
            members,
            properties: props,
            signatures: sigs,
            call_signature_count: count,
            index_infos,
            ..Default::default()
        };
        if base.flags.contains(TypeFlags::Object)
            && let Some(base_obj) = base.as_object()
        {
            let structured = face_structured(
                table,
                props,
                base_obj.structured.signatures.clone(),
                base_obj.structured.call_signature_count,
                base_obj.structured.index_infos.clone(),
            );
            return Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: ObjectFlags::Anonymous,
                id: crate::checker::types::next_type_id(),
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::Object(ObjectTypeData { node: None,
                    structured,
                    target: None,
                    mapper: None,
                    type_arguments: Vec::new(),
                }),
            });
        }
        let face = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
                structured: face_structured(
                    table,
                    props,
                    Vec::new(),
                    0,
                    Vec::new(),
                ),
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        });
        Arc::new(Type {
            flags: TypeFlags::Intersection,
            object_flags: ObjectFlags::None,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Intersection(IntersectionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types: vec![base, face],
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn add_optional_undefined(&mut self, t: Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("add_optional_undefined"); 
        if !self.strict_null_checks {
            return t;
        }

        if t.flags.contains(TypeFlags::Any) && t.intrinsic_name() == Some("error") {
            return t;
        }
        let already = t.flags.contains(TypeFlags::Undefined)
            || (t.flags.contains(TypeFlags::Union)
                && t.types()
                    .is_some_and(|ts| ts.iter().any(|c| c.flags.contains(TypeFlags::Undefined))));
        if already {
            return t;
        }
        self.get_union_type(vec![t, self.undefined_type()])
    }

    pub(crate) fn strip_optional_undefined(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("strip_optional_undefined"); 
        if t.flags.contains(TypeFlags::Union)
            && let Some(ts) = t.types()
        {
            let kept: Vec<Arc<Type>> = ts
                .iter()
                .filter(|c| !c.flags.contains(TypeFlags::Undefined))
                .cloned()
                .collect();
            if !kept.is_empty() && kept.len() != ts.len() {
                return if kept.len() == 1 {
                    kept.into_iter().next().expect("nonempty")
                } else {
                    self.get_union_type(kept)
                };
            }
        }
        Arc::clone(t)
    }
}
impl Checker {
    /// None = 纯 alias 语义已消费完毕，调用方按合并符号的非 alias 意义继续
    fn get_type_of_symbol_alias_inner(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_of_symbol_alias_inner"); 
        // import X = require("./m")：binder export_symbol 指向模块符号，Go
        // getTargetOfImportEqualsDeclaration 经 resolveExternalModuleSymbol
        // 再取 export= 符号，优先走该解析
        let is_require_import_equals = symbol.declarations.iter().any(|d| {
            matches!(
                &d.data,
                tsox_frontend::ast::NodeData::ImportEqualsDeclaration(ied)
                    if matches!(
                        &ied.module_reference.data,
                        tsox_frontend::ast::NodeData::ExternalModuleReference(_)
                    )
            )
        });
        if is_require_import_equals {
            if let Some(resolved) = self.resolve_import_alias_target_symbol(symbol)
                && !Arc::ptr_eq(&resolved, symbol)
            {
                let t = self.get_type_of_symbol(&resolved);
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
        }
        let target = self.follow_alias(symbol);
        if let Some(target) = target
            && !Arc::ptr_eq(&target, symbol)
            && !target.flags.contains(SymbolFlags::Alias)
        {
            let t = self.get_type_of_symbol(&target);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return Some(t);
        }
        // binder 未挂 export_symbol 的 import 别名（ImportClause default、
        // js export= 成员）：检查期解析目标符号
        if let Some(resolved) = self.resolve_import_alias_target_symbol(symbol)
            && !Arc::ptr_eq(&resolved, symbol)
        {
            let t = self.get_type_of_symbol(&resolved);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return Some(t);
        }
        // 合并符号（re-export 局部符号带 Alias 位）无值/类型意义：any
        if !symbol
            .flags
            .intersects(SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE)
        {
            return Some(self.get_any_type());
        }
        None
    }
}
