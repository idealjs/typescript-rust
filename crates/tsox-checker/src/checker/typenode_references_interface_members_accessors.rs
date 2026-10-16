#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn add_index_signature_member(
        &mut self,
        member: &Arc<Node>,
        index_infos: &mut Vec<Arc<crate::checker::IndexInfo>>,
    ) { ::tsox_core::fntrace::enter("add_index_signature_member"); 
        let NodeData::IndexSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let mut key_type = None;
        let value_type;
        if let Some(param) = data.parameters.iter().next() {
            if let NodeData::ParameterDeclaration(pd) = &param.data {
                key_type = pd
                    .type_node
                    .as_ref()
                    .map(|t| self.get_type_from_type_node(t));
            }
        }
        value_type = Some(self.get_type_from_type_node(&data.type_node));
        // 解析窗内急切求值的索引值型若为别名重入断路 error（引用了进行中的
        // 别名），按成员节点记账；Go 索引信息惰性解析在窗外取值时别名已缓存
        // 声明型（unionTypeWithIndexedLiteralType：Idx 的 [string]: U 在 U 体
        // 求值窗内只得断路 error，毒随并集成员驻留声明型缓存，上下文属性投影
        // 读到 error 致初值字面量误拓宽）。真错误（未知名等）不记账
        if let Some(vt) = value_type.as_ref()
            && crate::checker::utilities::is_type_error(vt)
            && !self.type_resolution_stack.is_empty()
            && let NodeData::TypeReferenceNode(tr) = &data.type_node.data
            && let Some(sym) = self.resolve_entity_name(
                &tr.type_name,
                SymbolFlags::TYPE,
                true,
                false,
                None,
            )
        {
            let sym = self.get_merged_symbol(&sym);
            if sym.flags.contains(SymbolFlags::TypeAlias)
                && self.is_resolving(
                    Arc::as_ptr(&sym),
                    crate::checker::checker::TypeResolutionProperty::DeclaredType,
                )
            {
                self.transient_alias_index_values
                    .insert(member.id() as usize, Arc::clone(&sym));
            }
        }
        let is_readonly = member
            .modifiers()
            .as_ref()
            .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
        // Go resolveStructuredTypeMembers：同键索引签名去重（重复声明由
        // 2374 检查报错，类型只保留一个）
        if let Some(k) = &key_type
            && index_infos
                .iter()
                .any(|info| info.key_type.as_ref().is_some_and(|e| e.id == k.id))
        {
            return;
        }
        index_infos.push(Arc::new(crate::checker::IndexInfo {
            key_type,
            value_type,
            is_readonly,
            declaration: Some(Arc::clone(member)),
            index_symbol: None,
            components: Vec::new(),
        }));
    }

    /// Go getTypeOfAccessors（checker.go:19604-19618）成员级解析序：getter 注解 →
    /// setter 参数注解 → getter 体返回推断（加宽）；均无则 any。get/set 对按
    /// 同 staticness 同名取（Go 经符号声明 GetDeclarationOfKind，符号按
    /// staticness 分表；本侧语法级等价过滤见 worker）
    pub(crate) fn resolve_accessor_pair_type(
        &mut self,
        accessor: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_accessor_pair_type");
        self.resolve_accessor_pair_type_with_symbol(accessor, None)
    }

    pub(crate) fn resolve_accessor_pair_type_with_symbol(
        &mut self,
        accessor: &Arc<Node>,
        symbol: Option<&Arc<Symbol>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_accessor_pair_type_with_symbol");
        self.push_scope(accessor);
        let result = self.resolve_accessor_pair_type_worker(accessor, symbol);
        self.pop_scope();
        result
    }

    fn resolve_accessor_pair_type_worker(
        &mut self,
        accessor: &Arc<Node>,
        symbol: Option<&Arc<Symbol>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_accessor_pair_type_worker");
        // TypeLiteral/MappedType 容器的访问器是动态名匿名符号（__computed，
        // binder declare_symbol 动态名臂不入容器成员表），类容器扫描于此
        // 不可达；Go getTypeOfAccessors（checker.go:19611-19612）恒经符号
        // 声明取对——type literal 内 get/set 各为独立匿名符号，声明天然
        // 只含自身 kind，注解序即正确
        if accessor
            .parent()
            .is_some_and(|p| matches!(p.kind, SyntaxKind::TypeLiteral | SyntaxKind::MappedType))
        {
            let bound = self.program.symbol_map().symbol_of(accessor);
            let getter = bound
                .and_then(|s| tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(s, SyntaxKind::GetAccessor));
            let setter = bound
                .and_then(|s| tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(s, SyntaxKind::SetAccessor));
            if let Some(g) = getter
                && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &g.data
                && let Some(tn) = &gd.type_node
            {
                return self.get_type_from_type_node(tn);
            }
            if let Some(s) = setter
                && let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) = &s.data
                && let Some(param) = sd.parameters.iter().next()
                && let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data
                && let Some(tn) = &pd.type_node
            {
                return self.get_type_from_type_node(tn);
            }
            return self.get_any_type();
        }
        // 对象字面量容器的访问器（binder 符号按成员声明入表）：Go
        // getTypeOfAccessors 按符号声明取 get/set 对（checker.go:19611-19612）
        // 容器无关；无注解有体时以空聚合语法判定落 voidType（Go
        // getReturnTypeFromBody types 空分支 checker.go:21374-21387），有值
        // return 走体推断（Go getTypeFromBody checker.go:19623-19626，
        // gettersAndSettersTypesAgree o1 落 number），无体维持 any
        let literal_parent = accessor.parent();
        if literal_parent
            .as_deref()
            .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
        {
            let props: &[Arc<Node>] = match literal_parent.as_deref().map(|p| &p.data) {
                Some(tsox_frontend::ast::NodeData::ObjectLiteralExpression(d)) => {
                    d.properties.nodes.as_slice()
                }
                _ => &[],
            };
            let getter = props.iter().find(|m| {
                m.kind == SyntaxKind::GetAccessor && Self::member_names_match(m, accessor)
            });
            let setter = props.iter().find(|m| {
                m.kind == SyntaxKind::SetAccessor && Self::member_names_match(m, accessor)
            });
            if let Some(g) = getter
                && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &g.data
                && let Some(tn) = &gd.type_node
            {
                return self.get_type_from_type_node(tn);
            }
            if let Some(s) = setter
                && let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) = &s.data
                && let Some(param) = sd.parameters.iter().next()
                && let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data
                && let Some(tn) = &pd.type_node
            {
                return self.get_type_from_type_node(tn);
            }
            if let Some(g) = getter
                && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &g.data
                && let Some(body) = &gd.body
            {
                if Self::body_has_no_value_return(body) {
                    return self.void_type();
                }
                // return 表达式型经 get_type_of_node 读 check 期 type_node_links
                // 驻留缓存（体已随 check_object_literal 的成员走查驻留，wc1d_2.rs
                // check_node_deferred → check_class_accessor_member check_statement），
                // 不重产体期诊断；环场景 call_return_query_depth 豁免界承接（重入
                // errorType 断路，同类容器臂语义，Go pushTypeResolution 环检出
                // 不标记帧 checker.go:13431）
                let saved_depth = self.call_return_query_depth;
                self.call_return_query_depth = self.call_return_query_depth.saturating_add(1);
                let inferred = self.infer_method_return_type(g, &Some(Arc::clone(body)));
                self.call_return_query_depth = saved_depth;
                return self.get_widened_type(&inferred);
            }
            return self.get_any_type();
        }
        // Go declareClassMember（binder.go:431-437）按 staticness 分表：static 成员
        // 进类符号 Exports、实例进 Members，get/set 只能取同 staticness 的同名
        // 声明；无 staticness 过滤时 static get x 会误配先声明的实例 getter
        //（cloduleWithDuplicateMember1：static get x 推断成实例 getter 的
        // number 而非自身体推的 string）。接口成员非类容器，class_members_of
        // 为空表——成员型经 resolve_deferred_accessor_member_types 预解析
        //（typenode_references_interface_members_properties.rs:328）；对象
        // 字面量访问器按声明符号回退取配对（见下方符号回退臂）
        let class = accessor.parent();
        let accessor_is_static = accessor.has_syntactic_modifier(ModifierFlags::Static);
        let getter = class.as_ref().and_then(|cls| {
            Self::class_members_of(cls).iter().find(|m| {
                m.kind == SyntaxKind::GetAccessor
                    && Self::member_names_match(m, accessor)
                    && m.has_syntactic_modifier(ModifierFlags::Static) == accessor_is_static
            }).cloned()
        });
        let setter = class.as_ref().and_then(|cls| {
            Self::class_members_of(cls).iter().find(|m| {
                m.kind == SyntaxKind::SetAccessor
                    && Self::member_names_match(m, accessor)
                    && m.has_syntactic_modifier(ModifierFlags::Static) == accessor_is_static
            }).cloned()
        });
        // Go getTypeOfAccessors（checker.go:19611-19612）：getter/setter 按
        // 符号声明取（GetDeclarationOfKind(symbol, Kind)）。对象字面量访问器
        // 非类容器、class_members_of 为空表，缺配对臂时体推断永不运行、
        // 访问器型驻留 any（.types 呈 self: any 而非惰性 typeof a）——按
        // 声明符号回退找配对；类容器命中臂（staticness 过滤）不变
        let getter = getter.or_else(|| {
            symbol.and_then(|s| self.get_declaration_of_kind(s, SyntaxKind::GetAccessor))
        });
        // Go 经合并符号恒可见 get/set 两侧；本仓 binder 对非字面量计算名成员
        // 走 bindAnonymousDeclaration 每声明独立 __computed 符号
        // （symbols_binder_2.rs:175-193，get [k]/set [k] 分属两符号）——符号
        // 回退臂对单侧符号仍缺配对，再按容器成员节点扫描补臂
        //（computedPropertiesWithSetterAssignment：set [k] 声明名行误呈 setter
        // 参数注解 Iterable<string> 而非 getter 型 Set<string>）
        let getter = getter.or_else(|| Self::accessor_container_sibling(accessor, SyntaxKind::GetAccessor));
        let setter = setter.or_else(|| {
            symbol.and_then(|s| self.get_declaration_of_kind(s, SyntaxKind::SetAccessor))
        });
        let setter = setter.or_else(|| Self::accessor_container_sibling(accessor, SyntaxKind::SetAccessor));
        if let Some(g) = getter.as_ref()
            && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &g.data
            && let Some(tn) = &gd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        if let Some(s) = setter.as_ref()
            && let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) = &s.data
            && let Some(param) = sd.parameters.iter().next()
            && let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        if let Some(g) = getter.as_ref()
            && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &g.data
            && let Some(body) = &gd.body
        {
            let accessor = g.clone();
            // Go getTypeOfAccessors 的体推断同步执行（非 checkNodeDeferred）：
            // 体期 this.x 重入命中本符号帧计环（TS7023），不开 rt_infer 豁免界
            let saved_depth = self.call_return_query_depth;
            self.call_return_query_depth = self.call_return_query_depth.saturating_add(1);
            let inferred = self.infer_method_return_type(&accessor, &Some(Arc::clone(body)));
            self.call_return_query_depth = saved_depth;
            return self.get_widened_type(&inferred);
        }
        self.get_any_type()
    }

    /// Go getReturnTypeFromBody 空聚合分支（checker.go:21374-21387）的语法
    /// 判定：体内无带表达式的 return——嵌套函数体的 return 归属内层，不下探
    pub(crate) fn body_has_no_value_return(body: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("body_has_no_value_return");
        !Self::contains_value_return(body)
    }

    fn contains_value_return(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::ReturnStatement {
            return matches!(
                &node.data,
                tsox_frontend::ast::NodeData::ReturnStatement(r) if r.expression.is_some()
            );
        }
        if matches!(
            node.kind,
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::ArrowFunction
                | SyntaxKind::Constructor
        ) {
            return false;
        }
        tsox_frontend::ast::node_data_generated::for_each_child(node, |c| {
            Self::contains_value_return(c)
        })
    }

    /// Go 判定同一成员：标识符按文本、well-known 计算名按内部名
    fn member_names_match(a: &Arc<Node>, b: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("member_names_match");
        let key = |n: &Arc<Node>| -> Option<String> {
            let name = n.name()?;
            match name.kind {
                SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                    Some(name.text().to_string())
                }
                SyntaxKind::ComputedPropertyName => {
                    let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &name.data
                    else {
                        return None;
                    };
                    // Go binder lateBindMember：const 变量（unique symbol）计算名
                    // 绑定为变量名的 late-bound 符号名，get/set 同名合并。知名
                    // 符号（Symbol.iterator 等）优先按内部名；其余 Identifier 内
                    // 层按变量文本对齐 late-bound 名（member_names_match 的调用
                    // 场景为同容器成员对，两侧同形，文本比对即符号名比对）
                    if let Some(well_known) =
                        crate::binder::symbols_binder_4::well_known_symbol_member_name(&cd.expression)
                    {
                        return Some(well_known);
                    }
                    if cd.expression.kind == SyntaxKind::Identifier {
                        return Some(cd.expression.text().to_string());
                    }
                    None
                }
                _ => None,
            }
        };
        match (key(a), key(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// 非类容器（接口/类型字面量）的访问器配对节点扫描。binder 对非字面量
    /// 计算名成员每声明发独立匿名 __computed 符号（不合并），符号级
    /// GetDeclarationOfKind 取不到对侧声明；Go 同形态经 late-bound 符号合并
    /// （getLateBoundSymbol）使配对恒可见——此处按容器成员节点同文匹配补齐
    pub(crate) fn accessor_container_sibling(
        accessor: &Arc<Node>,
        kind: SyntaxKind,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("accessor_container_sibling");
        let parent = accessor.parent()?;
        let members: &[Arc<Node>] = match &parent.data {
            NodeData::InterfaceDeclaration(d) => &d.members.nodes,
            NodeData::TypeLiteralNode(d) => &d.members.nodes,
            _ => return None,
        };
        let own_name = accessor.name()?;
        members
            .iter()
            .find(|m| {
                m.kind == kind
                    && m.name().is_some_and(|n| {
                        Self::member_names_match(m, accessor)
                            || (n.kind == SyntaxKind::ComputedPropertyName
                                && own_name.kind == SyntaxKind::ComputedPropertyName
                                && n.text() == own_name.text())
                    })
            })
            .cloned()
    }

    pub(crate) fn add_get_accessor_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
        deferred: &mut Vec<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("add_get_accessor_member"); 
        let NodeData::GetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
        if is_static_modifier(&data.modifiers) {
            return;
        }
        let name = self.member_declaration_name(&data.name);
        // Go declareSymbolEx：__missing 名成员不入容器符号表
        if name == crate::binder::symbols::INTERNAL_SYMBOL_NAME_MISSING {
            return;
        }
        if name.is_empty() {
            return;
        }

        match symbol_table.get(&name).cloned() {
            Some(existing) => {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).flags |= SymbolFlags::GetAccessor;
                    (*existing_mut).declarations.push(Arc::clone(member));
                }
            }
            None => {
                // Go bindPropertyWorker（binder.go:701-705）：get accessor 符号仅
                // GetAccessor 位（无 Property）——与 set 臂同理，Property 位会使
                // getWriteTypeOfSymbol 走 Property 臂回读型而弃 setter 写型
                let mut symbol = Symbol::new(
                    SymbolFlags::GetAccessor,
                    name.clone(),
                );
                symbol.declarations.push(Arc::clone(member));
                symbol.value_declaration = Some(Arc::clone(member));
                let symbol = Arc::new(symbol);
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
            }
        }
        deferred.push(Arc::clone(member));
    }

    pub(crate) fn add_set_accessor_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
        deferred: &mut Vec<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("add_set_accessor_member"); 
        let NodeData::SetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
        if is_static_modifier(&data.modifiers) {
            return;
        }
        let name = self.member_declaration_name(&data.name);
        // Go declareSymbolEx：__missing 名成员不入容器符号表
        if name == crate::binder::symbols::INTERNAL_SYMBOL_NAME_MISSING {
            return;
        }
        if name.is_empty() {
            return;
        }

        match symbol_table.get(&name).cloned() {
            Some(existing) => {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).flags |= SymbolFlags::SetAccessor;
                    (*existing_mut).declarations.push(Arc::clone(member));
                }
            }
            None => {
                // Go bindPropertyWorker（binder.go:703-705）：set accessor 符号仅
                // SetAccessor 位（无 Property）——Property 位会把
                // getWriteTypeOfSymbol（checker.go:17409-17413）导入 Property 臂
                // 取 getter 读型，set 写型（setter 参数注解）失配
                // （computedPropertiesWithSetterAssignment：foo.k = ['foo'] 以
                // Set<string> 为目标报 TS2322）
                let mut symbol = Symbol::new(
                    SymbolFlags::SetAccessor,
                    name.clone(),
                );
                symbol.declarations.push(Arc::clone(member));
                symbol.value_declaration = Some(Arc::clone(member));
                let symbol = Arc::new(symbol);
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
            }
        }
        deferred.push(Arc::clone(member));
    }

    // Go resolveObjectTypeMembers：accessor 成员型在全成员入表后解析，
    // 体推断期的 this.x 自引用（多态 this 约束即本壳）才可见
    pub(crate) fn resolve_deferred_accessor_member_types(
        &mut self,
        deferred: &[Arc<Node>],
        symbol_table: &SymbolTable,
    ) { ::tsox_core::fntrace::enter("resolve_deferred_accessor_member_types"); 
        for member in deferred {
            let name = match &member.data {
                NodeData::GetAccessorDeclaration(d) => self.member_declaration_name(&d.name),
                NodeData::SetAccessorDeclaration(d) => self.member_declaration_name(&d.name),
                _ => continue,
            };
            let Some(sym) = symbol_table.get(&name) else {
                continue;
            };
            if self
                .value_symbol_links
                .get(sym)
                .and_then(|l| l.resolved_type.clone())
                .is_some()
            {
                continue;
            }
            // Go resolveTypeMembers 容器型构造不解析成员型：成员型经
            // getTypeOfSymbol 惰性定型。守卫两翼：本成员的 Type 帧在栈（同符号
            // 重入），或任一 DeclaredType 帧在栈（容器自身构建中——accessor 注解
            // 引用同容器，mappedTypeCircularReferenceInAccessor：get bestFriend():
            // User → User 成员表重建再解析 bestFriend，构建期注解求值必然重入
            // 半成品 declared type 误报 TS2502）。跳过后待构建完成，惰性期
            // get_type_of_symbol 首调时 declared type 已缓存，注解求值成功
            if self.is_resolving(
                Arc::as_ptr(sym) as *const Symbol,
                crate::checker::TypeResolutionProperty::Type,
            ) || self.type_resolution_stack.iter().any(|entry| {
                entry.property == crate::checker::TypeResolutionProperty::DeclaredType
            }) {
                continue;
            }
            let in_type_literal = member
                .parent()
                .is_some_and(|p| p.kind == SyntaxKind::TypeLiteral);
            // 类/接口成员经符号定型（accessor 臂 push/pop 帧闭环，体推断重入
            // 计环报 TS7023）；TypeLiteral 成员维持字面量局部解析
            let prop_type = if in_type_literal {
                self.accessor_member_prop_type(member, &name, true)
            } else {
                self.get_type_of_symbol(sym)
            };
            self.value_symbol_links
                .get_or_default(sym)
                .resolved_type = Some(prop_type);
        }
    }

    fn accessor_member_prop_type(
        &mut self,
        member: &Arc<Node>,
        name: &str,
        in_type_literal: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("accessor_member_prop_type"); 
        if in_type_literal {
            self.type_literal_accessor_member_type(member, name)
        } else {
            self.resolve_accessor_pair_type(member)
        }
    }

    pub(crate) fn add_call_signature_member(
        &mut self,
        member: &Arc<Node>,
        call_signatures: &mut Vec<Arc<Signature>>,
    ) { ::tsox_core::fntrace::enter("add_call_signature_member"); 
        let NodeData::CallSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let suppress = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"));
        if suppress {
            self.push_ts2304_suppression();
        }

        self.push_scope(member);
        let return_type = match data.type_node.as_ref() {
            Some(tn) => self.get_type_from_type_node(tn),
            None => self.get_any_type(),
        };
        let sig = self.build_signature_from_function_like_type_node(
            &data.parameters,
            return_type,
            false,
            None,
            Some(Arc::clone(member)),
        );
        self.pop_scope();
        if suppress {
            self.pop_ts2304_suppression();
        }
        call_signatures.push(sig);
    }

    pub(crate) fn add_construct_signature_member(
        &mut self,
        member: &Arc<Node>,
        construct_signatures: &mut Vec<Arc<Signature>>,
    ) { ::tsox_core::fntrace::enter("add_construct_signature_member"); 
        let NodeData::ConstructSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let suppress = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"));
        if suppress {
            self.push_ts2304_suppression();
        }

        self.push_scope(member);
        let return_type = match data.type_node.as_ref() {
            Some(tn) => self.get_type_from_type_node(tn),
            None => self.get_any_type(),
        };
        let sig = self.build_signature_from_function_like_type_node(
            &data.parameters,
            return_type,
            true,
            None,
            Some(Arc::clone(member)),
        );
        self.pop_scope();
        if suppress {
            self.pop_ts2304_suppression();
        }
        construct_signatures.push(sig);
    }

    pub(crate) fn add_constructor_properties(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) { ::tsox_core::fntrace::enter("add_constructor_properties"); 
        let NodeData::ConstructorDeclaration(data) = &member.data else {
            unreachable!()
        };
        for param in data.parameters.iter() {
            let NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.name.kind != SyntaxKind::Identifier {
                continue;
            }
            let Some(modifiers) = &pd.modifiers else {
                continue;
            };
            if !modifiers.modifier_flags.intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            ) {
                continue;
            }
            let name = pd.name.text().to_string();
            if name.is_empty() || symbol_table.get(&name).is_some() {
                continue;
            }
            let prop_type = match pd.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => match pd.initializer.as_ref() {
                    Some(init) => self.get_type_of_node(init),
                    None => self.get_any_type(),
                },
            };
            // Go bindParameter+getTypeForVariableLikeDeclaration：可选参数属性
            // 符号带 Optional 位，strictNullChecks 下属性类型补 | undefined
            let is_optional = pd.question_token.is_some();
            let prop_type = if is_optional && self.strict_null_checks {
                self.get_union_type(vec![prop_type, self.undefined_type()])
            } else {
                prop_type
            };
            let mut flags = SymbolFlags::Property;
            if is_optional {
                flags |= SymbolFlags::Optional;
            }
            let mut symbol = Symbol::new(flags, name.clone());

            symbol.declarations.push(Arc::clone(param));
            symbol.value_declaration = Some(Arc::clone(param));
            if modifiers.modifier_flags.contains(ModifierFlags::Readonly) {
                symbol.check_flags |= CheckFlags::Readonly;
            }
            let symbol = Arc::new(symbol);
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name, Arc::clone(&symbol));
            props.push(symbol);
        }
    }
}
