#![allow(unused_imports)]

use crate::checker::exports::*;

impl Checker {
    pub fn get_unknown_signature(&self) -> Option<Arc<Signature>> {
        self.unknown_signature.get().cloned()
    }

    pub fn get_name_type_of_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        self.value_symbol_links
            .get(symbol)
            .and_then(|links| links.name_type.clone())
    }

    pub fn get_global_symbol(
        &self,
        name: &str,
        _meaning: SymbolFlags,
        _diagnostic: Option<&Message>,
    ) -> Option<Arc<Symbol>> {
        self.globals.get(name).cloned()
    }

    pub fn get_global_symbol_by_name(
        &self,
        name: &str,
        _meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        self.globals.get(name).cloned()
    }

    pub fn get_global_type_by_name(&self, name: &str) -> Option<Arc<Type>> {
        let _symbol = self.globals.get(name)?;
        None
    }

    pub fn get_symbol_by_name(&self, name: &str, _meaning: SymbolFlags) -> Option<Arc<Symbol>> {
        self.globals.get(name).cloned()
    }

    pub fn get_merged_symbol_public(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        Some(Arc::clone(symbol))
    }

    pub fn try_find_ambient_module(&self, _module_name: &str) -> Option<Arc<Symbol>> {
        None
    }

    pub fn get_immediate_aliased_symbol(&self, _symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        None
    }

    pub fn get_type_only_alias_declaration(&self, _symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
        None
    }

    pub fn resolve_external_module_name(
        &self,
        _module_specifier: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        None
    }

    pub fn get_declared_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
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
    ) -> Option<ResolutionMode> {
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

    pub fn type_predicate_to_string(&self, _t: &TypePredicate) -> String {
        String::new()
    }

    pub fn get_expanded_parameters(
        &self,
        _signature: &Arc<Signature>,
        _skip_union_expanding: bool,
    ) -> Vec<Vec<Arc<Symbol>>> {
        Vec::new()
    }

    pub fn get_resolved_signature(&self, _node: &Arc<Node>) -> Option<Arc<Signature>> {
        None
    }

    pub fn get_contextual_type_for_argument_at_index(
        &self,
        _node: &Arc<Node>,
        _arg_index: usize,
    ) -> Option<Arc<Type>> {
        None
    }

    pub fn get_index_signatures_at_location(&self, _node: &Arc<Node>) -> Vec<Arc<Node>> {
        Vec::new()
    }

    pub fn get_resolved_symbol(&self, _node: &Arc<Node>) -> Option<Arc<Symbol>> {
        None
    }

    pub fn get_jsx_fragment_factory(&self, _location: &Arc<Node>) -> String {
        String::new()
    }

    pub fn resolve_name(
        &self,
        _name: &str,
        _location: &Arc<Node>,
        _meaning: SymbolFlags,
        _exclude_globals: bool,
    ) -> Option<Arc<Symbol>> {
        None
    }

    pub fn get_symbol_flags(&self, symbol: &Arc<Symbol>) -> SymbolFlags {
        symbol.flags
    }

    pub fn get_base_types(&mut self, t: &Arc<Type>) -> Vec<Arc<Type>> {
        let Some(class_sym) = t.symbol.clone() else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for decl in class_sym.declarations.iter() {
            let heritage = match &decl.data {
                tsox_frontend::ast::NodeData::ClassDeclaration(d) => d.heritage_clauses.clone(),
                tsox_frontend::ast::NodeData::ClassExpression(d) => d.heritage_clauses.clone(),
                _ => continue,
            };
            let Some(clauses) = heritage else {
                continue;
            };
            for clause in clauses.iter() {
                let tsox_frontend::ast::NodeData::HeritageClause(hc) = &clause.data else {
                    continue;
                };
                for h in hc.types.iter() {
                    let tsox_frontend::ast::NodeData::ExpressionWithTypeArguments(d) = &h.data
                    else {
                        continue;
                    };
                    result.push(self.get_type_of_node(&d.expression));
                }
            }
        }
        result
    }

    pub fn get_base_constructor_type_of_class(&self, _t: &Arc<Type>) -> Option<Arc<Type>> {
        None
    }

    pub fn get_rest_type_of_signature(&self, _sig: &Arc<Signature>) -> Option<Arc<Type>> {
        None
    }

    // Go checker.isContextSensitive：递归判定（函数/嵌套箭头/||、??/条件/数组/对象字面量/括号）
    pub fn is_context_sensitive(&self, node: &Arc<Node>) -> bool {
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
    ) -> bool {
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
        &self,
        type_arguments: &[Arc<Type>],
        _type_parameters: &[Arc<Type>],
        _min_type_argument_count: usize,
        _is_java_script_implicit_any: bool,
    ) -> Vec<Arc<Type>> {
        type_arguments.to_vec()
    }

    pub fn get_min_type_argument_count(&self, type_parameters: &[Arc<Type>]) -> usize {
        type_parameters.len()
    }

    pub fn get_union_type_ex(
        &self,
        types: Vec<Arc<Type>>,
        _union_reduction: UnionReduction,
    ) -> Arc<Type> {
        self.build_union_from_types(types)
    }

    pub fn requires_adding_implicit_undefined(&self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn remove_missing_or_undefined_type(&self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    pub fn compare_symbols(&self, _s1: &Arc<Symbol>, _s2: &Arc<Symbol>) -> i32 {
        0
    }

    pub fn get_default_keyword_type(&self) -> Option<Arc<Type>> {
        self.get_global_type_by_name("default")
    }

    pub fn get_promise_type(&self) -> Option<Arc<Type>> {
        self.global_promise_type.get().cloned()
    }

    pub fn get_promise_like_type(&self) -> Option<Arc<Type>> {
        self.get_global_type_by_name("PromiseLike")
    }

    pub fn create_type_checker_cache(&self) {}

    pub fn clear_possible_type_requests(&mut self) {}
}

fn body_contains_this(node: &Arc<Node>) -> bool {
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

fn for_each_return_expression(body: &Arc<Node>) -> Option<Arc<Node>> {
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
