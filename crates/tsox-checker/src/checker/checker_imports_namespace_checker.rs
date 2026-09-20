#![allow(unused_imports)]

use crate::checker::checker_imports_namespace::*;

impl Checker {
    pub(crate) fn enclosing_function_is_generator(&self, node: &Arc<Node>) -> bool {
        let mut cur = node.parent();
        while let Some(n) = cur {
            let in_name_of_current = tsox_frontend::ast::node_data_generated::node_name(&n)
                .is_some_and(|name| {
                    name.loc.pos() <= node.loc.pos() && node.loc.end() <= name.loc.end()
                });
            if in_name_of_current {
                cur = n.parent();
                continue;
            }
            // 装饰器延迟求值：yield 语境取被装饰实体之外的函数
            //（类成员装饰器跳过成员、类装饰器跳过类）
            if n.kind == SyntaxKind::Decorator
                && let Some(decorated) = n.parent()
            {
                cur = decorated.parent();
                continue;
            }
            match &n.data {
                tsox_frontend::ast::NodeData::FunctionDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }
                tsox_frontend::ast::NodeData::FunctionExpression(d) => {
                    return d.asterisk_token.is_some();
                }
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => {
                    return d.asterisk_token.is_some();
                }

                tsox_frontend::ast::NodeData::ArrowFunction(_)
                | tsox_frontend::ast::NodeData::GetAccessorDeclaration(_)
                | tsox_frontend::ast::NodeData::SetAccessorDeclaration(_)
                | tsox_frontend::ast::NodeData::ConstructorDeclaration(_)
                // 类字段初始化器/静态块是独立容器：其中的 yield 不在生成器上下文
                | tsox_frontend::ast::NodeData::PropertyDeclaration(_)
                | tsox_frontend::ast::NodeData::PropertySignatureDeclaration(_)
                | tsox_frontend::ast::NodeData::ClassStaticBlockDeclaration(_) => return false,
                _ => {}
            }
            cur = n.parent();
        }
        false
    }

    pub(crate) fn get_array_element_type(&self, t: &Arc<Type>) -> Arc<Type> {
        match &t.data {
            crate::checker::TypeData::Object(obj) => {
                if let Some(elem) = obj.type_arguments.first() {
                    return Arc::clone(elem);
                }
                self.get_any_type()
            }
            crate::checker::TypeData::EvolvingArray(ea) => ea
                .element_type
                .clone()
                .unwrap_or_else(|| self.get_any_type()),
            _ => self.get_any_type(),
        }
    }

    pub(crate) fn is_empty_array_literal(&self, node: &Arc<Node>) -> bool {
        matches!(
            &node.data,
            tsox_frontend::ast::NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
        )
    }

    pub(crate) fn get_missing_required_properties(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Vec<String> {
        let Some(source_struct) = source.as_structured() else {
            return Vec::new();
        };
        let Some(target_struct) = target.as_structured() else {
            return Vec::new();
        };
        let mut missing = Vec::new();
        for target_prop in &target_struct.properties {
            if target_prop.flags.contains(SymbolFlags::Optional) {
                continue;
            }
            if source_struct.members.get(&target_prop.name).is_none() {
                missing.push(target_prop.name.clone());
            }
        }
        missing
    }

    pub(crate) fn get_property_name_from_node(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(id) => id.text.clone(),
            NodeData::StringLiteral(s) => s.text.clone(),
            NodeData::NumericLiteral(n) => {
                tsox_core::jsnum::Number::from_string(&n.text).to_string()
            }
            NodeData::ComputedPropertyName(cd) => {
                // `Symbol.<知名符号>` 计算成员用内部名 `__@<name>`
                //（与 binder member_name_text 一致，两侧命中同一键）
                if let Some(internal) =
                    crate::binder::symbols_binder_4::well_known_symbol_member_name(&cd.expression)
                {
                    return internal;
                }
                // 字面量计算名按字面量名入表（与 binder member_name_text
                // 共用 computed_member_literal_name，保持两侧同键）
                if let Some(literal) =
                    crate::binder::symbols_binder_4::computed_member_literal_name(&cd.expression)
                {
                    return literal;
                }
                String::new()
            }
            _ => node.text().to_string(),
        }
    }

    /// 成员声明位的名字（Go lateBindMember 语义）：早绑定名之外，
    /// 计算名为实体名表达式且其类型可用作属性名（string/number 字面量、
    /// unique symbol）时，以类型推导的名字入表
    pub(crate) fn member_declaration_name(&mut self, name: &Arc<Node>) -> String {
        let early = self.get_property_name_from_node(name);
        if !early.is_empty() || !matches!(name.data, NodeData::ComputedPropertyName(_)) {
            return early;
        }
        let NodeData::ComputedPropertyName(cd) = &name.data else {
            unreachable!();
        };
        if !tsox_frontend::ast::is_entity_name_expression(&cd.expression) {
            return early;
        }
        let t = self.get_type_of_node(&cd.expression);
        if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(
            &t,
        ) {
            return crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(&t);
        }
        early
    }

    pub(crate) fn get_constituent_property(
        &mut self,
        object_type: &Arc<Type>,
        name: &str,
    ) -> Option<std::sync::Arc<tsox_frontend::ast::Symbol>> {
        let apparent = self.get_apparent_type(object_type);
        let parts: Vec<Arc<Type>> = if apparent
            .flags
            .contains(crate::checker::types::TypeFlags::Union)
        {
            match &apparent.data {
                crate::checker::types::TypeData::Union(u) => u.union_or_intersection.types.clone(),
                _ => vec![apparent],
            }
        } else {
            vec![apparent]
        };
        for p in parts {
            if let Some(sym) = self.get_property_of_type(&p, name) {
                return Some(sym);
            }
        }
        None
    }

    pub(crate) fn loop_has_escaping_break(n: &Arc<Node>, direct: bool) -> bool {
        match n.kind {
            SyntaxKind::BreakStatement => {
                matches!(
                    &n.data,
                    tsox_frontend::ast::NodeData::BreakStatement(d) if d.label.is_some()
                ) || direct
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor => false,
            _ => {
                let nested = matches!(
                    n.kind,
                    SyntaxKind::WhileStatement
                        | SyntaxKind::DoStatement
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::SwitchStatement
                );
                let mut found = false;
                tsox_frontend::ast::node_data_generated::for_each_child(n, |child| {
                    if Self::loop_has_escaping_break(child, direct && !nested) {
                        found = true;
                        true
                    } else {
                        false
                    }
                });
                found
            }
        }
    }

    pub(crate) fn function_body_has_explicit_return(body: &Arc<Node>) -> bool {
        fn walk(n: &Arc<Node>) -> bool {
            match n.kind {
                SyntaxKind::ReturnStatement => return true,

                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return false,
                _ => {}
            }
            let mut found = false;
            tsox_frontend::ast::node_data_generated::for_each_child(n, |child| {
                if walk(child) {
                    found = true;
                    true
                } else {
                    false
                }
            });
            found
        }
        walk(body)
    }

    pub(crate) fn has_same_named_type_symbol(&self, name: &str) -> bool {
        let type_meaning = SymbolFlags::Interface
            | SymbolFlags::Class
            | SymbolFlags::TypeParameter
            | SymbolFlags::TypeAlias
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum;
        let symbol_map = self.program.symbol_map();
        for &container_id in self.scope_stack.iter().rev() {
            if let Some(locals) = symbol_map.locals.get(&container_id)
                && let Some(sym) = locals.get(name)
                && sym.flags.intersects(type_meaning)
            {
                return true;
            }
            if let Some(container_sym) = symbol_map.symbols.get(&container_id)
                && (container_sym
                    .members
                    .get(name)
                    .is_some_and(|s| s.flags.intersects(type_meaning))
                    || container_sym
                        .exports
                        .get(name)
                        .is_some_and(|s| s.flags.intersects(type_meaning)))
            {
                return true;
            }
        }
        self.globals
            .get(name)
            .is_some_and(|s| s.flags.intersects(type_meaning))
    }

    pub(crate) fn namespace_usable_as_value(&mut self, namespace: &Arc<Symbol>) -> bool {
        let state_instantiated = namespace
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
            .any(|d| {
                module_is_instantiated(d, self.compiler_options.should_preserve_const_enums())
            });
        state_instantiated || self.namespace_has_value_side(namespace)
    }
}
