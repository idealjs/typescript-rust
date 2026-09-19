use crate::checker::checker_prop_access::*;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::is_access_expression;
use tsox_frontend::ast::is_function_like_kind;

impl Checker {
    pub(crate) fn check_property_not_used_before_declaration(
        &mut self,
        prop: &Arc<Symbol>,
        node: &Arc<Node>,
        right: &Arc<Node>,
    ) {
        // 合成实例成员符号缺 value_declaration/parent，回源 binder 声明符号
        let prop = prop
            .declarations
            .first()
            .and_then(|d| {
                let d = d.clone();
                self.program.symbol_map().symbol_of(&d).map(Arc::clone)
            })
            .unwrap_or_else(|| Arc::clone(prop));
        let Some(value_declaration) = prop.value_declaration.clone() else {
            return;
        };
        if self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file)
        {
            return;
        }
        let declaration_name = right.text().to_string();
        let not_declared_before_use = !self.declaration_before_use(&value_declaration, right);
        let mut diagnostic: Option<tsox_frontend::ast::Diagnostic> = None;
        if self.is_in_property_initializer_or_class_static_block(node)
            && !is_optional_property_declaration(&value_declaration)
            && !is_nested_access(node)
            && not_declared_before_use
            && !is_static_method_declaration(&value_declaration)
            && (self.compiler_options.get_use_define_for_class_fields()
                || !self.is_property_declared_in_ancestor_class(&prop, &value_declaration))
        {
            diagnostic = Some(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                right.loc,
                PROPERTY_0_IS_USED_BEFORE_ITS_INITIALIZATION,
                vec![declaration_name.clone()],
            ));
        } else if value_declaration.kind == SyntaxKind::ClassDeclaration
            && node.parent().is_some_and(|p| p.kind != SyntaxKind::TypeReference)
            && !node_has_ambient_modifier(&value_declaration)
            && not_declared_before_use
        {
            diagnostic = Some(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                right.loc,
                CLASS_0_USED_BEFORE_ITS_DECLARATION,
                vec![declaration_name.clone()],
            ));
        }
        if let Some(mut diag) = diagnostic {
            let related_file = self
                .get_source_file_of_node(&value_declaration)
                .or_else(|| self.current_file.clone());
            let related_loc = value_declaration
                .name()
                .map(|n| n.loc)
                .unwrap_or(value_declaration.loc);
            diag.related_information.push(tsox_frontend::ast::Diagnostic::new(
                related_file,
                related_loc,
                tsox_core::diagnostics::messages_generated::X_0_IS_DECLARED_HERE,
                vec![declaration_name],
            ));
            self.diagnostics.add(diag);
        }
    }

    // Go isBlockScopedNameDeclaredBeforeUse（本调用点只涉及属性/类声明，
    // 变量与解构元素分支不在该调用点触发）
    fn declaration_before_use(&mut self, declaration: &Arc<Node>, usage: &Arc<Node>) -> bool {
        if !same_root(declaration, usage) {
            return true;
        }
        let prop_no_init = matches!(
            &declaration.data,
            NodeData::PropertyDeclaration(d)
                if d.initializer.is_none() && !has_exclamation_token(declaration)
        );
        if declaration.loc.pos() <= usage.loc.pos() && !(prop_no_init && is_this_property(usage)) {
            return match declaration.kind {
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                    !self.usage_in_class_computed_name_or_decorator(declaration, usage)
                }
                SyntaxKind::PropertyDeclaration => {
                    !is_property_immediately_referenced(declaration, usage, false)
                }
                _ => true,
            };
        }
        // 声明在用法之后：延迟求值的用法合法
        if usage_in_export_introducer(usage) {
            return true;
        }
        if self.is_used_in_function_or_instance_property(usage, declaration) {
            if self.compiler_options.get_emit_standard_class_fields()
                && containing_class(declaration).is_some()
                && matches!(
                    declaration.kind,
                    SyntaxKind::PropertyDeclaration | SyntaxKind::Parameter
                )
            {
                return !is_property_immediately_referenced(declaration, usage, true);
            }
            return true;
        }
        false
    }

    // Go isPropertyInitializedInStaticBlocks：在 [start,end] 区间内的静态块的
    // return flow 上查 this.prop 的流类型，不含 undefined 即视为已初始化
    fn is_property_initialized_in_static_blocks(
        &mut self,
        usage: &Arc<Node>,
        declaration: &Arc<Node>,
        static_blocks: &[Arc<Node>],
        start_pos: usize,
        end_pos: usize,
    ) -> bool {
        let Some(symbol) = self.program.symbol_map().symbol_of(declaration).cloned() else {
            return false;
        };
        let declared = self.get_type_of_symbol(&symbol);
        let initial = if self.type_contains_undefined_local(&declared) {
            declared.clone()
        } else {
            self.get_union_type(vec![Arc::clone(&declared), self.undefined_type()])
        };
        let target = crate::checker::flow::FlowRef::Node(
            usage
                .parent()
                .filter(|p| is_access_expression(p))
                .unwrap_or_else(|| Arc::clone(usage)),
        );
        for block in static_blocks {
            if block.loc.pos() < start_pos || block.loc.pos() > end_pos {
                continue;
            }
            let Some(flow) = self.program.symbol_map().flow_node_of(block).cloned() else {
                continue;
            };
            let key = self.flow_cache_key(&target, &flow, &initial);
            if let Some(cached) = self.flow_type_cache.get(&key) {
                if !self.type_contains_undefined_local(cached) {
                    return true;
                }
                continue;
            }
            self.flow_type_cache.insert(key, Arc::clone(&declared));
            let mut query = crate::checker::flow::FlowQuery::default();
            let narrowed =
                self.type_at_flow_node(&declared, &initial, &flow, &target, 0, &mut query);
            self.flow_type_cache
                .insert(self.flow_cache_key(&target, &flow, &initial), Arc::clone(&narrowed));
            if !self.type_contains_undefined_local(&narrowed) {
                return true;
            }
        }
        false
    }

    // Go isBlockScopedNameDeclaredBeforeUse 类分支：用法处于该类的计算属性名
    // 或（非 legacy）装饰器内时不算“声明先于使用”；装饰器内的非 IIFE 函数延迟合法
    pub(crate) fn usage_in_class_computed_name_or_decorator(
        &self,
        declaration: &Arc<Node>,
        usage: &Arc<Node>,
    ) -> bool {
        let mut cur = usage.parent();
        while let Some(n) = cur {
            if Arc::ptr_eq(&n, declaration) {
                return false;
            }
            let computed = n.kind == SyntaxKind::ComputedPropertyName
                && n.parent()
                    .and_then(|p| p.parent())
                    .is_some_and(|pp| Arc::ptr_eq(&pp, declaration));
            let decorator = !self.legacy_decorators
                && n.kind == SyntaxKind::Decorator
                && decorator_attached_to_class_member(&n, declaration);
            if computed {
                return true;
            }
            if decorator {
                // 装饰器内若处于非 IIFE 函数中则延迟合法（不算“未声明先使用”）
                let mut inner = Some(usage.clone());
                while let Some(m) = inner {
                    if Arc::ptr_eq(&m, &n) {
                        return true;
                    }
                    if is_function_like_kind(m.kind) && !is_immediately_invoked(&m) {
                        return false;
                    }
                    inner = m.parent();
                }
                return true;
            }
            cur = n.parent();
        }
        false
    }

    // Go isUsedInFunctionOrInstanceProperty：用法处于延迟求值位置
    fn is_used_in_function_or_instance_property(
        &mut self,
        usage: &Arc<Node>,
        declaration: &Arc<Node>,
    ) -> bool {
        let mut cur = Some(usage.clone());
        while let Some(n) = cur {
            if is_function_like_kind(n.kind) && !is_immediately_invoked(&n) {
                return true;
            }
            if n.kind == SyntaxKind::ClassStaticBlockDeclaration {
                return declaration.loc.pos() < usage.loc.pos();
            }
            if let Some(parent) = n.parent()
                && parent.kind == SyntaxKind::PropertyDeclaration
                && is_property_initializer(&parent, &n)
            {
                if parent.has_syntactic_modifier(ModifierFlags::Static) {
                    if declaration.kind == SyntaxKind::MethodDeclaration {
                        return true;
                    }
                    if declaration.kind == SyntaxKind::PropertyDeclaration
                        && nodes_share_class_parent(declaration, usage)
                        && declaration
                            .name()
                            .is_some_and(|name| matches!(name.kind, SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier))
                    {
                        let class = declaration.parent().unwrap_or_else(|| Arc::clone(declaration));
                        let start_pos = class.loc.pos();
                        let end_pos = n.loc.pos();
                        let static_blocks: Vec<Arc<Node>> = class_members(&class)
                            .into_iter()
                            .filter(|m| m.kind == SyntaxKind::ClassStaticBlockDeclaration)
                            .collect();
                        if self.is_property_initialized_in_static_blocks(
                            usage,
                            declaration,
                            &static_blocks,
                            start_pos,
                            end_pos,
                        ) {
                            return true;
                        }
                    }
                } else {
                    let decl_instance_prop =
                        declaration.kind == SyntaxKind::PropertyDeclaration
                            && !declaration.has_syntactic_modifier(ModifierFlags::Static);
                    if !decl_instance_prop || !nodes_share_class_parent(declaration, usage) {
                        return true;
                    }
                }
            }
            cur = n.parent();
        }
        false
    }

    // Go isInPropertyInitializerOrClassStaticBlock：FindAncestorFalse 表示继续上溯，
    // 仅 Quit 语义（TypeQuery/JsxClosingElement/Block-函数体/ArrowFunction-按参）终止；
    // Block 的父是函数式声明（不含 ArrowFunction，静态块也不算）才终止
    fn is_in_property_initializer_or_class_static_block(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent();
        while let Some(n) = cur {
            match n.kind {
                SyntaxKind::PropertyDeclaration | SyntaxKind::ClassStaticBlockDeclaration => {
                    return true
                }
                SyntaxKind::TypeQuery | SyntaxKind::JsxClosingElement => return false,
                SyntaxKind::ArrowFunction => return false,
                SyntaxKind::Block => {
                    let parent_is_fn_like = n.parent().is_some_and(|p| {
                        matches!(
                            p.kind,
                            SyntaxKind::FunctionDeclaration
                                | SyntaxKind::MethodDeclaration
                                | SyntaxKind::Constructor
                                | SyntaxKind::GetAccessor
                                | SyntaxKind::SetAccessor
                                | SyntaxKind::FunctionExpression
                        )
                    });
                    if parent_is_fn_like {
                        return false;
                    }
                }
                _ => {}
            }
            cur = n.parent();
        }
        false
    }

    fn is_property_declared_in_ancestor_class(
        &mut self,
        prop: &Arc<Symbol>,
        value_declaration: &Arc<Node>,
    ) -> bool {
        let Some(parent_class) = value_declaration.parent() else {
            return false;
        };
        let Some(extends_expr) = class_extends_expression(&parent_class) else {
            return false;
        };
        let Some(base_sym) = self.resolve_identifier(&extends_expr) else {
            return false;
        };
        let Some(base_class_node) = base_sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ClassDeclaration)
            .cloned()
        else {
            return false;
        };
        let base_instance = self.build_class_instance_type_with_base(&base_class_node);
        self.get_property_of_type(&base_instance, &prop.name)
            .is_some_and(|super_prop| {
                super_prop.value_declaration.is_some()
                    || !super_prop.declarations.is_empty()
            })
    }
}

fn is_nested_access(node: &Arc<Node>) -> bool {
    if let NodeData::PropertyAccessExpression(data) = &node.data {
        return is_access_expression(&data.expression);
    }
    false
}

fn class_members(class: &Arc<Node>) -> Vec<Arc<Node>> {
    match &class.data {
        NodeData::ClassDeclaration(d) => d.members.iter().cloned().collect(),
        NodeData::ClassExpression(d) => d.members.iter().cloned().collect(),
        _ => Vec::new(),
    }
}

// 取类首个 extends 子句的基类表达式节点（Go 只看 baseTypes[0]）
fn class_extends_expression(class: &Arc<Node>) -> Option<Arc<Node>> {
    let clauses = match &class.data {
        NodeData::ClassDeclaration(d) => d.heritage_clauses.as_ref()?,
        NodeData::ClassExpression(d) => d.heritage_clauses.as_ref()?,
        _ => return None,
    };
    for clause in clauses.iter() {
        if let NodeData::HeritageClause(hc) = &clause.data
            && hc.token == SyntaxKind::ExtendsKeyword
            && let Some(first) = hc.types.iter().next()
        {
            if let NodeData::ExpressionWithTypeArguments(e) = &first.data {
                return Some(Arc::clone(&e.expression));
            }
        }
    }
    None
}

// Go：装饰器挂在该类自身或其直接成员（方法/存取器/属性/参数）上
fn decorator_attached_to_class_member(decorator: &Arc<Node>, class: &Arc<Node>) -> bool {
    let Some(decorated) = decorator.parent() else {
        return false;
    };
    if Arc::ptr_eq(&decorated, class) {
        return true;
    }
    match decorated.kind {
        SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::PropertyDeclaration => decorated
            .parent()
            .is_some_and(|p| Arc::ptr_eq(&p, class)),
        SyntaxKind::Parameter => decorated
            .parent()
            .and_then(|p| p.parent())
            .is_some_and(|pp| Arc::ptr_eq(&pp, class)),
        _ => false,
    }
}

fn is_static_method_declaration(decl: &Arc<Node>) -> bool {
    decl.kind == SyntaxKind::MethodDeclaration
        && decl.has_syntactic_modifier(ModifierFlags::Static)
}

fn is_optional_property_declaration(decl: &Arc<Node>) -> bool {
    matches!(
        &decl.data,
        NodeData::PropertyDeclaration(d)
            if d.postfix_token
                .as_ref()
                .is_some_and(|t| t.kind == SyntaxKind::QuestionToken)
    )
}

fn has_exclamation_token(decl: &Arc<Node>) -> bool {
    matches!(
        &decl.data,
        NodeData::PropertyDeclaration(d)
            if d.postfix_token
                .as_ref()
                .is_some_and(|t| t.kind == SyntaxKind::ExclamationToken)
    )
}

fn node_has_ambient_modifier(decl: &Arc<Node>) -> bool {
    decl.has_syntactic_modifier(ModifierFlags::Ambient)
}

fn is_this_property(usage: &Arc<Node>) -> bool {
    usage.parent().is_some_and(|p| {
        p.kind == SyntaxKind::PropertyAccessExpression
            && matches!(&p.data, NodeData::PropertyAccessExpression(d) if d.expression.kind == SyntaxKind::ThisKeyword)
    })
}

fn usage_in_export_introducer(usage: &Arc<Node>) -> bool {
    usage.parent().is_some_and(|p| {
        matches!(p.kind, SyntaxKind::ExportSpecifier)
            || matches!(&p.data, NodeData::ExportAssignment(e) if e.is_export_equals)
    })
}

fn containing_class(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if matches!(n.kind, SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression) {
            return Some(n);
        }
        cur = n.parent();
    }
    None
}

fn nodes_share_class_parent(declaration: &Arc<Node>, usage: &Arc<Node>) -> bool {
    match (containing_class(declaration), containing_class(usage)) {
        (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
        _ => false,
    }
}

fn is_property_initializer(property: &Arc<Node>, candidate: &Arc<Node>) -> bool {
    matches!(
        &property.data,
        NodeData::PropertyDeclaration(d) if d
            .initializer
            .as_ref()
            .is_some_and(|init| Arc::ptr_eq(init, candidate))
    )
}

// Go GetImmediatelyInvokedFunctionExpression：函数体经括号包裹后紧跟调用视为 IIFE
fn is_immediately_invoked(func: &Arc<Node>) -> bool {
    let mut wrapper = Arc::clone(func);
    while let Some(p) = wrapper.parent()
        && p.kind == SyntaxKind::ParenthesizedExpression
    {
        wrapper = p;
    }
    wrapper.parent().is_some_and(|p| {
        matches!(&p.data, NodeData::CallExpression(c) if Arc::ptr_eq(&c.expression, &wrapper))
    })
}

// Go isPropertyImmediatelyReferencedWithinDeclaration
fn is_property_immediately_referenced(
    declaration: &Arc<Node>,
    usage: &Arc<Node>,
    stop_at_any_property_declaration: bool,
) -> bool {
    if usage.loc.pos() + usage.loc.len() > declaration.loc.pos() + declaration.loc.len() {
        return false;
    }
    let mut cur = Some(usage.clone());
    while let Some(n) = cur {
        if Arc::ptr_eq(&n, declaration) {
            break;
        }
        match n.kind {
            SyntaxKind::ArrowFunction => return false,
            SyntaxKind::PropertyDeclaration => {
                return stop_at_any_property_declaration
                    && n.parent().is_some_and(|p| {
                        declaration
                            .parent()
                            .is_some_and(|dp| Arc::ptr_eq(&p, &dp))
                    })
            }
            SyntaxKind::Block => {
                if let Some(p) = n.parent()
                    && matches!(
                        p.kind,
                        SyntaxKind::MethodDeclaration
                            | SyntaxKind::GetAccessor
                            | SyntaxKind::SetAccessor
                    )
                {
                    return false;
                }
            }
            _ => {}
        }
        cur = n.parent();
    }
    true
}

fn same_root(a: &Arc<Node>, b: &Arc<Node>) -> bool {
    let root_of = |mut n: Arc<Node>| loop {
        match n.parent() {
            Some(p) => n = p,
            None => return n,
        }
    };
    Arc::ptr_eq(&root_of(a.clone()), &root_of(b.clone()))
}
