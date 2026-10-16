#![allow(unused_imports)]

use crate::checker::exports::*;

impl Checker {
    pub fn get_unknown_signature(&self) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("get_unknown_signature"); 
        self.unknown_signature.get().cloned()
    }

    pub fn get_name_type_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_name_type_of_symbol"); 
        self.value_symbol_links
            .get(symbol)
            .and_then(|links| links.name_type.clone())
    }

    pub fn get_global_symbol(
        &self,
        name: &str,
        _meaning: SymbolFlags,
        _diagnostic: Option<&Message>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_symbol"); 
        self.globals.get(name).cloned()
    }

    pub fn get_global_symbol_by_name(
        &self,
        name: &str,
        _meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_symbol_by_name"); 
        self.globals.get(name).cloned()
    }

    pub fn get_global_type_by_name(&mut self, name: &str) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_global_type_by_name"); 
        let symbol: Arc<Symbol> = self.globals.get(name).cloned()?;
        if !symbol
            .flags
            .intersects(SymbolFlags::Class.union(SymbolFlags::Interface))
        {
            return None;
        }
        Some(self.get_declared_type_of_symbol(&symbol))
    }

    pub fn get_symbol_by_name(&self, name: &str, _meaning: SymbolFlags) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_by_name"); 
        self.globals.get(name).cloned()
    }

    pub fn get_merged_symbol_public(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_merged_symbol_public"); 
        Some(Arc::clone(symbol))
    }

    pub fn try_find_ambient_module(&self, module_name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("try_find_ambient_module");
        // Go tryFindAmbientModule（checker.go:16459-16470）：相对说明符不查
        // 全局；ambient 模块符号名带引号入 globals，命中经合并符号返回
        //（含增广）
        if tsox_core::tspath::is_external_module_name_relative(module_name) {
            return None;
        }
        let quoted = format!("\"{}\"", module_name);
        let symbol = self.globals.get(&quoted)?.clone();
        Some(self.get_merged_symbol(&symbol))
    }

    pub fn get_immediate_aliased_symbol(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_immediate_aliased_symbol"); 
        if let Some(target) = self
            .alias_symbol_links
            .get(symbol)
            .and_then(|l| l.immediate_target.clone())
        {
            return Some(target);
        }
        let node = self.get_declaration_of_alias_symbol(symbol)?;
        let target = self.get_target_of_alias_declaration(&node);
        self.alias_symbol_links.get_or_default(symbol).immediate_target = target.clone();
        target
    }

    pub fn get_type_only_alias_declaration(&self, _symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_only_alias_declaration"); 
        None
    }

    pub fn resolve_external_module_name(
        &self,
        module_specifier: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_external_module_name");
        // Go resolveExternalModuleName（checker.go:15942）→
        // resolveExternalModuleNameWorker：仅接受 StringLiteralLike 说明符，
        // 经 resolveExternalModule 落到文件/ambient 模块符号
        if !tsox_frontend::ast::is_string_literal_like(module_specifier) {
            return None;
        }
        let spec = module_specifier.text().trim_matches(['"', '\'', '`']);
        // Go 经程序模块解析结果定位目标文件；本仓快速索引走查命中不了
        // package.json typings 指向的包入口（@raymondfeng/pkg2 →
        // dist/index.d.ts），未命中按同一程序解析兜底
        self.resolve_module_file_symbol(spec).or_else(|| {
            self.resolve_module_file_by_program_resolution(module_specifier, spec)
        })
    }

    pub fn get_declared_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_symbol"); 
        // Class/Interface 维持既有实现：create_declared_type_of_class_or_interface
        // 递归回本函数，走 try_get 分派会成环。
        // Go tryGetDeclaredTypeOfSymbol（checker.go:25072-25073）Class|Interface
        // 同一接取 getDeclaredTypeOfClassOrInterface，kind 按 Class 位选
        //（checker.go:18331）——合并符号（class + 同名 interface）的声明型是
        // class 实例型（build_class_instance_type_with_base 已把同名 interface
        // 声明成员并入），故 Class 位先于 Interface 位判定，声明型恒与
        // is_class_instance_side 的 get_declared_type_of_class_or_interface
        // 收敛同一对象（Go nodebuilderimpl.go:349-352 实例/静态侧渲染分野）
        if symbol.flags.contains(tsox_frontend::ast::SymbolFlags::Class) {
            // class 声明节点从 declarations 按形态取：合并符号的
            // valueDeclaration 可能是先声明的 namespace 件
            //（binder merge 仅在为空时设置）
            let class_node = symbol
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        tsox_frontend::ast::SyntaxKind::ClassDeclaration
                            | tsox_frontend::ast::SyntaxKind::ClassExpression
                    )
                })
                .cloned();
            if let Some(d) = class_node {
                let class_type = self.build_class_instance_type_with_base(&d);
                if symbol.flags.intersects(tsox_frontend::ast::SymbolFlags::Interface) {
                    // Go mergeSymbol 原地并符号（checker.go:14957-14966）：增广后
                    // 类符号唯一，同名 interface 成员并入同一声明型——build 节点
                    // 缓存壳上原地并，纯符号视图与合并符号视图收敛同一对象，
                    // isClassInstanceSide（nodebuilderimpl.go:349-352）按
                    // getDeclaredTypeOfClassOrInterface(t.symbol) 同一对象判定
                    // 实例侧渲染裸名而非 typeof
                    let iface_type = self.resolve_interface_type_ex(symbol, None);
                    let already_merged = match (class_type.as_structured(), iface_type.as_structured())
                    {
                        (Some(c), Some(i)) => i
                            .members
                            .entries
                            .keys()
                            .all(|k| c.members.entries.contains_key(k)),
                        _ => false,
                    };
                    if !already_merged {
                        self.merge_base_into_shell(&class_type, &iface_type);
                    }
                }
                return class_type;
            }
            return self.resolve_interface_type_ex(symbol, None);
        }
        if symbol.flags.contains(tsox_frontend::ast::SymbolFlags::Interface) {
            return self.resolve_interface_type_ex(symbol, None);
        }
        // Go getDeclaredTypeOfSymbol：其余形态经 tryGetDeclaredTypeOfSymbol
        // 分派（TypeParameter/TypeAlias/Enum/EnumMember/Alias），无接取 errorType
        self.try_get_declared_type_of_symbol(symbol)
            .unwrap_or_else(|| self.error_type())
    }

    pub fn get_resolution_mode_override(
        &mut self,
        attrs: &Arc<Node>,
        report_errors: bool,
    ) -> Option<ResolutionMode> { ::tsox_core::fntrace::enter("get_resolution_mode_override"); 
        use tsox_frontend::ast::SyntaxKind;
        let data = match &attrs.data {
            tsox_frontend::ast::NodeData::ImportAttributes(d) => d,
            _ => return None,
        };
        let is_assertions = data.token == SyntaxKind::AssertKeyword;
        if data.attributes.len() != 1 {
            if report_errors {
                let msg = if is_assertions {
                    tsox_core::diagnostics::messages_generated::
                        TYPE_IMPORT_ASSERTIONS_SHOULD_HAVE_EXACTLY_ONE_KEY_RESOLUTION_MODE_WITH_VALUE_IMPORT_OR_REQUIRE
                } else {
                    tsox_core::diagnostics::messages_generated::
                        TYPE_IMPORT_ATTRIBUTES_SHOULD_HAVE_EXACTLY_ONE_KEY_RESOLUTION_MODE_WITH_VALUE_IMPORT_OR_REQUIRE
                };
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    attrs.loc,
                    msg,
                    Vec::new(),
                ));
            }
            return None;
        }
        let elem = &data.attributes.nodes[0];
        let (name, value) = match &elem.data {
            tsox_frontend::ast::NodeData::ImportAttribute(d) => (d.name.clone(), d.value.clone()),
            _ => return None,
        };
        if !matches!(name.kind, SyntaxKind::StringLiteral) {
            return None;
        }
        if name.text() != "resolution-mode" {
            if report_errors {
                let msg = if is_assertions {
                    tsox_core::diagnostics::messages_generated::
                        X_RESOLUTION_MODE_IS_THE_ONLY_VALID_KEY_FOR_TYPE_IMPORT_ASSERTIONS
                } else {
                    tsox_core::diagnostics::messages_generated::
                        X_RESOLUTION_MODE_IS_THE_ONLY_VALID_KEY_FOR_TYPE_IMPORT_ATTRIBUTES
                };
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name.loc,
                    msg,
                    Vec::new(),
                ));
            }
            return None;
        }
        if !matches!(value.kind, SyntaxKind::StringLiteral) {
            return None;
        }
        match value.text() {
            "import" => Some(ResolutionMode::ESNext),
            "require" => Some(ResolutionMode::CommonJS),
            _ => {
                if report_errors {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        value.loc,
                        tsox_core::diagnostics::messages_generated::
                            X_RESOLUTION_MODE_SHOULD_BE_EITHER_REQUIRE_OR_IMPORT,
                        Vec::new(),
                    ));
                }
                None
            }
        }
    }

    pub fn type_predicate_to_string(&self, _t: &TypePredicate) -> String { ::tsox_core::fntrace::enter("type_predicate_to_string"); 
        String::new()
    }

    pub fn get_expanded_parameters(
        &self,
        _signature: &Arc<Signature>,
        _skip_union_expanding: bool,
    ) -> Vec<Vec<Arc<Symbol>>> { ::tsox_core::fntrace::enter("get_expanded_parameters"); 
        Vec::new()
    }

    pub fn get_resolved_signature(&mut self, node: &Arc<Node>) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("get_resolved_signature"); 
        let cached = self
            .signature_links
            .get(node)
            .and_then(|l| l.resolved_signature.clone());
        if let Some(cached) = &cached
            && !self.signature_is_resolving_signature(cached)
        {
            return Some(Arc::clone(cached));
        }
        // Go checker.go:8915-8928：call 解析前把 resolutionStart 压到当前
        // type-resolution 栈底，签名/实参解析对窗口外符号型的重入（如属性
        // 初始化器经 call 实参反依赖符号自身）不构成环，仅 cached 为空时重设
        let save_resolution_start = self.resolution_start;
        if cached.is_none() {
            self.resolution_start = self.type_resolution_stack.len();
        }
        let marker = self.resolving_signature();
        self.signature_links
            .get_or_default(node)
            .resolved_signature = Some(Arc::clone(&marker));
        let result = self.resolve_signature(node, CheckMode::Normal);
        self.resolution_start = save_resolution_start;
        if result.as_ref().is_some_and(|r| Arc::ptr_eq(r, &marker)) {
            return result;
        }
        let links = self.signature_links.get_or_default(node);
        let finalized = match links.resolved_signature.as_ref() {
            Some(l) if !Arc::ptr_eq(l, &marker) => links.resolved_signature.clone(),
            _ => result,
        };
        links.resolved_signature = finalized.clone();
        finalized
    }

    pub fn resolve_signature(
        &mut self,
        node: &Arc<Node>,
        check_mode: CheckMode,
    ) -> Option<Arc<Signature>> { ::tsox_core::fntrace::enter("resolve_signature"); 
        match node.kind {
            SyntaxKind::CallExpression => self.resolve_call_expression(node, None, check_mode),
            SyntaxKind::NewExpression => self.resolve_new_expression(node, None, check_mode),
            SyntaxKind::TaggedTemplateExpression => {
                self.resolve_tagged_template_expression(node, None, check_mode)
            }
            SyntaxKind::Decorator => self.resolve_decorator(node, None, check_mode),
            SyntaxKind::JsxOpeningFragment
            | SyntaxKind::JsxOpeningElement
            | SyntaxKind::JsxSelfClosingElement => {
                self.resolve_jsx_opening_like_element(node, None, check_mode.bits())
            }
            SyntaxKind::BinaryExpression => {
                self.resolve_instanceof_expression(node, None, check_mode)
            }
            _ => None,
        }
    }

    pub fn get_contextual_type_for_argument_at_index(
        &self,
        _node: &Arc<Node>,
        _arg_index: usize,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_contextual_type_for_argument_at_index"); 
        None
    }

    pub fn get_index_signatures_at_location(&self, _node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_index_signatures_at_location"); 
        Vec::new()
    }

    pub fn get_resolved_symbol(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_resolved_symbol"); 
        self.symbol_node_links
            .get(node)
            .and_then(|l| l.resolved_symbol.clone())
    }

    pub fn get_jsx_fragment_factory(&self, _location: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_jsx_fragment_factory"); 
        String::new()
    }

    pub fn resolve_name(
        &self,
        name: &str,
        location: &Arc<Node>,
        meaning: SymbolFlags,
        exclude_globals: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_name"); 
        // Go checker.resolveName = binder.NameResolver.Resolve：自 location 起
        // 逐层上溯查容器 locals/成员表（meaning 过滤，条件类型 infer 参数仅
        // trueType 可见），末端 globals 回退。起点节点自身 locals 按 Go 语义
        // lastLocation==nil：函数类容器不做可见性限制，条件类型起点不可见
        use crate::checker::checker_resolve_checker::ANCESTRY_CONTAINERS;

        if ANCESTRY_CONTAINERS.contains(&location.kind)
            && location.kind != tsox_frontend::ast::SyntaxKind::ConditionalType
            && !Self::is_global_source_file(location)
            && let Some(sym) = self
                .program
                .symbol_map()
                .locals
                .get(&location.id())
                .and_then(|l| l.get(name))
            && self.meaning_hit(sym, meaning)
        {
            return Some(Arc::clone(sym));
        }

        let mut chain: std::collections::HashMap<u64, (Arc<Node>, Arc<Node>)> =
            std::collections::HashMap::new();
        {
            let mut child = Arc::clone(location);
            let mut ancestor = location.parent();
            while let Some(a) = ancestor {
                chain.insert(a.id(), (Arc::clone(&a), Arc::clone(&child)));
                child = Arc::clone(&a);
                ancestor = a.parent();
            }
        }
        let module_meaning = meaning & SymbolFlags::MODULE_MEMBER;
        let enum_meaning = meaning & SymbolFlags::EnumMember;
        let type_meaning = meaning & SymbolFlags::TYPE;
        let decorator_skip =
            crate::checker::checker_resolve_checker::decorator_skip_container_ids(location);
        if let Some(sym) = self.ancestry_lookup(
            location,
            &chain,
            name,
            meaning,
            module_meaning,
            enum_meaning,
            type_meaning,
            decorator_skip.as_ref(),
        ) {
            return Some(sym);
        }
        if !exclude_globals
            && let Some(sym) = self.globals.get(name)
            && sym
                .flags
                .intersects(meaning.union(SymbolFlags::GlobalLookup))
        {
            return Some(Arc::clone(sym));
        }
        None
    }

    pub fn get_symbol_flags(&self, symbol: &Arc<Symbol>) -> SymbolFlags { ::tsox_core::fntrace::enter("get_symbol_flags"); 
        symbol.flags
    }

    pub fn get_rest_type_of_signature(&self, _sig: &Arc<Signature>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_rest_type_of_signature"); 
        None
    }

    // Go checker.isContextSensitive：递归判定（函数/嵌套箭头/||、??/条件/数组/对象字面量/括号）
    pub fn is_context_sensitive(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_context_sensitive"); 
        use tsox_frontend::ast::NodeData;
        match &node.data {
            NodeData::FunctionExpression(d) => self.is_context_sensitive_fn_like(
                d.type_parameters.is_some(),
                &d.parameters,
                d.type_node.as_ref(),
                Some(&d.body),
                false,
            ),
            NodeData::ArrowFunction(d) => self.is_context_sensitive_fn_like(
                d.type_parameters.is_some(),
                &d.parameters,
                d.type_node.as_ref(),
                Some(&d.body),
                true,
            ),
            NodeData::MethodDeclaration(d) => self.is_context_sensitive_fn_like(
                d.type_parameters.is_some(),
                &d.parameters,
                d.type_node.as_ref(),
                d.body.as_ref(),
                false,
            ),
            NodeData::FunctionDeclaration(d) => self.is_context_sensitive_fn_like(
                d.type_parameters.is_some(),
                &d.parameters,
                d.type_node.as_ref(),
                d.body.as_ref(),
                false,
            ),
            NodeData::ObjectLiteralExpression(d) => d.properties.iter().any(|p| self.is_context_sensitive(p)),
            NodeData::ArrayLiteralExpression(d) => d.elements.iter().any(|e| self.is_context_sensitive(e)),
            NodeData::ConditionalExpression(d) => {
                self.is_context_sensitive(&d.when_true) || self.is_context_sensitive(&d.when_false)
            }
            NodeData::BinaryExpression(d)
                if matches!(
                    d.operator_token.kind,
                    SyntaxKind::BarBarToken | SyntaxKind::QuestionQuestionToken
                ) =>
            {
                self.is_context_sensitive(&d.left) || self.is_context_sensitive(&d.right)
            }
            NodeData::PropertyAssignment(d) => self.is_context_sensitive(&d.initializer),
            NodeData::ParenthesizedExpression(d) => self.is_context_sensitive(&d.expression),
            _ => false,
        }
    }

    fn is_context_sensitive_fn_like(
        &self,
        has_type_parameters: bool,
        parameters: &tsox_frontend::ast::NodeList,
        type_node: Option<&Arc<Node>>,
        body: Option<&Arc<Node>>,
        is_arrow: bool,
    ) -> bool { ::tsox_core::fntrace::enter("is_context_sensitive_fn_like"); 
        use tsox_frontend::ast::NodeData;
        if has_type_parameters {
            return false;
        }
        // Go HasContextSensitiveParameters：任一参数无注解即敏感
        if parameters.iter().any(|p| {
            matches!(&p.data, NodeData::ParameterDeclaration(pd) if pd.type_node.is_none())
        }) {
            return true;
        }
        // 非箭头：首参非显式 this 时，函数体含 this 引用即敏感
        if !is_arrow {
            let first_is_this = parameters.iter().next().is_some_and(|p| {
                matches!(&p.data, NodeData::ParameterDeclaration(pd)
                    if matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this"))
            });
            if !first_is_this
                && let Some(b) = body
                && body_contains_this(b)
            {
                return true;
            }
        }
        // Go hasContextSensitiveReturnExpression：无返回注解时按 return 表达式递归
        if type_node.is_none()
            && let Some(b) = body
        {
            if b.kind != SyntaxKind::Block {
                return self.is_context_sensitive(b);
            }
            if let Some(hit) = for_each_return_expression(b) {
                return self.is_context_sensitive(&hit);
            }
        }
        false
    }

    pub fn fill_missing_type_arguments(
        &mut self,
        type_arguments: &[Arc<Type>],
        type_parameters: &[Arc<Type>],
        _min_type_argument_count: usize,
        is_java_script_implicit_any: bool,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("fill_missing_type_arguments"); 
        let num_type_parameters = type_parameters.len();
        if num_type_parameters == 0 {
            return Vec::new();
        }
        let num_type_arguments = type_arguments.len();
        if is_java_script_implicit_any || num_type_arguments < num_type_parameters {
            let mut result: Vec<Arc<Type>> = type_arguments.to_vec();
            let error_type = self.error_type();
            result.resize(num_type_parameters, error_type);
            let base_default_type = if is_java_script_implicit_any {
                self.any_type()
            } else {
                self.unknown_type()
            };
            for i in num_type_arguments..num_type_parameters {
                let mut default_type = self.get_default_from_type_parameter(&type_parameters[i]);
                if is_java_script_implicit_any {
                    let unknown = self.unknown_type();
                    let empty_object = self.empty_object_type();
                    if default_type.as_ref().is_some_and(|d| {
                        self.is_type_identical_to(d, &unknown)
                            || self.is_type_identical_to(d, &empty_object)
                    }) {
                        default_type = Some(self.any_type());
                    }
                }
                result[i] = match default_type {
                    Some(d) => {
                        let mapper = Arc::new(crate::checker::mig::w9a::new_type_mapper(
                            type_parameters.to_vec(),
                            result.clone(),
                        ));
                        self.instantiate_type(&d, Some(&mapper))
                    }
                    None => Arc::clone(&base_default_type),
                };
            }
            return result;
        }
        type_arguments.to_vec()
    }

    pub fn get_min_type_argument_count(&mut self, type_parameters: &[Arc<Type>]) -> usize { ::tsox_core::fntrace::enter("get_min_type_argument_count");
        for (i, tp) in type_parameters.iter().enumerate() {
            if self.get_default_from_type_parameter(tp).is_some() {
                return i;
            }
        }
        type_parameters.len()
    }

    pub fn get_union_type_ex(
        &mut self,
        types: Vec<Arc<Type>>,
        union_reduction: UnionReduction,
        alias: Option<&TypeAlias>,
        origin: Option<&Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_union_type_ex");
        // Go getUnionTypeEx（checker.go:27167-27192）：空/单成员短路保
        // 实例同一性；联合与单型两成员组合经 unionOfUnionTypes 驻留（键为
        // 双侧 id 排序 + 归约档 + alias 键，origin 恒 nil 入 worker）；
        // 其余归约档（含 None）一律直通 getUnionTypeWorker——None 档在
        // worker 内仅跳过归约段（checker.go:27203），排序插入、named-union
        // origin、PrimitiveUnion/ContainsIntersections 标志与 unionTypes
        // 驻留照常执行，不存在轻量形态
        if types.is_empty() {
            return self.never_type();
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("len checked");
        }
        if types.len() == 2
            && origin.is_none()
            && (types[0].flags.intersects(TypeFlags::UNION) || types[1].flags.intersects(TypeFlags::UNION))
        {
            let (mut id1, mut id2) = (types[0].id, types[1].id);
            if id1 > id2 {
                std::mem::swap(&mut id1, &mut id2);
            }
            let key = UnionOfUnionKey { id1, id2, r: union_reduction, a: get_alias_key(alias) };
            if let Some(t) = self.union_of_union_types.get(&key) {
                return Arc::clone(t);
            }
            let t = self.get_union_type_worker(types, union_reduction, alias, None);
            self.union_of_union_types.insert(key, Arc::clone(&t));
            return t;
        }
        self.get_union_type_worker(types, union_reduction, alias, origin)
    }

    pub fn requires_adding_implicit_undefined(&self, _node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("requires_adding_implicit_undefined"); 
        false
    }

    pub fn remove_missing_or_undefined_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_missing_or_undefined_type");
        if self.exact_optional_property_types {
            let missing = self.missing_type();
            return self.remove_type(t, &missing);
        }
        self.get_type_with_facts(t, crate::checker::types::TypeFacts::NE_UNDEFINED)
    }

    pub fn compare_symbols(&self, _s1: &Arc<Symbol>, _s2: &Arc<Symbol>) -> i32 { ::tsox_core::fntrace::enter("compare_symbols"); 
        0
    }

    pub fn get_default_keyword_type(&mut self) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_default_keyword_type"); 
        self.get_global_type_by_name("default")
    }

    pub fn get_promise_type(&self) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_promise_type"); 
        self.global_promise_type.get().cloned()
    }

    pub fn get_promise_like_type(&mut self) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_promise_like_type"); 
        self.get_global_type_by_name("PromiseLike")
    }

    pub fn create_type_checker_cache(&self) { ::tsox_core::fntrace::enter("create_type_checker_cache"); }

    pub fn clear_possible_type_requests(&mut self) { ::tsox_core::fntrace::enter("clear_possible_type_requests"); }
}

fn body_contains_this(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("body_contains_this"); 
    if node.kind == SyntaxKind::ThisKeyword {
        return true;
    }
    if matches!(node.data, tsox_frontend::ast::NodeData::FunctionExpression(_))
        || matches!(node.data, tsox_frontend::ast::NodeData::ArrowFunction(_))
        || matches!(node.data, tsox_frontend::ast::NodeData::MethodDeclaration(_))
    {
        return false;
    }
    let mut hit = false;
    tsox_frontend::ast::node_data_generated::for_each_child(node, |c| {
        hit = body_contains_this(c);
        hit
    });
    hit
}

fn for_each_return_expression(body: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("for_each_return_expression"); 
    if body.kind == SyntaxKind::ReturnStatement {
        if let tsox_frontend::ast::NodeData::ReturnStatement(data) = &body.data {
            return data.expression.clone();
        }
        return None;
    }
    if matches!(body.data, tsox_frontend::ast::NodeData::FunctionExpression(_))
        || matches!(body.data, tsox_frontend::ast::NodeData::ArrowFunction(_))
    {
        return None;
    }
    let mut hit: Option<Arc<Node>> = None;
    tsox_frontend::ast::node_data_generated::for_each_child(body, |c| {
        if let Some(e) = for_each_return_expression(c) {
            hit = Some(e);
            return true;
        }
        false
    });
    hit
}

fn get_alias_key(alias: Option<&TypeAlias>) -> CacheHashKey { ::tsox_core::fntrace::enter("get_alias_key");
    let mut b = KeyBuilder::new();
    b.write_alias(alias);
    b.hash()
}
