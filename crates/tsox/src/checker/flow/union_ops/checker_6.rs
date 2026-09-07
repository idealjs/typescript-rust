#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn get_property_of_type_cached(
        &self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Symbol>> {
        if let TypeData::Mapped(m) = &t.data
            && m.type_parameter.is_some()
        {
            let sym = Symbol::new(SymbolFlags::Property, name.to_string());
            return Some(Arc::new(sym));
        }

        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                return Some(Arc::clone(sym));
            }
        }

        let is_array_like = self.is_array_type(t) || matches!(&t.data, TypeData::EvolvingArray(_));
        if is_array_like && let Some(array_sym) = self.globals.get("Array") {
            if let Some(declared) = self
                .type_alias_links
                .get(array_sym)
                .and_then(|l| l.declared_type.clone())
                && let Some(structured) = declared.as_structured()
                && let Some(member) = structured.members.get(name)
            {
                return Some(Arc::clone(member));
            }

            if let Some(member) = array_sym.members.get(name) {
                return Some(Arc::clone(member));
            }
        }

        if t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::Anonymous)
            && let Some(structured) = t.as_structured()
            && structured.call_signature_count > 0
            && !self.is_array_type(t)
            && !matches!(&t.data, TypeData::EvolvingArray(_))
        {
            if let Some(function_sym) = self.globals.get("Function") {
                if let Some(member) = function_sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }

        if let Some(interface_name) = self.primitive_interface_name(t) {
            if let Some(sym) = self.globals.get(interface_name) {
                if let Some(member) = sym.members.get(name) {
                    return Some(Arc::clone(member));
                }
            }
        }
        None
    }

    pub(crate) fn primitive_interface_name(&self, t: &Arc<Type>) -> Option<&'static str> {
        if t.flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral)
        {
            Some("String")
        } else if t
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral)
        {
            Some("Number")
        } else if t
            .flags
            .intersects(TypeFlags::Boolean | TypeFlags::BooleanLiteral)
        {
            Some("Boolean")
        } else if t
            .flags
            .intersects(TypeFlags::BigInt | TypeFlags::BigIntLiteral)
        {
            Some("BigInt")
        } else {
            None
        }
    }

    pub(crate) fn get_property_type_of_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> Option<Arc<Type>> {
        let sym = self.get_property_of_type(t, name)?;
        Some(self.get_type_of_symbol(&sym))
    }

    pub(crate) fn type_has_property(&self, t: &Arc<Type>, name: &str) -> PropertyPresence {
        if let Some(structured) = t.as_structured() {
            if let Some(sym) = structured.members.get(name) {
                if sym.flags.contains(SymbolFlags::Optional) {
                    return PropertyPresence::Maybe;
                }
                return PropertyPresence::Definitely;
            }
            if !structured.index_infos.is_empty() {
                return PropertyPresence::Maybe;
            }
            return PropertyPresence::DefinitelyNot;
        }

        if t.flags.contains(TypeFlags::Object) {
            return PropertyPresence::Maybe;
        }

        PropertyPresence::DefinitelyNot
    }

    pub(crate) fn get_instance_type_of_constructor(
        &mut self,
        ctor_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if let Some(prop_sym) = self.get_property_of_type(ctor_type, "prototype") {
            let prop_type = self.get_type_of_symbol(&prop_sym);
            if !prop_type.flags.contains(TypeFlags::Any) {
                return Some(prop_type);
            }
        }

        let construct_sigs = self.get_signatures_of_type(ctor_type, SignatureKind::Construct);
        if !construct_sigs.is_empty() {
            let mut return_types: Vec<Arc<Type>> = Vec::new();
            for sig in &construct_sigs {
                if let Some(rt) = self.get_return_type_of_signature(sig) {
                    if !return_types.iter().any(|t| Arc::ptr_eq(t, &rt)) {
                        return_types.push(rt);
                    }
                }
            }
            if !return_types.is_empty() {
                return Some(self.get_union_type(return_types));
            }
        }
        None
    }

    pub(crate) fn get_accessed_property_name_from_node(node: &Arc<Node>) -> Option<String> {
        match &node.data {
            NodeData::StringLiteral(s) => Some(s.text.clone()),
            NodeData::NumericLiteral(n) => Some(n.text.clone()),
            NodeData::Identifier(id) => Some(id.text.clone()),
            NodeData::PropertyAccessExpression(pa) => Some(pa.name.text().to_string()),
            NodeData::ElementAccessExpression(ea) => {
                Self::get_accessed_property_name_from_node(&ea.argument_expression)
            }

            NodeData::BindingElement(be) => be
                .property_name
                .as_ref()
                .map(|pn| pn.text().to_string())
                .or_else(|| be.name.as_ref().map(|n| n.text().to_string())),
            _ => None,
        }
    }

    pub(crate) fn discriminant_alias_access(
        &self,
        expr: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Node>> {
        if expr.kind != SyntaxKind::Identifier {
            return None;
        }
        let sym = self.resolve_identifier(expr)?;
        if !self.symbol_is_const_variable(&sym) {
            return None;
        }
        let decl = Arc::clone(sym.value_declaration.as_ref()?);

        if let Some(init) = Self::candidate_variable_declaration_initializer(&decl) {
            if matches!(
                init.kind,
                SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
            ) {
                if let Some(recv) = init.expression() {
                    if self.is_symbol_identifier(recv, symbol) {
                        return Some(init);
                    }
                }
            }
        }

        if decl.kind == SyntaxKind::BindingElement {
            let NodeData::BindingElement(be) = &decl.data else {
                return None;
            };
            if be.dot_dot_dot_token.is_none() && be.initializer.is_none() {
                let pattern = decl.parent.as_ref()?;
                let var_decl = Arc::clone(pattern.parent.as_ref()?);
                if let Some(init) = Self::candidate_variable_declaration_initializer(&var_decl) {
                    let init_matches = match init.kind {
                        SyntaxKind::Identifier => self.is_symbol_identifier(&init, symbol),
                        SyntaxKind::PropertyAccessExpression
                        | SyntaxKind::ElementAccessExpression => init
                            .expression()
                            .is_some_and(|recv| self.is_symbol_identifier(recv, symbol)),
                        _ => false,
                    };
                    if init_matches {
                        return Some(decl);
                    }
                }
            }
        }
        None
    }

    pub(crate) fn candidate_variable_declaration_initializer(
        decl: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let NodeData::VariableDeclaration(data) = &decl.data else {
            return None;
        };
        if data.type_node.is_some() {
            return None;
        }
        data.initializer.as_ref().map(Self::skip_parentheses)
    }

    pub(crate) fn is_property_access_on_reference(
        &self,
        node: &Arc<Node>,
        reference: &Arc<Node>,
    ) -> bool {
        let mut r = reference;
        loop {
            match &r.data {
                NodeData::ParenthesizedExpression(p) => r = &p.expression,
                NodeData::NonNullExpression(n) => r = &n.expression,
                _ => break,
            }
        }
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => self.is_matching_reference(r, &pa.expression),
            NodeData::ElementAccessExpression(ea) => self.is_matching_reference(r, &ea.expression),
            _ => false,
        }
    }

    pub(crate) fn is_property_access_on_symbol(
        &self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        match &node.data {
            NodeData::PropertyAccessExpression(pa) => {
                pa.question_dot_token.is_none() && self.is_symbol_identifier(&pa.expression, symbol)
            }
            NodeData::ElementAccessExpression(ea) => {
                ea.question_dot_token.is_none() && self.is_symbol_identifier(&ea.expression, symbol)
            }
            _ => false,
        }
    }
}
