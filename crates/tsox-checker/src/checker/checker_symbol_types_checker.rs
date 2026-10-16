#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

impl Checker {
    pub fn get_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_symbol"); 
        // Go getTypeOfSymbol 的全部调用点以合并符号为入参（getSymbol
        // checker.go:2275、getSymbolOfDeclaration checker.go:15186、
        // getSymbolAtLocation checker.go:33454 均过 getMergedSymbol），类型缓存
        // （symbolLinks/typeNodeLinks）皆挂合并符号；本侧部分解析路径
        // （check_function_declaration 的 resolve_identifier 等）直接携带节点侧
        // 符号，其单声明节点缓存（单签名函数型）会劫持合并符号型——入口先归并，
        // 无合并记录恒等返回
        let symbol = &self.get_merged_symbol(symbol);
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
        // Go getTypeOfSymbol（checker.go:17470-17471）：CheckFlagsMapped 符号
        // 分派 getTypeOfMappedSymbol——映射成员符号按 containing_type/key_type
        // 惰性实例化模板求型并驻留；须先于 Variable|Property 臂（checker.go:
        // 17479），合成成员无 value_declaration 不可走声明臂
        if symbol
            .check_flags
            .intersects(tsox_frontend::ast::CheckFlags::Mapped)
        {
            return self.get_type_of_mapped_symbol(symbol);
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
            let t = self.resolve_accessor_pair_type_with_symbol(&decl, Some(symbol));
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
            // Go getTypeOfFuncClassEnumModuleWorker 尾部（checker.go:17908-17912）：
            // strictNullChecks 下 Optional 方法符号型并 undefined
            let optional_tail =
                self.strict_null_checks && symbol.flags.intersects(SymbolFlags::Optional);
            if let Some(links) = self.value_symbol_links.get(symbol)
                && let Some(ref t) = links.resolved_type
            {
                // 预填命中同径补尾，但 undefined 只落在取型出口不落预填驻留值：
                // 成员表预填（add_method_signature_member/add_method_declaration_member）
                // 的纯签名函数型被过载合并臂 as_structured().call_signatures() 与
                // 直读 resolved_type 的消费者消费，联合驻留使合并臂读空签名、
                // 直读位拿到非基线形态（contextualTypeIterableUnions 回归）；
                // 包装范围限 MethodSignature（类方法成员预填保持基线口径）
                if optional_tail && decl.kind == SyntaxKind::MethodSignature {
                    return self.get_optional_type(Arc::clone(t), true);
                }
                return Arc::clone(t);
            }
            // Go getTypeOfFuncClassEnumModule 经 resolveAnonymousTypeMembers 取
            // getSignaturesOfSymbol（checker.go:20983-21005）：全函数式声明
            // 签名入集，实现体签名仅当紧邻前一同父同类声明（decl.Pos()==
            // previous.End() 或前声明 Reparsed）时排除——实现与过载间夹有
            // 同名 static 成员即非紧邻，实现签名留在集内呈多签名型。
            // 签名构建不复用 signature_links 缓存：get_signature_from_declaration
            // 可能在调用解析实例化窗口内构建并驻留带类型参克隆的签名（帧
            // 键按名等价命中跨声明同名类型参），Method 臂消费即呈 TS2719
            // 两不同 T 与 T_1 渲染（staticAnonymousTypeNotReferencingTypeParameter）。
            // 沿 push_scope + type_argument_stack/name_frames 隔离的新鲜构建
            // 对齐 getSignatureFromDeclaration（checker.go:21014）非实例化语义
            let mut overload_sigs: Vec<Arc<Signature>> = Vec::new();
            for (i, d) in symbol.declarations.iter().enumerate() {
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
                if i > 0 && has_body {
                    let previous = &symbol.declarations[i - 1];
                    let same_parent = d
                        .parent()
                        .as_ref()
                        .zip(previous.parent().as_ref())
                        .map(|(a, b)| a.id() == b.id())
                        .unwrap_or(false);
                    if same_parent
                        && d.kind == previous.kind
                        && (d.pos() == previous.end()
                            || previous.flags.intersects(NodeFlags::Reparsed))
                    {
                        continue;
                    }
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
                overload_sigs.push(sig);
            }
            if overload_sigs.len() > 1 {
                let t = self.create_function_or_constructor_type(overload_sigs, false);
                let t = if optional_tail {
                    self.get_optional_type(t, true)
                } else {
                    t
                };
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                return t;
            }
            if decl.kind == SyntaxKind::MethodSignature {
                let t = self.create_function_or_constructor_type(overload_sigs, false);
                let t = if optional_tail {
                    self.get_optional_type(t, true)
                } else {
                    t
                };
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
            let t = if optional_tail {
                self.get_optional_type(t, true)
            } else {
                t
            };
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
            // Go getTypeOfFuncClassEnumModuleWorker 对类符号恒产出匿名片构型
            //（checker.go:17897-17919），any 驻留不可能是类符号的合法值型——
            // 缓存命中也视同未解析走重算；构造型带构造签名由类声明节点急建
            // 产物（get_type_of_class_declaration）提供
            let pure_class = symbol.flags.contains(SymbolFlags::Class)
                && !symbol.flags.intersects(
                    SymbolFlags::Function | SymbolFlags::VARIABLE | SymbolFlags::Property,
                );
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
                    if pure_class && t.flags.contains(TypeFlags::Any) {
                        // 类符号 any 驻留视同未解析，落声明节点重算
                    } else {
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
            }

            // 无注解参数的 any 可能是节点级缓存的占位（get_type_of_node 污染），
            // 须放行到 on-demand 上下文定型
            let param_any_placeholder = |decl: &Arc<Node>, t: &Arc<Type>| {
                decl.kind == SyntaxKind::Parameter && t.flags.contains(TypeFlags::Any)
            };

            // Go getTypeOfSymbol（checker.go:17476-17484）：Function 位符号的型
            // 经 getTypeOfFuncClassEnumModule（checker.go:17888-17895）只读
            // links.resolvedType，未命中由 resolveStructuredTypeMembers →
            // getSignaturesOfSymbol（checker.go:20993-21002）聚合全部函数声明
            // 签名；声明节点缓存（check_function_declaration 挂单声明函数型）
            // 不作为多声明符号的型来源——过载/跨文件合并（lib.dom alert + 本文件
            // declare function alert）符号落节点缓存即漏聚合，.types 呈单签名
            // 裸函数型而非 N 签名对象字面量（nodebuilderimpl.go:2884/2922）
            if symbol.flags.contains(SymbolFlags::Function)
                && symbol
                    .declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::FunctionDeclaration)
                    .count()
                    > 1
            {
                if let Some(base) = self.build_overload_function_type(symbol) {
                    let t = self.attach_function_expando_type(symbol, base);
                    if !crate::checker::utilities::is_type_error(&t) {
                        self.value_symbol_links
                            .get_or_default(symbol)
                            .resolved_type = Some(Arc::clone(&t));
                    }
                    return t;
                }
            }

            // 环断路器 in-flight error（递归接口构建窗口）不作为声明型返回，
            // 落到 on-demand 重解析
            if let Some(decl) = &symbol.value_declaration {
                if let Some(links) = self.type_node_links.get(decl) {
                    if let Some(ref t) = links.resolved_type {
                        if !param_any_placeholder(decl, t)
                            && !crate::checker::utilities::is_type_error(t)
                            && !(pure_class && t.flags.contains(TypeFlags::Any))
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
                            && !(pure_class && t.flags.contains(TypeFlags::Any))
                        {
                            return Arc::clone(t);
                        }
                    }
                }
            }

            if let Some(t) = self.resolve_symbol_declared_type_on_demand(symbol) {
                // 同 resolve_symbol_declared_type_on_demand：error（环断路器
                // in-flight 产物）不驻留；类符号建型在途 any 占位同不驻留
                if !crate::checker::utilities::is_type_error(&t)
                    && !(pure_class && t.flags.contains(TypeFlags::Any))
                {
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
        } else if symbol.flags.contains(SymbolFlags::Alias)
            && !symbol.flags.contains(SymbolFlags::ValueModule)
            && !symbol.flags.intersects(SymbolFlags::ENUM)
        {
            // Go getTypeOfSymbol（checker.go:17476-17491）分派序：Accessor →
            // Variable|Property（17479）→ Function|Method|Class|Enum|ValueModule
            //（17482，ValueModule 位先于 Alias）→ EnumMember（17485）→ Alias
            //（17488，末位）。import 别名与局部声明合并的符号（如 var a +
            // import a = M 的 FSV|Alias，Go .symbols 基线单符号双声明）按
            // Variable 位走变量路径取隐式 any，别名位不得抢先；仅 NamespaceModule
            // 与 Alias 并存的合并符号（import {N} 合并非实例化 namespace N）无
            // 任何组 4 位，落 Alias 位 getTypeOfAlias——经别名解析取导入目标
            // 符号类型，非按本地 namespace 成员自构模块成员型。环守卫：
            // `export import A = require(./b)` 与 b 的
            // `export import B = require(./a)` 互指时，别名类型解析经模块命名
            // 空间成员枚举形成无限递归；in-flight 符号重入返回 any
            if self.alias_type_resolution_stack.contains(&symbol.id()) {
                return self.get_any_type();
            }
            self.alias_type_resolution_stack.push(symbol.id());
            let result = self.get_type_of_symbol_alias_inner(symbol);
            self.alias_type_resolution_stack.pop();
            if let Some(t) = result {
                return t;
            }
            self.get_any_type()
        } else if symbol.flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule) {
            // Go getTypeOfSymbol（checker.go:17462-17491）分派无 NamespaceModule
            // 臂：仅 NamespaceModule（未实例化模块/模块增广符号）落末位
            // errorType（any 韵味，渲染 any）；带 ValueModule 位走
            // getTypeOfFuncClassEnumModule（checker.go:17482），本侧以
            // resolve_namespace_type 同构
            if !symbol.flags.contains(SymbolFlags::ValueModule) {
                return self.error_type();
            }
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
        let annotated_parent = self.expando_parent_has_type_annotation(symbol);
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
            let rhs_type = if annotated_parent {
                None
            } else {
                Some(self.with_declaring_file_context(&node, |c| {
                    let t = c.check_expression_for_mutable_location(&bin.right, CheckMode::Normal);
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
                }))
            };
            let prop = Arc::new(tsox_frontend::ast::Symbol::new(
                SymbolFlags::Property,
                name.clone(),
            ));
            let prop_mut = Arc::as_ptr(&prop) as *mut tsox_frontend::ast::Symbol;
            unsafe {
                (*prop_mut).declarations.push(Arc::clone(&node));
            }
            match rhs_type {
                Some(rhs_type) => {
                    self.value_symbol_links.insert(
                        &prop,
                        ValueSymbolLinks {
                            resolved_type: Some(rhs_type),
                            ..Default::default()
                        },
                    );
                }
                None => {
                    self.expando_deferred_props
                        .insert(Arc::as_ptr(&prop) as *const tsox_frontend::ast::Symbol as u64);
                }
            }
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
        // Go resolveAliasWorker（checker.go:17236-17247）useResolvedLinks 臂：
        // aliasTarget 已解析的别名直接取缓存目标——合成 default 包装的 default
        // 成员（create_default_property_wrapper_for_module 挂 alias_target）
        // 由此呈目标型，否则落到尾部无 VALUE 位呈 any
        if let Some(target) = self
            .alias_symbol_links
            .get(symbol)
            .and_then(|l| l.alias_target.clone())
            && !Arc::ptr_eq(&target, symbol)
        {
            let t = self.get_type_of_symbol(&target);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return Some(t);
        }
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
        // `export = React`（ExportAssignment 声明的纯 Alias 符号）：Go
        // getTypeOfSymbol Alias 位 → resolveAlias（checker.go:17236）经
        // getTargetOfAliasDeclaration（checker.go:16686 ExportAssignment 臂）
        // 解到赋值目标实体（namespace React），类型取目标符号类型
        //（`import React from "react"` 得 typeof React）。binder 对 export=
        // 只置 Alias 位不挂 export_symbol，此处按声明形态走检查期解析
        if symbol
            .declarations
            .iter()
            .any(|d| matches!(&d.data, tsox_frontend::ast::NodeData::ExportAssignment(_)))
            && let Some(resolved) = self.resolve_export_assignment_target(symbol)
            && !Arc::ptr_eq(&resolved, symbol)
        {
            let t = self.get_type_of_symbol(&resolved);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return Some(t);
        }
        // `export as namespace X`（NamespaceExportDeclaration 声明的纯 Alias
        // 符号）：Go resolveAlias 经 getTargetOfAliasDeclaration
        //（checker.go:16699 NamespaceExportDeclaration 臂）→
        // getTargetOfNamespaceExportDeclaration（checker.go:15847-15854）→
        // resolveExternalModuleSymbol(宿主文件符号, dontResolveAlias=true)
        //（checker.go:16488-16497）——文件有 export= 时解到 export= 符号本身
        //（不解穿），resolveAlias 对 Alias 位目标递归到赋值目标
        //（export = Foo → typeof Foo；目标为 any 导入 → any）；无 export=
        // 才落宿主文件模块符号（typeof import(...)）。该别名 binder 不挂
        // export_symbol，follow_alias 返回自身即断链呈 any
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::NamespaceExportDeclaration)
            .cloned()
        {
            if let Some(resolved) = self.get_target_of_namespace_export_declaration(&decl)
                && !Arc::ptr_eq(&resolved, symbol)
            {
                let t = self.get_type_of_symbol(&resolved);
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
        }
        // `export { X }`（声明形态纯 ExportSpecifier 的别名符号）：Go
        // getTypeOfSymbol Alias 位 → getTypeOfAlias → resolveAlias 经
        // getTargetOfAliasDeclaration（checker.go:16686）ExportSpecifier 臂，
        // 无 from 解到本容器绑定、from 解到模块成员，类型取目标符号类型。该
        // 形态不进 resolve_import_alias_target_symbol 的声明形态查（缺
        // ExportSpecifier 位），follow_alias 无 export_symbol 断链，按声明
        // 形态显式走通用别名解析
        if !symbol.declarations.is_empty()
            && symbol
                .declarations
                .iter()
                .all(|d| d.kind == SyntaxKind::ExportSpecifier)
        {
            let resolved = self.resolve_alias_base(Arc::clone(symbol));
            if !Arc::ptr_eq(&resolved, symbol) {
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
        // `export { X } from "m"` re-export specifier（binder 建纯 Alias 不挂
        // export_symbol，bind_walk_binder_2.rs:218-237，follow_alias 断链）：
        // Go getTypeOfSymbol Alias 位 → resolveAlias（checker.go:17224）经
        // getTargetOfAliasDeclaration（checker.go:16693-16694）ExportSpecifier
        // 臂 getTargetOfExportSpecifier(node, Value|Type|Namespace,
        // dontRecursivelyResolve)——带模块说明符时 getExternalModuleMember 按
        // 声明文件模块解析取同员，类型取目标符号类型（typeof import("m").X）
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ExportSpecifier)
            && let Some(resolved) = self.get_target_of_export_specifier(
                decl,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
                true,
            )
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
