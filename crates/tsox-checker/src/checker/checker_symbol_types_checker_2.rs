#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

enum BindingPathSeg {
    Prop(String, bool),
    Index(usize),
    Rest,
}

impl Checker {
    pub(crate) fn resolve_symbol_declared_type_on_demand(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("resolve_symbol_declared_type_on_demand"); 
        // Go getTypeOfVariableOrParameterOrPropertyWorker：变量类符号类型解析
        // 进 type resolution 栈，同符号重入即环（reportCircularityError）
        let deferred_expando_prop = self
            .expando_deferred_props
            .contains(&(Arc::as_ptr(symbol) as *const Symbol as u64));
        let uses_frame = deferred_expando_prop
            || symbol.declarations.iter().any(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::Parameter
                        | SyntaxKind::PropertyDeclaration
                        | SyntaxKind::PropertySignature
                        | SyntaxKind::PropertyAssignment
                        | SyntaxKind::ShorthandPropertyAssignment
                        | SyntaxKind::VariableDeclaration
                        | SyntaxKind::BindingElement
                        | SyntaxKind::ExportAssignment
                        | SyntaxKind::JsxAttribute
                )
            });
        if !uses_frame {
            return self.resolve_symbol_declared_type_on_demand_inner(symbol);
        }
        if !self.push_type_resolution(
            Arc::as_ptr(symbol) as *const Symbol,
            crate::checker::TypeResolutionProperty::Type,
        ) {
            let plain_binding_element = symbol.declarations.iter().any(|d| {
                matches!(
                    &d.data,
                    NodeData::BindingElement(be) if be.initializer.is_none()
                )
            });
            if plain_binding_element {
                return Some(self.circular_constraint_type());
            }
            return Some(self.report_circularity_error(symbol));
        }
        self.variable_type_frame_depth += 1;
        // Go getTypeOfVariableOrParameterOrPropertyWorker（checker.go:17802-17870）
        // 的声明型解析无环境替换帧：声明型恒对声明本体求值（Go 单例世界无全局
        // 代入栈，实例化由消费端 mapper 承担——getTypeOfInstantiatedSymbol
        // checker.go:17455-17469 = instantiateType(getTypeOfSymbol(target), mapper)）。
        // 本仓接口实例化走帧重建（resolve_interface_pass 推接口实参帧），成员表
        // 构建期 property_signature_type_deferred 把声明成员型推迟到首个
        // get_type_of_symbol（check_interface_members 不解析成员注解，注解节点
        // 缓存与符号声明型同刻首填）——该首解析若落在实例化窗内（Exclude 条件
        // 式 extends 比较链经实例化成员符号 target 回指声明符号，
        // checker_symbol_types_checker.rs Instantiated 分支），注解内容器型参
        // 引用被帧代入烤进声明型缓存（符号 links 与注解节点缓存同染），显示位
        // 声明成员注解呈窗内实参型参（explicitMembersBeforeInherited 的 >subs
        // 应呈 IndentationSubTree<L>[]，NodeBase<L> 实例窗首解析代入 IndentationTree
        // 的 L 后阴影判定按跨声明改名 L_1）——接口成员注解按声明形态解析（与
        // get_type_parameter_from_symbol 约束清帧同款，
        // typenode_references_checker_7.rs:69-76），实例成员型仍由
        // Instantiated 分支的 instantiate_type(声明型, mapper) 承担代入
        let interface_member_annotation = symbol.declarations.iter().any(|d| {
            d.kind == SyntaxKind::PropertySignature
                && d.parent().is_some_and(|p| p.kind == SyntaxKind::InterfaceDeclaration)
        });
        let result = if interface_member_annotation {
            let saved_stack = std::mem::take(&mut self.type_argument_stack);
            let saved_frames = std::mem::take(&mut self.type_argument_name_frames);
            let r = self.resolve_symbol_declared_type_on_demand_inner(symbol);
            self.type_argument_stack = saved_stack;
            self.type_argument_name_frames = saved_frames;
            r
        } else {
            self.resolve_symbol_declared_type_on_demand_inner(symbol)
        };
        self.variable_type_frame_depth -= 1;
        if !self.pop_type_resolution() {
            return Some(self.report_circularity_error(symbol));
        }
        result
    }

    // Go getContextualTypeForAssignmentExpression（checker.go:31585-31632）赋值
    // 声明臂：binary 挂 binder 符号（binary.Symbol != nil）且宿主为带注解
    // VariableDeclaration 时，目标上下文 = 注解型上的同名属性型——
    // PropertyAccess 按名字文本（31599-31601），ElementAccess 经
    // checkExpressionCached(实参) 折叠名型（31602-31608，[mySymbol] 实参
    // unique symbol → mySymbol）；宿主非带注解变量恒 None（31606/31612），
    // 不回落 getTypeOfExpression(left)——延迟成员解析窗内该调用经属性访问
    // 回指延迟符号自环。调用点声明即赋值声明（expando 成员符号的声明表），
    // 免 binary.Symbol gate。注解属性取型走赋值目标写位读路径
    // assignment_target_type（写型 = 成员声明型按 declared_type_parameter_types
    // × type_arguments 重代入，checker_assignment2_checker.rs:237-280）——r9 的
    // expando_annotated_property_type（get_property_of_type +
    // substituted_member_type_of：resolve_interface_type_ex 重建成员表 +
    // Instantiated 分支 instantiate_type 代入）在延迟成员解析窗内对带实参
    // 注解产出未代入 Partial<P>，属性级映射臂 keyof P 基约束空取不到
    // color，fresh "red" 无字面量语境拓宽为 string
    //（expandoFunctionContextualTypes anchor 行17 defaultProps 应
    // { color: "red" } 呈 { color: string }）；写位路径即发射位 LHS 直查链
    //（同例 .types 行22 >MyComponent.defaultProps 与行27 对象字面量保真
    // 同源），成员缺失时 None 回落 RHS 驻留型
    fn expando_assignment_target_contextual_type(
        &mut self,
        binary_node: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("expando_assignment_target_contextual_type");
        use tsox_frontend::ast::NodeData;
        let bin = match &binary_node.data {
            NodeData::BinaryExpression(b) => b,
            _ => return None,
        };
        let (base, arg): (&Arc<Node>, Option<&Arc<Node>>) = match &bin.left.data {
            NodeData::PropertyAccessExpression(pa)
                if matches!(&pa.expression.data, NodeData::Identifier(_)) =>
            {
                (&pa.expression, None)
            }
            NodeData::ElementAccessExpression(ea)
                if matches!(&ea.expression.data, NodeData::Identifier(_)) =>
            {
                (&ea.expression, Some(&ea.argument_expression))
            }
            _ => return None,
        };
        let resolved = self.resolve_identifier(base)?;
        let sym = self.get_export_symbol_of_value_symbol_if_exported(&resolved);
        if sym
            .flags
            .contains(tsox_frontend::ast::SymbolFlags::ModuleExports)
        {
            return None;
        }
        let vd = sym.value_declaration.as_ref()?;
        let NodeData::VariableDeclaration(vdd) = &vd.data else {
            return None;
        };
        if vdd.type_node.is_none() {
            return None;
        }
        match (&bin.left.data, arg) {
            (NodeData::PropertyAccessExpression(_), _) => self.assignment_target_type(&bin.left),
            (NodeData::ElementAccessExpression(_), Some(arg)) => {
                use crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name;
                let name_type = self.check_expression_cached(arg);
                if is_type_usable_as_property_name(&name_type) {
                    self.assignment_target_type(&bin.left)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // Go getWidenedTypeForAssignmentDeclaration（checker.go:19134-19174）
    // BinaryExpression 臂：getAssignmentDeclarationInitializerType
    //（checker.go:19188-19214）default 臂 checkExpressionForMutableLocation
    // (RHS, Normal) 逐声明收集，getUnionType 合并、getWidenedType 收尾，
    // 无就绪门——重查上下文由内部 getContextualType(RHS) 经赋值目标注解链
    // 取得。空数组隐式 any 报告挂 hasParentWithTypeAnnotation 门
    //（checker.go:19207），本 helper 仅服务有注解父符号的延迟成员（无注解
    // 父符号走附着期急切路径），报告位不达。
    // 本仓 check 诊断与 .types 发射是两个 Checker 实例：诊断走
    // build_checker_internal 一次性实例（program_3.rs get_semantic_diagnostics），
    // 发射走 get_type_checker_for_file 池实例（m4w_2.rs）——赋值语句只在
    // 诊断实例检查过，发射实例冷缓存下 RHS 驻留型判存恒空（warm 缓存对
    // 发射不可见），r9 就绪门恒闭使延迟成员显示位呈 any
    //（expandoFunctionContextualTypes anchor 行17 defaultProps: any、
    // expandoFunctionExpressionsWithDynamicNames2 行18、
    // expandoPropertyEmptyArrayWidening 行14）。主臂改注解上下文直取
    //（expando_assignment_target_contextual_type，binder 符号+注解 type
    // node 状态无关，两实例皆可求）并显式压栈，重查内部 get_contextual_type
    // 经 find_contextual_node 恒命中（容器级查询命中压栈位，属性级上下文由
    // check_property_assignment 经容器上下文自举，fresh 字面量以 regular 位
    // 存活）；上下文不可得回落 RHS 驻留型判存（诊断实例赋值语句检查后的
    // 臂，check_assignment_compat 经 get_type_of_node(RHS) 写回）。就绪门
    // 弃用背景：附着同窗的 const 声明可赋值性检查先于赋值语句，RHS 缓存
    // 未建时返回 None 落未驻留 any 保 TS2322 消失（r9 修复），主臂的注解
    // 上下文产实型（{ color: 'red' }/true/never[]）同样可赋
    fn resolve_expando_deferred_member_type(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("resolve_expando_deferred_member_type");
        let mut types: Vec<Arc<Type>> = Vec::new();
        for d in &symbol.declarations {
            let tsox_frontend::ast::NodeData::BinaryExpression(bin) = &d.data else {
                continue;
            };
            let resolved = self.with_declaring_file_context(d, |c| {
                match c.expando_assignment_target_contextual_type(d) {
                    Some(ctx) => {
                        let context_node = c.get_context_node(&bin.right);
                        c.push_contextual_type(&context_node, &ctx, false);
                        let t = c.check_expression_for_mutable_location(
                            &bin.right,
                            crate::checker::types_alias_symbol_links::CheckMode::Normal,
                        );
                        c.pop_contextual_type();
                        Some(t)
                    }
                    None => c
                        .type_node_links
                        .get(&bin.right)
                        .and_then(|l| l.resolved_type.clone())
                        .map(|_| {
                            c.check_expression_for_mutable_location(
                                &bin.right,
                                crate::checker::types_alias_symbol_links::CheckMode::Normal,
                            )
                        }),
                }
            });
            if let Some(t) = resolved {
                types.push(t);
            }
        }
        if types.is_empty() {
            return None;
        }
        let t = if types.len() == 1 {
            types.pop().unwrap()
        } else {
            self.get_union_type(types)
        };
        Some(self.get_widened_type(&t))
    }

    fn resolve_symbol_declared_type_on_demand_inner(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("resolve_symbol_declared_type_on_demand_inner"); 
        use tsox_frontend::ast::NodeData;
        // 有注解父符号（StatelessComponent<P> 形）的 expando 赋值成员：附着期
        // 不急切求型（attach_function_expando_type 标记），成员型按 Go
        // getWidenedTypeForAssignmentDeclaration（checker.go:19134-19174）在
        // 赋值语句已检查后惰性解析——checkExpressionForMutableLocation(RHS)
        // 带赋值目标上下文重查保住 regular 字面量，getWidenedType 收尾只拓
        // fresh 位
        if self
            .expando_deferred_props
            .contains(&(Arc::as_ptr(symbol) as *const Symbol as u64))
        {
            return self.resolve_expando_deferred_member_type(symbol);
        }
        // 合并符号（UMD 全局 export as namespace + declare global 变量）声明
        // 列表混有非变量声明：优先取变量/属性/参数类声明
        let decl = symbol
            .declarations
            .iter()
            .find(|d| {
                matches!(
                    d.data,
                    NodeData::VariableDeclaration(_)
                        | NodeData::PropertyDeclaration(_)
                        | NodeData::PropertySignatureDeclaration(_)
                        | NodeData::PropertyAssignment(_)
                        | NodeData::ShorthandPropertyAssignment(_)
                        | NodeData::ParameterDeclaration(_)
                        | NodeData::BindingElement(_)
                        | NodeData::EnumMember(_)
                        | NodeData::JsxAttribute(_)
                )
            })
            .cloned()
            .or_else(|| symbol.value_declaration.clone())
            .or_else(|| symbol.declarations.first().cloned())?;
        // Go getTypeForVariableLikeDeclaration（checker.go:17636-17657）首段：
        // for..in 变量取 string（右操作数索引型为类型参数/索引型时取其
        // extract-string 型），for..of 变量取右侧迭代元素型。本函数是变量类
        // 符号声明型的活动求值路径，缺此段时无注解无初始化式的 for-in 变量
        // 声明型落 any
        if let NodeData::VariableDeclaration(_) = &decl.data {
            let in_for_iteration = decl
                .parent()
                .and_then(|p| p.parent())
                .map(|grand| {
                    matches!(
                        grand.kind,
                        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                    )
                })
                .unwrap_or(false);
            if in_for_iteration {
                return self.get_widened_type_for_variable_like_declaration(&decl, true);
            }
        }
        let type_node_and_init: (Option<Arc<Node>>, Option<Arc<Node>>) = match &decl.data {
            // 函数表达式/箭头函数变量（const getProps = () => {}）：按需建型并挂 expando
            NodeData::VariableDeclaration(d)
                if d.type_node.is_none()
                    && d.initializer.as_ref().is_some_and(|init| {
                        matches!(
                            init.kind,
                            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
                        )
                    }) =>
            {
                let init = d.initializer.clone().expect("checked above");
                let base = self.get_type_of_node(&init);
                let t = self.attach_function_expando_type(symbol, base);
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertyDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertySignatureDeclaration(d) => (
                (d.type_node.kind != SyntaxKind::MissingDeclaration)
                    .then(|| Arc::clone(&d.type_node)),
                None,
            ),
            NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            // Go getTypeOfSymbol：Function 位（函数表达式/箭头函数符号）经
            // getTypeOfFuncClassEnumModule 即时驻留匿名壳，体自引读壳不重入
            //（checker.go:17482-17484、17888-17894），不依赖节点型缓存
            NodeData::FunctionExpression(_) | NodeData::ArrowFunction(_) => {
                let base = self.type_of_function_expression_symbol_shell(symbol, &decl);
                let t = self.attach_function_expando_type(symbol, base);
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            // 局部函数声明：按需建函数型（外层返回推断在体检查前消费标识符引用），
            // 与 check_function_declaration 同步挂 expando 属性（binder 期 exports 已就绪）
            NodeData::FunctionDeclaration(_) => {

                // Go getTypeOfFuncClassEnumModule 的符号型为匿名壳，
                // getSignaturesOfType 经 resolveDeclaredMembers 按
                // getSignaturesOfSymbol 聚合全部过载声明：多声名符号先建
                // 过载集（跳过检查的 lib 文件也要按需聚合），单声明回落单签名
                let base = match self.build_overload_function_type(symbol) {
                    Some(t) => t,
                    None => self.get_type_of_function_like(&decl),
                };
                let t = self.attach_function_expando_type(symbol, base);
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            // 局部类声明：按需建实例型（return new C() 在体检查前消费）；
            // 建型在途的重入是基型环路窗口，Go getTypeOfFuncClassEnumModuleWorker
            // 环路期仍产出匿名片构型、links.resolvedType 不落 any
            //（checker.go:17888-17919），此处不缓存占位，防污染符号型
            NodeData::ClassDeclaration(_) => {
                if self.class_type_resolution_stack.contains(&decl.id()) {
                    return None;
                }
                let t = self.get_type_of_class_declaration(&decl);
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            // Go checkJsxAttribute（jsx.go:895）：有初始化式按可变位置求值
            //（checkExpressionForMutableLocation——fresh 字面量拓宽为基元，
            // content="x" → string），{expr} 包装经 checkJsxExpression 展开内层
            //（jsx.go:92），无初始化式是 true 语法糖
            NodeData::JsxAttribute(d) => {
                return Some(match d.initializer.as_ref() {
                    Some(init) => {
                        let expr = match &init.data {
                            NodeData::JsxExpression(je) => je.expression.clone(),
                            _ => Some(Arc::clone(init)),
                        };
                        match expr {
                            Some(e) => self.check_expression_for_mutable_location(
                                &e,
                                crate::checker::types_alias_symbol_links::CheckMode::Normal,
                            ),
                            None => self.get_any_type(),
                        }
                    }
                    None => self.true_type(),
                });
            }
            NodeData::BindingElement(_) => {
                let t = self.binding_element_type(&decl)?;
                return Some(
                    self.widen_type_for_variable_like_declaration(Some(t), &decl, true),
                );
            }
            // Go getTypeOfVariableOrParameterOrPropertyWorker：export= 符号
            // 类型 = 表达式（或注解型）拓宽
            NodeData::ExportAssignment(d) => {
                let t = if d.type_node.kind != SyntaxKind::MissingDeclaration {
                    self.get_type_from_type_node(&d.type_node)
                } else {
                    let expr_t = self.get_type_of_node(&d.expression);
                    self.get_widened_type(&expr_t)
                };
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            // Go getTypeOfVariableOrParameterOrPropertyWorker：PropertyAssignment
            // 走 checkPropertyAssignment（checker.go:17587-17588）
            NodeData::PropertyAssignment(_) => {
                let t = self.check_property_assignment(
                    &decl,
                    crate::checker::types_alias_symbol_links::CheckMode::Normal,
                );
                return Some(t);
            }
            // Go getTypeOfVariableOrParameterOrPropertyWorker：ShorthandPropertyAssignment
            // 走 checkShorthandPropertyAssignment(decl, true, Normal)
            //（checker.go:17577-17578）——名行按 checkExpressionForMutableLocation
            // 经名字解析取外层变量声明型（{ t, x } 简写名行 T），缺臂时符号型
            // 落 get_type_of_symbol 尾 any
            NodeData::ShorthandPropertyAssignment(_) => {
                let t = self.check_shorthand_property_assignment(
                    &decl,
                    true,
                    crate::checker::types_alias_symbol_links::CheckMode::Normal,
                );
                return Some(t);
            }
            // Go getTypeOfSymbol 的 EnumMember 独立分支：getTypeOfEnumMember =
            // memoized getDeclaredTypeOfEnumMember（无 type resolution 帧；
            // 成员型经 getDeclaredTypeOfEnum 按需求值，求值器报 TS2565/
            // TS18033，值缺失成员取 createComputedEnumType 计算枚举型）
            NodeData::EnumMember(_) => {
                return Some(self.get_type_of_enum_member(symbol));
            }
            _ => return None,
        };
        if type_node_and_init.0.is_none() && type_node_and_init.1.is_none() {
            // 无注解参数：上下文定型（IIFE 实参 / 调用上下文签名），rest 参数优先走上下文
            if decl.kind == SyntaxKind::Parameter {
                // JS 无注解形参：@param 标签充当注解
                //（Go reparseHosted 的 JSDocParameterTag 通道：解析期把
                // @param 型落成 param.Type，checker.go:17655/17672 的
                // tryGetTypeFromTypeNode 因此先行命中——注解先于访问器回退）
                if let Some(t) = self.jsdoc_type_annotation(&decl) {
                    self.value_symbol_links.get_or_default(symbol).resolved_type =
                        Some(Arc::clone(&t));
                    return Some(t);
                }
                // Go getTypeForVariableLikeDeclaration（checker.go:17696-17716）：
                // set 访问器值参数无注解时取 getter 签名返回型；签名基础设施
                // 未建（get_signature_from_declaration 空壳）时按 getter 注解 →
                // 体推断的最小等价实现取读型
                if let Some(fn_node) = decl.parent()
                    && tsox_frontend::ast::node_data_generated::is_set_accessor_declaration(&fn_node)
                    && self.has_bindable_name(&fn_node)
                    && let Some(parent_symbol) = self.get_symbol_of_declaration(&fn_node)
                    && let Some(getter) =
                        self.get_declaration_of_kind(&parent_symbol, SyntaxKind::GetAccessor)
                {
                    let this_parameter = self.get_accessor_this_parameter(&fn_node);
                    if this_parameter.as_ref().is_some_and(|tp| Arc::ptr_eq(tp, &decl)) {
                        // Go：declaration == this 参数时取 getter 的 this 型
                        //（getterSignature.thisParameter 的符号型）
                        if let Some(getter_this) = self.get_accessor_this_parameter(&getter)
                            && let Some(sym) = self.get_symbol_of_declaration(&getter_this)
                        {
                            return Some(self.get_type_of_symbol(&sym));
                        }
                        return Some(self.get_any_type());
                    }
                    return Some(self.getter_signature_return_type(&getter));
                }
                let placeholder = self.error_type();
                let existing = self
                    .value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type
                    .replace(placeholder);
                // Go getWidenedTypeForVariableLikeDeclaration（checker.go:17623-17626）
                // 对 getTypeForVariableLikeDeclaration 的结果（含 17712-17718 的
                // 上下文参数型分支）统一经 widenTypeForVariableLikeDeclaration →
                // getWidenedType（checker.go:19347）收尾：上下文型携带的绑定模式
                // 隐含型成员（fresh 对象字面量，Go getWidenedTypeOfObjectLiteral
                // checker.go:19486-19512 剥 ObjectLiteral）在暴露给 relate 前先
                // 拓宽，propertiesRelatedTo 对象字面量目标规则（relater.go:4434-4443）
                // 不因未经拓宽的 fresh 字面量目标误触发
                let t = self
                    .contextual_type_of_parameter(&decl)
                    .map(|t| self.widen_type_for_variable_like_declaration(Some(t), &decl, true));
                match &t {
                    Some(t) => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type =
                            Some(Arc::clone(t));
                    }
                    None => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
                    }
                }
                if t.is_some() {
                    return t;
                }
                // Go getTypeForVariableLikeDeclaration 尾段（checker.go:17766-17769）：
                // 无注解/无初始化式/无上下文的绑定模式参数取模式隐含型
                //（[a]→[any]，{a}→{ a: any; }），经 getWidenedTypeForVariableLikeDeclaration
                //（checker.go:17623-17626）拓宽。reportErrors 侧的逐元素
                // reportImplicitAny（TS7031）由 check_parameter_implicit_any
                // 承担，此处不再重报
                if let Some(name) = decl.name()
                    && tsox_frontend::ast::node_data_generated::is_binding_pattern(&name)
                {
                    let t = self.get_type_from_binding_pattern(&name, false, false);
                    return Some(
                        self.widen_type_for_variable_like_declaration(Some(t), &decl, true),
                    );
                }
            }
            // rest 参数无注解：元素类型数组（无上下文时 any[]）
            if let NodeData::ParameterDeclaration(d) = &decl.data {
                if d.dot_dot_dot_token.is_some() {
                    let elem = self.get_any_type();
                    return Some(self.create_array_type(elem));
                }
            }
            if decl.kind == SyntaxKind::VariableDeclaration
                && decl
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::CatchClause)
            {
                return Some(if self.use_unknown_in_catch_variables {
                    self.get_unknown_type()
                } else {
                    self.get_any_type()
                });
            }
            // Go getTypeOfVariableOrParameterOrPropertyWorker（checker.go:17577-17578）：
            // 无注解无初始化式的变量声明经 getWidenedTypeForVariableLikeDeclaration
            // 委托 getTypeForVariableLikeDeclaration——noImplicitAny 下无初始化式
            // var/let 取 autoType 供控制流按赋值收窄为赋值型拓宽
            //（checker.go:17700-17713）；其余回落 any。for-in/for-of 头声明臂
            //（Go checker.go:17636-17657）由本函数入口的 for 迭代拦截段覆盖，
            // 此处不重做
            if decl.kind == SyntaxKind::VariableDeclaration {
                let in_for_iteration = decl
                    .parent()
                    .as_ref()
                    .and_then(|p| p.parent())
                    .is_some_and(|grand| {
                        matches!(
                            grand.kind,
                            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                        )
                    });
                if !in_for_iteration {
                    let t = self.get_type_for_variable_like_declaration(
                        &decl,
                        true,
                        crate::checker::types_alias_symbol_links::CheckMode::Normal,
                    );
                    let report_errors =
                        !self.is_parameter_of_context_sensitive_signature(symbol);
                    return Some(
                        self.widen_type_for_variable_like_declaration(t, &decl, report_errors),
                    );
                }
            }
            // Go getTypeOfVariableOrParameterOrPropertyWorker（checker.go:17584-17586）：
            // 无注解无初始化式的 PropertySignature 在 getTypeForVariableLikeDeclaration
            // 落 nil，widenTypeForVariableLikeDeclaration 的 nil 臂取隐式 any
            //（checker.go:19339-19351）
            if decl.kind == SyntaxKind::PropertySignature {
                let report_errors = !self.is_parameter_of_context_sensitive_signature(symbol);
                return Some(
                    self.widen_type_for_variable_like_declaration(None, &decl, report_errors),
                );
            }
            return None;
        }

        // Go getTypeForVariableLikeDeclaration（checker.go:17656-17672）：catch
        // 变量声明在初始化式推断前短路——无注解时按 useUnknownInCatchVariables
        // 取 unknown/any，初始化式不参与符号型（catch (e = 1) 的 >e 由此呈
        // unknown；有注解的 any/unknown 由下方注解臂原样返回）
        if decl.kind == SyntaxKind::VariableDeclaration
            && type_node_and_init.0.is_none()
            && decl
                .parent()
                .as_ref()
                .is_some_and(|p| p.kind == SyntaxKind::CatchClause)
        {
            return Some(if self.use_unknown_in_catch_variables {
                self.get_unknown_type()
            } else {
                self.get_any_type()
            });
        }

        let placeholder = self.error_type();
        let existing = self
            .value_symbol_links
            .get_or_default(symbol)
            .resolved_type
            .replace(Arc::clone(&placeholder));
        let result = self.with_declaring_file_context(&decl, |checker| {
            let (type_node, initializer) = match &decl.data {
                NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                NodeData::PropertyDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                NodeData::PropertySignatureDeclaration(d) => (
                    (d.type_node.kind != SyntaxKind::MissingDeclaration)
                        .then(|| Arc::clone(&d.type_node)),
                    None,
                ),
                NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                _ => (None, None),
            };
            if let Some(tn) = type_node {
                // Go getTypeForVariableLikeDeclaration（checker.go:17640-17662）：
                // 声明型存在即 addOptionalityEx(declaredType, isProperty, isOptional)，
                // isOptional = includeOptionality && isOptionalDeclaration（HasQuestionToken），
                // strictNullChecks 下 optional 声明型并 undefined（depth?: D → D | undefined）
                let declared = checker.get_type_from_type_node(&tn);
                let is_property = tsox_frontend::ast::node_data_generated::is_property_declaration(&decl)
                    && !tsox_frontend::ast::has_accessor_modifier(&decl)
                    || tsox_frontend::ast::node_data_generated::is_property_signature_declaration(&decl);
                // Go isOptionalDeclaration = HasQuestionToken（utilities.go:338-341）：
                // postfix_token 是 ?/! 合并字段，x!: string 的 ExclamationToken
                // 不可判 optional，按 token kind 判 QuestionToken（与
                // m1g_2.rs get_type_for_variable_like_declaration 同源）
                let is_optional = crate::checker::utilities_has_only_expression_initialization::is_optional_declaration(&decl);
                Some(checker.add_optionality_ex(&declared, is_property, is_optional))
            } else if decl.kind == SyntaxKind::Parameter {
                // Go getTypeOfVariableOrParameterOrPropertyWorker（checker.go:17572-17574）：
                // 参数先上下文定型，无果走 getWidenedTypeForVariableLikeDeclaration
                //（checker.go:17621-17624）——initializer 推断 + undefined 拓宽型
                // 转 any + getWidenedType 全量拓宽
                let param = Arc::clone(&decl);
                checker
                    .contextual_type_of_parameter(&param)
                    .or_else(|| {
                        checker.get_widened_type_for_variable_like_declaration(&decl, false)
                    })
            } else {
                let owner_class = match &decl.data {
                    NodeData::PropertyDeclaration(_) => decl
                        .parent()
                        .as_ref()
                        .filter(|p| {
                            matches!(
                                p.kind,
                                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                            )
                        })
                        .cloned(),
                    _ => None,
                };
                if let Some(class) = owner_class.as_ref() {
                    let this_type = checker.build_class_instance_type_with_base(class);
                    checker.this_type_stack.push(this_type);
                }
                let t = initializer.map(|init| {
                    if !checker
                        .get_combined_node_flags(&decl)
                        .intersects(NodeFlags::Constant)
                        && checker.is_null_or_undefined(&init)
                    {
                        return checker.auto_type();
                    }
                    if checker.is_empty_array_literal(&init) {
                        return checker.auto_array_type();
                    }
                    // Go checkDeclarationInitializer（checker.go:17775-17789）：
                    // getQuickTypeOfExpression 快路径，否则 checkExpressionCachedEx
                    //（flow 声明型，fresh 不常规化）；不得走 get_type_of_expression
                    //（TypeOnly 重查对 as const 产出 any）或 get_type_of_node
                    //（getRegularTypeOfExpression 把 fresh 常规化）
                    let raw = checker.get_quick_type_of_expression(&init).unwrap_or_else(
                        || {
                            checker.check_expression_cached_ex(
                                &init,
                                crate::checker::types_alias_symbol_links::CheckMode::Normal,
                            )
                        },
                    );
                    let owned =
                        checker.widen_unique_symbol_for_declaration(&decl, &raw);
                    let widened = checker
                        .widen_type_inferred_from_initializer(&decl, &owned);
                    checker.widen_initializer_type(&widened)
                });
                if owner_class.is_some() {
                    checker.this_type_stack.pop();
                }
                t
            }
        });

        let result = match (&result, &decl.data) {
            (Some(t), NodeData::ParameterDeclaration(pd)) if pd.question_token.is_some() => {
                Some(self.add_optional_undefined(Arc::clone(t)))
            }

            (Some(t), NodeData::PropertySignatureDeclaration(psd))
                if psd
                    .postfix_token
                    .as_ref()
                    .is_some_and(|tk| tk.kind == SyntaxKind::QuestionToken) =>
            {
                Some(self.get_optional_type(Arc::clone(t), true))
            }
            (Some(t), NodeData::PropertyDeclaration(pd))
                if pd
                    .postfix_token
                    .as_ref()
                    .is_some_and(|tk| tk.kind == SyntaxKind::QuestionToken) =>
            {
                Some(self.get_optional_type(Arc::clone(t), true))
            }
            _ => result,
        };
        let result = match (&result, &decl.data) {
            (Some(t), NodeData::PropertySignatureDeclaration(_))
                if !crate::checker::utilities::is_type_error(t) =>
            {
                let instantiation = self.value_symbol_links.get(symbol).map(|l| {
                    (l.instantiation_params.clone(), l.instantiation_args.clone())
                });
                match instantiation {
                    Some((params, args))
                        if !params.is_empty() && params.len() == args.len() =>
                    {
                        Some(self.substitute_infer_type_parameters(t, &params, &args))
                    }
                    _ => result,
                }
            }
            _ => result,
        };
        // 窗口内已发生真实赋值（嵌套 assignContextualParameterTypes 经
        // assign_parameter_type 写入位置型 + 初始化器回退的定型）：
        // Go 侧该赋值直接写 links.resolvedType 且后续读取返回它，
        // 窗口自身的原始位置型（如 rest 位 never）不回写覆盖
        let assigned_in_window = match self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
        {
            Some(current) => {
                !crate::checker::utilities::is_type_error(&current)
                    && !Arc::ptr_eq(&current, &placeholder)
                    && !existing.as_ref().is_some_and(|e| Arc::ptr_eq(&current, e))
            }
            None => false,
        };
        if assigned_in_window {
            return self
                .value_symbol_links
                .get(symbol)
                .and_then(|l| l.resolved_type.clone())
                .or(result);
        }
        match &result {
            Some(t) => {
                // 递归类型在构建窗口内经环断路器拿到 in-flight error：不驻留，
                // 恢复窗口前状态（占位 any 泄漏会把合并符号永久定格为 any，
                // 如 UMD 全局 + declare global const 的 typeof 链）
                if !crate::checker::utilities::is_type_error(t) {
                    self.value_symbol_links.get_or_default(symbol).resolved_type =
                        Some(Arc::clone(t));
                } else {
                    self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
                }
            }
            None => {
                self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
            }
        }
        result
    }

    /// Go getBindingElementTypeFromParentType：strictNullChecks 下模式父声明
    /// 的初始化式不可能是 undefined 时，根类型剔除 undefined 成分
    pub(crate) fn filter_binding_parent_undefined(
        &mut self,
        decl: &Arc<Node>,
        t: Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("filter_binding_parent_undefined"); 
        if !self.strict_null_checks {
            return t;
        }
        let init = match &decl.data {
            NodeData::VariableDeclaration(d) => d.initializer.clone(),
            NodeData::ParameterDeclaration(d) => d.initializer.clone(),
            _ => None,
        };
        let Some(init) = init else {
            return t;
        };
        let init_t = self.get_type_of_node(&init);
        if Self::type_may_be_undefined(&init_t) {
            return t;
        }
        if t.is_union() {
            if let Some(members) = t.types() {
                let kept: Vec<Arc<Type>> = members
                    .iter()
                    .filter(|m| !Self::type_may_be_undefined(m))
                    .cloned()
                    .collect();
                return match kept.len() {
                    0 => self.never_type(),
                    1 => kept.into_iter().next().expect("nonempty"),
                    _ => self.get_union_type(kept),
                };
            }
            return t;
        }
        if Self::type_may_be_undefined(&t) {
            return self.never_type();
        }
        t
    }

    /// Go getTypeFactsWorker 的 EQUndefined 位：any/unknown/未定类型视为可能
    fn type_may_be_undefined(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("type_may_be_undefined"); 
        if t.flags.intersects(
            TypeFlags::Any
                | TypeFlags::Unknown
                | TypeFlags::Undefined
                | TypeFlags::TypeParameter
                | TypeFlags::Index
                | TypeFlags::Conditional,
        ) {
            return true;
        }
        t.is_union()
            && t.types()
                .is_some_and(|ts| ts.iter().any(Self::type_may_be_undefined))
    }

    /// 绑定元素解析属性时，把源类型的属性符号挂为 container（显示限定名用）。
    fn link_binding_element_container(&mut self, elem: &Arc<Node>, t: &Arc<Type>, name: &str) { ::tsox_core::fntrace::enter("link_binding_element_container"); 
        let Some(sym) = t.symbol.clone() else { return };
        let member = self
            .resolve_interface_type_ex(&sym, None)
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
            .or_else(|| sym.members.entries.get(name).cloned());
        let Some(member) = member else { return };
        if let Some(s) = self.program.symbol_map().symbol_of(elem) {
            let links = self.value_symbol_links.get_or_default(s);
            if links.container_symbol.is_none() {
                links.container_symbol = Some(Arc::clone(&member));
            }
        }
        // 成员自身的限定容器 = 根类型符号（如 I），供 qualified_symbol_name 使用
        if member.parent().is_none() {
            let mlinks = self.value_symbol_links.get_or_default(&member);
            if mlinks.container_symbol.is_none() {
                mlinks.container_symbol = Some(sym);
            }
        }
    }

    /// 绑定元素类型：沿模式链上行到根声明取类型，再按属性/索引路径逐层查。
    pub(crate) fn binding_element_type(&mut self, elem: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("binding_element_type"); 
        use tsox_frontend::ast::NodeData;
        if let Some(sym) = self.program.symbol_map().symbol_of(elem)
            && let Some(t) = self
                .value_symbol_links
                .get(sym)
                .and_then(|l| l.resolved_type.clone())
            && !crate::checker::utilities::is_type_error(&t)
        {
            return Some(t);
        }
        let mut path: Vec<(Arc<Node>, BindingPathSeg)> = Vec::new();
        let mut cur = Arc::clone(elem);
        loop {
            match &cur.data {
                NodeData::BindingElement(d) => {
                    let parent_kind = cur.parent().as_ref().map(|p| p.kind);
                    if d.dot_dot_dot_token.is_some() {
                        // rest 元素：类型是模式容器的“剩余部分”，不是属性查找
                        path.push((Arc::clone(&cur), BindingPathSeg::Rest));
                    } else if parent_kind == Some(tsox_frontend::ast::SyntaxKind::ArrayBindingPattern) {
                        let pattern = cur.parent().expect("checked kind above");
                        let index = match &pattern.data {
                            NodeData::BindingPattern(bp) => bp
                                .elements
                                .iter()
                                .position(|e| Arc::ptr_eq(e, &cur)),
                            _ => None,
                        }?;
                        let renamed = d.property_name.as_ref().and_then(|n| match &n.data {
                            NodeData::NumericLiteral(num) => num.text.parse::<usize>().ok(),
                            _ => None,
                        });
                        path.push((Arc::clone(&cur), BindingPathSeg::Index(renamed.unwrap_or(index))));
                    } else {
                        let renamed = d.property_name.as_ref().and_then(|n| match &n.data {
                            NodeData::Identifier(i) => Some(i.text.clone()),
                            NodeData::StringLiteral(s) => Some(s.text.clone()),
                            NodeData::NumericLiteral(num) => Some(num.text.clone()),
                            // Go getLiteralTypeFromPropertyName（checker.go:28363）：
                            // bigint 名经 GetPropertyNameForPropertyNameNode 取文本
                            // 作 string 字面量索引
                            NodeData::BigIntLiteral(b) => Some(b.text.clone()),
                            // Go getBindingElementTypeFromParentType
                            //（checker.go:18801-18803）：计算属性名经
                            // getLiteralTypeFromPropertyName 折叠表达式常量
                            //（const itemId = 'some-id' 的 { [itemId]: x }），
                            // 落空时不得以绑定目标名当属性名
                            NodeData::ComputedPropertyName(_) => {
                                let pn = Arc::clone(n);
                                self.get_literal_property_name_text(&pn)
                            }
                            _ => None,
                        });
                        let seg = renamed.clone().or_else(|| {
                            d.name.as_ref().and_then(|n| match &n.data {
                                NodeData::Identifier(i) => Some(i.text.clone()),
                                _ => None,
                            })
                        })?;
                        path.push((Arc::clone(&cur), BindingPathSeg::Prop(seg, renamed.is_some())));
                    }
                    cur = Arc::clone(cur.parent().as_ref()?);
                }
                NodeData::BindingPattern(_) => {
                    cur = Arc::clone(cur.parent().as_ref()?);
                }
                NodeData::ParameterDeclaration(d) => {
                    let mut t = match &d.type_node {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => {
                            // Go getTypeForBindingElementParent：优先取上下文定型
                            // 已写入符号链接的参数型，未定型才落上下文重解析
                            let memoized = self
                                .program
                                .symbol_map()
                                .symbol_of(&cur)
                                .and_then(|s| {
                                    self.value_symbol_links
                                        .get(s)
                                        .and_then(|l| l.resolved_type.clone())
                                })
                                .filter(|t| !crate::checker::utilities::is_type_error(t));
                            match memoized {
                                Some(t) => t,
                                None => {
                                    let param = Arc::clone(&cur);
                                    match self.contextual_type_of_parameter(&param) {
                                        Some(t) => t,
                                        None => {
                                            if let Some(init) = &d.initializer {
                                                // Go checkDeclarationInitializer（checker.go:17775-17789）：
                                                // quick 快路径否则 checkExpressionCachedEx 全量检查
                                                let raw = self
                                                    .get_quick_type_of_expression(&init)
                                                    .unwrap_or_else(|| {
                                                        binding_element_root_initializer_type(
                                                            self,
                                                            init,
                                                        )
                                                    });
                                                let widened_literal =
                                                    self.get_widened_literal_type_for_initializer(&cur, &raw);
                                                self.binding_element_root_widen(&widened_literal)
                                            } else if tsox_frontend::ast::is_binding_pattern(&d.name) {
                                                // Go getTypeForVariableLikeDeclaration 尾段
                                                // （checker.go:17766-17770）：无注解/无上下文/无初始化式
                                                // 的绑定模式参数取模式隐含型（{ first, ...rest } →
                                                // { first: any; [x: string]: any; }）。缺此回退时本臂
                                                // 落 None，get_type_of_symbol 尾段以 any 兜底并污染首个
                                                // 引用的节点型缓存。TS7031 由 check_parameter_implicit_any
                                                // 承担，reportErrors 取 false
                                                self.get_type_from_binding_pattern(&d.name, false, false)
                                            } else {
                                                return None;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    };
                    // Go getBindingElementTypeFromParentType（checker.go:18769-18771）：
                    // 父类型为 any 时绑定元素直接取 any，rest 不构造剩余属性对象型
                    if crate::checker::mig::wc1b::is_type_any(&t) {
                        return Some(t);
                    }
                    t = self.filter_binding_parent_undefined(&cur, t);
                    let root_annotated = d.type_node.is_some();
                    for (seg_elem, seg) in path.iter().rev() {
                        t = self.binding_path_step(seg_elem, t, seg)?;
                        t = self.binding_element_initializer_tail(seg_elem, t, root_annotated);
                    }
                    return Some(t);
                }
                NodeData::VariableDeclaration(d) => {
                    let mut t = match (&d.type_node, &d.initializer) {
                        (Some(tn), _) => self.get_type_from_type_node(tn),
                        (None, Some(init)) => {
                            let memoized = self
                                .program
                                .symbol_map()
                                .symbol_of(&cur)
                                .and_then(|s| {
                                    self.value_symbol_links
                                        .get(s)
                                        .and_then(|l| l.resolved_type.clone())
                                })
                                .filter(|t| !crate::checker::utilities::is_type_error(t));
                            match memoized {
                                Some(t) => t,
                                None => {
                                    // Go getTypeForBindingElementParent→
                                    // getTypeOfVariableLikeDeclaration 初始化式臂→
                                    // checkDeclarationInitializer（checker.go:17775-17789）：
                                    // quick 快路径否则 checkExpressionCachedEx 全量检查
                                    //（对象字面量经 checkObjectLiteralExpression 建型，
                                    // 上下文型/patternForType 机制在此激活）；get_type_of_node
                                    // 走 compute_type_of_node 原始建型绕开该机制
                                    let raw = self
                                        .get_quick_type_of_expression(&init)
                                        .unwrap_or_else(|| {
                                            binding_element_root_initializer_type(
                                                self,
                                                init,
                                            )
                                        });
                                    // Go widenTypeInferredFromInitializer：解构根类型按
                                    // 初始化式拓宽（fresh 字面量 6 → number）
                                    let widened_literal =
                                        self.get_widened_literal_type_for_initializer(&cur, &raw);
                                    self.binding_element_root_widen(&widened_literal)
                                }
                            }
                        }
                        // for-in/of 头声明无初始化式：迭代类型即根类型
                        //（Go getTypeForVariableLikeDeclaration 的 ForIn/ForOf 分支）
                        (None, None) => self.initial_type_of_declaration(&cur)?,
                    };
                    // Go getBindingElementTypeFromParentType（checker.go:18769-18771）：
                    // 父类型为 any 时绑定元素直接取 any，rest 不构造剩余属性对象型
                    if crate::checker::mig::wc1b::is_type_any(&t) {
                        return Some(t);
                    }
                    t = self.filter_binding_parent_undefined(&cur, t);
                    let root_annotated = d.type_node.is_some();
                    for (seg_elem, seg) in path.iter().rev() {
                        t = self.binding_path_step(seg_elem, t, seg)?;
                        t = self.binding_element_initializer_tail(seg_elem, t, root_annotated);
                    }
                    return Some(t);
                }
                _ => return None,
            }
        }
    }

    // Go getBindingElementTypeFromParentType 尾段（checker.go:18839-18850）：
    // 带默认值的绑定元素路径步进后按初始化式定型——根声明有注解时
    // strictNullChecks 下初始化式非 undefined 事实即剥 undefined 成分；
    // 无注解时与初始化式类型取联合并按声明拓宽。初始化式型沿 Go
    // checkDeclarationInitializer 的 getQuickTypeOfExpression 快路径取；
    // 非 quick 且未缓存时不在此强制全量检查——解构初始化式内兄弟引用的
    // 判别收窄（kind === "a" 收窄 payload）依赖语句级检查期建立的条件流，
    // Go 流图 bind 期成型故任意检查点皆收窄，本侧语句级才成型，符号型
    // 解析期提前检查会以未收窄并集产出 TS2339 并缓存节点型
    //（dependentDestructuringBindingElementInitializer）。初始化式经
    // checkBindingElement 的可赋值性检查保证 ⊆ 路径型的 nonUndefined，
    // Go 联合（checker.go:18850）在该前提下化简即 nonUndefinedType
    //（controlFlowInitializedDestructuringVariables：+a 非 quick 且语句级
    // check_expression 语法遍历不落节点缓存，放弃臂保 undefined 成分即呈
    // number | undefined vs number），故初始化式形态恒不产出 undefined 时
    // 放弃臂即取 nonUndefinedType，其余形态维持原样待缓存命中路径补准
    // Go getApplicableIndexInfoForName（checker.go:20140-20145）：晚期绑定名
    // 走 esSymbol 适用索引查询，其余按名字面量型取适用索引信息
    fn binding_element_initializer_tail(
        &mut self,
        elem: &Arc<Node>,
        t: Arc<Type>,
        root_annotated: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("binding_element_initializer_tail");
        use tsox_frontend::ast::NodeData;
        let NodeData::BindingElement(_) = &elem.data else {
            return t;
        };
        let Some(init) = tsox_frontend::ast::mig::m3b::initializer(elem) else {
            return t;
        };
        let init_t = match self.get_quick_type_of_expression(init) {
            Some(t) => t,
            None => match self
                .type_node_links
                .get(init)
                .and_then(|l| l.resolved_type.clone())
            {
                Some(t) => t,
                None => {
                    if self.initializer_type_excludes_undefined(init) {
                        return self.get_non_undefined_type(&t);
                    }
                    return t;
                }
            },
        };
        if root_annotated {
            if self.strict_null_checks && !self.has_type_facts(&init_t, TypeFacts::IS_UNDEFINED) {
                return self.get_non_undefined_type(&t);
            }
            return t;
        }
        let non_undefined = self.get_non_undefined_type(&t);
        let union = self.get_union_type_ex(
            vec![non_undefined, init_t],
            crate::checker::exports_union_reduction::UnionReduction::Subtype,
            None,
            None,
        );
        self.widen_type_inferred_from_initializer(elem, &union)
    }

    // 初始化式语法形态恒不产出 undefined 值的窄集：前缀数值/逻辑化
    //（+a/-a/!a/~a 的 ToNumber/Boolean 语义）、比较与相等、成员/instanceof、
    // 模板串（插值恒字符串化）、对象/数组字面量、正则、函数。放弃臂仅对
    // 这些形态按 Go 联合的可赋值化简剥路径型 undefined；可能 undefined 的
    // 形态（标识符/调用/属性访问/条件/await/undefined 字面量等）维持原样，
    // 由语句级检查后的缓存命中路径补准
    fn initializer_type_excludes_undefined(&self, expr: &Arc<Node>) -> bool {
        use tsox_frontend::ast::{NodeData, SyntaxKind};
        let expr = Self::skip_parentheses(expr);
        match expr.kind {
            SyntaxKind::PrefixUnaryExpression => {
                matches!(&expr.data,
                    NodeData::PrefixUnaryExpression(d)
                        if matches!(
                            d.operator,
                            SyntaxKind::PlusToken
                                | SyntaxKind::MinusToken
                                | SyntaxKind::ExclamationToken
                                | SyntaxKind::TildeToken
                        ))
            }
            SyntaxKind::BinaryExpression => matches!(&expr.data,
                NodeData::BinaryExpression(d)
                    if matches!(
                        d.operator_token.kind,
                        SyntaxKind::EqualsEqualsToken
                            | SyntaxKind::ExclamationEqualsToken
                            | SyntaxKind::EqualsEqualsEqualsToken
                            | SyntaxKind::ExclamationEqualsEqualsToken
                            | SyntaxKind::LessThanToken
                            | SyntaxKind::LessThanEqualsToken
                            | SyntaxKind::GreaterThanToken
                            | SyntaxKind::GreaterThanEqualsToken
                            | SyntaxKind::InKeyword
                            | SyntaxKind::InstanceOfKeyword
                    )),
            SyntaxKind::TemplateExpression
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::ArrowFunction
            | SyntaxKind::FunctionExpression => true,
            _ => false,
        }
    }

    // 所在调用带显式类型实参时按实参实例化上下文签名（Go inferSignature：
    // 显式实参直接固定映射；错误实参落 error 型，参数位显示 any）
    fn substitute_explicit_call_type_args(
        &mut self,
        call: &Arc<Node>,
        sig: &Arc<Signature>,
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("substitute_explicit_call_type_args"); 
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let ta = match &call.data {
            NodeData::CallExpression(d) => d.type_arguments.as_ref(),
            NodeData::NewExpression(d) => d.type_arguments.as_ref(),
            _ => None,
        };
        let Some(ta) = ta else {
            return Arc::clone(sig);
        };
        if ta.len() != sig.type_parameters.len() {
            return Arc::clone(sig);
        }
        let args: Vec<Arc<Type>> = ta
            .iter()
            .map(|t| self.get_type_from_type_node(t))
            .collect();
        self.get_signature_instantiation(sig, &args, &[])
    }

    pub(crate) fn contextual_type_of_parameter(
        &mut self,
        param: &Arc<Node>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("contextual_type_of_parameter"); 
        use tsox_frontend::ast::NodeData;
        let host = param.parent()?;
        let (host_kind_ok, host_type_params, host_params) = match &host.data {
            NodeData::FunctionExpression(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            NodeData::ArrowFunction(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            NodeData::FunctionDeclaration(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            NodeData::MethodDeclaration(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            _ => (false, None, None),
        };
        if !host_kind_ok || host_type_params.is_some() {
            return None;
        }
        let host_params = host_params?;
        let param_index = host_params.iter().position(|p| Arc::ptr_eq(p, param));
        let param_index = param_index?;
        // 对象字面量方法：上下文签名取字面量上下文类型的同名属性（Go getContextualSignatureForObjectLiteralMethod）
        if matches!(host.data, NodeData::MethodDeclaration(_)) {
            let obj_lit = host.parent()?;
            if obj_lit.kind != SyntaxKind::ObjectLiteralExpression {
                return None;
            }
            let ctx = self.get_contextual_type(&obj_lit, ContextFlags::None)?;
            let method_name = match &host.data {
                NodeData::MethodDeclaration(d) => self.get_property_name_from_node(&d.name),
                _ => String::new(),
            };
            let prop_type = self.get_type_of_property_of_contextual_type(&ctx, &method_name)?;
            let sigs =
                self.get_signatures_of_type(&prop_type, crate::checker::SignatureKind::Call);
            // Go getContextualSignatureForObjectLiteralMethod：getSingleSignature
            //（恰一个调用签名才用，多重载不给上下文参数型）
            if sigs.len() != 1 {
                return None;
            }
            let sig = sigs[0].clone();
            let is_rest = matches!(&param.data, NodeData::ParameterDeclaration(pd) if pd.dot_dot_dot_token.is_some());
            let is_this_param = param_index == 0
                && matches!(&param.data, NodeData::ParameterDeclaration(pd)
                    if matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this")
                        || pd.name.kind == SyntaxKind::ThisKeyword);
            return self
                .contextual_param_type_at(&sig, &host_params, param_index, param, is_rest, is_this_param)
                .into();
        }
        let mut call = host.parent()?;
        let mut in_parens = false;
        while call.kind == tsox_frontend::ast::SyntaxKind::ParenthesizedExpression {
            in_parens = true;
            call = call.parent()?;
        }
        let call_ctx = match &call.data {
            NodeData::CallExpression(d) => {
                // IIFE：参数类型取对应实参的 widened 类型（Go GetImmediatelyInvokedFunctionExpression 分支）
                let callee_is_host = (!in_parens && Arc::ptr_eq(&d.expression, &host))
                    || (in_parens
                        && host
                            .parent()
                            .as_ref()
                            .is_some_and(|p| Arc::ptr_eq(p, &d.expression)));
                if callee_is_host {
                    let call_id = call.id();
                    if !self.resolving_contextual_calls.insert(call_id) {
                        return None;
                    }
                    let args: Vec<Arc<Node>> = d.arguments.iter().cloned().collect();
                    let result = (|| {
                        if param_index < args.len() {
                            let raw = self.get_type_of_node(&args[param_index]);
                            let widened = self.widen_argument_type_deep(&raw);
                            return Some(widened);
                        }
                        if matches!(&param.data, NodeData::ParameterDeclaration(pd) if pd.initializer.is_some())
                        {
                            return None;
                        }
                        Some(self.undefined_type())
                    })();
                    self.resolving_contextual_calls.remove(&call_id);
                    return result;
                }
                self.get_contextual_type_for_argument(&call, &host)
            }
            NodeData::NewExpression(_) => self.get_contextual_type_for_argument(&call, &host),
            _ => None,
        };
        let ctx = call_ctx.or_else(|| self.get_contextual_type(&host, ContextFlags::None))?;
        // Go getContextuallyTypedParameterType → getContextualSignature（联合
        // 逐成分、全同才用；多重载且 noImplicitAny 关闭时无上下文签名，
        // 参数走隐式 any）
        let gated = self.contextual_signature_of_type(&ctx, &host)?;
        let sig = self.substitute_explicit_call_type_args(&call, &gated);
        let is_rest = matches!(&param.data, NodeData::ParameterDeclaration(pd) if pd.dot_dot_dot_token.is_some());
        let is_this_param = param_index == 0
            && matches!(&param.data, NodeData::ParameterDeclaration(pd)
                if matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this")
                    || pd.name.kind == SyntaxKind::ThisKeyword);
        self.contextual_param_type_at(&sig, &host_params, param_index, param, is_rest, is_this_param)
    }

    fn widen_argument_type_deep(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("widen_argument_type_deep"); 
        let widened = self.get_widened_type(t);
        let TypeData::Object(o) = &widened.data else {
            return widened;
        };
        if widened.symbol.is_some() {
            return widened;
        }
        let mut new_props: Vec<Arc<Symbol>> = Vec::with_capacity(o.structured.properties.len());
        let mut new_members = SymbolTable::new();
        let mut changed = false;
        for p in &o.structured.properties {
            let pt = self.get_type_of_symbol(p);
            let wpt = self.widen_argument_type_deep(&pt);
            if Arc::ptr_eq(&pt, &wpt) {
                new_members.insert(p.name.clone(), Arc::clone(p));
                new_props.push(Arc::clone(p));
            } else {
                let np = Arc::new(Symbol::new(p.flags, p.name.clone()));
                self.value_symbol_links.insert(
                    &np,
                    ValueSymbolLinks {
                        resolved_type: Some(wpt),
                        ..Default::default()
                    },
                );
                new_members.insert(np.name.clone(), Arc::clone(&np));
                new_props.push(np);
                changed = true;
            }
        }
        if !changed {
            return widened;
        }
        let mut rebuilt = Type::new(
            widened.flags,
            TypeData::Object(ObjectTypeData { node: None,
                structured: StructuredTypeData {
                    members: new_members,
                    properties: new_props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
        rebuilt.object_flags = widened.object_flags;
        Arc::new(rebuilt)
    }

    /// 元素的 computed 属性名节点（仅显式 property_name 位）
    pub(crate) fn binding_element_computed_property_name(elem: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binding_element_computed_property_name");
        let tsox_frontend::ast::NodeData::BindingElement(be) = &elem.data else {
            return None;
        };
        let pn = be.property_name.as_ref()?;
        (pn.kind == tsox_frontend::ast::SyntaxKind::ComputedPropertyName)
            .then(|| Arc::clone(pn))
    }

    /// Go getBindingElementTypeFromParentType 对象非 rest 臂经 getIndexedAccessTypeEx
    /// （checker.go:18802-18804）取元素型：属性未命中时按名种别查适用索引签名
    /// （数字名先 number 后 string 索引，字符串名仅 string 索引），命中取值型，
    /// 无诊断；两侧皆未命中才走调用方的 TS2339 报错路径
    fn binding_element_index_signature_type(
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("binding_element_index_signature_type");
        let structured = t.as_structured()?;
        let numeric = !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit());
        let by_key = |want_number: bool| {
            structured
                .index_infos
                .iter()
                .find(|info| {
                    info.key_type
                        .as_ref()
                        .map(|k| {
                            k.flags.contains(if want_number {
                                TypeFlags::Number
                            } else {
                                TypeFlags::String
                            })
                        })
                        .unwrap_or(false)
                })
                .and_then(|info| info.value_type.clone())
        };
        if numeric {
            by_key(true).or_else(|| by_key(false))
        } else {
            by_key(false)
        }
    }

    /// Go widenTypeInferredFromInitializer（checker.go:17864-17869）=
    /// getWidenedLiteralTypeForInitializer（checker.go:17876-17882），主体止于
    /// getWidenedLiteralType（checker.go:27017-27032）：联合根按成分仅加宽
    /// 字面量类（union 臂 mapType，Literal 归约重建、恒等保原联合），对象
    /// 字面量成分（fresh {} 等）原样保留。full widen（widen_object_literal_type
    /// 逐成分剥 ObjectLiteral 旗标，checker.go:19508-19509 newAnonymousType）
    /// 会令联合属性构造（checker.go:22807-22810 无 spread 空 object literal
    /// 成分补 undefined 臂）退化为 ReadPartial（getPropertyOfUnionOrIntersectionType
    /// checker.go:22670-22676 过滤），解构属性缺省误报 TS2339 且符号落 any。
    /// 非联合根维持直接对象字面量/演化数组的既有经验拓宽路径
    fn binding_element_root_widen(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Union) {
            return Arc::clone(t);
        }
        let regularized = self.get_regular_type_of_literal_type(t);
        self.widen_initializer_type(&regularized)
    }

    fn binding_path_step(
        &mut self,
        elem: &Arc<Node>,
        t: Arc<Type>,
        seg: &BindingPathSeg,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("binding_path_step"); 
        // 带默认值的元素（a = expr）：类型由默认值提供，属性缺失不诊断
        //（Go getTypeForVariableLikeDeclaration 的 hasInitializer 分支）
        let elem_has_initializer = matches!(
            &elem.data,
            NodeData::BindingElement(d) if d.initializer.is_some()
        );
        let diagnostics_allowed = !self.in_ambient_declaration_context()
            && !elem_has_initializer
            && !t.flags.intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Never)
            && !crate::checker::utilities::is_type_error(&t);
        if let BindingPathSeg::Rest = seg {
            return Some(self.rest_element_type(elem, &t));
        }
        if let BindingPathSeg::Prop(name, renamed) = seg {
            // Go isTypeUsableAsPropertyName：computed 名表达式解析失败（error 型）
            // 时属性查找不可用，错误已由名表达式自身报告
            if let Some(pn) = Self::binding_element_computed_property_name(elem)
                && let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &pn.data
            {
                let name_expr_type = self.get_type_of_node(&cd.expression);
                if crate::checker::utilities::is_type_error(&name_expr_type) {
                    return None;
                }
                if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&name_expr_type)
                {
                    // 计算名表达式取型落空（解出非字面量成分）时回退路径构造期的
                    // 字面量名（binding_element_type 经 get_literal_property_name_text
                    // 折叠 const itemId = 'some-id'）
                    let mut derived = crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(&name_expr_type);
                    if derived.is_empty() {
                        derived = name.clone();
                    }
                    if *renamed {
                        self.link_binding_element_container(elem, &t, &derived);
                    }
                    // Go getBindingElementTypeFromParentType（checker.go:18801-18803）
                    // →getIndexedAccessTypeEx→getPropertyTypeForIndexType 尾段
                    //（checker.go:28702）getApplicableIndexInfoForName
                    //（checker.go:20140）按名取适用索引信息。联合父型（带默认值的
                    // 绑定元素经 getUnionIndexInfos checker.go:14229-14246 合并索引
                    // 信息，任一成分缺索引即联合无索引信息，属性通道落空）逐成分走
                    // getApplicableIndexInfoForName 同款查询：命中成分并集值类型、
                    // 缺索引成分跳过（crashInGetTextOfComputedPropertyName：
                    // { [s: string]: AB; } | {} 的 [itemId] 位呈 AB）
                    let result = if t.flags.contains(TypeFlags::Union) {
                        let constituents = t.types().map(|ts| ts.to_vec()).unwrap_or_default();
                        let mut values: Vec<Arc<Type>> = Vec::new();
                        for c in &constituents {
                            if let Some(info) =
                                self.get_applicable_index_info_for_name(c, &derived)
                                && let Some(v) = &info.value_type
                            {
                                values.push(Arc::clone(v));
                            }
                        }
                        (!values.is_empty()).then(|| self.get_union_type(values))
                    } else {
                        self.get_type_of_property_of_type(&t, &derived)
                    };
                    if result.is_none() && diagnostics_allowed {
                        let display = self.boxed_declared_type_for_display(&t);
                        let type_str = self.type_to_string(&display);
                        let name_node = Self::binding_element_name_node(elem)
                            .unwrap_or_else(|| Arc::clone(elem));
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            name_node.loc,
                            tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                            vec![derived, type_str],
                        ));
                    }
                    return result;
                }
                if name_expr_type.flags.intersects(TypeFlags::TypeParameter) {
                    let constraint = self
                        .get_constraint_of_type_parameter(&name_expr_type)
                        .or_else(|| self.get_base_constraint_of_type(&name_expr_type));
                    let index_like = constraint.as_ref().is_some_and(|c| {
                        c.flags.intersects(
                            TypeFlags::String | TypeFlags::Number | TypeFlags::ESSymbol,
                        )
                    });
                    if index_like {
                        return Some(self.get_indexed_access_type(&t, &name_expr_type));
                    }
                }
                // computed string 名走索引签名通道（Go getIndexedAccessTypeEx）：
                // 无匹配 string 索引签名报 TS2537，命中取值类型
                if name_expr_type.flags.intersects(TypeFlags::String) {
                    let structured = t.as_structured();
                    let match_info = structured.and_then(|s| {
                        s.index_infos.iter().find(|info| {
                            info.key_type
                                .as_ref()
                                .map(|k| k.flags.intersects(TypeFlags::String))
                                .unwrap_or(false)
                        })
                    });
                    if let Some(info) = match_info {
                        return info.value_type.clone();
                    }
                    if diagnostics_allowed {
                        let display = self.boxed_declared_type_for_display(&t);
                        let type_str = self.type_to_string(&display);
                        let key_str = self.type_to_string(&name_expr_type);
                        // Go 报错锚定计算名内的表达式节点
                        let anchor_loc = cd.expression.loc;
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            anchor_loc,
                            tsox_core::diagnostics::messages_generated::
                                TYPE_0_HAS_NO_MATCHING_INDEX_SIGNATURE_FOR_TYPE_1,
                            vec![type_str, key_str],
                        ));
                    }
                    return None;
                }
                if name_expr_type.flags.intersects(TypeFlags::Number) {
                    let structured = t.as_structured();
                    let match_info = structured.and_then(|s| {
                        s.index_infos.iter().find(|info| {
                            info.key_type
                                .as_ref()
                                .map(|k| k.flags.intersects(TypeFlags::Number))
                                .unwrap_or(false)
                        })
                    });
                    if let Some(info) = match_info {
                        return info.value_type.clone();
                    }
                }
                if diagnostics_allowed {
                    let type_str = if cd.expression.kind == SyntaxKind::BigIntLiteral {
                        "bigint".to_string()
                    } else {
                        self.type_to_string(&name_expr_type)
                    };
                    let anchor_loc = cd.expression.loc;
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        anchor_loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                        vec![type_str],
                    ));
                }
                return Some(self.get_any_type());
            }
            if *renamed {
                self.link_binding_element_container(elem, &t, name);
            }
            if !Self::binding_element_computed_property_name(elem).is_some() {
                if let Some(prop) = self.get_property_of_type(&t, name) {
                    self.mark_property_as_referenced(&prop, None);
                }
            }
            let result = match self.get_type_of_property_of_type(&t, name) {
                Some(t) => Some(t),
                None => Self::binding_element_index_signature_type(&t, name),
            };
            if result.is_none() && diagnostics_allowed {
                let name_node = Self::binding_element_name_node(elem)
                    .unwrap_or_else(|| Arc::clone(elem));
                // Go checkIndexedAccess（checker.go:28801）：索引节点为 bigint
                // 字面量时不报属性缺失，改报 TS2538 且类型参数固定 "bigint"
                if name_node.kind == tsox_frontend::ast::SyntaxKind::BigIntLiteral {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        name_node.loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                        vec!["bigint".to_string()],
                    ));
                    return None;
                }
                let display = self.boxed_declared_type_for_display(&t);
                let type_str = self.type_to_string(&display);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name_node.loc,
                    tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                    vec![name.clone(), type_str],
                ));
            }
            return result;
        }
        let BindingPathSeg::Index(index) = seg else {
            return None;
        };
        let index = *index;
        if let TypeData::Tuple(tuple) = &t.data {
            return tuple
                .element_infos
                .get(index)
                .and_then(|info| info.type_.clone());
        }
        if t.object_flags.contains(crate::checker::types::ObjectFlags::Tuple)
            && let Some(structured) = t.as_structured()
            && let Some(info) = structured.index_infos.first()
        {
            return info.value_type.clone();
        }
        if let Some(elem) = self.get_array_element_type_of(&t) {
            return Some(elem);
        }
        if let Some(prop) = self.get_type_of_property_of_type(&t, &index.to_string()) {
            return Some(prop);
        }
        if diagnostics_allowed {
            let pattern = elem
                .parent()
                .filter(|p| p.kind == tsox_frontend::ast::SyntaxKind::ArrayBindingPattern);
            return Some(self.check_iterated_type_or_element_type(
                crate::checker::checker_iteration::IterationUse::Destructuring,
                &t,
                pattern.as_ref(),
            ));
        }
        None
    }

    /// rest 绑定元素类型：对象模式取排除同模式其他绑定名后的剩余属性对象；
    /// 数组模式取剩余元素数组（Go getRestTypeAtObject / getRestTypeAtPosition）。
    pub(crate) fn rest_element_type(&mut self, elem: &Arc<Node>, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("rest_element_type"); 
        let is_array_pattern = elem
            .parent()
            .is_some_and(|p| p.kind == tsox_frontend::ast::SyntaxKind::ArrayBindingPattern);
        if is_array_pattern {
            if t.is_union()
                && let Some(members) = t.types()
            {
                let mapped: Vec<Arc<Type>> = members
                    .iter()
                    .map(|m| self.rest_element_type(elem, m))
                    .collect();
                return self.get_union_type(mapped);
            }
            if self.is_tuple_type(t) {
                let index = elem
                    .parent()
                    .and_then(|pattern| Self::binding_element_index(&pattern, elem))
                    .unwrap_or(0);
                let mut rest: Vec<Arc<Type>> = Vec::new();
                let mut i = index;
                while let Some(el) = self.get_tuple_element_type(t, i) {
                    rest.push(el);
                    i += 1;
                }
                let element_t = if rest.is_empty() {
                    self.never_type()
                } else {
                    self.get_union_type(rest)
                };
                return self.create_array_type(element_t);
            }
            if let Some(el) = self.get_array_element_type_of(t) {
                return self.create_array_type(el);
            }
            return Arc::clone(t);
        }
        let source = self.remove_nullable_constituents(t);
        if source.flags.contains(TypeFlags::Never) {
            return Arc::new(Type::new(
                TypeFlags::Object,
                TypeData::Object(ObjectTypeData::default()),
            ));
        }
        if source.is_union()
            && let Some(members) = source.types()
        {
            let mapped: Vec<Arc<Type>> = members
                .iter()
                .map(|m| self.rest_element_type(elem, m))
                .collect();
            return self.get_union_type(mapped);
        }
        let excluded = Self::pattern_excluded_property_names(elem);
        let source_props = self.get_properties_of_type(&source);
        let kept: Vec<Arc<Symbol>> = source_props
            .into_iter()
            .filter(|p| !excluded.contains(&p.name))
            .filter(|p| self.is_spreadable_property(p))
            .collect();
        let index_infos = self.get_index_infos_of_type(&source);
        let mut members = crate::checker::types::SymbolTable::default();
        for p in &kept {
            members.insert(p.name.clone(), Arc::clone(p));
        }
        let mut rebuilt = Type::new(
            TypeFlags::Object,
            TypeData::Object(ObjectTypeData { node: None,
                structured: crate::checker::types_impl_chunk::StructuredTypeData {
                    members,
                    properties: kept,
                    index_infos,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
        rebuilt.object_flags = source.object_flags | crate::checker::types::ObjectFlags::ObjectRestType;
        Arc::new(rebuilt)
    }

    fn remove_nullable_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_nullable_constituents"); 
        if !t.is_union() {
            if t.flags.intersects(TYPE_FLAGS_NULLABLE) {
                return self.never_type();
            }
            return Arc::clone(t);
        }
        let Some(members) = t.types() else {
            return Arc::clone(t);
        };
        let kept: Vec<Arc<Type>> = members
            .iter()
            .filter(|m| !m.flags.intersects(TYPE_FLAGS_NULLABLE))
            .cloned()
            .collect();
        if kept.len() == members.len() {
            return Arc::clone(t);
        }
        match kept.len() {
            0 => self.never_type(),
            1 => Arc::clone(&kept[0]),
            _ => self.get_union_type(kept),
        }
    }

    pub(crate) fn is_spreadable_property(&self, prop: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_spreadable_property"); 
        let no_private_ident = !prop.declarations.iter().any(|d| {
            matches!(
                &d.data,
                NodeData::PropertyDeclaration(pd) if pd.name.kind == tsox_frontend::ast::SyntaxKind::PrivateIdentifier
            ) || Self::is_method_or_accessor(d)
                && d.name().is_some_and(|n| n.kind == tsox_frontend::ast::SyntaxKind::PrivateIdentifier)
        });
        let not_accessor_like = !prop.flags.intersects(
            tsox_frontend::ast::SymbolFlags::Method
                | tsox_frontend::ast::SymbolFlags::GetAccessor
                | tsox_frontend::ast::SymbolFlags::SetAccessor,
        );
        if no_private_ident && not_accessor_like {
            return true;
        }
        !prop
            .declarations
            .iter()
            .any(|d| d.parent().is_some_and(|p| tsox_frontend::ast::is_class_like(&p)))
    }

    fn is_method_or_accessor(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_method_or_accessor"); 
        use tsox_frontend::ast::SyntaxKind as K;
        matches!(node.kind, K::MethodDeclaration | K::MethodSignature | K::GetAccessor | K::SetAccessor)
    }

    /// 同一对象模式中其他元素绑定的属性名（rest 类型需排除这些属性）
    fn pattern_excluded_property_names(elem: &Arc<Node>) -> Vec<String> { ::tsox_core::fntrace::enter("pattern_excluded_property_names"); 
        let mut out = Vec::new();
        let Some(pattern) = elem
            .parent()
            .filter(|p| p.kind == tsox_frontend::ast::SyntaxKind::ObjectBindingPattern)
        else {
            return out;
        };
        let NodeData::BindingPattern(bp) = &pattern.data else {
            return out;
        };
        for e in bp.elements.nodes.iter() {
            if Arc::ptr_eq(e, elem) {
                continue;
            }
            let NodeData::BindingElement(be) = &e.data else {
                continue;
            };
            if let Some(pn) = &be.property_name {
                match &pn.data {
                    NodeData::Identifier(i) => out.push(i.text.clone()),
                    NodeData::StringLiteral(s) => out.push(s.text.clone()),
                    NodeData::NumericLiteral(n) => out.push(n.text.clone()),
                    _ => {}
                }
            } else if let Some(n) = &be.name
                && n.kind == tsox_frontend::ast::SyntaxKind::Identifier
            {
                out.push(n.text().to_string());
            }
        }
        out
    }

    pub(crate) fn with_declaring_file_context<T>(
        &mut self,
        decl: &Arc<Node>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T { ::tsox_core::fntrace::enter("with_declaring_file_context"); 
        let saved_file = self.current_file.take();
        let saved_id = self.current_file_id;
        let saved_symbol = self.current_file_symbol.take();
        let mut pushed = 0usize;
        if let Some(file) = self.get_source_file_of_node(decl) {
            self.current_file = Some(Arc::clone(&file));
            self.current_file_id = file.node.id();
            self.current_file_symbol = self.program.symbol_map().symbol_of(&file.node).cloned();

            let mut chain: Vec<Arc<Node>> = Vec::new();
            let mut cur = decl.parent();
            while let Some(n) = cur {
                if matches!(
                    n.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::ModuleDeclaration
                        | SyntaxKind::Block
                        | SyntaxKind::CatchClause
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::MethodSignature
                        | SyntaxKind::CallSignature
                        | SyntaxKind::ConstructSignature
                        | SyntaxKind::FunctionType
                        | SyntaxKind::ConstructorType
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::InterfaceDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                        | SyntaxKind::TypeAliasDeclaration
                        | SyntaxKind::MappedType
                        | SyntaxKind::EnumDeclaration
                ) {
                    chain.push(Arc::clone(&n));
                    if n.kind == SyntaxKind::SourceFile {
                        break;
                    }
                }
                cur = n.parent();
            }
            for scope in chain.iter().rev() {
                self.push_scope(scope);
                pushed += 1;
            }
        }
        let result = f(self);
        for _ in 0..pushed {
            self.pop_scope();
        }
        self.current_file = saved_file;
        self.current_file_id = saved_id;
        self.current_file_symbol = saved_symbol;
        result
    }

    pub(crate) fn get_type_of_merged_namespace_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_merged_namespace_symbol"); 
        // Go getTypeOfFuncClassEnumModule（checker.go:17890-17894）：值侧类型
        // 缓存于 valueSymbolLinks.resolvedType，与 declaredTypeLinks 的声明型
        // （getDeclaredTypeOfEnum 枚举成员联合）分槽
        if let Some(cached) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
        {
            return cached;
        }

        // Go getTypeOfFuncClassEnumModuleWorker：enum+namespace 合并符号的值类型
        // 是单一匿名对象，枚举成员与 namespace 导出共同构成属性
        let has_enum = symbol.flags.intersects(SymbolFlags::ENUM);
        let value_type = if has_enum {
            self.resolve_enum_value_type(symbol)
        } else {
            self.get_value_type_of_symbol(symbol)
        };

        let ns_type = self.resolve_namespace_type(symbol);

        let (call_sigs, construct_sigs) = match &value_type.data {
            TypeData::Object(obj) => {
                let cs = obj.structured.call_signatures().to_vec();
                let xs = obj.structured.construct_signatures().to_vec();
                (cs, xs)
            }
            _ => (Vec::new(), Vec::new()),
        };
        let merged = if has_enum {
            let (Some(enum_obj), Some(ns_obj)) = (value_type.as_object(), ns_type.as_object())
            else {
                return value_type;
            };
            let mut structured = StructuredTypeData::default();
            structured.members = enum_obj.structured.members.clone();
            structured.properties = enum_obj.structured.properties.clone();
            structured.index_infos = ns_obj.structured.index_infos.clone();
            for (name, sym) in ns_obj.structured.members.entries.iter() {
                if !structured.members.entries.contains_key(name) {
                    structured.members.insert(name.clone(), Arc::clone(sym));
                }
            }
            for prop in &ns_obj.structured.properties {
                if !structured
                    .properties
                    .iter()
                    .any(|p| p.name == prop.name)
                {
                    structured.properties.push(Arc::clone(prop));
                }
            }
            Arc::new(Type {
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
            })
        } else if call_sigs.is_empty() && construct_sigs.is_empty() {
            ns_type
        } else {
            let ns_obj = match &ns_type.data {
                TypeData::Object(obj) => obj,
                _ => {
                    self.value_symbol_links
                        .get_or_default(symbol)
                        .resolved_type = Some(Arc::clone(&value_type));
                    return value_type;
                }
            };
            let ns_structured = &ns_obj.structured;
            let mut structured = StructuredTypeData::default();
            structured.members = ns_structured.members.clone();
            structured.properties = ns_structured.properties.clone();
            structured.index_infos = ns_structured.index_infos.clone();

            let existing_sigs = ns_structured.signatures.clone();
            let existing_call_count = ns_structured.call_signature_count;
            structured.call_signature_count = call_sigs.len() + existing_call_count;
            structured.signatures = call_sigs;
            structured
                .signatures
                .extend(existing_sigs[..existing_call_count].to_vec());
            structured.signatures.extend(construct_sigs);
            structured
                .signatures
                .extend(existing_sigs[existing_call_count..].to_vec());
            Arc::new(Type {
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
            })
        };

        self.value_symbol_links
            .get_or_default(symbol)
            .resolved_type = Some(Arc::clone(&merged));
        merged
    }

    // Go getReturnTypeOfSignature(getter 签名) 的最小等价：注解优先，
    // 无注解有体走体返回推断，均无落 any
    fn getter_signature_return_type(&mut self, getter: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("getter_signature_return_type");
        let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &getter.data else {
            return self.get_any_type();
        };
        if let Some(tn) = &gd.type_node {
            return self.get_type_from_type_node(tn);
        }
        if let Some(body) = gd.body.clone() {
            return self.infer_method_return_type(getter, &Some(body));
        }
        self.get_any_type()
    }
}

// Go checkDeclarationInitializer（checker.go:17775-17789）：quick 快路径否则
// checkExpressionCachedEx 全量检查（checker.go:7977-7995 缓存优先+流栈暂存），
// 非对象/数组字面量形态读节点型缓存落 compute_type_of_node 原始建型时，
// 箭头/IIFE 的返回型被急切解析，自引根符号的在飞解析产物为空对象型而非环
// 静默 any（blockScopedVariablesUseBeforeDef foo16 `(() => a)()` 应 any）。
// 全量检查窗内 check_identifier 的流收窄链会重入解析中根符号的解构元素解析
//（Go 体检查/返回型惰性延后到帧出栈后，本侧即时成型），push_type_resolution
// 环检测会毒化当前帧致 pop 报 TS7022——以 rt_infer_boundary_marks 圈出本检查
// 窗：窗内对窗下符号帧的环重入不毒化，落 report_circularity_error 静默 any
//（对齐 Go strict:false reportCircularityError 静默位 checker.go:19935-19957）
fn binding_element_root_initializer_type(
    checker: &mut Checker,
    init: &Arc<Node>,
) -> Arc<Type> {
    match init.kind {
        SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression => checker
            .check_expression_cached_ex(
                init,
                crate::checker::types_alias_symbol_links::CheckMode::Normal,
            ),
        _ => {
            let mark = checker.type_resolution_stack.len();
            checker.rt_infer_boundary_marks.push(mark);
            let t = checker.check_expression_cached_ex(
                init,
                crate::checker::types_alias_symbol_links::CheckMode::Normal,
            );
            checker.rt_infer_boundary_marks.pop();
            t
        }
    }
}
