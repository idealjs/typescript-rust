#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn collect_return_expressions(node: &Arc<Node>, out: &mut Vec<Arc<Node>>) {
        crate::ast::node_data_generated::for_each_child(node, |child| match child.kind {
            SyntaxKind::ReturnStatement => {
                if let crate::ast::NodeData::ReturnStatement(r) = &child.data
                    && let Some(expr) = &r.expression
                {
                    out.push(Arc::clone(expr));
                }
                false
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => false,
            _ => {
                Self::collect_return_expressions(child, out);
                false
            }
        });
    }

    pub(crate) fn subtree_contains_this(node: &Arc<Node>) -> bool {
        let mut found = false;
        fn walk(root: &Arc<Node>, n: &Arc<Node>, found: &mut bool) {
            if *found {
                return;
            }
            if n.kind == SyntaxKind::ThisKeyword {
                *found = true;
                return;
            }

            if !Arc::ptr_eq(n, root)
                && matches!(
                    n.kind,
                    SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                )
            {
                return;
            }
            crate::ast::node_data_generated::for_each_child(n, |c| {
                walk(root, c, found);
                *found
            });
        }

        if matches!(
            node.kind,
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
        ) {
            return false;
        }
        walk(node, node, &mut found);
        found
    }

    pub(crate) fn getter_return_reaches_this(&mut self, accessor: &Arc<Node>) -> bool {
        let crate::ast::NodeData::GetAccessorDeclaration(gd) = &accessor.data else {
            return false;
        };
        let Some(body) = &gd.body else {
            return false;
        };
        let mut returns = Vec::new();
        Self::collect_return_expressions(&body, &mut returns);
        if returns.is_empty() {
            return false;
        }

        let mut this_aliases: Vec<String> = Vec::new();
        crate::ast::node_data_generated::for_each_child(&body, |stmt| {
            if stmt.kind == SyntaxKind::VariableStatement
                && let crate::ast::NodeData::VariableStatement(vs) = &stmt.data
                && let crate::ast::NodeData::VariableDeclarationList(vdl) =
                    &vs.declaration_list.data
            {
                for decl in vdl.declarations.iter() {
                    if let (Some(name), Some(init)) = (decl.name(), {
                        match &decl.data {
                            crate::ast::NodeData::VariableDeclaration(vd) => vd.initializer.clone(),
                            _ => None,
                        }
                    }) {
                        if name.kind == SyntaxKind::Identifier && Self::subtree_contains_this(&init)
                        {
                            this_aliases.push(name.text().to_string());
                        }
                    }
                }
            }
            false
        });
        returns.iter().any(|r| {
            Self::subtree_contains_this(r) || {
                let mut hit = false;
                fn walk(n: &Arc<Node>, aliases: &[String], hit: &mut bool) {
                    if *hit {
                        return;
                    }
                    if n.kind == SyntaxKind::Identifier && aliases.iter().any(|a| a == n.text()) {
                        *hit = true;
                        return;
                    }
                    crate::ast::node_data_generated::for_each_child(n, |c| {
                        walk(c, aliases, hit);
                        *hit
                    });
                }
                walk(r, &this_aliases, &mut hit);
                hit
            }
        })
    }

    pub(crate) fn this_type_marker_argument(
        &self,
        t: &Arc<Type>,
        depth: usize,
    ) -> Option<Arc<Type>> {
        if depth > 4 {
            return None;
        }
        let constituent_types: Option<Vec<Arc<Type>>> = match &t.data {
            TypeData::Union(u) => Some(u.union_or_intersection.types.to_vec()),
            TypeData::Intersection(i) => Some(i.union_or_intersection.types.to_vec()),
            _ => None,
        };
        if let Some(types) = constituent_types {
            return types
                .iter()
                .find_map(|c| self.this_type_marker_argument(c, depth + 1));
        }
        let obj = t.as_object()?;
        if obj.type_arguments.len() == 1 && t.symbol.as_ref().is_some_and(|s| s.name == "ThisType")
        {
            return Some(Arc::clone(&obj.type_arguments[0]));
        }
        None
    }

    pub(crate) fn build_object_literal_this_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let crate::ast::NodeData::ObjectLiteralExpression(data) = &node.data else {
            return self.get_any_type();
        };
        let mut symbol_table = crate::ast::SymbolTable::new();
        let mut props: Vec<Arc<crate::ast::Symbol>> = Vec::new();
        for prop in data.properties.iter() {
            let Some(name_node) = Self::member_name_node(prop) else {
                continue;
            };
            if !matches!(
                name_node.kind,
                SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
            ) {
                continue;
            }
            let name = name_node.text().to_string();
            let (member_type, readonly) = match &prop.data {
                crate::ast::NodeData::PropertyAssignment(pa) => {
                    let t = self.get_type_of_node(&pa.initializer);
                    (self.get_widened_type_of_literal(&t), false)
                }
                crate::ast::NodeData::ShorthandPropertyAssignment(sa) => {
                    let t = self.get_type_of_node(&sa.name);
                    (t, false)
                }
                crate::ast::NodeData::GetAccessorDeclaration(gd) => {
                    let t = match &gd.type_node {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => self.get_any_type(),
                    };
                    (t, true)
                }

                crate::ast::NodeData::MethodDeclaration(_) => {
                    let mut method_sym =
                        crate::ast::Symbol::new(crate::ast::SymbolFlags::Method, name.clone());
                    method_sym.declarations = vec![Arc::clone(prop)];
                    let method_sym = Arc::new(method_sym);
                    symbol_table.insert(name, Arc::clone(&method_sym));
                    props.push(method_sym);
                    continue;
                }
                _ => continue,
            };
            let prop_sym = Arc::new(crate::ast::Symbol::new(
                crate::ast::SymbolFlags::Property,
                name.clone(),
            ));
            if readonly {
                let sym_mut = Arc::as_ptr(&prop_sym) as *mut crate::ast::Symbol;
                unsafe {
                    (*sym_mut).check_flags |= crate::ast::CheckFlags::Readonly;
                }
            }
            self.value_symbol_links.insert(
                &prop_sym,
                crate::checker::types::ValueSymbolLinks {
                    resolved_type: Some(member_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name, Arc::clone(&prop_sym));
            props.push(prop_sym);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Anonymous
                | crate::checker::types::ObjectFlags::ObjectLiteral,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: crate::checker::types::TypeData::Object(crate::checker::types::ObjectTypeData {
                structured: crate::checker::types::StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn check_object_literal_element(&mut self, node: &Arc<Node>) {
        if let Some(name) = node.name()
            && name.kind == SyntaxKind::PrivateIdentifier
        {
            self.diagnostics.add(crate::ast::Diagnostic::new(
                self.current_file.clone(),
                name.loc,
                crate::diagnostics::messages_generated::
                    PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                vec![],
            ));
            return;
        }

        match node.kind {
            SyntaxKind::PropertyAssignment => {
                if let crate::ast::NodeData::PropertyAssignment(data) = &node.data {
                    self.check_expression(&data.initializer);
                }
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                if let crate::ast::NodeData::ShorthandPropertyAssignment(data) = &node.data {
                    self.check_identifier_reference(&data.name);
                }
            }
            SyntaxKind::SpreadAssignment => {
                if let crate::ast::NodeData::SpreadAssignment(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                self.check_class_member(node);
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
    }
}
