#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn reduced_assignment_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
        evolving: bool,
    ) -> Arc<Type> {
        if evolving {
            return Arc::clone(assigned);
        }

        if declared.flags.contains(TypeFlags::Null)
            && (self.is_auto_array_type(assigned)
                || assigned.object_flags.contains(ObjectFlags::EvolvingArray))
        {
            return Arc::clone(assigned);
        }
        if !declared.is_union() {
            return Arc::clone(declared);
        }
        self.get_assignment_reduced_type(declared, assigned)
    }

    pub(crate) fn get_assignment_reduced_type(
        &mut self,
        declared: &Arc<Type>,
        assigned: &Arc<Type>,
    ) -> Arc<Type> {
        if Arc::ptr_eq(declared, assigned) {
            return Arc::clone(declared);
        }
        if assigned.flags.contains(TypeFlags::Never) {
            return Arc::clone(assigned);
        }
        let constituents = self.constituent_types(declared);
        let kept: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| self.type_maybe_assignable_to(assigned, t))
            .collect();
        let reduced = self.rebuild_union_or_never(declared, kept);
        if self.is_type_assignable_to(assigned, &reduced) {
            reduced
        } else {
            Arc::clone(declared)
        }
    }

    pub(crate) fn type_maybe_assignable_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        if !source.is_union() {
            return self.is_type_assignable_to(source, target);
        }
        let constituents = self.constituent_types(source);
        if constituents.iter().any(|t| Arc::ptr_eq(t, target)) {
            return true;
        }
        constituents
            .iter()
            .any(|t| self.is_type_assignable_to(t, target))
    }

    pub(crate) fn initial_type_of_declaration(&mut self, expr: &Arc<Node>) -> Option<Arc<Type>> {
        match &expr.data {
            NodeData::VariableDeclaration(vd) => {
                if let Some(init) = &vd.initializer {
                    if matches!(
                        &init.data,
                        NodeData::ArrayLiteralExpression(d) if d.elements.is_empty()
                    ) {
                        return Some(self.auto_array_type());
                    }
                    if matches!(
                        init.kind,
                        crate::ast::SyntaxKind::NullKeyword
                            | crate::ast::SyntaxKind::UndefinedKeyword
                    ) {
                        return Some(self.auto_type());
                    }
                    return Some(self.get_type_of_node(init));
                }
                let for_stmt = Self::for_in_or_of_statement_of(expr)?;
                let NodeData::ForInOrOfStatement(data) = &for_stmt.data else {
                    return None;
                };
                match for_stmt.kind {
                    SyntaxKind::ForInStatement => Some(self.string_type()),
                    SyntaxKind::ForOfStatement => {
                        let rhs = self.get_type_of_node(&data.expression);
                        Some(self.iterated_element_type(&rhs))
                    }
                    _ => None,
                }
            }
            NodeData::BindingElement(be) => {
                let pattern = Arc::clone(expr.parent.as_ref()?);
                let pattern_parent = Arc::clone(pattern.parent.as_ref()?);
                let parent_type = self.initial_type_of_declaration(&pattern_parent);
                let mut t = match (&parent_type, pattern.kind) {
                    (Some(parent_type), SyntaxKind::ObjectBindingPattern) => {
                        match Self::binding_element_property_name(expr) {
                            Some(name) => self.get_property_type_of_type(parent_type, &name),
                            None => None,
                        }
                    }
                    (Some(parent_type), SyntaxKind::ArrayBindingPattern)
                        if be.dot_dot_dot_token.is_none() =>
                    {
                        match Self::binding_element_index(&pattern, expr) {
                            Some(index) => self.destructured_array_element_type(parent_type, index),
                            None => None,
                        }
                    }
                    _ => None,
                };
                if let Some(default_expr) = &be.initializer {
                    let default_type = self.get_type_of_node(default_expr);

                    t = match t {
                        Some(t) => {
                            let non_undefined =
                                self.remove_flags_from_union(&t, TypeFlags::Undefined);
                            Some(self.get_union_type(vec![non_undefined, default_type]))
                        }
                        None => Some(default_type),
                    };
                }
                t
            }
            _ => None,
        }
    }

    pub(crate) fn binding_element_property_name(element: &Arc<Node>) -> Option<String> {
        let NodeData::BindingElement(be) = &element.data else {
            return None;
        };
        if let Some(pn) = &be.property_name {
            return Some(pn.text().to_string());
        }
        be.name.as_ref().map(|n| n.text().to_string())
    }

    pub(crate) fn binding_element_index(pattern: &Arc<Node>, element: &Arc<Node>) -> Option<usize> {
        let NodeData::BindingPattern(data) = &pattern.data else {
            return None;
        };
        data.elements
            .nodes
            .iter()
            .position(|e| Arc::ptr_eq(e, element))
    }

    pub(crate) fn destructured_array_element_type(
        &mut self,
        parent_type: &Arc<Type>,
        index: usize,
    ) -> Option<Arc<Type>> {
        if self.is_tuple_type(parent_type) {
            return self.get_tuple_element_type(parent_type, index);
        }
        if self.is_array_type(parent_type) {
            return Some(self.get_array_element_type(parent_type));
        }
        Some(self.get_any_type())
    }

    pub(crate) fn iterated_element_type(&mut self, rhs: &Arc<Type>) -> Arc<Type> {
        if rhs.is_union() {
            let parts: Vec<Arc<Type>> = self
                .constituent_types(rhs)
                .into_iter()
                .map(|c| self.iterated_element_type(&c))
                .filter(|t| !t.flags.contains(TypeFlags::Never))
                .collect();
            if parts.is_empty() {
                return self.get_any_type();
            }
            if parts.len() == 1 {
                return parts.into_iter().next().expect("exactly one");
            }
            return self.get_union_type(parts);
        }
        if self.is_array_type(rhs) {
            return self.get_array_element_type(rhs);
        }
        if rhs
            .flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            return self.string_type();
        }
        self.get_any_type()
    }

    pub(crate) fn for_in_or_of_statement_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let list = decl.parent.as_ref()?;
        if list.kind != SyntaxKind::VariableDeclarationList {
            return None;
        }
        let stmt = list.parent.as_ref()?;
        if matches!(
            stmt.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        ) {
            Some(Arc::clone(stmt))
        } else {
            None
        }
    }

    pub(crate) fn for_in_expression_of(decl: &Arc<Node>) -> Option<Arc<Node>> {
        let stmt = Self::for_in_or_of_statement_of(decl)?;
        if stmt.kind != SyntaxKind::ForInStatement {
            return None;
        }
        match &stmt.data {
            NodeData::ForInOrOfStatement(d) => Some(Arc::clone(&d.expression)),
            _ => None,
        }
    }

    pub(crate) fn get_property_of_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if let Some(sym) = self.get_property_of_type_cached(t, name) {
            return Some(sym);
        }
        if let Some(interface_sym) = self.unresolved_interface_symbol_of(t)
            && let Some(member) = self
                .resolve_interface_type_ex(&interface_sym, None)
                .as_structured()
                .and_then(|s| s.members.get(name))
        {
            return Some(Arc::clone(member));
        }
        None
    }

    pub(crate) fn unresolved_interface_symbol_of(&self, t: &Arc<Type>) -> Option<Arc<Symbol>> {
        if !t.flags.contains(crate::checker::types::TypeFlags::Object) {
            return None;
        }
        let sym = t.symbol.as_ref()?;
        let has_interface_decl = sym
            .declarations
            .iter()
            .any(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)));
        if !has_interface_decl {
            return None;
        }
        if self
            .type_alias_links
            .get(sym)
            .map(|l| l.declared_type.is_some())
            == Some(true)
        {
            return None;
        }
        if let Some(structured) = t.as_structured()
            && !structured.members.entries.is_empty()
        {
            return None;
        }
        Some(Arc::clone(sym))
    }
}
