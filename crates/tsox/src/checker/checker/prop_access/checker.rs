#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn is_array_mutation_method(&self, name: &str) -> bool {
        matches!(name, "push" | "unshift")
    }

    pub fn boxed_apparent_type_of_primitive(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        use crate::checker::types::TYPE_FLAGS_ENUM_LIKE;
        let name = if t.flags.intersects(
            TypeFlags::String
                | TypeFlags::StringLiteral
                | TypeFlags::Index
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping,
        ) {
            "String"
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral | TypeFlags::EnumLiteral)
            || (t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) && !t.flags.intersects(TypeFlags::String))
        {
            "Number"
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            "Boolean"
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            "BigInt"
        } else if t
            .flags
            .intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol)
        {
            "Symbol"
        } else {
            return None;
        };
        if let Some(cached) = self.boxed_global_types.get(name) {
            return Some(Arc::clone(cached));
        }

        let mut matching: Vec<Arc<Node>> = Vec::new();
        for file in &self.files {
            let statements = match &file.node.data {
                NodeData::SourceFile(data) => &data.statements,
                _ => continue,
            };
            for stmt in statements.iter() {
                if let NodeData::InterfaceDeclaration(d) = &stmt.data
                    && d.name.text() == name
                {
                    matching.push(Arc::clone(stmt));
                }
            }
        }
        let mut all_members: Vec<Arc<Node>> = Vec::new();
        for stmt in &matching {
            let NodeData::InterfaceDeclaration(d) = &stmt.data else {
                continue;
            };
            all_members.extend(d.members.iter().cloned());

            self.collect_boxed_heritage_members(stmt, &mut all_members, &mut Vec::new(), 0);
        }
        if all_members.is_empty() {
            return None;
        }
        let members = Arc::new(crate::ast::NodeList::new(all_members));
        let built = self.build_interface_type_from_members(&members);
        self.boxed_global_types
            .insert(name.to_string(), Arc::clone(&built));
        Some(built)
    }

    pub(crate) fn collect_boxed_heritage_members(
        &mut self,
        iface_stmt: &Arc<Node>,
        out: &mut Vec<Arc<Node>>,
        visited: &mut Vec<*const Node>,
        depth: usize,
    ) {
        if depth >= 6 {
            return;
        }
        if visited.contains(&Arc::as_ptr(iface_stmt)) {
            return;
        }
        visited.push(Arc::as_ptr(iface_stmt));
        let heritage = match &iface_stmt.data {
            NodeData::InterfaceDeclaration(d) => d.heritage_clauses.clone(),
            _ => return,
        };
        let Some(heritage) = heritage else {
            return;
        };
        for clause in heritage.iter() {
            let NodeData::HeritageClause(hc) = &clause.data else {
                continue;
            };
            if hc.token != SyntaxKind::ExtendsKeyword {
                continue;
            }
            for type_ref in hc.types.iter() {
                let expr = match &type_ref.data {
                    NodeData::ExpressionWithTypeArguments(ewa) => Arc::clone(&ewa.expression),
                    _ => continue,
                };
                let base_decls: Vec<Arc<Node>> =
                    self.with_declaring_file_context(iface_stmt, |c| match expr.kind {
                        SyntaxKind::Identifier => c
                            .resolve_identifier(&expr)
                            .map(|s| s.declarations.clone())
                            .unwrap_or_default(),
                        _ => c
                            .resolve_qualified_symbol(&expr)
                            .map(|s| s.declarations.clone())
                            .unwrap_or_default(),
                    });
                for decl in base_decls {
                    if let NodeData::InterfaceDeclaration(bd) = &decl.data {
                        out.extend(bd.members.iter().cloned());
                        self.collect_boxed_heritage_members(&decl, out, visited, depth + 1);
                    }
                }
            }
        }
    }

    pub(crate) fn global_interface_has_property(
        &mut self,
        symbol_name: &str,
        prop_name: &str,
    ) -> bool {
        if !self.global_interface_members.contains_key(symbol_name) {
            let names = self.collect_global_interface_member_names(symbol_name);
            self.global_interface_members
                .insert(symbol_name.to_string(), names);
        }
        self.global_interface_members
            .get(symbol_name)
            .map(|names| names.iter().any(|n| n == prop_name))
            .unwrap_or(false)
    }

    pub(crate) fn collect_global_interface_member_names(
        &self,
        interface_name: &str,
    ) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for file in &self.files {
            let statements: Vec<Arc<Node>> = match &file.node.data {
                NodeData::SourceFile(data) => data.statements.iter().cloned().collect(),
                _ => continue,
            };
            for stmt in &statements {
                let members = match &stmt.data {
                    NodeData::InterfaceDeclaration(d) if d.name.text() == interface_name => {
                        &d.members
                    }
                    _ => continue,
                };
                for member in members.iter() {
                    let member_name = match &member.data {
                        NodeData::PropertySignatureDeclaration(d) => {
                            self.get_property_name_from_node(&d.name)
                        }
                        NodeData::MethodSignatureDeclaration(d) => {
                            self.get_property_name_from_node(&d.name)
                        }
                        _ => continue,
                    };
                    if !member_name.is_empty() && !names.iter().any(|n| n == &member_name) {
                        names.push(member_name);
                    }
                }
            }
        }
        names
    }
}
