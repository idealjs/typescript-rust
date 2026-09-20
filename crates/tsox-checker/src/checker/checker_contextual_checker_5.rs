#![allow(unused_imports)]

use crate::checker::checker_contextual::*;

impl Checker {
    pub(crate) fn check_variable_used_before_assigned(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
    ) {
        if crate::checker::utilities::get_assignment_target_kind(node)
            == crate::checker::utilities::AssignmentKind::Definite
        {
            return;
        }

        if !self.strict_null_checks {
            return;
        }

        let is_plain_var = symbol.flags.contains(SymbolFlags::FunctionScopedVariable)
            && symbol
                .value_declaration
                .as_ref()
                .is_some_and(|d| d.kind == SyntaxKind::VariableDeclaration);
        if !symbol.flags.contains(SymbolFlags::BlockScopedVariable) && !is_plain_var {
            return;
        }

        let declaration = symbol.value_declaration.as_ref().or_else(|| {
            symbol.declarations.iter().find(|d| {
                matches!(
                    d.kind,
                    SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement
                )
            })
        });
        let Some(declaration) = declaration else {
            return;
        };

        // 自初始化式内引用：解析期环路径返回 any（Go reportCircularityError），
        // assumeInitialized 短路
        {
            let mut cur = node.parent();
            let mut inside_own = false;
            while let Some(a) = cur {
                if Arc::ptr_eq(&a, declaration) {
                    inside_own = true;
                    break;
                }
                if matches!(
                    a.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                ) {
                    break;
                }
                cur = a.parent();
            }
            if inside_own {
                return;
            }
        }

        // 纯声明（let x;）auto 型走下面类型守卫；此处只拦 ambient/断言声明
        let has_exclamation = matches!(
            &declaration.data,
            tsox_frontend::ast::NodeData::VariableDeclaration(vd) if vd.exclamation_token.is_some()
        );
        let ambient_context = |n: &Arc<Node>| -> bool {
            let mut cur = Some(Arc::clone(n));
            while let Some(c) = cur {
                if c.has_syntactic_modifier(ModifierFlags::Ambient) {
                    return true;
                }
                cur = c.parent();
            }
            false
        };
        let in_type_node = |n: &Arc<Node>| -> bool {
            let mut cur = n.parent();
            while let Some(a) = cur {
                if matches!(
                    a.kind,
                    SyntaxKind::InterfaceDeclaration
                        | SyntaxKind::TypeAliasDeclaration
                        | SyntaxKind::TypeLiteral
                ) {
                    return true;
                }
                cur = a.parent();
            }
            false
        };
        let file_is_declaration = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file);
        let spread_destructuring_target = node.parent().is_some_and(|p| {
            p.kind == SyntaxKind::SpreadElement
                && p.parent().is_some_and(|lit| {
                    lit.kind == SyntaxKind::ObjectLiteralExpression
                        && lit.parent().is_some_and(|gp| match &gp.data {
                            tsox_frontend::ast::NodeData::BinaryExpression(b) => {
                                Arc::ptr_eq(&b.left, &lit)
                            }
                            _ => gp.kind == SyntaxKind::ForOfStatement,
                        })
                })
        });
        if self
            .get_combined_modifier_flags(declaration)
            .contains(ModifierFlags::Ambient)
            || has_exclamation
            || ambient_context(declaration)
            || ambient_context(node)
            || file_is_declaration
            || in_type_node(node)
            || spread_destructuring_target
        {
            return;
        }

        let declared_type = self.get_type_of_symbol(symbol);
        if declared_type
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Void)
            || type_contains_undefined(&declared_type)
        {
            return;
        }

        let flow_container_of = |n: &Arc<Node>| -> Option<Arc<Node>> {
            let mut current = Arc::clone(n);
            loop {
                if matches!(
                    current.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::ModuleDeclaration
                        | SyntaxKind::PropertyDeclaration
                        | SyntaxKind::PropertySignature
                ) {
                    return Some(current);
                }
                current = Arc::clone(current.parent().as_ref()?);
            }
        };
        let same_scope = match (flow_container_of(node), flow_container_of(declaration)) {
            (Some(a), Some(b)) => Arc::ptr_eq(&a, &b),
            _ => true,
        };
        if !same_scope {
            return;
        }

        if node
            .parent()
            .as_ref()
            .is_some_and(|p| p.kind == SyntaxKind::NonNullExpression)
        {
            return;
        }

        if !self.strict_null_checks {
            return;
        }
        if let Some(flow_type) = self.get_definite_assignment_flow_type(symbol, node) {
            if type_contains_undefined(&flow_type) {
                let file = self.current_file.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    VARIABLE_0_IS_USED_BEFORE_BEING_ASSIGNED,
                    vec![name.to_string()],
                ));
            }
        }
    }

    pub(crate) fn push_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes += 1;
        if self.suppress_source_file.is_none() {
            self.suppress_source_file = self.current_file.as_ref().map(|f| f.node.id());
        }
    }

    pub(crate) fn pop_ts2304_suppression(&mut self) {
        self.suppress_cannot_find_name_in_type_nodes = self
            .suppress_cannot_find_name_in_type_nodes
            .saturating_sub(1);
        if self.suppress_cannot_find_name_in_type_nodes == 0 {
            self.suppress_source_file = None;
        }
    }

    pub(crate) fn ts2304_reporting_allowed_for(&self, node: &Arc<Node>) -> bool {
        if self.suppress_cannot_find_name_in_type_nodes == 0 {
            return true;
        }
        match (
            self.get_source_file_of_node(node),
            self.suppress_source_file,
        ) {
            (Some(f), Some(origin)) => {
                if f.node.id() == origin {
                    false
                } else {
                    !f.file_name.starts_with("bundled://")
                }
            }
            _ => false,
        }
    }

    pub(crate) fn push_scope(&mut self, node: &Arc<Node>) {
        self.scope_stack.push(node.id());
    }

    pub(crate) fn push_function_scope(&mut self, node: &Arc<Node>) {
        self.function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    pub(crate) fn pop_function_scope(&mut self) {
        self.function_scope_count -= 1;
        self.scope_stack.pop();
    }

    pub(crate) fn push_arrow_function_scope(&mut self, node: &Arc<Node>) {
        self.arrow_function_scope_count += 1;
        self.scope_stack.push(node.id());
    }

    pub(crate) fn pop_arrow_function_scope(&mut self) {
        self.arrow_function_scope_count -= 1;
        self.scope_stack.pop();
    }
}
