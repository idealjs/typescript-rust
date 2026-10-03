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

    pub fn try_find_ambient_module(&self, _module_name: &str) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("try_find_ambient_module"); 
        None
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
        _module_specifier: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_external_module_name"); 
        None
    }

    pub fn get_declared_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_declared_type_of_symbol"); 
        // Go tryGetDeclaredTypeOfSymbol 的 Class/Interface/TypeAlias 分支；
        // TypeParameter/Enum/Alias 等其余形态维持 any（尚未接入）
        if symbol.flags.contains(tsox_frontend::ast::SymbolFlags::Interface) {
            return self.resolve_interface_type_ex(symbol, None);
        }
        if symbol.flags.contains(tsox_frontend::ast::SymbolFlags::Class) {
            let class_node = symbol
                .value_declaration
                .clone()
                .or_else(|| symbol.declarations.first().cloned());
            if let Some(d) = class_node
                && matches!(
                    d.kind,
                    tsox_frontend::ast::SyntaxKind::ClassDeclaration
                        | tsox_frontend::ast::SyntaxKind::ClassExpression
                )
            {
                return self.build_class_instance_type_with_base(&d);
            }
            return self.resolve_interface_type_ex(symbol, None);
        }
        if symbol.flags.contains(tsox_frontend::ast::SymbolFlags::TypeAlias) {
            if let Some(t) = self.type_alias_links.get(symbol).and_then(|l| l.declared_type.clone()) {
                return t;
            }
        }
        self.any_type()
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
        if let Some(cached) = self
            .signature_links
            .get(node)
            .and_then(|l| l.resolved_signature.clone())
            && !self.signature_is_resolving_signature(&cached)
        {
            return Some(cached);
        }
        let marker = self.resolving_signature();
        self.signature_links
            .get_or_default(node)
            .resolved_signature = Some(Arc::clone(&marker));
        let result = self.resolve_signature(node, CheckMode::Normal);
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
        if let Some(sym) = self.ancestry_lookup(
            location,
            &chain,
            name,
            meaning,
            module_meaning,
            enum_meaning,
            type_meaning,
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

    pub fn get_min_type_argument_count(&self, type_parameters: &[Arc<Type>]) -> usize { ::tsox_core::fntrace::enter("get_min_type_argument_count"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_union_type_ex"); 
        // Go getUnionTypeEx：归约路径直通 getUnionTypeWorker（addTypesToUnion
        // 排序去重 + 字面量/受约束类型参数/子类型归约 + named-union origin +
        // unionTypes 驻留）；None 维持排序去重轻路径
        if union_reduction != UnionReduction::None {
            return self.get_union_type_worker(types, union_reduction, None, None);
        }
        self.build_union_from_types(types)
    }

    pub fn requires_adding_implicit_undefined(&self, _node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("requires_adding_implicit_undefined"); 
        false
    }

    pub fn remove_missing_or_undefined_type(&self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("remove_missing_or_undefined_type"); 
        Arc::clone(t)
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
